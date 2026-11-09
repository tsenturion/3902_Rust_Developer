/*
cargo run --offline --bin mutex_booking
cargo run --offline --bin mutex_booking -- poison-demo
*/

/*
Несколько терминалов бронируют места на один рейс. Проверка остатка и
запись брони выполняются под одной блокировкой. Повторный номер заказа
отклоняется. При отмене места возвращаются. Для быстрого просмотра
остатка терминал должен уметь узнать, занят ли журнал прямо сейчас.
В режиме poison-demo имитируется сбой, после которого остаток сверяют
по подтверждённым броням.
*/

use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex, TryLockError};
use std::thread;

struct BookingState {
    capacity: u16,
    free: u16,
    reservations: BTreeMap<u32, u16>,
}

impl BookingState {
    fn new(capacity: u16) -> Self {
        Self {
            capacity,
            free: capacity,
            reservations: BTreeMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BookingError {
    ZeroPlaces,
    Duplicate { order: u32 },
    NotEnough { requested: u16, free: u16 },
    UnknownOrder { order: u32 },
    Poisoned,
    Inconsistent,
}

fn reserve(book: &Mutex<BookingState>, order: u32, places: u16) -> Result<(), BookingError> {
    if places == 0 {
        return Err(BookingError::ZeroPlaces);
    }

    // Guard удерживает блокировку до выхода из функции, включая ранний возврат.
    let mut state = book.lock().map_err(|_| BookingError::Poisoned)?;
    if state.reservations.contains_key(&order) {
        return Err(BookingError::Duplicate { order });
    }
    let remaining = state
        .free
        .checked_sub(places)
        .ok_or(BookingError::NotEnough {
            requested: places,
            free: state.free,
        })?;
    state.reservations.insert(order, places);
    state.free = remaining;
    Ok(())
}

fn cancel(book: &Mutex<BookingState>, order: u32) -> Result<u16, BookingError> {
    let mut state = book.lock().map_err(|_| BookingError::Poisoned)?;
    let places = state
        .reservations
        .remove(&order)
        .ok_or(BookingError::UnknownOrder { order })?;
    state.free += places;
    Ok(places)
}

fn try_free(book: &Mutex<BookingState>) -> Result<Option<u16>, BookingError> {
    match book.try_lock() {
        Ok(state) => Ok(Some(state.free)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Poisoned(_)) => Err(BookingError::Poisoned),
    }
}

fn check_contention(book: &Arc<Mutex<BookingState>>) {
    let guard = book.lock().expect("Журнал примера доступен");
    // let _ = thread::spawn(move || drop(guard));
    let other = Arc::clone(book);
    // try_lock сразу возвращает результат, даже пока главный поток удерживает guard.
    let preview = thread::spawn(move || try_free(&other));
    let busy = preview.join().expect("Поток просмотра завершился");
    assert_eq!(busy, Ok(None));
    drop(guard);
    let available = try_free(book);
    assert_eq!(available, Ok(Some(8)));
    println!("Быстрый просмотр — под блокировкой {busy:?}, после освобождения {available:?}");
}

fn booking_demo() {
    let book = Arc::new(Mutex::new(BookingState::new(8)));
    check_contention(&book);
    assert_eq!(reserve(&book, 30, 0), Err(BookingError::ZeroPlaces));

    let workers = [31, 32, 33].map(|order| {
        let book = Arc::clone(&book);
        thread::spawn(move || (order, reserve(&book, order, 3)))
    });
    let outcomes = workers.map(|worker| worker.join());

    let mut accepted = Vec::new();
    let mut declined = Vec::new();
    for outcome in outcomes {
        let (order, result) = outcome.expect("Поток бронирования завершился");
        match result {
            Ok(()) => accepted.push(order),
            Err(BookingError::NotEnough {
                requested: 3,
                free: 2,
            }) => declined.push(order),
            other => panic!("Неожиданный результат заявки {order} — {other:?}"),
        }
    }

    // Победители зависят от планировщика. Проверяем общие итоги.
    assert_eq!((accepted.len(), declined.len()), (2, 1));
    {
        let state = book.lock().expect("Потоки бронирования освободили журнал");
        assert_eq!(state.reservations.len(), 2);
        assert_eq!(
            state.free + state.reservations.values().copied().sum::<u16>(),
            state.capacity
        );
        for order in &accepted {
            assert_eq!(state.reservations.get(order), Some(&3));
        }
        assert!(!state.reservations.contains_key(&declined[0]));
    }
    assert_eq!(try_free(&book), Ok(Some(2)));
    println!(
        "Подтверждено {}, отклонено {}, свободных мест 2",
        accepted.len(),
        declined.len()
    );

    let existing = accepted[0];
    let waiting = declined[0];
    assert_eq!(
        reserve(&book, existing, 1),
        Err(BookingError::Duplicate { order: existing })
    );
    assert_eq!(try_free(&book), Ok(Some(2)));
    assert_eq!(cancel(&book, existing), Ok(3));
    assert_eq!(
        cancel(&book, existing),
        Err(BookingError::UnknownOrder { order: existing })
    );
    assert_eq!(try_free(&book), Ok(Some(5)));
    assert_eq!(reserve(&book, waiting, 3), Ok(()));
    assert_eq!(try_free(&book), Ok(Some(2)));
    println!("После отмены места переданы ожидающей заявке");
}

fn restore_free(book: &Mutex<BookingState>) -> Result<(), BookingError> {
    let mut state = match book.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    };

    // Журнал броней — источник для восстановления. Снимаем флаг только после сверки.
    let occupied = state
        .reservations
        .values()
        .copied()
        .try_fold(0_u16, u16::checked_add)
        .ok_or(BookingError::Inconsistent)?;
    state.free = state
        .capacity
        .checked_sub(occupied)
        .ok_or(BookingError::Inconsistent)?;
    book.clear_poison();
    Ok(())
}

fn poison_demo() {
    let book = Arc::new(Mutex::new(BookingState::new(8)));
    assert_eq!(reserve(&book, 31, 3), Ok(()));
    let other = Arc::clone(&book);
    let failed = thread::spawn(move || {
        let mut state = other.lock().expect("Журнал доступен до сбоя");
        state.free = 0;
        panic!("учебный сбой при обновлении остатка");
    });

    assert!(failed.join().is_err());
    assert!(book.is_poisoned());
    assert_eq!(try_free(&book), Err(BookingError::Poisoned));
    assert_eq!(reserve(&book, 32, 4), Err(BookingError::Poisoned));
    println!("После паники — журнал помечен, обычная бронь отклонена");
    assert_eq!(restore_free(&book), Ok(()));
    assert!(!book.is_poisoned());
    assert_eq!(try_free(&book), Ok(Some(5)));
    assert_eq!(reserve(&book, 32, 4), Ok(()));
    assert_eq!(try_free(&book), Ok(Some(1)));
    println!("Остаток восстановлен по журналу. После новой брони свободно одно место");
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Turn {
    Zero,
    Odd,
    Even,
}

struct Sequence {
    next: u32,
    turn: Turn,
    values: Vec<u32>,
}

fn zero_even_odd(limit: u16) -> Vec<u32> {
    let limit = u32::from(limit);
    let state = Mutex::new(Sequence {
        next: 1,
        turn: Turn::Zero,
        values: Vec::new(),
    });
    let changed = Condvar::new();
    thread::scope(|scope| {
        for role in [Turn::Even, Turn::Odd, Turn::Zero] {
            let state = &state;
            let changed = &changed;
            scope.spawn(move || {
                let mut sequence = state.lock().expect("Последовательность доступна");
                loop {
                    sequence = changed
                        .wait_while(sequence, |current| {
                            current.next <= limit && current.turn != role
                        })
                        .expect("Ожидание очереди завершилось");
                    let next = sequence.next;
                    if next > limit {
                        break;
                    }
                    match role {
                        Turn::Zero => {
                            sequence.values.push(0);
                            sequence.turn = if next % 2 == 0 { Turn::Even } else { Turn::Odd };
                        }
                        Turn::Even | Turn::Odd => {
                            sequence.values.push(next);
                            sequence.next += 1;
                            sequence.turn = Turn::Zero;
                        }
                    }
                    changed.notify_all();
                }
            });
        }
    });
    state.into_inner().expect("Все этапы завершились").values
}

fn condition_demo() {
    let sequence = zero_even_odd(5);
    assert_eq!(sequence, [0, 1, 0, 2, 0, 3, 0, 4, 0, 5]);
    assert!(zero_even_odd(0).is_empty());
    assert_eq!(zero_even_odd(1), [0, 1]);
    println!("Условная переменная — {sequence:?}");
}

const DINERS: usize = 5;

#[derive(Default)]
struct TableState {
    next_ticket: usize,
    serving: usize,
    busy: [bool; DINERS],
}

#[derive(Default)]
struct Table {
    state: Mutex<TableState>,
    changed: Condvar,
}

struct Forks<'table> {
    table: &'table Table,
    left: usize,
    right: usize,
}

impl Table {
    fn take_forks(&self, diner: usize) -> Forks<'_> {
        let left = diner;
        let right = (diner + 1) % DINERS;
        let mut state = self.state.lock().expect("Стол доступен");
        let ticket = state.next_ticket;
        state.next_ticket = ticket.checked_add(1).expect("Номера очереди исчерпаны");
        let mut state = self
            .changed
            .wait_while(state, |state| {
                state.serving != ticket || state.busy[left] || state.busy[right]
            })
            .expect("Очередь к столу доступна");

        // Нельзя получить одну вилку отдельно. Обе выдаются в одной критической секции.
        state.busy[left] = true;
        state.busy[right] = true;
        state.serving += 1;
        self.changed.notify_all();
        Forks {
            table: self,
            left,
            right,
        }
    }
}

impl Drop for Forks<'_> {
    fn drop(&mut self) {
        let mut state = self
            .table
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.busy[self.left] = false;
        state.busy[self.right] = false;
        self.table.changed.notify_all();
    }
}

fn table_demo() {
    let table = Table::default();
    let meals = thread::scope(|scope| {
        let workers: [_; DINERS] = std::array::from_fn(|diner| {
            let table = &table;
            scope.spawn(move || {
                let mut meals = 0;
                for _ in 0..2 {
                    let _forks = table.take_forks(diner);
                    meals += 1;
                    thread::yield_now();
                }
                meals
            })
        });
        workers.map(|worker| worker.join().expect("Участник закончил работу"))
    });
    let state = table.state.into_inner().expect("Все вилки возвращены");
    assert_eq!(meals, [2; DINERS]);
    assert_eq!((state.serving, state.next_ticket), (10, 10));
    let free = state.busy.into_iter().filter(|busy| !busy).count();
    assert_eq!(free, DINERS);
    println!("Очередь к столу — приёмы {meals:?}, свободных вилок {free}");
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("poison-demo") {
        poison_demo();
    } else {
        booking_demo();
        condition_demo();
        table_demo();
    }
}
