/*
cargo run --offline --bin ownership_borrowing
*/

struct ExportJob {
    output_path: String,
    duration_seconds: u32,
    bytes_per_second: u32,
    captions: bool,
}

struct ExportPlan {
    job: ExportJob,
    required_bytes: u64,
}

/*
Сначала оценить объём записи и проверить путь назначения.
Затем изменить ту же заявку для предпросмотра. Ограничить длительность
45 секундами, скорость потока 100 000 байт/с и выключить субтитры.
Зарезервировать место и передать заявку в готовый план.
*/
fn main() {
    let mut job = ExportJob {
        output_path: String::from("exports/course/preview.mp4"),
        duration_seconds: 90,
        bytes_per_second: 300_000,
        captions: true,
    };

    // Две общие ссылки позволяют читать одну заявку.
    let size_view = &job;
    let path_view = &job;
    // Для записи требуется изменяемая ссылка.
    // size_view.duration_seconds = 30;
    let full_bytes = estimated_size(size_view);
    assert_eq!(full_bytes, 27_000_000);
    assert_eq!(path_view.output_path, "exports/course/preview.mp4");

    let preview_editor = &mut job;
    // На время изменения доступ к заявке идёт через preview_editor.
    // let _ = estimated_size(&job);
    // let second_editor = &mut job;
    // limit_preview(second_editor, 30, 90_000);
    limit_preview(preview_editor, 45, 100_000);
    assert_eq!(preview_editor.duration_seconds, 45);
    assert_eq!(preview_editor.bytes_per_second, 100_000);

    // Через редактор временно получаем общий доступ для расчёта.
    let bytes_from_editor = estimated_size(&*preview_editor);
    assert_eq!(bytes_from_editor, 4_500_000);
    assert!(!preview_editor.captions);

    // После последнего использования preview_editor можно снова читать job.
    let preview_bytes = estimated_size(&job);
    assert_eq!(preview_bytes, 4_500_000);
    assert_eq!(preview_bytes, bytes_from_editor);

    let mut available_bytes = 5_000_000;
    assert!(reserve_space(&mut available_bytes, preview_bytes));
    assert_eq!(available_bytes, 500_000);
    let extra_reserved = reserve_space(&mut available_bytes, 1_000_000);
    assert!(!extra_reserved);
    assert_eq!(available_bytes, 500_000);

    // Чтобы передать fixed_budget через &mut, его объявляют с mut.
    // let fixed_budget = 5_000_000_u64;
    // reserve_space(&mut fixed_budget, preview_bytes);

    let mut plan = into_export_plan(job);
    assert_eq!(plan.required_bytes, preview_bytes);
    println!("Полная запись — {full_bytes} байт, предпросмотр — {preview_bytes} байт");
    println!("План экспорта — {}", plan.job.output_path);
    println!("Свободное место после резерва — {available_bytes} байт");
    println!("Оценка через редактор — {bytes_from_editor} байт");
    println!("Дополнительный резерв — принят {extra_reserved}, остаток {available_bytes} байт");

    // Общий доступ к объёму и изменение пути относятся к разным полям.
    let bytes_for_notice = &plan.required_bytes;
    let staging_path = &mut plan.job.output_path;
    staging_path.push_str(".part");
    println!("Временный путь — {staging_path}, объём — {bytes_for_notice} байт");
    assert_eq!(plan.job.output_path, "exports/course/preview.mp4.part");
    assert_eq!(plan.required_bytes, preview_bytes);

    // Заявка теперь принадлежит плану.
    // println!("{}", job.output_path);

    // Ссылка должна использоваться, пока существует значение, на которое она указывает.
    // let saved_path;
    // {
    //     let temporary_path = String::from("exports/temporary.mp4");
    //     saved_path = &temporary_path;
    // }
    // println!("{saved_path}");
}

fn estimated_size(job: &ExportJob) -> u64 {
    u64::from(job.duration_seconds) * u64::from(job.bytes_per_second)
}

fn limit_preview(job: &mut ExportJob, max_seconds: u32, max_bytes_per_second: u32) {
    job.duration_seconds = job.duration_seconds.min(max_seconds);
    job.bytes_per_second = job.bytes_per_second.min(max_bytes_per_second);
    job.captions = false;
}

fn reserve_space(available_bytes: &mut u64, required_bytes: u64) -> bool {
    // * обращается к числу, на которое указывает ссылка.
    if *available_bytes < required_bytes {
        return false;
    }
    *available_bytes -= required_bytes;
    true
}

fn into_export_plan(job: ExportJob) -> ExportPlan {
    let required_bytes = estimated_size(&job);
    ExportPlan {
        job,
        required_bytes,
    }
}
