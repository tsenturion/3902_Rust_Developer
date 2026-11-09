/*
cargo run --offline --bin slices
*/

const HEADER_BYTES: usize = 2;
const CHECKSUM_BYTES: usize = 1;

/*
Транспортный буфер содержит пакет предпросмотра между двумя служебными байтами.
В пакете идут номер устройства, количество пикселей, значения яркости
и контрольный байт. Для предпросмотра ограничить яркость до 200 и записать
сумму скорректированных значений по модулю 256. Служебные байты буфера
должны сохраниться. Для передачи скопировать выбранное окно из двух пикселей.
*/
fn main() {
    let mut buffer = [0xAA_u8, 7, 4, 12, 180, 250, 90, 0, 0x55];
    let packet_start = 1;
    let packet_end = buffer.len() - 1;
    let packet: &[u8] = &buffer[packet_start..packet_end];
    let (header, body) = packet.split_at(HEADER_BYTES);
    let pixels = &body[..body.len() - CHECKSUM_BYTES];
    assert_eq!(header, &[7, 4]);
    assert_eq!(pixels.len(), 4);

    // Для записи нужен изменяемый слайс.
    // pixels[0] = 99;

    // pixels ещё используется для чтения.
    // adjust_preview(&mut buffer[packet_start..packet_end], 200);

    assert_eq!(&pixels[..2], &[12, 180]);
    assert_eq!(&pixels[2..], &[250, 90]);
    assert_eq!(&pixels[1..3], &[180, 250]);
    assert_eq!(&pixels[1..=2], &[180, 250]);
    let empty_window = &pixels[2..2];
    assert!(empty_window.is_empty());
    assert_eq!(pixel_window(pixels, 3, usize::MAX), &[90]);
    assert!(pixel_window(pixels, usize::MAX, usize::MAX).is_empty());
    println!("Окно до коррекции — {:?}", pixel_window(pixels, 1, 2));

    // Прямой диапазон за границей вызывает панику.
    // let invalid_end = pixels.len() + 1;
    // println!("{:?}", &pixels[..invalid_end]);

    // После чтения заимствуем только пакет и меняем его на месте.
    let packet: &mut [u8] = &mut buffer[packet_start..packet_end];
    assert!(adjust_preview(packet, 200));
    assert_eq!(&buffer[..], &[0xAA, 7, 4, 12, 180, 200, 90, 226, 0x55]);
    let adjusted_packet = &buffer[packet_start..packet_end];
    println!("Пакет после коррекции — {adjusted_packet:?}");

    let adjusted_pixels = &adjusted_packet[HEADER_BYTES..adjusted_packet.len() - CHECKSUM_BYTES];
    let mut transmission = [0_u8, 0];
    let destination = &mut transmission[..];
    let window = pixel_window(adjusted_pixels, 1, destination.len());
    assert_eq!(window.len(), destination.len());
    destination.copy_from_slice(window);
    assert_eq!(transmission, [180, 200]);
    println!("Окно для передачи — {transmission:?}");

    // У края копируем только фактическую длину окна.
    let mut tail_transmission = [0_u8; 2];
    let tail_window = pixel_window(adjusted_pixels, 3, tail_transmission.len());
    let copied_pixels = tail_window.len();
    tail_transmission[..copied_pixels].copy_from_slice(tail_window);
    assert_eq!(copied_pixels, 1);
    assert_eq!(tail_transmission, [90, 0]);
    println!("Неполное окно — {tail_transmission:?}, подготовлено пикселей — {copied_pixels}");

    let mut truncated = [7_u8, 4];
    assert!(!adjust_preview(&mut truncated, 200));
    let mut wrong_length = [7_u8, 5, 12, 180, 250, 90, 0];
    let wrong_length_accepted = adjust_preview(&mut wrong_length, 200);
    assert!(!wrong_length_accepted);
    assert_eq!(wrong_length, [7, 5, 12, 180, 250, 90, 0]);
    println!("Некорректная длина — принят {wrong_length_accepted}, пакет — {wrong_length:?}");

    let mut empty_packet = [7_u8, 0, 255];
    assert!(adjust_preview(&mut empty_packet, 200));
    assert_eq!(empty_packet, [7, 0, 0]);
}

fn pixel_window(pixels: &[u8], start: usize, requested: usize) -> &[u8] {
    // Сначала ограничиваем начало, затем длину в доступном остатке.
    let start = start.min(pixels.len());
    let end = start + requested.min(pixels.len() - start);
    &pixels[start..end]
}

fn adjust_preview(packet: &mut [u8], limit: u8) -> bool {
    if packet.len() < HEADER_BYTES + CHECKSUM_BYTES {
        return false;
    }

    let pixel_count = usize::from(packet[1]);
    if pixel_count != packet.len() - HEADER_BYTES - CHECKSUM_BYTES {
        return false;
    }

    // Две независимые индексации требуют два изменяемых заимствования packet.
    // let pixels = &mut packet[HEADER_BYTES..HEADER_BYTES + pixel_count];
    // let checksum = &mut packet[HEADER_BYTES + pixel_count..];
    // pixels[0] = checksum[0];

    // split_at_mut даёт изменяемые слайсы для непересекающихся частей.
    let (_, body) = packet.split_at_mut(HEADER_BYTES);
    let (pixels, checksum) = body.split_at_mut(pixel_count);
    let mut sum = 0_u8;
    let mut index = 0;
    while index < pixels.len() {
        pixels[index] = pixels[index].min(limit);
        sum = sum.wrapping_add(pixels[index]);
        index += 1;
    }
    checksum[0] = sum;
    true
}
