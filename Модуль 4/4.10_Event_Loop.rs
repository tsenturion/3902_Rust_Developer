/*
cargo run --offline --bin event_loop
*/

/*
Терминал резервирует ячейку для заказа. Сканер присылает номер ячейки
и заказа. Верное сканирование завершает выдачу, а истечение срока
освобождает бронь. После остановки терминал отбрасывает оставшиеся брони.

Таблицей броней владеет поток, выполняющий цикл. Сканер передаёт команды через канал
и получает уведомления после их обработки. Пока выполняется обработчик,
новые команды остаются в очереди. Поэтому обработчики обновляют состояние
и возвращают результат, а ожидание сосредоточено в самом цикле.
*/

use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Booking {
    locker: u8,
    order: u32,
}

enum Command {
    Reserve {
        booking: Booking,
        duration: Duration,
    },
    Scan(Booking),
    Stop,
}

#[derive(Debug, PartialEq, Eq)]
enum Notice {
    Reserved(Booking),
    Occupied(Booking),
    InvalidDuration(Booking),
    Collected(Booking),
    ScanRejected(Booking),
    Expired(Booking),
}

#[derive(Debug, PartialEq, Eq)]
enum StopReason {
    Requested,
    InputClosed,
}

#[derive(Debug, PartialEq, Eq)]
struct Exit {
    reason: StopReason,
    released_on_stop: usize,
}

// Порядок полей задаёт сортировку. При совпадении сроков порядок определяет
// номер ячейки, поэтому таймеры разных броней хранятся отдельно.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Deadline {
    at: Instant,
    locker: u8,
}

struct ActiveBooking {
    order: u32,
    deadline: Deadline,
}

#[derive(Default)]
struct Terminal {
    active: HashMap<u8, ActiveBooking>,
    deadlines: BTreeSet<Deadline>,
}

impl Terminal {
    fn run(mut self, commands: Receiver<Command>, mut emit: impl FnMut(Notice)) -> Exit {
        let reason = loop {
            let command = if let Some(deadline) = self.deadlines.first() {
                // Команда может разбудить цикл раньше ближайшего таймера.
                // Instant отсчитывает срок по монотонным часам.
                let wait = deadline.at.saturating_duration_since(Instant::now());
                match commands.recv_timeout(wait) {
                    Ok(command) => Some(command),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => break StopReason::InputClosed,
                }
            } else {
                match commands.recv() {
                    Ok(command) => Some(command),
                    Err(_) => break StopReason::InputClosed,
                }
            };

            // Срок мог наступить, пока цикл получал команду. Проверяем
            // таймеры и после команды, и после пробуждения по таймауту.
            let now = Instant::now();
            while let Some(deadline) = self.deadlines.first().copied()
                && deadline.at <= now
            {
                emit(Notice::Expired(self.release(deadline.locker)));
            }

            let notice = match command {
                Some(Command::Reserve { booking, duration }) => {
                    self.reserve(booking, duration, now)
                }
                Some(Command::Scan(booking)) => self.scan(booking),
                Some(Command::Stop) => break StopReason::Requested,
                None => continue,
            };
            emit(notice);
        };

        // run владеет терминалом. При выходе его таблица и таймеры
        // уничтожаются вместе с оставшимися бронями.
        Exit {
            reason,
            released_on_stop: self.active.len(),
        }
    }

    fn reserve(&mut self, booking: Booking, duration: Duration, now: Instant) -> Notice {
        if self.active.contains_key(&booking.locker) {
            return Notice::Occupied(booking);
        }
        let Some(at) = now.checked_add(duration) else {
            return Notice::InvalidDuration(booking);
        };

        let deadline = Deadline {
            at,
            locker: booking.locker,
        };
        self.active.insert(
            booking.locker,
            ActiveBooking {
                order: booking.order,
                deadline,
            },
        );
        self.deadlines.insert(deadline);
        Notice::Reserved(booking)
    }

    fn scan(&mut self, booking: Booking) -> Notice {
        match self.active.get(&booking.locker) {
            Some(active) if active.order == booking.order => {
                Notice::Collected(self.release(booking.locker))
            }
            _ => Notice::ScanRejected(booking),
        }
    }

    fn release(&mut self, locker: u8) -> Booking {
        let active = self
            .active
            .remove(&locker)
            .expect("Ячейка должна быть занята");
        // Завершение выдачи удаляет и бронь, и её таймер. Эту же ячейку
        // можно сразу забронировать заново с другим сроком.
        let removed = self.deadlines.remove(&active.deadline);
        assert!(removed, "У каждой активной брони должен быть таймер");
        Booking {
            locker,
            order: active.order,
        }
    }
}

fn send_and_expect(
    scanner: &Sender<Command>,
    notices: &Receiver<Notice>,
    command: Command,
    expected: Notice,
) {
    scanner
        .send(command)
        .expect("Терминал должен принимать команды");
    let received = notices
        .recv_timeout(Duration::from_secs(5))
        .expect("Терминал должен прислать уведомление");
    assert_eq!(received, expected);
}

