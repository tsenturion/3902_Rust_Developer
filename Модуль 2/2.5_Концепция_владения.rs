/*
cargo run --offline --bin ownership_concept
*/

struct ExportJob {
    lesson_number: u16,
    output_path: String,
}

/*
Заявка на экспорт получает путь без расширения. При постановке в очередь
заявку передают на подготовку, где к пути добавляется расширение .mp4.
Готовая заявка возвращается из подготовки. Для запуска экспорта
из неё извлекают номер занятия и подготовленный путь.
*/
fn main() {
    let prepared_job = {
        let base_path = String::from("exports/course/lesson-02");
        let job = ExportJob {
            lesson_number: 2,
            output_path: base_path,
        };

        // Владельцем строки стало поле output_path.
        // println!("{base_path}");

        let queued_job = job;
        // Присваивание переместило заявку в queued_job.
        // println!("{}", job.output_path);

        let prepared = prepare_export(queued_job);
        // Вызов передал заявку в функцию. Результат получен в prepared.
        // println!("{}", queued_job.output_path);

        assert_eq!(prepared.lesson_number, 2);
        prepared
    };

    // Результат блока передал готовую заявку во внешнюю переменную.
    assert_eq!(prepared_job.output_path, "exports/course/lesson-02.mp4");

    {
        // При распаковке строка переходит из поля в локальную переменную.
        let ExportJob {
            lesson_number,
            output_path,
        } = prepared_job;
        println!("Экспорт занятия {lesson_number} — {output_path}");
        // На выходе из блока локальная строка освобождает свои данные.
    }

    // Имя output_path доступно только во внутреннем блоке.
    // println!("{output_path}");
}

fn prepare_export(mut job: ExportJob) -> ExportJob {
    job.output_path.push_str(".mp4");
    job
}
