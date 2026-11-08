/*
cargo run --offline --bin loops
*/

const CLIPS_PER_BATCH: u16 = 3;
const RESERVED_SLOT: u16 = 2;

fn main() {
    let planned_clips = plan_clips(5, 45, 200);
    assert_eq!(planned_clips, 4);
    assert_eq!(plan_clips(5, 0, 200), 0);

    let batches = minimum_batches(planned_clips);
    assert_eq!(batches, 2);
    assert_eq!(minimum_batches(0), 0);
    assert_eq!(minimum_batches(6), 2);

    let clip_bytes = 3_000_000;
    let (fits, remaining_bytes) = fill_batch(7_500_000, clip_bytes);
    assert_eq!((fits, remaining_bytes), (2, 1_500_000));
    assert_eq!(fill_batch(0, clip_bytes), (0, 0));
    assert_eq!(fill_batch(7_500_000, 0), (0, 7_500_000));

    let normal = schedule_in_batches(3, 5_000_000, clip_bytes, planned_clips);
    assert_eq!(normal, (3, 3, 3));
    let stopped_at_request = schedule_in_batches(3, 5_000_000, clip_bytes, 2);
    assert_eq!(stopped_at_request, (2, 1, 1));
    assert_eq!(schedule_in_batches(2, 9_000_000, clip_bytes, 5), (4, 2, 0));
    assert_eq!(
        schedule_in_batches(3, 5_000_000, 0, planned_clips),
        (0, 0, 0)
    );
    assert_eq!(schedule_in_batches(3, 5_000_000, clip_bytes, 0), (0, 0, 0));

    let (scheduled, skipped_reserved, ended_by_bytes) = normal;
    let (early_scheduled, early_skipped_reserved, early_ended_by_bytes) = stopped_at_request;

    println!("По числу мест для {planned_clips} фрагментов нужно {batches} партии");
    println!("По объёму в одну партию входят {fits} фрагмента, остаток — {remaining_bytes} байт");
    println!("С учётом лимитов запланировано {scheduled} из {planned_clips} фрагментов");
    println!(
        "Обычный запрос: запланировано {scheduled}, continue: пропущено резервных мест {skipped_reserved}, continue 'batches: партий завершено по лимиту байтов {ended_by_bytes}"
    );
    println!(
        "Запрос 2, break 'batches: запланировано {early_scheduled}, continue: пропущено резервных мест {early_skipped_reserved}, continue 'batches: партий завершено по лимиту байтов {early_ended_by_bytes}"
    );

    // Условие while должно иметь тип bool.
    // while planned_clips { break; }

    // Выход с результатом доступен в loop.
    // while planned_clips > 0 { break 1; }

    // Количество фрагментов можно использовать как границу диапазона.
    // for index in planned_clips { let _ = index; }

    // Метка batches доступна внутри помеченного цикла.
    // break 'batches;
}

fn plan_clips(requested_clips: u16, clip_seconds: u16, recording_seconds: u32) -> u16 {
    if clip_seconds == 0 {
        return 0;
    }

    let mut planned = 0;
    // Верхняя граница диапазона 0..requested_clips исключена.
    for index in 0..requested_clips {
        let start = u32::from(index) * u32::from(clip_seconds);
        let end = start + u32::from(clip_seconds);
        if end > recording_seconds {
            break;
        }
        planned += 1;
        println!("Фрагмент {planned} — {start}..{end} с");
    }
    planned
}

fn minimum_batches(clip_count: u16) -> u16 {
    let mut remaining = clip_count;
    let mut batches = 0;
    loop {
        if remaining == 0 {
            // Значение после break становится результатом loop.
            break batches;
        }
        remaining = remaining.saturating_sub(CLIPS_PER_BATCH);
        batches += 1;
    }
}

fn fill_batch(available_bytes: u64, clip_bytes: u64) -> (u16, u64) {
    let mut remaining_bytes = available_bytes;
    let mut fits = 0;
    // Условие проверяется перед первой и каждой следующей итерацией.
    while clip_bytes != 0 && fits < CLIPS_PER_BATCH && remaining_bytes >= clip_bytes {
        remaining_bytes -= clip_bytes;
        fits += 1;
    }
    (fits, remaining_bytes)
}

fn schedule_in_batches(
    batch_count: u16,
    bytes_per_batch: u64,
    clip_bytes: u64,
    requested_clips: u16,
) -> (u16, u16, u16) {
    if clip_bytes == 0 {
        return (0, 0, 0);
    }

    let mut scheduled = 0;
    let mut skipped_reserved = 0;
    let mut ended_by_bytes = 0;
    'batches: for _ in 0..batch_count {
        let mut remaining_bytes = bytes_per_batch;
        // Диапазон 1..=CLIPS_PER_BATCH включает последнее место.
        for slot in 1..=CLIPS_PER_BATCH {
            if scheduled == requested_clips {
                break 'batches;
            }
            if slot == RESERVED_SLOT {
                skipped_reserved += 1;
                continue;
            }
            if remaining_bytes < clip_bytes {
                ended_by_bytes += 1;
                continue 'batches;
            }
            remaining_bytes -= clip_bytes;
            scheduled += 1;
        }
    }
    (scheduled, skipped_reserved, ended_by_bytes)
}
