/*
cargo run --offline --bin c10k
cargo run --offline --bin c10k -- --tcp
*/

/*
Сервис учёта остатков поддерживает постоянные соединения с 10 000 терминалов.
В одни моменты запросы есть у сотни клиентов, в другие — у всех.
Сравним работу диспетчера при полном обходе и при получении списка
готовых соединений. В обоих случаях обработчик и нагрузка одинаковы.

Соединения и время моделируются в памяти. Ответы представлены числом байт.
Считаем проверки готовности и обработанные запросы. Список готовых
соединений подаёт сценарий, как если бы его выдал системный механизм
уведомлений. Стоимость работы ОС и пропускная способность сети
выходят за пределы этих счётчиков.

С аргументом --tcp отдельно создаётся одна пара локальных TCP-сокетов.
Первое чтение неблокирующее и сразу возвращает WouldBlock. После записи
чтение переключается в блокирующий режим с ограничением времени.
*/

use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::mem::size_of;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

const CONNECTIONS: usize = 10_000;
const RESPONSE_BYTES: usize = 512;

type ConnectionId = usize;

#[derive(Clone, Copy)]
struct Limits {
    open_connections: usize,
    output_per_connection: usize,
    output_total: usize,
    idle_ticks: u64,
}

#[derive(Clone, PartialEq, Eq)]
struct Connection {
    pending_bytes: usize,
    last_progress: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum Admission {
    Accepted,
    ConnectionLimit,
    ServerLimit,
    Closed,
}

#[derive(Clone)]
struct Server {
    limits: Limits,
    connections: HashMap<ConnectionId, Connection>,
    queued_bytes: usize,
    next_id: ConnectionId,
}

impl Server {
    fn new(limits: Limits) -> Self {
        Self {
            limits,
            connections: HashMap::new(),
            queued_bytes: 0,
            next_id: 0,
        }
    }

    fn open(&mut self, now: u64) -> Option<ConnectionId> {
        // В реальном сервере каждому клиенту нужен сокет. При выборе
        // лимита учитывают также сокет слушателя и другие ресурсы процесса.
        if self.connections.len() >= self.limits.open_connections {
            return None;
        }
        let id = self.next_id;
        self.next_id = id.checked_add(1)?;
        self.connections.insert(
            id,
            Connection {
                pending_bytes: 0,
                last_progress: now,
            },
        );
        Some(id)
    }

    fn queue_response(&mut self, id: ConnectionId, bytes: usize, now: u64) -> Admission {
        let Some(connection) = self.connections.get_mut(&id) else {
            return Admission::Closed;
        };

        // Сначала проверяем бюджеты, затем принимаем ответ в очередь.
        // Сценарий может повторить попытку, когда часть очереди отправлена.
        if bytes > self.limits.output_per_connection - connection.pending_bytes {
            return Admission::ConnectionLimit;
        }
        if bytes > self.limits.output_total - self.queued_bytes {
            return Admission::ServerLimit;
        }
        connection.pending_bytes += bytes;
        connection.last_progress = now;
        self.queued_bytes += bytes;
        Admission::Accepted
    }

    fn write_ready(&mut self, id: ConnectionId, writable_bytes: usize, now: u64) -> Option<usize> {
        let connection = self.connections.get_mut(&id)?;
        // Доступная запись может принять только часть ответа. Остаток
        // остаётся в очереди до следующей возможности записи.
        let written = connection.pending_bytes.min(writable_bytes);
        connection.pending_bytes -= written;
        self.queued_bytes -= written;
        if written != 0 {
            connection.last_progress = now;
        }
        Some(written)
    }

