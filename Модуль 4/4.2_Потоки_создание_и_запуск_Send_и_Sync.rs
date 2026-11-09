/*
cargo run --offline --bin threads_send_sync
cargo run --offline --bin threads_send_sync -- panic-demo
*/

// Второй запуск показывает ожидаемую панику дочернего потока.

/*
Два дока проверяют партии сканов. Сканы с нулём коробов и сканы сверх
лимита отклоняются. Главный поток собирает сводку после завершения обоих
исполнителей. Если буферы созданы в главном потоке, исполнители могут
обработать их по отдельности и оставить отметки для последующего отчёта.
*/

use std::{io, thread};

const FIRST_DOCK: [u16; 3] = [6, 0, 20];
const SECOND_DOCK: [u16; 3] = [21, 5, 3];
const MAX_CARTONS: u16 = 20;

struct Rules {
    max_cartons: u16,
}

struct Scan {
    cartons: u16,
    accepted: bool,
}

impl Scan {
    fn new(cartons: u16) -> Self {
        Self {
            cartons,
            accepted: false,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct Summary {
    accepted: usize,
    rejected: usize,
    cartons: u128,
}

fn inspect(scans: &mut [Scan], rules: &Rules) -> Summary {
    let mut summary = Summary::default();
    for scan in scans {
        scan.accepted = scan.cartons > 0 && scan.cartons <= rules.max_cartons;
        if scan.accepted {
            summary.accepted += 1;
            summary.cartons += u128::from(scan.cartons);
        } else {
            summary.rejected += 1;
        }
    }
    summary
}

fn inspect_independently() -> (Summary, Summary) {
    // spawn запускает системный поток и возвращает JoinHandle.
    // Задания создают свои данные внутри потока и возвращают сводку во владение main.
    let first = thread::spawn(|| {
        inspect(
            &mut FIRST_DOCK.map(Scan::new),
            &Rules {
                max_cartons: MAX_CARTONS,
            },
        )
    });
    let second = thread::spawn(|| {
        inspect(
            &mut SECOND_DOCK.map(Scan::new),
            &Rules {
                max_cartons: MAX_CARTONS,
            },
        )
    });

    let main_id = thread::current().id();
    let first_id = first.thread().id();
    let second_id = second.thread().id();
    let separate = first_id != second_id && first_id != main_id && second_id != main_id;
    assert!(separate);
    println!("Потоки доков — два отдельных исполнителя {separate}");

    // Оба потока уже запущены. Каждый join ждёт свой поток, а сводки читаются по порядку доков.
    let first = first.join();
    let second = second.join();
    (
        first.expect("Сбой потока первого дока"),
        second.expect("Сбой потока второго дока"),
    )
}

fn named_worker_demo() -> io::Result<()> {
    let worker = thread::Builder::new()
        .name(String::from("контроль-дока"))
        .spawn(|| {
            let current = thread::current();
            (current.id(), current.name().map(str::to_owned))
        })?;
    let expected_id = worker.thread().id();
    let (actual_id, name) = worker.join().expect("Сбой именованного исполнителя");
    let same_id = expected_id == actual_id;
    assert!(same_id);
    assert_eq!(name.as_deref(), Some("контроль-дока"));
    println!(
        "Именованный поток — имя «{}», идентификатор совпал {same_id}",
        name.expect("Имя задано при запуске")
    );
    Ok(())
}

fn ordered_stages_demo() {
    let stages: [fn() -> &'static str; 3] = [|| "приём", || "проверка", || "отчёт"];
    // Новый этап запускается только после завершения предыдущего.
    let completed = stages.map(|stage| thread::spawn(stage).join().expect("Сбой этапа"));
    assert_eq!(completed, ["приём", "проверка", "отчёт"]);
    println!("Порядок этапов — {}", completed.join(" → "));
}

fn inspect_in_scope(first: &mut [Scan], second: &mut [Scan], rules: &Rules) -> (Summary, Summary) {
    // scope гарантирует завершение запущенных в нём потоков до выхода из области.
    thread::scope(|scope| {
        // Изменяемые буферы разные. Оба потока читают один rules через общие ссылки.
        let left = scope.spawn(|| inspect(first, rules));
        let right = scope.spawn(|| inspect(second, rules));
        let left = left.join();
        let right = right.join();
        (
            left.expect("Сбой проверки первого буфера"),
            right.expect("Сбой проверки второго буфера"),
        )
    })
}

fn inspect_split(scans: &mut [Scan], boundary: usize, rules: &Rules) {
    let (first, second) = scans.split_at_mut(boundary);
    thread::scope(|scope| {
        // Результат каждого задания остаётся в его части буфера. Область дождётся обоих.
        scope.spawn(|| {
            inspect(first, rules);
        });
        scope.spawn(|| {
            inspect(second, rules);
        });
    });
}

fn split_demo(rules: &Rules) {
    let mut scans = [6, 0, 20, 21, 5, 3].map(Scan::new);
    inspect_split(&mut scans, 3, rules);
    let flags = scans.map(|scan| scan.accepted);
    assert_eq!(flags, [true, false, true, false, true, true]);
    let mut single = [Scan::new(6)];
    inspect_split(&mut single, 0, rules);
    assert!(single[0].accepted);
    inspect_split(&mut [], 0, rules);
    println!("Разделённый буфер — отметки после автоматического ожидания {flags:?}");
}

// Тела пустые. Проверку выполняет компилятор по ограничениям типов.
fn requires_send<T: Send>() {}
fn requires_sync<T: Sync>() {}

fn check_contracts() {
    // Send позволяет передать значение другому потоку. Sync означает, что &T можно передать потоку.
    requires_send::<Summary>();
    requires_send::<&mut [Scan]>();
    requires_sync::<Rules>();
    requires_send::<&Rules>();
    println!(
        "Проверка типов — Summary и изменяемый слайс допускают передачу, Rules допускает общий доступ"
    );

    // requires_send::<std::rc::Rc<Summary>>();
    // requires_sync::<std::rc::Rc<Rules>>();
}

fn panic_demo() -> io::Result<()> {
    // Builder позволяет задать имя и получить ошибку создания потока как Result.
    let worker = thread::Builder::new()
        .name(String::from("проверка-join"))
        .spawn(|| {
            panic!("учебный сбой исполнителя");
        })?;
    let failure = worker
        .join()
        .expect_err("Исполнитель должен завершиться паникой");
    let message = failure
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| failure.downcast_ref::<String>().map(String::as_str));
    assert_eq!(message, Some("учебный сбой исполнителя"));
    println!(
        "join вернул Err — «{}», главный поток продолжил работу",
        message.expect("Паника содержит текст")
    );
    Ok(())
}

fn main() -> io::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("panic-demo") {
        return panic_demo();
    }

    let independent = inspect_independently();
    assert_eq!(
        independent.0,
        Summary {
            accepted: 2,
            rejected: 1,
            cartons: 26
        }
    );
    println!(
        "Сводки доков — первый {} коробов, второй {} коробов",
        independent.0.cartons, independent.1.cartons
    );
    named_worker_demo()?;
    ordered_stages_demo();
    assert_eq!(
        independent.1,
        Summary {
            accepted: 2,
            rejected: 1,
            cartons: 8
        }
    );

    let rules = Rules {
        max_cartons: MAX_CARTONS,
    };
    let mut first = FIRST_DOCK.map(Scan::new);
    let mut second = SECOND_DOCK.map(Scan::new);

    // Обычный spawn требует 'static для задания. Локальные ссылки подходят для scope.
    // let _worker = thread::spawn(|| inspect(&mut first, &rules));

    let scoped = inspect_in_scope(&mut first, &mut second, &rules);
    assert_eq!(scoped, independent);
    // После выхода из scope оба буфера снова доступны главному потоку.
    let first_flags = first.map(|scan| scan.accepted);
    let second_flags = second.map(|scan| scan.accepted);
    assert_eq!(first_flags, [true, false, true]);
    assert_eq!(second_flags, [false, true, true]);
    println!("Область потоков — отметки первого дока {first_flags:?}, второго {second_flags:?}");
    split_demo(&rules);
    check_contracts();

    let (first, second) = scoped;
    println!(
        "Два дока — принято {}, отклонено {}, коробов {}",
        first.accepted + second.accepted,
        first.rejected + second.rejected,
        first.cartons + second.cartons
    );
    println!("Проверка в scope оставила отметки в обоих буферах");
    Ok(())
}
