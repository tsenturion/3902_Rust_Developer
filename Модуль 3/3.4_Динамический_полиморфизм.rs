/*
cargo run --offline --bin dynamic_dispatch
cargo run --offline --bin dynamic_dispatch -- tsv
*/

/*
Один журнал нужен дежурному на экране и аналитикам для загрузки в таблицу.
Терминал по умолчанию выводит читаемый отчёт. Если при запуске передать tsv,
те же записи должны выйти в виде строк с разделителями-табуляциями.
В TSV нужно экранировать табуляции, переводы строк и обратные косые черты
внутри сообщения.
*/

use std::process::ExitCode;

#[derive(Clone, Copy)]
enum Level {
    Warning,
    Error,
}

impl Level {
    fn code(self) -> &'static str {
        match self {
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        }
    }
}

struct Incident {
    number: u32,
    level: Level,
    message: String,
}

trait ReportFormat {
    fn key(&self) -> &'static str;
    fn render(&self, incident: &Incident) -> String;
}

struct OperatorFormat {
    station: String,
}

impl ReportFormat for OperatorFormat {
    fn key(&self) -> &'static str {
        "operator"
    }

    fn render(&self, incident: &Incident) -> String {
        format!(
            "{} — {} №{} — {}",
            self.station,
            incident.level.code(),
            incident.number,
            incident.message
        )
    }
}

struct TsvFormat;

impl ReportFormat for TsvFormat {
    fn key(&self) -> &'static str {
        "tsv"
    }

    fn render(&self, incident: &Incident) -> String {
        let mut field = String::new();
        for character in incident.message.chars() {
            match character {
                '\\' => field.push_str("\\\\"),
                '\t' => field.push_str("\\t"),
                '\r' => field.push_str("\\r"),
                '\n' => field.push_str("\\n"),
                character => field.push(character),
            }
        }
        format!("{}\t{}\t{field}", incident.number, incident.level.code())
    }
}

fn choose_format<'format>(
    formats: &[&'format dyn ReportFormat],
    key: &str,
) -> Option<&'format dyn ReportFormat> {
    for &format in formats {
        if format.key() == key {
            return Some(format);
        }
    }
    None
}

fn build_report(format: &dyn ReportFormat, incidents: &[Incident]) -> String {
    let mut report = String::new();
    for incident in incidents {
        // &dyn ReportFormat содержит указатели на данные и таблицу методов.
        // Вызов выбирает render конкретной реализации по этой таблице.
        report.push_str(&format.render(incident));
        report.push('\n');
    }
    report
}

fn check_example(formats: &[&dyn ReportFormat], incidents: &[Incident]) {
    let Some(operator) = choose_format(formats, "operator") else {
        panic!("В примере должен быть формат для дежурного");
    };
    let Some(tsv) = choose_format(formats, "tsv") else {
        panic!("В примере должен быть формат TSV");
    };

    assert_eq!(
        build_report(operator, incidents),
        "Пост 7 — WARN №42 — таймаут шлюза\nПост 7 — ERROR №43 — потеряна связь\n"
    );
    assert_eq!(
        build_report(tsv, incidents),
        "42\tWARN\tтаймаут шлюза\n43\tERROR\tпотеряна связь\n"
    );
    assert_eq!(build_report(tsv, &[]), "");
    assert!(choose_format(formats, "xml").is_none());

    let with_control_chars = Incident {
        number: 44,
        level: Level::Warning,
        message: String::from("строка\tтаб\r\nпуть\\журнал"),
    };
    assert_eq!(
        tsv.render(&with_control_chars),
        "44\tWARN\tстрока\\tтаб\\r\\nпуть\\\\журнал"
    );
}

fn main() -> ExitCode {
    let operator = OperatorFormat {
        station: String::from("Пост 7"),
    };
    let tsv = TsvFormat;
    // Без аннотации Rust выведет тип элемента из первой ссылки.
    // let _formats = [&operator, &tsv];

    // Элементы имеют общий тип ссылки. Сами значения остаются у своих владельцев.
    let formats: [&dyn ReportFormat; 2] = [&operator, &tsv];
    let incidents = [
        Incident {
            number: 42,
            level: Level::Warning,
            message: String::from("таймаут шлюза"),
        },
        Incident {
            number: 43,
            level: Level::Error,
            message: String::from("потеряна связь"),
        },
    ];
    check_example(&formats, &incidents);

    let argument = std::env::args().nth(1);
    let key = argument.as_deref().unwrap_or("operator");
    let Some(format) = choose_format(&formats, key) else {
        eprintln!("Неизвестный формат «{key}». Доступны operator и tsv.");
        return ExitCode::FAILURE;
    };

    print!("{}", build_report(format, &incidents));
    ExitCode::SUCCESS
}

// Для объекта трейта методы должны быть доступны через таблицу вызовов.
// Здесь render имеет собственный параметр T, поэтому трейт несовместим с dyn.
/*
trait GenericFormat {
    fn render<T>(&self, value: T) -> String;
}

fn render_any(format: &dyn GenericFormat) {}
*/
