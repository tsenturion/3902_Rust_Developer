/*
cargo run --offline --bin resources
*/

use std::fs::{self, File};
use std::io::{self, Write};

/*
Сервис экспорта пишет отчёт в файл. Две операции могут открыть один и тот же
отчёт, но блокировку для записи должна удерживать только одна из них.
Проверить, что следующая операция получает блокировку после завершения блока,
явного закрытия файла и раннего возврата при пропуске записи.
*/
fn main() {
    fs::create_dir_all("target/resource_management").expect("Не удалось создать папку отчётов");
    let report_path = format!(
        "target/resource_management/export-{}.txt",
        std::process::id()
    );

    let next_writer;
    {
        let mut report = File::create(&report_path).expect("Не удалось создать отчёт");
        report.try_lock().expect("Не удалось заблокировать отчёт");
        writeln!(report, "Экспорт подготовлен.").expect("Не удалось записать начало отчёта");

        next_writer = open_for_append(&report_path);
        let conflict = next_writer
            .try_lock()
            .expect_err("Следующая операция должна дождаться освобождения файла");
        assert_eq!(io::Error::from(conflict).kind(), io::ErrorKind::WouldBlock);
    }

    // При выходе из блока первый файл закрывается и освобождает блокировку.
    next_writer
        .try_lock()
        .expect("После блока отчёт должен быть доступен");
    println!("После выхода из блока следующая операция получила блокировку.");

    let locked_writer = next_writer;
    let independent_writer = open_for_append(&report_path);
    let conflict = independent_writer
        .try_lock()
        .expect_err("Перемещение файла не должно освобождать блокировку");
    assert_eq!(io::Error::from(conflict).kind(), io::ErrorKind::WouldBlock);
    println!("После перемещения дескриптора блокировка сохраняется.");

    drop(locked_writer);
    independent_writer
        .try_lock()
        .expect("После закрытия перемещённого дескриптора отчёт должен быть доступен");
    println!("После закрытия перемещённого дескриптора независимая операция получила блокировку.");
    drop(independent_writer);
    assert!(!append_export_result(&report_path, false));

    // Ранний return тоже завершает доступ к файлу.
    assert!(append_export_result(&report_path, true));
    let contents = fs::read_to_string(&report_path).expect("Не удалось прочитать готовый отчёт");
    assert_eq!(contents, "Экспорт подготовлен.\nЭкспорт завершён.\n");
    println!("После явного закрытия и раннего возврата отчёт успешно дополнен.");
    println!("Отчёт сохранён в {report_path}");
}

fn open_for_append(path: &str) -> File {
    File::options()
        .read(true)
        .append(true)
        .open(path)
        .expect("Не удалось открыть отчёт для дополнения")
}

fn append_export_result(path: &str, export_finished: bool) -> bool {
    let mut report = open_for_append(path);
    report.try_lock().expect("Не удалось заблокировать отчёт");
    if !export_finished {
        return false;
    }

    writeln!(report, "Экспорт завершён.").expect("Не удалось дописать отчёт");
    true
}
