/*
cargo run --offline --bin practice_4_5
cargo test --offline --bin practice_4_5
*/

/*
Задача 1. Два этапа обработки буфера

Первый поток ограничивает показания верхним пределом прямо в буфере.
Второй получает этот буфер через join, суммирует исправленные значения
и возвращает буфер с результатом. Проверить, что выделенный блок памяти
сохранился на обоих этапах.
*/
mod buffer_pipeline {
    use std::thread;

    fn process(frame: Box<[u16]>, limit: u16) -> (Box<[u16]>, usize, u128) {
        let address = frame.as_ptr().addr();
        let normalized = thread::spawn(move || {
            let mut frame = frame;
            assert_eq!(frame.as_ptr().addr(), address);
            let mut clamped = 0;
            for value in &mut *frame {
                if *value > limit {
                    *value = limit;
                    clamped += 1;
                }
            }
            (frame, clamped)
        });

        let summarized = thread::spawn(move || {
            let (frame, clamped) = normalized.join().expect("Обработка показаний завершилась");
            assert_eq!(frame.as_ptr().addr(), address);
            let sum = frame.iter().map(|&value| u128::from(value)).sum();
            (frame, clamped, sum)
        });
        summarized.join().expect("Подсчёт показаний завершился")
    }

    pub fn run() {
        let frame: Box<[u16]> = Box::new([120, 180, 90, 150]);
        let address = frame.as_ptr();
        let (frame, clamped, sum) = process(frame, 150);
        assert_eq!(frame.as_ptr(), address);
        assert_eq!(&*frame, &[120, 150, 90, 150]);
        assert_eq!((clamped, sum), (1, 510));
        println!(
            "Буфер вернулся по прежнему адресу. Сумма после ограничения — {sum}, изменено показаний {clamped}"
        );
        let (empty, clamped, sum) = process(Box::new([]), 150);
        assert!(empty.is_empty());
        assert_eq!((clamped, sum), (0, 0));
        let (zero_limit, clamped, sum) = process(Box::new([0, u16::MAX]), 0);
        assert_eq!((&*zero_limit, clamped, sum), (&[0, 0][..], 1, 0));
    }
}

/*
Задача 2. Встречные перемещения между зонами

У каждой зоны отдельный мьютекс. Два потока могут одновременно переносить
товар в противоположных направлениях. Захватывать зоны всегда по возрастанию
номера. Перед записью проверить наличие товара и вместимость счётчика
получателя. При отказе сохранить оба остатка.
*/
mod transfers {
    use std::sync::Mutex;
    use std::thread;

    #[derive(Debug, PartialEq)]
    enum TransferError {
        UnknownZone(usize),
        SameZone,
        NotEnough,
        Overflow,
        Poisoned,
    }

    fn transfer(
        zones: &[Mutex<u16>],
        from: usize,
        to: usize,
        count: u16,
    ) -> Result<(), TransferError> {
        let source = zones.get(from).ok_or(TransferError::UnknownZone(from))?;
        let destination = zones.get(to).ok_or(TransferError::UnknownZone(to))?;
        if from == to {
            return Err(TransferError::SameZone);
        }

        let (lower, higher) = if from < to {
            (source, destination)
        } else {
            (destination, source)
        };
        let mut lower = lower.lock().map_err(|_| TransferError::Poisoned)?;
        let mut higher = higher.lock().map_err(|_| TransferError::Poisoned)?;
        let (source, destination) = if from < to {
            (&mut *lower, &mut *higher)
        } else {
            (&mut *higher, &mut *lower)
        };

        let source_after = source.checked_sub(count).ok_or(TransferError::NotEnough)?;
        let destination_after = destination
            .checked_add(count)
            .ok_or(TransferError::Overflow)?;
        *source = source_after;
        *destination = destination_after;
        Ok(())
    }

