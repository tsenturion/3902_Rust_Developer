/*
cargo run --offline --bin heap_memory
*/

/*
Терминал принимает пакет телеметрии по частям. Первые два байта содержат
длину полезных данных в формате little-endian. Терминал собирает фрагменты
в буфер и передаёт его обработчику. Для проверки можно сделать независимую
копию, а основной буфер после обработки использовать для следующего пакета.
*/

use std::collections::TryReserveError;
use std::mem::size_of_val;

struct Packet {
    number: u32,
    bytes: Vec<u8>,
}

fn append_fragment(buffer: &mut Vec<u8>, fragment: &[u8]) -> Result<(), TryReserveError> {
    // try_reserve принимает число дополнительных байтов относительно len.
    buffer.try_reserve(fragment.len())?;
    // При достаточной ёмкости push сохраняет выделенный блок.
    for &byte in fragment {
        buffer.push(byte);
    }
    Ok(())
}

fn payload(bytes: &[u8]) -> Option<&[u8]> {
    let [low, high, body @ ..] = bytes else {
        return None;
    };
    let declared = usize::from(u16::from_le_bytes([*low, *high]));
    (body.len() == declared).then_some(body)
}

fn main() {
    let first_fragment = [4_u8, 0, 10];
    let second_fragment = [20_u8, 30, 40];
    let mut input = Vec::<u8>::new();
    let owner_size = size_of_val(&input);
    // У массива байты входят в размер самого значения. Vec хранит служебные данные отдельно от буфера.
    assert_eq!(size_of_val(&first_fragment), first_fragment.len());
    assert_eq!((input.len(), input.capacity()), (0, 0));

    input.try_reserve(32).expect("Нужен начальный буфер пакета");
    assert_eq!(input.len(), 0);
    assert!(input.capacity() >= 32);
    let allocated = input.as_ptr();
    let initial_capacity = input.capacity();

    append_fragment(&mut input, &first_fragment).expect("Первый фрагмент помещается");
    assert_eq!(payload(&input), None);
    append_fragment(&mut input, &second_fragment).expect("Второй фрагмент помещается");
    assert_eq!(input.as_ptr(), allocated);
    assert_eq!(input.len(), 6);
    assert_eq!(input.capacity(), initial_capacity);
    assert_eq!(size_of_val(&input), owner_size);

    // Перемещаем владельца в пакет. Выделение с байтами остаётся тем же.
    let mut packet = Packet {
        number: 17,
        bytes: input,
    };
    assert_eq!(packet.bytes.as_ptr(), allocated);
    // Владение уже передано полю bytes.
    // let _ = input.len();

    let Some(body) = payload(&packet.bytes) else {
        panic!("Пакет примера должен содержать четыре байта данных");
    };
    // packet.bytes.clear();
    assert_eq!(body, [10, 20, 30, 40]);
    assert_eq!(body.as_ptr(), packet.bytes[2..].as_ptr());
    println!(
        "Пакет {} — размер Vec в байтах {}, длина буфера {}, ёмкость {}",
        packet.number,
        owner_size,
        packet.bytes.len(),
        packet.bytes.capacity()
    );
    let address_preserved = packet.bytes.as_ptr() == allocated;
    println!("Передача пакета — тот же адрес {address_preserved}, полезные данные {body:?}");

    // Для непустого Vec<u8> clone выделяет отдельный буфер с тем же содержимым.
    let mut sample = packet.bytes.clone();
    assert_eq!(sample, packet.bytes);
    assert_ne!(sample.as_ptr(), packet.bytes.as_ptr());
    sample[2] = 99;
    let changed = payload(&sample).expect("Копия сохранила заголовок пакета");
    assert_eq!(changed, [99, 20, 30, 40]);
    assert_eq!(body, [10, 20, 30, 40]);
    println!("Копия пакета — исходные данные {body:?}, изменённая копия {changed:?}");

    // clear удаляет элементы. Ёмкость и адрес выделенного буфера сохраняются.
    packet.bytes.clear();
    packet.number = 18;
    assert!(packet.bytes.is_empty());
    assert_eq!(packet.bytes.capacity(), initial_capacity);
    append_fragment(&mut packet.bytes, &[2, 0, 8, 9]).expect("Новый пакет помещается");
    assert_eq!(packet.bytes.as_ptr(), allocated);
    assert_eq!(payload(&packet.bytes), Some(&[8, 9][..]));
    println!("Пакет {} использовал тот же буфер", packet.number);

    // Просим на один байт больше свободного места. Аллокатор может расширить блок на месте.
    let additional = packet.bytes.capacity() - packet.bytes.len() + 1;
    packet
        .bytes
        .try_reserve(additional)
        .expect("Нужен запас для длинного пакета");
    assert!(packet.bytes.capacity() > initial_capacity);
    assert_eq!(payload(&packet.bytes), Some(&[8, 9][..]));
    let address_status = if packet.bytes.as_ptr() == allocated {
        "сохранился"
    } else {
        "изменился"
    };
    println!(
        "Ёмкость выросла с {initial_capacity} до {} байт. Адрес {address_status}",
        packet.bytes.capacity()
    );

    let capacity = packet.bytes.capacity();
    let address = packet.bytes.as_ptr();
    // len + usize::MAX переполняется. try_reserve вернёт ошибку до запроса к аллокатору.
    let rejected = packet.bytes.try_reserve(usize::MAX).is_err();
    assert!(rejected);
    assert_eq!(packet.bytes.capacity(), capacity);
    assert_eq!(packet.bytes.as_ptr(), address);
    assert_eq!(payload(&packet.bytes), Some(&[8, 9][..]));
    println!(
        "Ошибка резерва — запрос отклонён {rejected}, пакет сохранён {:?}",
        packet.bytes
    );

    let fixed = packet.bytes.into_boxed_slice();
    assert_eq!(fixed.as_ref(), [2, 0, 8, 9]);
    println!(
        "Фиксированный пакет — {} байта, данные {fixed:?}",
        size_of_val(fixed.as_ref())
    );
}