fn run_scanner_session() {
    let (scanner, commands) = channel();
    let (notifications, notices) = channel();
    let scanner_session = thread::spawn(move || {
        let initial = Booking {
            locker: 4,
            order: 731,
        };
        let next = Booking {
            locker: 4,
            order: 732,
        };
        let long = Booking {
            locker: 8,
            order: 733,
        };
        let normal_duration = Duration::from_secs(120);

        send_and_expect(
            &scanner,
            &notices,
            Command::Reserve {
                booking: initial,
                duration: normal_duration,
            },
            Notice::Reserved(initial),
        );
        send_and_expect(
            &scanner,
            &notices,
            Command::Reserve {
                booking: next,
                duration: normal_duration,
            },
            Notice::Occupied(next),
        );
        send_and_expect(
            &scanner,
            &notices,
            Command::Scan(next),
            Notice::ScanRejected(next),
        );
        send_and_expect(
            &scanner,
            &notices,
            Command::Scan(initial),
            Notice::Collected(initial),
        );
        println!(
            "Заказ {} выдан, таймер ячейки {} снят.",
            initial.order, initial.locker
        );

        send_and_expect(
            &scanner,
            &notices,
            Command::Reserve {
                booking: long,
                duration: normal_duration,
            },
            Notice::Reserved(long),
        );
        send_and_expect(
            &scanner,
            &notices,
            Command::Reserve {
                booking: next,
                duration: Duration::from_millis(30),
            },
            Notice::Reserved(next),
        );
        // Сканер пока не отправляет команд. Цикл сам проснётся в срок
        // короткой брони, хотя в очереди есть и более поздний таймер.
        let expired = notices
            .recv_timeout(Duration::from_secs(5))
            .expect("Терминал должен сообщить об истечении срока");
        assert_eq!(expired, Notice::Expired(next));
        println!(
            "Срок брони заказа {} истёк, ячейка {} свободна.",
            next.order, next.locker
        );

        scanner
            .send(Command::Stop)
            .expect("Терминал должен принять остановку");
    });

    let exit = Terminal::default().run(commands, |notice| {
        notifications
            .send(notice)
            .expect("Сканер должен принимать уведомления");
    });
    scanner_session
        .join()
        .expect("Сценарий сканера должен завершиться");
    assert_eq!(
        exit,
        Exit {
            reason: StopReason::Requested,
            released_on_stop: 1
        }
    );
    println!(
        "Терминал остановлен. Снято броней — {}.",
        exit.released_on_stop
    );
}

fn check_timer_before_queued_scan() {
    let (scanner, commands) = channel();
    let booking = Booking {
        locker: 1,
        order: 901,
    };
    let remaining = Booking {
        locker: 2,
        order: 902,
    };
    // Команды стоят в очереди до старта цикла. Нулевой срок позволяет
    // проверить порядок обработки независимо от скорости запуска.
    scanner
        .send(Command::Reserve {
            booking,
            duration: Duration::ZERO,
        })
        .expect("Канал открыт");
    scanner.send(Command::Scan(booking)).expect("Канал открыт");
    scanner
        .send(Command::Reserve {
            booking: remaining,
            duration: Duration::from_secs(120),
        })
        .expect("Канал открыт");
    // Цикл обработает накопленные команды, обнаружит закрытие канала
    // и завершится с активной бронью, которую снимет при выходе.
    drop(scanner);

    let mut notices = Vec::new();
    let exit = Terminal::default().run(commands, |notice| notices.push(notice));
    assert_eq!(
        notices,
        [
            Notice::Reserved(booking),
            Notice::Expired(booking),
            Notice::ScanRejected(booking),
            Notice::Reserved(remaining),
        ]
    );
    println!(
        "Срок и сканирование — истёкшая бронь обработана до уже поставленной в очередь команды."
    );
    assert_eq!(
        exit,
        Exit {
            reason: StopReason::InputClosed,
            released_on_stop: 1
        }
    );
    println!(
        "Канал сканера закрыт. Снято броней — {}.",
        exit.released_on_stop
    );
}

fn check_simultaneous_deadlines() {
    let mut terminal = Terminal::default();
    let now = Instant::now();
    let first = Booking {
        locker: 2,
        order: 904,
    };
    let second = Booking {
        locker: 7,
        order: 905,
    };
    assert_eq!(
        terminal.reserve(second, Duration::ZERO, now),
        Notice::Reserved(second)
    );
    assert_eq!(
        terminal.reserve(first, Duration::ZERO, now),
        Notice::Reserved(first)
    );

    let (scanner, commands) = channel();
    scanner.send(Command::Stop).expect("Канал открыт");
    let mut notices = Vec::new();
    let exit = terminal.run(commands, |notice| notices.push(notice));
    assert_eq!(notices, [Notice::Expired(first), Notice::Expired(second)]);
    assert_eq!(
        exit,
        Exit {
            reason: StopReason::Requested,
            released_on_stop: 0
        }
    );
    println!(
        "Одинаковый срок — освобождены ячейки {} и {} в порядке их номеров.",
        first.locker, second.locker
    );
}

fn main() {
    run_scanner_session();
    check_timer_before_queued_scan();
    check_simultaneous_deadlines();
}
