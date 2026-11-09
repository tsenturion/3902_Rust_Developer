// Блоки запускаются в PowerShell из корня проекта. --locked требует готовый Cargo.lock.
/*
cargo build --locked --offline --bin build_project
.\target\debug\build_project.exe 12000 20000 24000
*/

/*
cargo build --locked --offline --release --bin build_project
.\target\release\build_project.exe 12000 20000 24000
*/

/*
cargo build --locked --offline --profile checked-release --bin build_project --features build-report
.\target\checked-release\build_project.exe 12000 20000 24000
*/

/*
Терминал проверяет вес коробов перед отправкой. Вес каждого короба задаётся
в граммах отдельным аргументом. Короба тяжелее 20 кг откладываются.
Обычной сборке нужна краткая сводка, сборке с build-report — также номера
и вес отложенных коробов. Правила допуска одинаковы во всех сборках.
*/

use std::process::ExitCode;

const MAX_WEIGHT_GRAMS: u32 = 20_000;

enum InputError {
    MissingWeights,
    InvalidWeight { position: usize, text: String },
}

impl InputError {
    fn message(&self) -> String {
        match self {
            Self::MissingWeights => String::from("Передайте вес хотя бы одного короба в граммах"),
            Self::InvalidWeight { position, text } => {
                format!(
                    "Вес короба {position} должен быть положительным целым числом в граммах. Получено «{text}»"
                )
            }
        }
    }
}

fn read_weights(args: impl Iterator<Item = String>) -> Result<Vec<u32>, InputError> {
    let mut weights = Vec::new();
    for (index, text) in args.enumerate() {
        // Проверки пользовательского ввода нужны при любом профиле сборки.
        match text.parse::<u32>() {
            Ok(weight) if weight > 0 => weights.push(weight),
            _ => {
                return Err(InputError::InvalidWeight {
                    position: index + 1,
                    text,
                });
            }
        }
    }
    if weights.is_empty() {
        return Err(InputError::MissingWeights);
    }
    Ok(weights)
}

struct Summary {
    accepted: usize,
    postponed: usize,
    accepted_grams: u128,
}

impl Summary {
    fn check_count(&self, expected: usize) {
        debug_assert_eq!(
            self.accepted + self.postponed,
            expected,
            "Нарушен внутренний баланс коробов"
        );
    }
}

fn summarize(weights: &[u32]) -> Summary {
    let mut summary = Summary {
        accepted: 0,
        postponed: 0,
        accepted_grams: 0,
    };
    for &weight in weights {
        if weight <= MAX_WEIGHT_GRAMS {
            summary.accepted += 1;
            summary.accepted_grams += u128::from(weight);
        } else {
            summary.postponed += 1;
        }
    }
    // Это проверка внутреннего результата. Её включение задаёт профиль Cargo.
    summary.check_count(weights.len());
    summary
}

// Cargo включает эту функцию в сборку только при выборе build-report.
#[cfg(feature = "build-report")]
fn print_details(weights: &[u32]) {
    for (index, &weight) in weights.iter().enumerate() {
        if weight > MAX_WEIGHT_GRAMS {
            println!("Короб {} отложен. Вес в граммах — {weight}", index + 1);
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_some_and(|arg| arg == "--invariant-demo") {
        // Отдельная проба специально задаёт неверное ожидаемое число коробов.
        summarize(&[12_000]).check_count(2);
        println!("Проба инварианта — выполнение продолжилось");
        return ExitCode::SUCCESS;
    }

    let weights = match read_weights(args) {
        Ok(weights) => weights,
        Err(error) => {
            eprintln!("{}", error.message());
            return ExitCode::FAILURE;
        }
    };
    let summary = summarize(&weights);

    // cfg! даёт булево значение при компиляции. Атрибут #[cfg] включает сам код.
    let checks = if cfg!(debug_assertions) {
        "включены"
    } else {
        "выключены"
    };
    let details = if cfg!(feature = "build-report") {
        "включены"
    } else {
        "выключены"
    };
    println!(
        "Цель {} — проверки debug_assert {checks}, подробности {details}",
        env!("CARGO_BIN_NAME")
    );
    println!(
        "Принято {}, отложено {}, общий вес принятых в граммах {}",
        summary.accepted, summary.postponed, summary.accepted_grams
    );

    #[cfg(feature = "build-report")]
    print_details(&weights);

    ExitCode::SUCCESS
}