    fn snapshot(zones: &[Mutex<u16>; 2]) -> [u16; 2] {
        let first = zones[0].lock().expect("Первая зона доступна");
        let second = zones[1].lock().expect("Вторая зона доступна");
        [*first, *second]
    }

    pub fn run() {
        let zones = [Mutex::new(20), Mutex::new(10)];
        let (outgoing, incoming) = thread::scope(|scope| {
            let outgoing = scope.spawn(|| transfer(&zones, 0, 1, 4));
            let incoming = scope.spawn(|| transfer(&zones, 1, 0, 3));
            (
                outgoing.join().expect("Первый перенос завершился"),
                incoming.join().expect("Второй перенос завершился"),
            )
        });
        assert_eq!((outgoing, incoming), (Ok(()), Ok(())));
        assert_eq!(snapshot(&zones), [19, 11]);
        assert_eq!(transfer(&zones, 0, 0, 1), Err(TransferError::SameZone));
        assert_eq!(
            transfer(&zones, 0, 2, 1),
            Err(TransferError::UnknownZone(2))
        );
        assert_eq!(transfer(&zones, 1, 0, 12), Err(TransferError::NotEnough));
        assert_eq!(snapshot(&zones), [19, 11]);
        println!("Остатки после встречных переносов — {:?}", snapshot(&zones));

        let full = [Mutex::new(2), Mutex::new(u16::MAX)];
        let failure = transfer(&full, 0, 1, 1);
        let after = snapshot(&full);
        assert_eq!(failure, Err(TransferError::Overflow));
        assert_eq!(after, [2, u16::MAX]);
        println!("Отклонённый перенос — {failure:?}, остатки {after:?}");
    }
}

/*
Задача 3. Очередь расчёта площадей

Потоки забирают задания из общей очереди. Под блокировкой только извлечь
задание. Рассчитать площадь вне блокировки. Нулевая сторона делает
задание ошибочным. Атомарный счётчик учитывает все завершённые задания,
а отчёт хранит результат каждого расчёта по номеру.
*/
mod queue {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    struct Job {
        number: u32,
        width: u16,
        length: u16,
    }

    #[derive(Debug, PartialEq)]
    enum AreaError {
        ZeroSide,
    }

    type Report = (u32, Result<u32, AreaError>);

    fn work(queue: &Mutex<VecDeque<Job>>, finished: &AtomicUsize) -> Vec<Report> {
        let mut results = Vec::new();
        loop {
            let job = {
                let mut pending = queue.lock().expect("Очередь доступна");
                pending.pop_front()
            };
            let Some(job) = job else { break };

            let area = if job.width == 0 || job.length == 0 {
                Err(AreaError::ZeroSide)
            } else {
                Ok(u32::from(job.width) * u32::from(job.length))
            };
            results.push((job.number, area));
            finished.fetch_add(1, Ordering::Relaxed);
        }
        results
    }

    fn process(jobs: VecDeque<Job>) -> (Vec<Report>, usize) {
        let queue = Arc::new(Mutex::new(jobs));
        let finished = Arc::new(AtomicUsize::new(0));
        let workers = [(); 3].map(|()| {
            let queue = Arc::clone(&queue);
            let finished = Arc::clone(&finished);
            thread::spawn(move || work(&queue, &finished))
        });

        let outcomes = workers.map(|worker| worker.join());
        let mut report = Vec::new();
        for outcome in outcomes {
            report.extend(outcome.expect("Рабочий поток завершился"));
        }
        report.sort_by_key(|(number, _)| *number);
        let count = finished.load(Ordering::Relaxed);
        assert_eq!(count, report.len());
        assert!(queue.lock().expect("Очередь доступна").is_empty());
        (report, count)
    }

