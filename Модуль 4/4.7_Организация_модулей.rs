/*
cargo run --offline --bin module_organization -- 3:20 1:5 3:15
cargo test --offline --bin module_organization
*/

/*
Диспетчер передаёт записи простоя вида «док:минуты». Отчёт суммирует
простои по каждому доку и выводит их по порядку номеров. При неверной
строке расчёт отклоняется целиком, а терминал сообщает её позицию.

Дерево модулей отчёта размещено в каталоге 4.7_report.
Модуль cli остаётся в этом файле рядом с точкой входа.
*/

use std::process::ExitCode;

const MAX_DOCK: u8 = 12;

#[path = "4.7_report/mod.rs"]
mod report;

mod cli {
    use crate::report::{self, ReportError, Summary};

    fn render(summary: &Summary) -> String {
        let mut output = format!(
            "Простоев {}, всего минут {}",
            summary.events, summary.total_minutes
        );
        for (dock, minutes) in &summary.by_dock {
            output.push_str(&format!("\nДок {dock} — {minutes} мин."));
        }
        output
    }

    pub(crate) fn run(rows: &[String]) -> Result<String, ReportError> {
        Ok(render(&report::build(rows)?))
    }

    pub(crate) fn message(error: ReportError) -> String {
        match error {
            ReportError::EmptyInput => String::from("Передайте хотя бы одну запись простоя"),
            ReportError::InvalidRecord { position, text } => {
                format!(
                    "Запись {position} «{text}» недопустима. Укажите док от 1 до {} и положительное число минут",
                    crate::MAX_DOCK
                )
            }
        }
    }
}

fn main() -> ExitCode {
    let rows: Vec<_> = std::env::args().skip(1).collect();

    // Для запуска используется вход cli::run. Эти два пути закрыты.
    // let _ = report::input::parse(&rows);
    // let _ = cli::render;

    match cli::run(&rows) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", cli::message(error));
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cli;
    use crate::report::{self, ReportError, Summary};
    use std::collections::BTreeMap;

    #[test]
    fn combines_events_by_dock() {
        let rows = ["3:20", "1:5", "3:15"].map(String::from);
        assert_eq!(
            report::build(&rows),
            Ok(Summary {
                events: 3,
                total_minutes: 40,
                by_dock: BTreeMap::from([(1, 5), (3, 35)]),
            })
        );
        assert_eq!(
            cli::run(&rows),
            Ok(String::from(
                "Простоев 3, всего минут 40\nДок 1 — 5 мин.\nДок 3 — 35 мин."
            ))
        );
    }

    #[test]
    fn reports_first_invalid_row() {
        for text in ["13:4", "2:0", "2:-1", "2:65536", "2:3:4", "2"] {
            let rows = ["3:20", text, "0:5"].map(String::from);
            assert_eq!(
                report::build(&rows),
                Err(ReportError::InvalidRecord {
                    position: 2,
                    text: text.to_owned(),
                })
            );
        }
    }

    #[test]
    fn checks_input_boundaries() {
        assert_eq!(report::build(&[]), Err(ReportError::EmptyInput));
        let rows = [String::from("12:65535")];
        let summary = report::build(&rows).expect("Граничная запись допустима");
        assert_eq!(summary.by_dock.get(&12), Some(&65_535));
    }
}
