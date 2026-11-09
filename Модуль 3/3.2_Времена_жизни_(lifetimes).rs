/*
cargo run --offline --bin lifetimes
*/

/*
Журнал контроллера содержит строки вида «WARN E42 таймаут шлюза».
Найти первую запись с нужным кодом и вернуть её поля как слайсы
исходного текста. Некорректные строки пропускать, описание может
отсутствовать. Поисковый запрос нужен только во время поиска.
Если сообщение требуется после выгрузки журнала из памяти, сохранить
его в отчёте отдельной строкой.
*/

#[derive(Clone, Copy, Debug, PartialEq)]
enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    // Слайс строкового литерала может использоваться всё время работы программы.
    fn default_detail(self) -> &'static str {
        match self {
            Self::Info => "Информационное событие",
            Self::Warn => "Проверьте устройство",
            Self::Error => "Требуется вмешательство",
        }
    }
}

#[derive(Debug, PartialEq)]
struct Entry<'text> {
    level: Level,
    code: &'text str,
    detail: &'text str,
}

// У функции одна входная ссылка. В Entry<'_> Rust подставляет её время жизни.
fn parse_entry(line: &str) -> Option<Entry<'_>> {
    let (level, rest) = line.split_once(' ')?;
    let level = match level {
        "INFO" => Level::Info,
        "WARN" => Level::Warn,
        "ERROR" => Level::Error,
        _ => return None,
    };
    let (code, detail) = rest.split_once(' ').unwrap_or((rest, ""));
    if code.is_empty() {
        return None;
    }
    Some(Entry {
        level,
        code,
        detail: detail.trim(),
    })
}

// Здесь две входные ссылки. 'text связывает результат только с текстом журнала.
fn find_entry<'text>(text: &'text str, code: &str) -> Option<Entry<'text>> {
    for line in text.lines() {
        let Some(entry) = parse_entry(line) else {
            continue;
        };
        if entry.code == code {
            return Some(entry);
        }
    }
    None
}

struct LogView<'text> {
    text: &'text str,
}

impl<'text> LogView<'text> {
    fn new(text: &'text str) -> Self {
        Self { text }
    }

    // Для метода с &self опущенное время жизни результата связано с заимствованием self.
    fn raw(&self) -> &str {
        self.text
    }

    // Результат привязан к исходному тексту, поэтому может пережить LogView.
    fn find(&self, code: &str) -> Option<Entry<'text>> {
        find_entry(self.text, code)
    }
}

// Результат может прийти из любой строки. 'a должен быть допустим для обеих ссылок.
fn operator_detail<'a>(detail: &'a str, fallback: &'a str) -> &'a str {
    if detail.is_empty() { fallback } else { detail }
}

fn main() {
    let mut source = String::from(
        "UNKNOWN E42 неподдерживаемый уровень\nWARN E42 таймаут шлюза\nERROR E42 повтор\nWARN E07",
    );

    assert_eq!(find_entry("", "E42"), None);
    assert_eq!(find_entry(&source, "E99"), None);
    assert_eq!(parse_entry("WARN  текст без кода"), None);

    let incident = {
        let view = LogView::new(&source);
        let query = String::from("E42");
        assert_eq!(view.raw(), source.as_str());
        let Some(entry) = view.find(&query) else {
            panic!("В журнале примера должна быть запись E42");
        };
        entry
    };

    // Изменение source здесь нарушило бы ещё действующее заимствование.
    // source.push('\n');

    assert_eq!(
        incident,
        Entry {
            level: Level::Warn,
            code: "E42",
            detail: "таймаут шлюза"
        }
    );
    println!("{} — {}", incident.code, incident.detail);

    // После последнего использования incident его заимствование больше не требуется.
    source.push_str("\nINFO I09 контроллер восстановлен");
    assert_eq!(
        find_entry(&source, "I09").map(|entry| entry.detail),
        Some("контроллер восстановлен")
    );

    let Some(without_detail) = find_entry(&source, "E07") else {
        panic!("В журнале примера должна быть запись E07");
    };
    let hint: &'static str = without_detail.level.default_detail();
    assert_eq!(without_detail.detail, "");
    let message = operator_detail(without_detail.detail, hint);
    assert_eq!(message, "Проверьте устройство");
    println!("{} — {message}", without_detail.code);

    // Отчёт получает собственную строку, чтобы она сохранилась после удаления буфера.
    let exported = {
        let imported = String::from("ERROR E90 потеряна связь");
        let Some(entry) = find_entry(&imported, "E90") else {
            panic!("В журнале примера должна быть запись E90");
        };
        let fallback = String::from(entry.level.default_detail());
        operator_detail(entry.detail, &fallback).to_owned()
    };
    assert_eq!(exported, "потеряна связь");
    println!("Сообщение в отчёте — {exported}");

    // raw возвращает слайс на время заимствования view.
    /*
    let raw = { let view = LogView::new(&source); view.raw() };
    assert!(!raw.is_empty());
    */

    // Общий результат operator_detail нельзя вынести за пределы жизни запасной строки.
    /*
    let message = {
        let fallback = String::from("Описание отсутствует");
        operator_detail("таймаут шлюза", &fallback)
    };
    assert_eq!(message, "таймаут шлюза");
    */
}

// При двух входных ссылках нужно указать, откуда берутся поля возвращаемой записи.
// fn without_origin(text: &str, code: &str) -> Option<Entry<'_>> { find_entry(text, code) }

// Аннотация 'static не продлевает жизнь локального String.
// fn from_local() -> &'static str { String::from("WARN E42 таймаут").as_str() }
