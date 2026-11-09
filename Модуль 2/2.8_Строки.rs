/*
cargo run --offline --bin strings
*/

/*
Заголовок записи приходит с лишними пробелами и переносами.
Подготовить полное имя для журнала экспорта и сокращённое для списка.
Для списка действует лимит в байтах. Срез должен заканчиваться
на границе UTF-8. Когда к полному заголовку добавится отметка готовности,
короткий заголовок сохранится отдельной строкой.
*/
fn main() {
    let raw_title: &str = "\t  Урок\t02   🦀\n";
    let mut title = normalize_title(raw_title);
    assert_eq!(title, "Урок 02 🦀");
    assert!(title.starts_with("Урок"));
    assert!(title.contains("02"));
    assert!(title.ends_with('🦀'));
    assert_eq!(normalize_title("\t\u{00A0}\n"), "");

    // len даёт размер в байтах. chars перебирает скалярные значения Unicode.
    let (bytes, scalars) = text_metrics(&title);
    assert_eq!((bytes, scalars), (16, 9));
    assert_eq!("Я".as_bytes(), &[0xD0, 0xAF]);
    let accented_text = "е\u{0301}";
    assert_eq!(text_metrics(accented_text), (4, 2));

    let view: &str = title.as_str();
    let prefix_for_list = utf8_prefix(view, 14);
    assert_eq!(prefix_for_list, "Урок 02 ");

    // prefix_for_list ещё нужен для подготовки короткого заголовка.
    // title.push('!');
    let compact_title = prefix_for_list.trim_end().to_owned();

    title.push_str(" · ");
    title.push('✓');
    assert_eq!(title, "Урок 02 🦀 · ✓");
    assert_eq!(compact_title, "Урок 02");

    // push принимает один char.
    // title.push("✓");

    // as_bytes даёт доступ к байтам по индексу.
    // let _ = title[0];

    // Оператор + принимает левую строку во владение.
    let prefix = String::from("[Экспорт] ");
    let status = prefix + title.as_str();
    // println!("{prefix}");
    let output_directory = r"C:\Exports\course";
    assert_eq!(output_directory, "C:\\Exports\\course");
    let log_line = format!("{status} → {output_directory}");
    assert_eq!(status, "[Экспорт] Урок 02 🦀 · ✓");

    println!("{log_line}");
    println!("Заголовок для списка — {compact_title}");
    println!("После нормализации — {bytes} байт и {scalars} значений char");

    let cyrillic_crab = "Я🦀";
    let prefix_five = utf8_prefix(cyrillic_crab, 5);
    let prefix_six = utf8_prefix(cyrillic_crab, 6);
    assert_eq!(utf8_prefix(cyrillic_crab, 1), "");
    assert_eq!(utf8_prefix(cyrillic_crab, 2), "Я");
    assert_eq!(prefix_five, "Я");
    assert_eq!(prefix_six, "Я🦀");
    assert_eq!(utf8_prefix(cyrillic_crab, usize::MAX), "Я🦀");
    assert_eq!(utf8_prefix("", 10), "");
    println!("Префикс «{cyrillic_crab}» при лимите 5 байт — «{prefix_five}»");
    println!("Префикс «{cyrillic_crab}» при лимите 6 байт — «{prefix_six}»");

    // Знак ударения занимает отдельные байты, хотя текст выглядит одной буквой.
    let accented_prefix = utf8_prefix(accented_text, 2);
    assert_eq!(accented_prefix, "е");
    println!("Префикс «{accented_text}» при лимите 2 байта — «{accented_prefix}»");

    // Индекс 1 указывает внутрь двухбайтовой буквы Я.
    // let cyrillic = "Я";
    // println!("{}", &cyrillic[..1]);
}

fn normalize_title(input: &str) -> String {
    let mut normalized = String::new();
    for word in input.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    normalized
}

fn text_metrics(text: &str) -> (usize, usize) {
    let mut scalars = 0;
    for _ in text.chars() {
        scalars += 1;
    }
    (text.len(), scalars)
}

fn utf8_prefix(text: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}