    pub fn run() {
        let (report, count) = process(VecDeque::from([
            Job {
                number: 4,
                width: 80,
                length: 120,
            },
            Job {
                number: 1,
                width: 0,
                length: 120,
            },
            Job {
                number: 3,
                width: 50,
                length: 40,
            },
            Job {
                number: 2,
                width: 20,
                length: 30,
            },
        ]));
        assert_eq!(
            report,
            [
                (1, Err(AreaError::ZeroSide)),
                (2, Ok(600)),
                (3, Ok(2_000)),
                (4, Ok(9_600))
            ]
        );
        assert_eq!(process(VecDeque::new()), (Vec::new(), 0));
        assert_eq!(
            process(VecDeque::from([Job {
                number: 8,
                width: u16::MAX,
                length: u16::MAX,
            }])),
            (vec![(8, Ok(u32::from(u16::MAX).pow(2)))], 1)
        );
        println!("Обработано заданий из очереди — {count}");
        println!("Отчёт очереди — {report:?}");
    }
}

/*
Задача 4. Выдача ограниченной серии талонов

Параллельно выдавать уникальные номера из диапазона [первый, конец).
Исчерпание серии возвращает None. При повторной попытке учитывать
номер, который уже успел получить другой поток. Проверить диапазон,
заканчивающийся на usize::MAX.
*/
mod tickets {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    fn issue(next: &AtomicUsize, end: usize) -> Option<usize> {
        let mut number = next.load(Ordering::Relaxed);
        while number < end {
            // compare_exchange_weak может вернуть Err даже при совпадении. Снова проверяем предел.
            match next.compare_exchange_weak(
                number,
                number + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(issued) => return Some(issued),
                Err(observed) => number = observed,
            }
        }
        None
    }

    pub fn run() {
        let next = AtomicUsize::new(100);
        let results = thread::scope(|scope| {
            let workers = [(); 5].map(|()| scope.spawn(|| issue(&next, 103)));
            workers.map(|worker| worker.join().expect("Выдача талона завершилась"))
        });

        let exhausted = results.iter().filter(|result| result.is_none()).count();
        assert_eq!(exhausted, 2);
        let mut issued: Vec<_> = results.into_iter().flatten().collect();
        issued.sort_unstable();
        assert_eq!(issued, [100, 101, 102]);
        assert_eq!(next.load(Ordering::Relaxed), 103);
        assert_eq!(issue(&next, 103), None);
        let last = AtomicUsize::new(usize::MAX - 1);
        assert_eq!(issue(&last, usize::MAX), Some(usize::MAX - 1));
        assert_eq!(issue(&last, usize::MAX), None);
        println!("Выданы уникальные талоны {issued:?}");
        println!(
            "Серия исчерпана — без талона {exhausted} запроса, следующая позиция {}",
            next.into_inner()
        );
    }
}

/*
Задача 5. Возврат рабочего буфера после ошибки

У рабочих потоков общий пул буферов. Пока буфер выдан, он принадлежит
одному заданию. При завершении задания очистить его и вернуть в пул
через Drop, в том числе при раннем возврате с ошибкой. После возврата
следующее задание должно получить тот же блок памяти.
*/
mod buffer_pool {
    use std::sync::{Arc, Mutex};
    use std::thread;

    struct Pool {
        free: Mutex<Vec<Box<[u8]>>>,
    }

    struct Lease {
        pool: Arc<Pool>,
        buffer: Option<Box<[u8]>>,
    }

    fn checkout(pool: &Arc<Pool>) -> Option<Lease> {
        let buffer = pool.free.lock().expect("Пул доступен").pop()?;
        Some(Lease {
            pool: Arc::clone(pool),
            buffer: Some(buffer),
        })
    }

    impl Lease {
        fn bytes(&mut self) -> &mut [u8] {
            self.buffer
                .as_deref_mut()
                .expect("У выданной аренды есть буфер")
        }
    }

    impl Drop for Lease {
        fn drop(&mut self) {
            if let Some(mut buffer) = self.buffer.take() {
                buffer.fill(0);
                // Если пул повреждён, буфер освободится при выходе из блока.
                if let Ok(mut free) = self.pool.free.lock() {
                    free.push(buffer);
                }
            }
        }
    }

