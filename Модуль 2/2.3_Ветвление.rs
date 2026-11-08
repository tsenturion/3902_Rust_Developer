/*
cargo run --offline --bin branching
*/

#[derive(Debug, PartialEq)]
enum ExportDecision {
    EmptyRecording,
    InsufficientSpace,
    NeedsConfirmation,
    Ready,
}

fn main() {
    let preview_bytes = recording_bytes(90, true);
    let full_bytes = recording_bytes(90, false);
    let available_bytes = 9_000_000;
    let empty_decision = decide_export(0, available_bytes, false);
    let full_decision = decide_export(full_bytes, available_bytes, false);
    let unconfirmed_decision = decide_export(preview_bytes, available_bytes, false);
    let decision = decide_export(preview_bytes, available_bytes, true);

    assert_eq!(preview_bytes, 9_000_000);
    assert_eq!(full_bytes, 27_000_000);
    assert_eq!(decision, ExportDecision::Ready);
    assert_eq!(empty_decision, ExportDecision::EmptyRecording);
    assert_eq!(full_decision, ExportDecision::InsufficientSpace);
    assert_eq!(unconfirmed_decision, ExportDecision::NeedsConfirmation);

    // Меняем только объём свободного места на один байт вокруг границы.
    let below_limit = decide_export(preview_bytes, available_bytes - 1, true);
    let above_limit = decide_export(preview_bytes, available_bytes + 1, true);
    assert_eq!(below_limit, ExportDecision::InsufficientSpace);
    assert_eq!(above_limit, ExportDecision::Ready);

    assert!(fits_repeated_exports(preview_bytes, available_bytes, 1));
    assert!(!fits_repeated_exports(preview_bytes, available_bytes, 2));
    let zero_copies_fit = fits_repeated_exports(0, available_bytes, 0);
    let one_empty_copy_fits = fits_repeated_exports(0, available_bytes, 1);
    assert!(zero_copies_fit);
    assert!(!one_empty_copy_fits);

    println!("Предпросмотр — {preview_bytes} байт, полная запись — {full_bytes} байт");
    println!(
        "Приоритет решения — пустая запись {empty_decision:?}, полная без подтверждения {full_decision:?}, предпросмотр без подтверждения {unconfirmed_decision:?}"
    );
    println!(
        "Граница места — на байт меньше {below_limit:?}, ровно {decision:?}, на байт больше {above_limit:?}"
    );
    println!("Пустой фрагмент — ноль копий {zero_copies_fit}, одна копия {one_empty_copy_fits}");
    // if без else выполняет действие только при истинном условии.
    if decision == ExportDecision::Ready {
        println!("Экспорт разрешён.");
        // Это условие проверяется только внутри разрешённого экспорта.
        if preview_bytes == available_bytes {
            println!("Свободное место будет занято полностью.");
        }
    }

    // Условие должно иметь тип bool.
    // if preview_bytes { println!("Экспорт"); }

    // Обе ветки выражения должны давать совместимые типы.
    // let _ = if preview_bytes <= available_bytes { preview_bytes } else { false };

    // Для выбора числа нужен результат и при ложном условии.
    // let _: u64 = if preview_bytes > 0 { preview_bytes };
}

fn recording_bytes(seconds: u32, preview: bool) -> u64 {
    // Результат выбранной ветки инициализирует переменную.
    let bytes_per_second: u32 = if preview { 100_000 } else { 300_000 };
    u64::from(seconds) * u64::from(bytes_per_second)
}

fn decide_export(required_bytes: u64, available_bytes: u64, confirmed: bool) -> ExportDecision {
    if required_bytes == 0 {
        return ExportDecision::EmptyRecording;
    }

    // Срабатывает первая истинная ветка. Проверка места предшествует подтверждению.
    if required_bytes > available_bytes {
        ExportDecision::InsufficientSpace
    } else if !confirmed {
        ExportDecision::NeedsConfirmation
    } else {
        ExportDecision::Ready
    }
}

fn fits_repeated_exports(clip_bytes: u64, available_bytes: u64, copies: u64) -> bool {
    // Ноль копий сразу даёт true. Для остальных случаев && защищает деление.
    copies == 0 || (clip_bytes != 0 && available_bytes / clip_bytes >= copies)
}