    fn expire_idle(&mut self, now: u64) -> usize {
        // Отдельный проход очистки. Его работа не входит в сравнение
        // способов доставки уведомлений о готовности.
        let before = self.connections.len();
        let mut discarded_bytes = 0;
        self.connections.retain(|_, connection| {
            let idle = now.saturating_sub(connection.last_progress);
            if idle >= self.limits.idle_ticks {
                discarded_bytes += connection.pending_bytes;
                return false;
            }
            true
        });
        self.queued_bytes -= discarded_bytes;
        before - self.connections.len()
    }
}

#[derive(Default)]
struct DispatchStats {
    inspected: usize,
    answered: usize,
    written: usize,
}

fn dispatch(
    server: &mut Server,
    candidates: impl Iterator<Item = Option<ConnectionId>>,
    now: u64,
) -> DispatchStats {
    let mut stats = DispatchStats::default();
    for candidate in candidates {
        stats.inspected += 1;
        let Some(id) = candidate else {
            continue;
        };
        assert_eq!(
            server.queue_response(id, RESPONSE_BYTES, now),
            Admission::Accepted
        );
        let written = server
            .write_ready(id, RESPONSE_BYTES, now)
            .expect("Соединение открыто");
        assert_eq!(written, RESPONSE_BYTES);
        stats.answered += 1;
        stats.written += written;
    }
    stats
}

fn compare_dispatch(ready_count: usize) {
    let mut scanned = Server::new(Limits {
        open_connections: CONNECTIONS,
        output_per_connection: 8 * 1024,
        output_total: 8 * 1024 * 1024,
        idle_ticks: 30,
    });
    for expected_id in 0..CONNECTIONS {
        assert_eq!(scanned.open(0), Some(expected_id));
    }
    let mut notified = scanned.clone();

    // Размещаем активных клиентов по всей таблице соединений. Один
    // источник нагрузки даёт и флаги для обхода, и список для уведомлений.
    let ready_ids: Vec<_> = (0..ready_count)
        .map(|index| index * CONNECTIONS / ready_count)
        .collect();
    let mut readable = vec![false; CONNECTIONS];
    for &id in &ready_ids {
        readable[id] = true;
    }

    let all_candidates = readable
        .into_iter()
        .enumerate()
        .map(|(id, is_ready)| is_ready.then_some(id));
    let ready_candidates = ready_ids.into_iter().map(Some);

    let all = dispatch(&mut scanned, all_candidates, 1);
    let ready = dispatch(&mut notified, ready_candidates, 1);
    assert_eq!(all.inspected, CONNECTIONS);
    assert_eq!(ready.inspected, ready_count);
    assert_eq!(all.answered, ready_count);
    assert_eq!(ready.answered, ready_count);
    assert_eq!(all.written, ready.written);
    assert_eq!((scanned.queued_bytes, notified.queued_bytes), (0, 0));
    assert!(scanned.connections == notified.connections);
    println!(
        "Готовых соединений — {}. Проверок при обходе — {}, по уведомлениям — {}. Ответов в каждом случае — {}.",
        ready_count, all.inspected, ready.inspected, ready.answered
    );
}

fn check_slow_client() {
    let mut server = Server::new(Limits {
        open_connections: 3,
        output_per_connection: 1_024,
        output_total: 1_536,
        idle_ticks: 5,
    });
    let slow = server.open(0).expect("Есть место для медленного клиента");
    let fast = server.open(0).expect("Есть место для быстрого клиента");
    let idle = server
        .open(0)
        .expect("Есть место для простаивающего клиента");
    assert_eq!(server.open(0), None);
    println!("Лимит соединений: четвёртое соединение отклонено при максимуме 3.");

    assert_eq!(server.queue_response(slow, 1_024, 0), Admission::Accepted);
    assert_eq!(
        server.queue_response(slow, 1, 1),
        Admission::ConnectionLimit
    );
    assert_eq!(server.queue_response(fast, 512, 1), Admission::Accepted);
    assert_eq!(server.queue_response(fast, 1, 1), Admission::ServerLimit);
    assert_eq!(server.queued_bytes, 1_536);
    println!(
        "Обратное давление: лимит клиента отклонил 1 байт при 1024 в очереди. Общий лимит отклонил 1 байт при 1536 в очереди."
    );

    // Быстрый клиент получает ответ, пока медленный освободил только часть
    // своей очереди. После записи освобождается и общий бюджет.
    assert_eq!(server.write_ready(slow, 256, 2), Some(256));
    println!("Частичная запись: отправлено 256 байт, у медленного клиента осталось 768.");
    assert_eq!(server.write_ready(fast, 512, 3), Some(512));
    assert_eq!(server.write_ready(slow, 0, 4), Some(0));
    assert_eq!(server.queue_response(fast, 1, 4), Admission::Accepted);
    println!("Повтор после освобождения очереди: ответ в 1 байт принят.");
    assert_eq!(server.write_ready(fast, 1, 5), Some(1));
    assert_eq!(server.queue_response(fast, 512, 6), Admission::Accepted);
    assert_eq!(server.queued_bytes, 768 + 512);

    // Медленный клиент с неотправленным остатком и молчащий клиент
    // закрываются по таймауту. Освобождаются места и 768 байт очереди.
    assert_eq!(server.expire_idle(7), 2);
    assert!(!server.connections.contains_key(&idle));
    assert_eq!(server.queue_response(slow, 1, 7), Admission::Closed);
    assert_eq!(server.queued_bytes, 512);
    assert_eq!(server.write_ready(fast, 512, 7), Some(512));
    assert_eq!(server.queued_bytes, 0);
    assert!(server.open(7).is_some());
    assert!(server.open(7).is_some());
    assert_eq!(server.open(7), None);
    println!("Таймаут освободил два места и удалил 768 байт из очереди медленного клиента.");
}

fn show_local_tcp() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let mut sender = TcpStream::connect(listener.local_addr()?)?;
    let (mut receiver, _) = listener.accept()?;