    fn receive(mut lease: Lease, payload: &[u8]) -> Result<(), usize> {
        let buffer = lease.bytes();
        let mut payload = payload.iter();
        for slot in buffer {
            let Some(&byte) = payload.next() else {
                return Ok(());
            };
            *slot = byte;
        }
        if payload.len() > 0 {
            return Err(payload.len());
        }
        Ok(())
    }

    pub fn run() {
        let pool = Arc::new(Pool {
            free: Mutex::new(vec![Box::new([0; 4])]),
        });
        let mut lease = checkout(&pool).expect("В пуле есть один буфер");
        let address = lease.bytes().as_ptr();
        let got_second = thread::scope(|scope| {
            scope
                .spawn(|| checkout(&pool).is_some())
                .join()
                .expect("Проверка пула завершилась")
        });
        assert!(!got_second);
        println!("Выдача из пула — второй поток получил буфер {got_second}");

        let failed = thread::spawn(move || receive(lease, &[10, 20, 30, 40, 50]));
        let extra = failed
            .join()
            .expect("Приём завершился")
            .expect_err("Пакет должен превышать ёмкость");
        assert_eq!(extra, 1);

        let mut recycled = checkout(&pool).expect("Буфер вернулся после ошибки");
        assert_eq!(recycled.bytes().as_ptr(), address);
        assert_eq!(recycled.bytes(), &[0, 0, 0, 0]);
        println!(
            "После ошибки тот же буфер вернулся в пул очищенным. Лишних байтов {extra}, буфер {:?}",
            recycled.bytes()
        );
        assert_eq!(receive(recycled, &[10, 20]), Ok(()));
        assert_eq!(pool.free.lock().expect("Пул доступен").len(), 1);
    }
}

/*
Задача 6. Подтверждение веса по этапам

Первый поток публикует суммарный вес трёх мест, второй ждёт его и сверяет с лимитом.
Главный поток читает решение и вес только после подтверждения готовности
решения. Вес на границе лимита проходит. Этапы упорядочены через Release
и Acquire, а ожидание в учебном примере ограничено по времени.
*/
mod approval {
    use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    const WEIGHT_READY: u8 = 1;
    const DECISION_READY: u8 = 2;

    fn wait_for(stage: &AtomicU8, target: u8) {
        let started = Instant::now();
        while stage.load(Ordering::Acquire) < target {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "Этап не завершился вовремя"
            );
            thread::yield_now();
        }
    }

    fn approve(weights: [u16; 3], limit: u32) -> (u32, bool) {
        let stage = AtomicU8::new(0);
        let total = AtomicU32::new(0);
        let accepted = AtomicBool::new(false);

        thread::scope(|scope| {
            let weighing = scope.spawn(|| {
                let weight = weights.into_iter().map(u32::from).sum();
                total.store(weight, Ordering::Relaxed);
                stage.store(WEIGHT_READY, Ordering::Release);
            });
            let checking = scope.spawn(|| {
                wait_for(&stage, WEIGHT_READY);
                accepted.store(total.load(Ordering::Relaxed) <= limit, Ordering::Relaxed);
                stage.store(DECISION_READY, Ordering::Release);
            });

            // Публикация решения следует за чтением опубликованного веса.
            wait_for(&stage, DECISION_READY);
            let result = (
                total.load(Ordering::Relaxed),
                accepted.load(Ordering::Relaxed),
            );
            weighing.join().expect("Взвешивание завершилось");
            checking.join().expect("Проверка завершилась");
            result
        })
    }

    pub fn run() {
        let at_limit = approve([5, 7, 8], 20);
        let over_limit = approve([5, 7, 9], 20);
        assert_eq!(at_limit, (20, true));
        assert_eq!(over_limit, (21, false));
        assert_eq!(approve([0, 0, 0], 0), (0, true));
        assert_eq!(
            approve([u16::MAX; 3], u32::from(u16::MAX)),
            (u32::from(u16::MAX) * 3, false)
        );
        println!(
            "Вес {} кг принят. Вес {} кг при том же лимите отклонён",
            at_limit.0, over_limit.0
        );
    }
}

