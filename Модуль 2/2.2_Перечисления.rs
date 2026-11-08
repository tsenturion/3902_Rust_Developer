/*
cargo run --offline --bin enums
*/

#[derive(Debug, PartialEq)]
enum ExportRequest {
    FullRecording,
    Snapshot(u64),
    ContactSheet(u16, u16),
    Clip {
        start_seconds: u64,
        duration_seconds: u32,
    },
}

struct ExportJob {
    id: u32,
    request: ExportRequest,
}

fn main() {
    let mut job = ExportJob {
        id: 17,
        request: ExportRequest::FullRecording,
    };
    assert_eq!(job.request, ExportRequest::FullRecording);

    // Две заявки на снимок несут разные данные, но имеют один вариант.
    let snapshot = ExportRequest::Snapshot(90);
    let later_snapshot = ExportRequest::Snapshot(120);
    let same_values = snapshot == later_snapshot;
    let same_variant = std::mem::discriminant(&snapshot) == std::mem::discriminant(&later_snapshot);
    assert!(!same_values);
    assert!(same_variant);
    println!("Снимки: совпадает вариант — {same_variant}, совпадают значения — {same_values}");
    job.request = snapshot;
    println!("Заявка {} на снимок — {:?}", job.id, job.request);

    // У листа миниатюр позиционные поля идут по порядку: столбцы, затем строки.
    // У Clip именованные поля показывают назначение каждого числа при создании.
    let columns = 4;
    let rows = 3;
    let sheet_job = ExportJob {
        id: 18,
        request: ExportRequest::ContactSheet(columns, rows),
    };
    println!(
        "Заявка {} на лист миниатюр (столбцы, строки) — {:?}",
        sheet_job.id, sheet_job.request
    );

    // Индекс второго занятия равен 1. До его начала проходит 45 минут.
    job.request = lesson_clip(1, 45);
    assert_eq!(
        job.request,
        ExportRequest::Clip {
            start_seconds: 2_700,
            duration_seconds: 2_700,
        }
    );
    assert_ne!(
        std::mem::discriminant(&job.request),
        std::mem::discriminant(&later_snapshot)
    );
    println!("Заявка {} на фрагмент — {:?}", job.id, job.request);

    // Значение перечисления создаётся из объявленного варианта.
    // job.request = ExportRequest::AudioOnly;

    // Вариант Snapshot требует время снимка.
    // job.request = ExportRequest::Snapshot;

    // Для Clip требуется заполнить оба поля.
    // job.request = ExportRequest::Clip { start_seconds: 2_700 };

    // duration_seconds принадлежит варианту Clip.
    // let _ = job.request.duration_seconds;
}

fn lesson_clip(lesson_index: u16, lesson_minutes: u16) -> ExportRequest {
    let duration_seconds = u32::from(lesson_minutes) * 60;
    let start_seconds = u64::from(lesson_index) * u64::from(duration_seconds);
    ExportRequest::Clip {
        start_seconds,
        duration_seconds,
    }
}
