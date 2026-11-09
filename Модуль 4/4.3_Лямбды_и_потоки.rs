/*
cargo run --offline --bin closures_and_threads
*/

/*
Терминал получает партии этикеток от сканеров. В каждой строке код K-
с тремя цифрами. Пробелы по краям удаляются, латинская k приводится
к верхнему регистру. Партия отклоняется при первой неверной строке.
Партии независимы, поэтому их можно проверять отдельными потоками.
Отчёт должен сохранить порядок подачи партий и указать причину отказа.
*/

use std::{io, rc::Rc, thread};

struct Batch {
    number: u32,
    rows: Vec<String>,
}

impl Batch {
    fn new(number: u32, rows: &[&str]) -> Self {
        Self {
            number,
            rows: rows.iter().map(|&row| row.to_owned()).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ImportError {
    InvalidLabel { line: usize, text: String },
    Launch(io::ErrorKind),
    Panicked,
}

type ImportResult = Result<Vec<String>, ImportError>;

#[derive(Debug, PartialEq, Eq)]
struct Report {
    number: u32,
    result: ImportResult,
}

fn prepare(batch: Batch) -> impl FnOnce() -> ImportResult + Send + 'static {
    // Задание забирает строки во владение, потому что into_iter расходует Vec.
    || {
        let mut labels = Vec::with_capacity(batch.rows.len());
        for (index, text) in batch.rows.into_iter().enumerate() {
            let label = text.trim().to_ascii_uppercase();
            let valid = label.strip_prefix("K-").is_some_and(|digits| {
                digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit())
            });
            if !valid {
                return Err(ImportError::InvalidLabel {
                    line: index + 1,
                    text,
                });
            }
            labels.push(label);
        }
        Ok(labels)
    }
}

fn import_in_parallel(batches: Vec<Batch>) -> Vec<Report> {
    let mut pending = Vec::with_capacity(batches.len());
    for batch in batches {
        let number = batch.number;
        let task = prepare(batch);
        // Создание замыкания только подготавливает данные. spawn запускает его в потоке.
        let worker = thread::Builder::new()
            .name(format!("партия-{number}"))
            .spawn(task);
        // let _ = task();
        pending.push((number, worker));
    }

    let mut reports = Vec::with_capacity(pending.len());
    for (number, worker) in pending {
        // Даже после отказа читаем остальные дескрипторы. Порядок строк задаёт pending.
        let result = match worker {
            Ok(handle) => match handle.join() {
                Ok(import) => import,
                Err(_) => Err(ImportError::Panicked),
            },
            Err(error) => Err(ImportError::Launch(error.kind())),
        };
        reports.push(Report { number, result });
    }
    reports
}

fn copy_capture_demo() {
    let mut limit = 2_usize;
    let task = move || limit;
    limit = 4;
    let worker_limit = thread::spawn(task).join().expect("Сбой проверки лимита");
    assert_eq!((worker_limit, limit), (2, 4));
    println!("Снимок лимита — у задания {worker_limit}, в главном потоке {limit}");
}

fn field_capture_demo() {
    struct ScannerInput {
        batch: Batch,
        preview: Rc<String>,
    }

    let input = ScannerInput {
        batch: Batch::new(9, &[" k-009 "]),
        preview: Rc::new(String::from("Сканер-1")),
    };
    let task = move || {
        let import = prepare(input.batch);
        import()
    };
    // Задание забирает только batch. Локальный предпросмотр остаётся здесь.
    let labels = thread::spawn(task)
        .join()
        .expect("Сбой проверки этикеток")
        .expect("В примере этикетка корректна");
    assert_eq!(labels, ["K-009"]);
    println!(
        "Захват поля — этикетки {labels:?}, предпросмотр «{}»",
        input.preview
    );
}

fn ordered_output_demo() {
    let count = 3;
    let first = thread::spawn(move || vec!["foo"; count]);
    let second = thread::spawn(move || vec!["bar"; count]);
    let first = first.join();
    let second = second.join();
    let output: String = first
        .expect("Сбой задания foo")
        .into_iter()
        .zip(second.expect("Сбой задания bar"))
        .flat_map(|(foo, bar)| [foo, bar])
        .collect();
    assert_eq!(output, "foobarfoobarfoobar");
    println!("Сборка вывода — {output}");
}

fn main() {
    // Подготовленное задание можно проверить напрямую, а затем создать новое для потока.
    let local = prepare(Batch::new(1, &[" k-005 ", "K-108"]));
    let labels = local().expect("Учебные этикетки корректны");
    assert_eq!(labels, ["K-005", "K-108"]);
    println!("Прямой вызов задания — {labels:?}");
    copy_capture_demo();
    field_capture_demo();
    let invalid = prepare(Batch::new(2, &["К-001"]));
    assert_eq!(
        invalid(),
        Err(ImportError::InvalidLabel {
            line: 1,
            text: String::from("К-001")
        })
    );
    let too_long = prepare(Batch::new(3, &["K-1234"]));
    assert_eq!(
        too_long(),
        Err(ImportError::InvalidLabel {
            line: 1,
            text: String::from("K-1234")
        })
    );

    assert!(import_in_parallel(vec![]).is_empty());
    let reports = import_in_parallel(vec![
        Batch::new(17, &[" k-101 ", "K-102"]),
        Batch::new(5, &["K-105", "K-1X0", "K-106"]),
        Batch::new(41, &["k-210"]),
        Batch::new(11, &[]),
    ]);
    for report in &reports {
        let number = report.number;
        match &report.result {
            Ok(labels) => println!("Партия {number} — приняты этикетки {labels:?}"),
            Err(ImportError::InvalidLabel { line, text }) => {
                println!("Партия {number} — отклонена строка {line} с кодом «{text}»");
            }
            Err(ImportError::Launch(kind)) => {
                println!("Партия {number} — запуск потока завершился ошибкой {kind:?}")
            }
            Err(ImportError::Panicked) => println!("Партия {number} — сбой исполнителя"),
        }
    }
    assert_eq!(
        reports,
        [
            Report {
                number: 17,
                result: Ok(vec![String::from("K-101"), String::from("K-102")])
            },
            Report {
                number: 5,
                result: Err(ImportError::InvalidLabel {
                    line: 2,
                    text: String::from("K-1X0")
                })
            },
            Report {
                number: 41,
                result: Ok(vec![String::from("K-210")])
            },
            Report {
                number: 11,
                result: Ok(vec![])
            },
        ]
    );
    ordered_output_demo();
}
