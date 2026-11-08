/*
cargo run --offline --bin functions
*/

const MINUTES_PER_ACADEMIC_HOUR: u32 = 45;
const SECONDS_PER_MINUTE: u32 = 60;
const BYTES_PER_MEBIBYTE: u64 = 1_048_576;

fn main() {
    // При вызове передаём аргументы в порядке объявления параметров.
    let course_hours = 20;
    let recording_seconds = academic_hours_to_seconds(course_hours);
    let required_bytes = {
        let video_bytes = stream_size_bytes(recording_seconds, 300_000);
        let audio_bytes = stream_size_bytes(recording_seconds, 16_000);
        // Последнее выражение даёт значение всего блока.
        video_bytes + audio_bytes
    };
    let required_mib = required_mebibytes(required_bytes);
    let available_mib = 16_384;

    assert_eq!(recording_seconds, 54_000);
    assert_eq!(required_bytes, 17_064_000_000);
    assert_eq!(required_mib, 16_274);
    assert_eq!(stream_size_bytes(academic_hours_to_seconds(1), 1), 2_700);
    assert!(fits_on_drive(required_mib, available_mib));
    assert!(!fits_on_drive(required_mib, 16_000));

    print_estimate(course_hours, required_bytes, required_mib, available_mib);

    // Один расчёт проверяем на точной границе и при нехватке 1 МиБ.
    let smaller_drive_mib = required_mib - 1;
    println!("Граница вместимости для тех же записей:");
    println!(
        "Доступно {required_mib} МиБ — помещаются: {}",
        fits_on_drive(required_mib, required_mib)
    );
    println!(
        "Доступно {smaller_drive_mib} МиБ — помещаются: {}",
        fits_on_drive(required_mib, smaller_drive_mib)
    );

    // Неверное количество аргументов.
    // stream_size_bytes(recording_seconds);

    // Неверный тип второго аргумента.
    // stream_size_bytes(recording_seconds, 300_000.0);
}

// Функцию можно объявить ниже места вызова. Тип результата указан после ->.
fn academic_hours_to_seconds(hours: u16) -> u32 {
    u32::from(hours) * MINUTES_PER_ACADEMIC_HOUR * SECONDS_PER_MINUTE
}

fn stream_size_bytes(seconds: u32, bytes_per_second: u32) -> u64 {
    u64::from(seconds) * u64::from(bytes_per_second)
}

// Явный return передаёт значение вызывающему коду.
fn required_mebibytes(bytes: u64) -> u64 {
    return bytes.div_ceil(BYTES_PER_MEBIBYTE);
}

fn fits_on_drive(required_mib: u64, available_mib: u64) -> bool {
    required_mib <= available_mib
}

fn print_estimate(course_hours: u16, bytes: u64, required_mib: u64, available_mib: u64) {
    println!("Объём записей за {course_hours} ак. ч. — {bytes} байт");
    println!("Требуется {required_mib} МиБ, доступно {available_mib} МиБ");
    println!(
        "Записи помещаются — {}",
        fits_on_drive(required_mib, available_mib)
    );
}

// Для каждого параметра нужен тип.
// fn missing_parameter_type(hours) -> u16 {
//     hours
// }

// Точка с запятой превращает выражение в инструкцию.
// fn missing_result(hours: u16) -> u32 {
//     u32::from(hours) * 45 * 60;
// }