    receiver.set_nonblocking(true)?;
    let mut bytes = [0; 4];
    let error = receiver
        .read(&mut bytes)
        .expect_err("Пустой сокет не готов к чтению");
    assert_eq!(error.kind(), ErrorKind::WouldBlock);
    println!("TCP: чтение пустого неблокирующего сокета сразу вернуло WouldBlock.");

    // Отправляем данные перед блокирующим чтением с таймаутом.
    receiver.set_nonblocking(false)?;
    receiver.set_read_timeout(Some(Duration::from_secs(1)))?;
    sender.write_all(b"ping")?;
    receiver.read_exact(&mut bytes)?;
    assert_eq!(&bytes, b"ping");
    println!("TCP: после записи получены байты ping.");

    sender.shutdown(Shutdown::Write)?;
    assert_eq!(receiver.read(&mut bytes)?, 0);
    println!("TCP: после shutdown(Write) чтение вернуло 0 байт (EOF).");
    Ok(())
}

fn show_memory_budget() {
    const MIB: u64 = 1024 * 1024;
    // Условный резерв адресного пространства под стек одного потока.
    // Потребление физической памяти зависит от ОС и использованных страниц.
    let stack_per_thread = 2 * MIB;
    let thread_stack_reserve = CONNECTIONS as u64 * stack_per_thread;
    let state_bytes = CONNECTIONS * size_of::<Connection>();
    println!(
        "Поток на соединение при резерве 2 МиБ на стек потребовал бы {} МиБ адресного пространства.",
        thread_stack_reserve / MIB
    );
    // Отдельно учитывают таблицу соединений, пользовательские буферы,
    // буферы ядра и прочее состояние протокола.
    println!(
        "Поля состояния {} соединений занимают {} байт.",
        CONNECTIONS, state_bytes
    );
}

fn main() {
    compare_dispatch(0);
    compare_dispatch(100);
    compare_dispatch(CONNECTIONS);
    show_memory_budget();
    check_slow_client();
    if std::env::args().any(|arg| arg == "--tcp") {
        show_local_tcp().expect("Локальный TCP-пример должен завершиться успешно");
    }
}