/*
Задача 7. Сборка групп H2O

Один поток представляет один атом. Перед записью должны собраться
два атома водорода и один кислорода. Следующая группа получает места
только после записи всех атомов текущей группы.
*/
mod molecules {
    use std::io;
    use std::sync::{Condvar, Mutex};
    use std::thread;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Atom {
        H,
        O,
    }

    const GROUP: [usize; 2] = [2, 1];

    struct Assembly {
        reserved: [usize; 2],
        emitted: Vec<Atom>,
        cancelled: bool,
    }

    fn bond(atom: Atom, state: &Mutex<Assembly>, changed: &Condvar) {
        let slot = match atom {
            Atom::H => 0,
            Atom::O => 1,
        };
        let state = state.lock().expect("Сборка доступна");
        let mut state = changed
            .wait_while(state, |state| {
                !state.cancelled && state.reserved[slot] == GROUP[slot]
            })
            .expect("Ожидание места завершилось");
        if state.cancelled {
            return;
        }

        state.reserved[slot] += 1;
        if state.reserved == GROUP {
            changed.notify_all();
        }
        let mut state = changed
            .wait_while(state, |state| !state.cancelled && state.reserved != GROUP)
            .expect("Ожидание группы завершилось");
        if state.cancelled {
            return;
        }

        state.emitted.push(atom);
        // Места открываются после записи всех участников, включая тех, кто проснулся позже.
        if state.emitted.len() % 3 == 0 {
            state.reserved = [0, 0];
            changed.notify_all();
        }
    }

    fn build(atoms: &[Atom]) -> io::Result<Option<Vec<Atom>>> {
        let oxygen = atoms.iter().filter(|&&atom| atom == Atom::O).count();
        let hydrogen = atoms.len() - oxygen;
        if hydrogen % 2 != 0 || hydrogen / 2 != oxygen {
            return Ok(None);
        }

        let state = Mutex::new(Assembly {
            reserved: [0, 0],
            emitted: Vec::with_capacity(atoms.len()),
            cancelled: false,
        });
        let changed = Condvar::new();
        thread::scope(|scope| {
            for &atom in atoms {
                let state = &state;
                let changed = &changed;
                if let Err(error) =
                    thread::Builder::new().spawn_scoped(scope, move || bond(atom, state, changed))
                {
                    // При отказе запуска отпускаем и тех, кто ждёт недостающего участника.
                    state.lock().expect("Сборка доступна").cancelled = true;
                    changed.notify_all();
                    return Err(error);
                }
            }
            Ok(())
        })?;
        Ok(Some(
            state.into_inner().expect("Сборка завершилась").emitted,
        ))
    }

    fn assert_groups(atoms: &[Atom], expected: usize) {
        assert_eq!(atoms.len(), expected * 3);
        for group in atoms.chunks_exact(3) {
            assert_eq!(group.iter().filter(|&&atom| atom == Atom::H).count(), 2);
            assert_eq!(group.iter().filter(|&&atom| atom == Atom::O).count(), 1);
        }
    }

    pub fn run() {
        use Atom::{H, O};
        let built = build(&[O, O, H, H, H, H])
            .expect("Потоки запущены")
            .expect("Состав сбалансирован");
        assert_groups(&built, 2);
        let reversed = build(&[H, H, H, H, O, O])
            .expect("Потоки запущены")
            .expect("Состав сбалансирован");
        assert_groups(&reversed, 2);
        assert_eq!(build(&[]).expect("Пустой вход допустим"), Some(vec![]));
        assert_eq!(build(&[H, O]).expect("Некорректный вход обработан"), None);
        println!(
            "Группы H2O — собрано {}, в каждой два H и один O",
            built.len() / 3
        );
    }
}

