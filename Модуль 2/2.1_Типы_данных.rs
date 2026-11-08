/*
cargo run --offline --bin data_types
*/

fn main() {
    // Аннотация, суффикс литерала и вывод типа из контекста.
    let default_integer = 42;
    let default_float = 0.5;
    let temperature: i16 = -12;
    let correction = 3;
    let corrected_temperature = temperature + correction;
    assert_eq!(corrected_temperature, -9_i16);

    println!(
        "Типы по умолчанию — {} и {}, поправка по контексту — {}",
        std::any::type_name_of_val(&default_integer),
        std::any::type_name_of_val(&default_float),
        std::any::type_name_of_val(&correction)
    );

    // Четыре записи одного целого значения.
    let decimal: u8 = 42;
    let binary = 0b0010_1010_u8;
    let octal = 0o52_u8;
    let hexadecimal = 0x2A_u8;
    assert_eq!(decimal, binary);
    assert_eq!(decimal, octal);
    assert_eq!(decimal, hexadecimal);
    println!("Четыре записи одного числа — {decimal}, {binary}, {octal}, {hexadecimal}");

    println!("\nЦелые числа, включительные границы");
    println!("i8 {}..={} | u8 0..={}", i8::MIN, i8::MAX, u8::MAX);
    println!("i16 {}..={} | u16 0..={}", i16::MIN, i16::MAX, u16::MAX);
    println!("i32 {}..={} | u32 0..={}", i32::MIN, i32::MAX, u32::MAX);
    println!("i64 {}..={} | u64 0..={}", i64::MIN, i64::MAX, u64::MAX);
    println!(
        "i128 {}..={} | u128 0..={}",
        i128::MIN,
        i128::MAX,
        u128::MAX
    );
    println!(
        "isize {}..={} | usize 0..={} | ширина {} бит",
        isize::MIN,
        isize::MAX,
        usize::MAX,
        usize::BITS
    );

    // Размер архива выходит за границу u32. Расширяем оба числа до умножения.
    let block_bytes: u16 = 4_096;
    let block_count: u32 = 1_500_000;
    let archive_bytes = u64::from(block_bytes) * u64::from(block_count);
    let beyond_u64 = u128::from(u64::MAX) + 1;
    let below_i64 = i128::from(i64::MIN) - 1;
    assert_eq!(archive_bytes, 6_144_000_000_u64);
    assert_eq!(beyond_u64, 18_446_744_073_709_551_616_u128);
    assert_eq!(below_i64, -9_223_372_036_854_775_809_i128);

    // Размер буфера и знаковое смещение имеют ширину платформы.
    let buffer_bytes: usize = 4_096 * 3;
    let offset_from_end: isize = -16;
    let data_position = buffer_bytes.saturating_add_signed(offset_from_end);
    assert_eq!(data_position, 12_272);
    println!("Архив — {archive_bytes} байт, позиция в буфере — {data_position}");

    // Целочисленное деление отбрасывает дробную часть, % даёт остаток.
    let full_blocks = 7_u32 / 2;
    let remaining_units = 7_u32 % 2;
    assert_eq!(full_blocks, 3);
    assert_eq!(remaining_units, 1);
    let negative_quotient = -7_i32 / 2;
    assert_eq!(negative_quotient, -3);
    println!(
        "Целочисленное деление — 7 / 2 = {full_blocks}, 7 % 2 = {remaining_units}, −7 / 2 = {negative_quotient}"
    );

    // Два явных способа обработать переполнение счётчика.
    let counter = u8::MAX;
    let wrapped_counter = counter.wrapping_add(1);
    let capped_counter = counter.saturating_add(1);
    assert_eq!(wrapped_counter, 0);
    assert_eq!(capped_counter, 255);
    assert_eq!(i8::MAX.wrapping_add(1), i8::MIN);
    println!(
        "После 255 — {wrapped_counter} с переходом через границу, {capped_counter} с насыщением"
    );

    // u16::from сохраняет значение u8. Сужение через as может отбросить часть данных.
    let widened = u16::from(250_u8);
    let narrowed = 300_u16 as u8;
    let unsigned = -1_i8 as u8;
    let integer_part = -12.75_f64 as i32;
    let clamped_float = 300.0_f64 as u8;
    assert_eq!(widened, 250);
    assert_eq!(narrowed, 44);
    assert_eq!(unsigned, 255);
    assert_eq!(integer_part, -12);
    assert_eq!(clamped_float, 255);
    println!("300_u16 в u8 — {narrowed}, −12.75 в i32 — {integer_part}");

    println!("\nЧисла с плавающей точкой");
    let f32_large = 16_777_216.0_f32;
    let f64_large = 16_777_216.0_f64;
    assert_eq!(f32_large + 1.0, f32_large);
    assert_eq!(f64_large + 1.0, 16_777_217.0);
    assert_eq!(7.0_f64 / 2.0, 3.5);
    assert_eq!(1.25e3_f64, 1_250.0);
    println!(
        "16 777 216 + 1 — {} в f32 и {} в f64",
        f32_large + 1.0,
        f64_large + 1.0
    );
    println!(
        "Максимальные конечные значения — {:e} и {:e}",
        f32::MAX,
        f64::MAX
    );

    let measured_sum = 0.1_f64 + 0.2;
    let deviation = (measured_sum - 0.3).abs();
    let tolerance = 1e-12_f64;
    assert!(deviation < tolerance);
    println!("0.1 + 0.2 = {measured_sum:.17} руб., отклонение {deviation:e} руб.");

    // Величины с фиксированным шагом можно считать в целых минимальных единицах.
    let first_price_kopecks = 10_u32;
    let second_price_kopecks = 20_u32;
    let total_price_kopecks = first_price_kopecks + second_price_kopecks;
    assert_eq!(total_price_kopecks, 30);
    println!("Точный расчёт в копейках — {total_price_kopecks}");

    let positive_infinity = f64::INFINITY;
    let negative_infinity = f64::NEG_INFINITY;
    let not_a_number = f64::NAN;
    let negative_zero = -0.0_f64;
    let subnormal = f32::MIN_POSITIVE / 2.0;
    assert!(positive_infinity.is_infinite());
    assert!(negative_infinity.is_infinite() && negative_infinity.is_sign_negative());
    assert!(not_a_number.is_nan());
    assert!(measured_sum.is_finite());
    assert_eq!(negative_zero, 0.0);
    assert!(negative_zero.is_sign_negative());
    assert!(subnormal.is_subnormal());
    println!(
        "Особые значения — бесконечность {positive_infinity}, NaN {not_a_number}, отрицательный ноль {negative_zero}"
    );

    // Сравнения и логические операции дают значения bool.
    let sensor_online: bool = true;
    let backup_sensor_online: bool = false;
    let temperature_in_range = corrected_temperature >= -30 && corrected_temperature <= 60;
    let reading_accepted = sensor_online && temperature_in_range;
    let warning = !sensor_online || !temperature_in_range;
    assert!(reading_accepted);
    assert!(!warning);
    assert!(!(backup_sensor_online && temperature_in_range));
    assert_eq!(u8::from(reading_accepted), 1);
    assert_eq!(u8::from(backup_sensor_online), 0);
    println!("\nИзмерение принято — {reading_accepted}, предупреждение — {warning}");

    // Один char хранит скалярное значение Unicode. В UTF-8 длина зависит от символа.
    let latin: char = 'A';
    let cyrillic = 'Я';
    let euro = '€';
    let crab = '\u{1F980}';
    let latin_byte: u8 = b'A';
    assert_eq!(crab, '🦀');
    assert_eq!(latin_byte, 65);
    assert_eq!(u32::from(char::MAX), 0x10_FFFF);
    assert_eq!(std::mem::size_of_val(&cyrillic), 4);
    assert_eq!(latin.len_utf8(), 1);
    assert_eq!(cyrillic.len_utf8(), 2);
    assert_eq!(euro.len_utf8(), 3);
    assert_eq!(crab.len_utf8(), 4);
    assert!('\n'.is_whitespace());
    println!(
        "\n{cyrillic} — U+{:04X}, {} байта UTF-8, {crab} — U+{:04X}, {} байта UTF-8",
        u32::from(cyrillic),
        cyrillic.len_utf8(),
        u32::from(crab),
        crab.len_utf8()
    );
    println!(
        "Буква {latin} как char занимает {} байта, байтовый литерал b'A' даёт {latin_byte}",
        std::mem::size_of_val(&latin)
    );

    // Примеры строк, которые компилятор отклоняет.
    // let _: u8 = 256;
    // let _: u16 = 42_u8;
    // let _: bool = 1;
    // let _: char = 'AB';
}
