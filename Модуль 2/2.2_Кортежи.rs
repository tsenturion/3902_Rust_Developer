/*
cargo run --offline --bin tuples
*/

fn main() {
    // Ширина, высота и признак субтитров занимают фиксированные позиции.
    let mut frame: (u16, u16, bool) = (1_920, 1_080, false);
    assert_eq!(frame.0, 1_920);
    assert_eq!(frame.1, 1_080);
    assert!(!frame.2);
    frame.2 = true;
    let (width, height, subtitles) = frame;
    assert!(subtitles);

    let full_size = (width, height);
    let (full_pixels, full_rgb_bytes) = rgb_frame_metrics(full_size);
    assert_eq!(full_pixels, 2_073_600);
    assert_eq!(full_rgb_bytes, 6_220_800);

    // Фрагмент объединяет две пары. Первая задаёт начало, вторая задаёт размер.
    let crop = ((100_u16, 40_u16), (640_u16, 360_u16));
    let ((left, top), preview_size) = crop;
    let (right, bottom) = crop_end((left, top), preview_size);
    assert_eq!((right, bottom), (740, 400));
    assert!(right <= u32::from(width) && bottom <= u32::from(height));

    // Для расчёта объёма предпросмотра достаточно второго поля результата.
    let (_, preview_rgb_bytes) = rgb_frame_metrics(preview_size);
    assert_eq!(preview_rgb_bytes, 691_200);

    // При том же размере фрагмент может упираться ровно в границы кадра.
    let edge_origin = (1_280, 720);
    let (edge_right, edge_bottom) = crop_end(edge_origin, preview_size);
    assert_eq!(
        (edge_right, edge_bottom),
        (u32::from(width), u32::from(height))
    );

    // Сначала сравниваются первые поля. Площадь кадра считаем отдельно.
    let wide = (1_280, 720);
    let tall = (1_024, 1_080);
    assert!(wide > tall);
    let (wide_pixels, _) = rgb_frame_metrics(wide);
    let (tall_pixels, _) = rgb_frame_metrics(tall);
    assert!(wide_pixels < tall_pixels);
    let same_width = (1_280, 960);
    assert!(wide < same_width);

    // Запятая задаёт кортеж из одного поля.
    let frame_number: (u32,) = (1,);
    let (number,) = frame_number;

    println!("Кадр {number} — {width}×{height}, субтитры — {subtitles}");
    println!("Полный кадр — {full_pixels} пикселей и {full_rgb_bytes} байт RGB");
    println!(
        "Фрагмент {crop:?} — x={left}..{right}, y={top}..{bottom}, {preview_rgb_bytes} байт RGB"
    );
    println!("Фрагмент у края — правая граница {edge_right}, нижняя граница {edge_bottom}");
    println!(
        "Сравнение размеров — {wide:?} > {tall:?}: {}, площади: {wide_pixels} и {tall_pixels}",
        wide > tall
    );
    println!(
        "При равной ширине — {wide:?} < {same_width:?}: {}",
        wide < same_width
    );

    // Четвёртого поля в этом кортеже нет.
    // let _ = frame.3;

    // При распаковке нужно учесть все три поля.
    // let (width, height) = frame;

    // Третье поле имеет тип bool.
    // frame.2 = 1_u8;

    // Без запятой скобки группируют число.
    // let number: (u32,) = (1_u32);
}

fn rgb_frame_metrics(size: (u16, u16)) -> (u32, u64) {
    let (width, height) = size;
    let pixels = u32::from(width) * u32::from(height);
    let rgb_bytes = u64::from(pixels) * 3;
    (pixels, rgb_bytes)
}

// Правая и нижняя границы исключены из фрагмента.
fn crop_end(origin: (u16, u16), size: (u16, u16)) -> (u32, u32) {
    let (left, top) = origin;
    let (width, height) = size;
    (
        u32::from(left) + u32::from(width),
        u32::from(top) + u32::from(height),
    )
}
