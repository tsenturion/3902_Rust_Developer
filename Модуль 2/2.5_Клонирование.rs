/*
cargo run --offline --bin cloning
*/

#[derive(Clone)]
struct ExportProfile {
    directory: String,
    file_stem: String,
    frames_per_second: u16,
    captions: bool,
}

/*
Из общего профиля экспорта нужно получить настройки предпросмотра.
У предпросмотра своя папка, суффикс имени и частота кадров.
Для архива требуется только отдельное имя. Настроенный предпросмотр
нужно сохранить для заявки в очереди, а перед следующим заданием
восстановить рабочий профиль из шаблона.
*/
fn main() {
    let template = ExportProfile {
        directory: String::from("exports/course"),
        file_stem: String::from("lesson-02"),
        frames_per_second: 30,
        captions: true,
    };

    // derive(Clone) клонирует поля профиля по отдельности; строки можно менять независимо.
    let mut working_profile = template.clone();
    working_profile.directory.push_str("/preview");
    working_profile.file_stem.push_str("-preview");
    working_profile.frames_per_second = 15;
    working_profile.captions = false;

    assert_eq!(working_profile.directory, "exports/course/preview");
    assert_eq!(working_profile.file_stem, "lesson-02-preview");
    assert_eq!(working_profile.frames_per_second, 15);
    assert!(!working_profile.captions);
    assert_eq!(template.directory, "exports/course");
    assert_eq!(template.file_stem, "lesson-02");
    assert_eq!(template.frames_per_second, 30);
    assert!(template.captions);

    println!("Шаблон — {}/{}.mp4", template.directory, template.file_stem);
    println!(
        "Предпросмотр — {}/{}.mp4, {} кадров/с",
        working_profile.directory, working_profile.file_stem, working_profile.frames_per_second
    );

    // Для отдельного имени достаточно клонировать одно поле.
    let mut archive_file_stem = template.file_stem.clone();
    archive_file_stem.push_str("-archive");
    assert_eq!(archive_file_stem, "lesson-02-archive");
    assert_eq!(template.file_stem, "lesson-02");
    println!("Имя для архива — {archive_file_stem}.mp4");

    // Заявка в очереди сохраняет настройки предпросмотра перед восстановлением.
    let queued_profile = working_profile.clone();

    // clone_from заменяет содержимое существующего рабочего профиля.
    working_profile.clone_from(&template);
    assert_eq!(queued_profile.directory, "exports/course/preview");
    assert_eq!(queued_profile.file_stem, "lesson-02-preview");
    assert_eq!(queued_profile.frames_per_second, 15);
    assert!(!queued_profile.captions);
    assert_eq!(working_profile.directory, template.directory);
    assert_eq!(working_profile.file_stem, template.file_stem);
    assert_eq!(
        working_profile.frames_per_second,
        template.frames_per_second
    );
    assert_eq!(working_profile.captions, template.captions);
    println!(
        "Заявка в очереди — {}/{}.mp4, {} кадров/с, субтитры — {}",
        queued_profile.directory,
        queued_profile.file_stem,
        queued_profile.frames_per_second,
        queued_profile.captions
    );
    println!(
        "Рабочий профиль — {}/{}.mp4, {} кадров/с, субтитры — {}",
        working_profile.directory,
        working_profile.file_stem,
        working_profile.frames_per_second,
        working_profile.captions
    );
    println!("Профиль для следующего задания восстановлен из шаблона.");

    // Источник clone_from должен иметь тип ExportProfile.
    // working_profile.clone_from(&archive_file_stem);
}

// Для derive(Clone) клонирование должно поддерживаться всеми полями.
// #[derive(Clone)]
// struct OpenReport {
//     file: std::fs::File,
// }
