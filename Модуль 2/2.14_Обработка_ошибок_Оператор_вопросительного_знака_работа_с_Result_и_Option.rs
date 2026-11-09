/*
cargo run --offline --bin error_handling
*/

/*
Терминал приёмки получает этикетки вида «IN:ячейка;количество;примечание».
Примечание необязательно. Ячейки пронумерованы с единицы, количество должно
быть положительным. Отклонённая этикетка не меняет остатки и не мешает
обработать следующую строку. После отказа оператор видит его причину.
*/

use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
enum Field {
    Cell,
    Quantity,
}

#[derive(Debug, PartialEq)]
enum ReceiveError {
    InvalidLabel {
        text: String,
    },
    InvalidNumber {
        field: Field,
        text: String,
        source: ParseIntError,
    },
    UnknownCell {
        cell: u16,
    },
    EmptyDelivery,
    StockLimit {
        cell: u16,
        current: u16,
        incoming: u16,
    },
}

#[derive(Debug, PartialEq)]
struct Delivery {
    cell: u16,
    quantity: u16,
    note: Option<String>,
}

#[derive(Debug, PartialEq)]
struct Receipt {
    delivery: Delivery,
    on_hand: u16,
}

fn split_label(raw: &str) -> Option<(&str, &str, Option<&str>)> {
    // В функции с Option оператор ? извлекает содержимое Some или сразу возвращает None.
    let body = raw.trim().strip_prefix("IN:")?;
    let (cell, rest) = body.split_once(';')?;
    let (quantity, note) = match rest.split_once(';') {
        Some((quantity, note)) => (quantity, Some(note.trim()).filter(|text| !text.is_empty())),
        None => (rest, None),
    };
    Some((cell.trim(), quantity.trim(), note))
}

fn read_number(text: &str, field: Field) -> Result<u16, ReceiveError> {
    // map_err меняет только Err. В ошибке остаются поле, исходный текст и причина от parse.
    text.parse().map_err(|source| ReceiveError::InvalidNumber {
        field,
        text: text.to_owned(),
        source,
    })
}

fn parse_label(raw: &str) -> Result<Delivery, ReceiveError> {
    // Для Result сначала превращаем отсутствие частей этикетки в конкретную ошибку.
    // ok_or_else создаёт строку с исходной этикеткой только при None.
    let (cell, quantity, note) = split_label(raw).ok_or_else(|| ReceiveError::InvalidLabel {
        text: raw.to_owned(),
    })?;

    // ? извлекает значение из Ok или возвращает Err из parse_label до следующих проверок.
    let cell = read_number(cell, Field::Cell)?;
    let quantity = read_number(quantity, Field::Quantity)?;
    if quantity == 0 {
        return Err(ReceiveError::EmptyDelivery);
    }

    // У parse_label результат типа Result, поэтому None надо превратить в Err.
    // let _ = split_label(raw)?;
    Ok(Delivery {
        cell,
        quantity,
        note: note.map(|text| text.to_owned()),
    })
}

struct Warehouse {
    on_hand: [u16; 3],
}

impl Warehouse {
    fn receive(&mut self, label: &str) -> Result<Receipt, ReceiveError> {
        let delivery = parse_label(label)?;
        let cell = delivery.cell;
        // and_then ищет ячейку только если номер удалось превратить в индекс.
        let balance = cell
            .checked_sub(1)
            .and_then(|index| self.on_hand.get_mut(usize::from(index)))
            .ok_or(ReceiveError::UnknownCell { cell })?;

        // checked_add даёт None при переполнении. Здесь достаточно готовой ошибки в ok_or.
        let updated = balance
            .checked_add(delivery.quantity)
            .ok_or(ReceiveError::StockLimit {
                cell,
                current: *balance,
                incoming: delivery.quantity,
            })?;
        // Все проверки пройдены. До этой строки ни один путь с ошибкой не меняет остаток.
        *balance = updated;
        Ok(Receipt {
            delivery,
            on_hand: updated,
        })
    }
}

fn describe_error(error: &ReceiveError) -> String {
    match error {
        ReceiveError::InvalidLabel { text } => format!("неверный формат этикетки «{text}»"),
        ReceiveError::InvalidNumber {
            field,
            text,
            source,
        } => {
            let name = match field {
                Field::Cell => "ячейка",
                Field::Quantity => "количество",
            };
            format!("в поле «{name}» неверное число «{text}» ({source})")
        }
        ReceiveError::UnknownCell { cell } => format!("ячейки {cell} нет на складе"),
        ReceiveError::EmptyDelivery => String::from("количество должно быть больше нуля"),
        ReceiveError::StockLimit {
            cell,
            current,
            incoming,
        } => {
            format!("сумма {current} и {incoming} превышает предел учёта ячейки {cell}")
        }
    }
}

fn main() {
    let mut warehouse = Warehouse {
        on_hand: [8, u16::MAX - 2, 0],
    };

    assert_eq!(split_label("IN:3"), None);
    assert_eq!(split_label("OUT:3;4"), None);
    assert_eq!(split_label("IN:3;4;  "), Some(("3", "4", None)));
    assert_eq!(
        split_label(" IN:1; 12; партия А7 "),
        Some(("1", "12", Some("партия А7")))
    );
    assert_eq!(
        parse_label("IN:3"),
        Err(ReceiveError::InvalidLabel {
            text: String::from("IN:3")
        })
    );
    assert!(matches!(
        parse_label("IN:X;3"),
        Err(ReceiveError::InvalidNumber {
            field: Field::Cell,
            ..
        })
    ));
    assert!(matches!(
        parse_label("IN:1;-1"),
        Err(ReceiveError::InvalidNumber {
            field: Field::Quantity,
            ..
        })
    ));
    assert_eq!(
        warehouse.receive("IN:0;2"),
        Err(ReceiveError::UnknownCell { cell: 0 })
    );
    assert_eq!(
        warehouse.receive("IN:4;2"),
        Err(ReceiveError::UnknownCell { cell: 4 })
    );
    assert_eq!(
        warehouse.receive("IN:1;0"),
        Err(ReceiveError::EmptyDelivery)
    );
    assert_eq!(warehouse.on_hand, [8, u16::MAX - 2, 0]);

    let labels = [
        "IN:1;12;партия А7",
        "IN:1;много",
        "IN:2;3",
        "IN:3;4",
        "IN:2;2;  ",
    ];
    let mut accepted = 0;
    let mut rejected = 0;
    for label in labels {
        let before = warehouse.on_hand;
        match warehouse.receive(label) {
            Ok(receipt) => {
                let delivery = &receipt.delivery;
                // as_deref даёт Option<&str>. Запасной текст нужен только для квитанции.
                let note = delivery.note.as_deref().unwrap_or("без примечания");
                println!(
                    "Ячейка {} — принято {} ед., остаток {}, {note}",
                    delivery.cell, delivery.quantity, receipt.on_hand
                );
                accepted += 1;
            }
            Err(error) => {
                assert_eq!(warehouse.on_hand, before);
                println!("Этикетка отклонена — {}", describe_error(&error));
                rejected += 1;
            }
        }
    }
    assert_eq!((accepted, rejected), (3, 2));
    assert_eq!(warehouse.on_hand, [20, u16::MAX, 4]);
    println!("Принято этикеток — {accepted}, отклонено — {rejected}");

    // main возвращает (). Для обработки сбоя здесь используется match.
    // let _ = warehouse.receive("IN:1;3")?;
}