/*
Задача 8. Упорядоченный вывод FizzBuzz

Четыре исполнителя отвечают за разные значения. На каждом шаге
состояние содержит один текущий номер, который записывает только
подходящий исполнитель. После последнего шага все завершают ожидание.
*/
mod ordered_fizzbuzz {
    use std::io;
    use std::sync::{Condvar, Mutex};
    use std::thread;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Role {
        Number,
        Fizz,
        Buzz,
        Both,
    }

    impl Role {
        fn for_number(number: u32) -> Self {
            match (number % 3 == 0, number % 5 == 0) {
                (true, true) => Self::Both,
                (true, false) => Self::Fizz,
                (false, true) => Self::Buzz,
                (false, false) => Self::Number,
            }
        }

        fn render(self, number: u32) -> String {
            match self {
                Self::Number => number.to_string(),
                Self::Fizz => String::from("fizz"),
                Self::Buzz => String::from("buzz"),
                Self::Both => String::from("fizzbuzz"),
            }
        }
    }

    struct Output {
        next: u32,
        entries: Vec<(u32, Role)>,
    }

    fn emit(role: Role, limit: u32, state: &Mutex<Output>, changed: &Condvar) {
        let mut state = state.lock().expect("Очередь вывода доступна");
        loop {
            state = changed
                .wait_while(state, |state| {
                    state.next <= limit && Role::for_number(state.next) != role
                })
                .expect("Ожидание номера завершилось");
            let number = state.next;
            if number > limit {
                return;
            }
            state.entries.push((number, role));
            state.next += 1;
            changed.notify_all();
        }
    }

    fn sequence(limit: u16) -> io::Result<Vec<String>> {
        let state = Mutex::new(Output {
            next: 1,
            entries: Vec::with_capacity(usize::from(limit)),
        });
        let limit = u32::from(limit);
        let changed = Condvar::new();
        thread::scope(|scope| {
            for role in [Role::Buzz, Role::Both, Role::Fizz, Role::Number] {
                let state = &state;
                let changed = &changed;
                if let Err(error) = thread::Builder::new()
                    .spawn_scoped(scope, move || emit(role, limit, state, changed))
                {
                    state.lock().expect("Очередь вывода доступна").next = limit + 1;
                    changed.notify_all();
                    return Err(error);
                }
            }
            Ok(())
        })?;
        let output = state.into_inner().expect("Вывод завершился");
        Ok(output
            .entries
            .into_iter()
            .map(|(number, role)| role.render(number))
            .collect())
    }

    pub fn run() {
        let output = sequence(15).expect("Потоки вывода запущены");
        assert_eq!(
            output,
            [
                "1", "2", "fizz", "4", "buzz", "fizz", "7", "8", "fizz", "buzz", "11", "fizz",
                "13", "14", "fizzbuzz"
            ]
        );
        assert_eq!(sequence(1).expect("Потоки вывода запущены"), ["1"]);
        assert!(sequence(0).expect("Пустой вывод допустим").is_empty());
        println!("Порядок FizzBuzz — {}", output.join(", "));
    }
}

fn main() {
    buffer_pipeline::run();
    transfers::run();
    queue::run();
    tickets::run();
    buffer_pool::run();
    approval::run();
    molecules::run();
    ordered_fizzbuzz::run();
}

#[cfg(test)]
mod tests {
    #[test]
    fn buffer_pipeline() {
        super::buffer_pipeline::run();
    }

    #[test]
    fn transfers() {
        super::transfers::run();
    }

    #[test]
    fn queue() {
        super::queue::run();
    }

    #[test]
    fn tickets() {
        super::tickets::run();
    }

    #[test]
    fn buffer_pool() {
        super::buffer_pool::run();
    }

    #[test]
    fn approval() {
        super::approval::run();
    }

    #[test]
    fn molecules() {
        super::molecules::run();
    }

    #[test]
    fn ordered_fizzbuzz() {
        super::ordered_fizzbuzz::run();
    }
}
