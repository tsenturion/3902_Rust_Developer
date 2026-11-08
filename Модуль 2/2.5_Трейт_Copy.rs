/*
cargo run --offline --bin copy_trait
*/

#[derive(Clone, Copy)]
struct ClipWindow {
    start_seconds: u32,
    duration_seconds: u16,
    captions: bool,
}

/*
Для одного фрагмента записи нужно подготовить исходное окно экспорта
и короткий предпросмотр. Предпросмотр длится 30 секунд и идёт без субтитров.
Затем нужно рассчитать запасной вариант, если старт предпросмотра
сдвинется на 5 секунд. Исходные параметры нужны для сравнения вариантов.
*/
fn main() {
    let original = ClipWindow {
        start_seconds: 2_700,
        duration_seconds: 120,
        captions: true,
    };

    // Присваивание создаёт копию значения ClipWindow.
    let mut preview = original;
    preview.duration_seconds = 30;
    preview.captions = false;

    // Функция принимает ClipWindow по значению. Исходные параметры ещё доступны.
    let original_plan = export_plan(original);
    let preview_plan = export_plan(preview);
    assert_eq!(original.duration_seconds, 120);
    assert_eq!(preview.duration_seconds, 30);
    assert_eq!(original_plan, (2_700, 2_820, true));
    assert_eq!(preview_plan, (2_700, 2_730, false));

    // Кортеж из чисел и bool тоже поддерживает Copy.
    let mut delayed_preview = preview_plan;
    let delay_seconds = 5;
    delayed_preview.0 += delay_seconds;
    delayed_preview.1 += delay_seconds;
    assert_eq!(preview_plan, (2_700, 2_730, false));
    assert_eq!(delayed_preview, (2_705, 2_735, false));

    let (original_start, original_end, original_captions) = original_plan;
    let (preview_start, preview_end, preview_captions) = preview_plan;
    let (delayed_start, delayed_end, _) = delayed_preview;
    println!("Исходный клип — {original_start}..{original_end} с, субтитры — {original_captions}");
    println!("Предпросмотр — {preview_start}..{preview_end} с, субтитры — {preview_captions}");
    println!("Предпросмотр с задержкой — {delayed_start}..{delayed_end} с");
}

fn export_plan(window: ClipWindow) -> (u64, u64, bool) {
    let start_seconds = u64::from(window.start_seconds);
    let end_seconds = start_seconds + u64::from(window.duration_seconds);
    (start_seconds, end_seconds, window.captions)
}

// Для Copy требуется Clone.
// #[derive(Copy)]
// struct WithoutClone {
//     duration_seconds: u16,
// }

// Все поля структуры с Copy должны поддерживать Copy.
// #[derive(Clone, Copy)]
// struct NamedClip {
//     title: String,
// }
