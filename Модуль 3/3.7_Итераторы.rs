/*
cargo run --offline --bin collection_iterators
*/

/*
Склад обходит список талонов, проверяет вес коробов, сверяет загрузку
с планом и разбивает остаток груза на рейсы по вместимости машины.
*/

use std::collections::{BTreeMap, HashSet};
use std::iter::{FusedIterator, repeat_n};

fn cursor_demo() {
    let weights = [120_u16, 80, 150];
    let mut cursor = weights.iter();
    let first: Option<&u16> = cursor.next();
    let second = cursor.next();
    let third = cursor.next();
    let end = cursor.next();
    let repeated_end = cursor.next();
    // next сдвигает курсор. Для итератора слайса конец остаётся концом.
    assert_eq!(
        [first, second, third, end, repeated_end],
        [Some(&120), Some(&80), Some(&150), None, None]
    );

    println!("Курсор — {first:?}, {second:?}, {third:?}, {end:?}, {repeated_end:?}");
}

fn access_demo() {
    let mut labels = vec![String::from("К-101"), String::from("К-102")];
    let incoming = labels.iter();
    let mut received = 0;
    for label in incoming {
        if label.starts_with("К-") {
            received += 1;
        }
    }

    // iter даёт &String, iter_mut даёт &mut String, into_iter передаёт сами String.
    assert_eq!(received, 2);
    for label in labels.iter_mut() {
        label.push_str("-Г");
    }
    assert_eq!(labels, ["К-101-Г", "К-102-Г"]);

    let mut outgoing = labels.into_iter();
    let first: String = outgoing.next().expect("В очереди есть первый талон");
    let second: String = outgoing.next().expect("В очереди есть второй талон");
    assert_eq!((first.as_str(), second.as_str()), ("К-101-Г", "К-102-Г"));
    assert_eq!(outgoing.next(), None);
    println!("Доступ — прочитано {received}, переданы {first} и {second}");
}

fn pipeline_demo() {
    let arrivals = [("К-101", 120_u16), ("К-102", 80), ("К-103", 150)];
    let mut checked = 0;
    let selection = arrivals
        .iter()
        .inspect(|(code, _)| {
            checked += 1;
            println!("Конвейер — проверяем {code}");
        })
        .filter(|(_, weight)| *weight >= 100)
        .map(|(code, _)| *code);

    // До collect ни inspect, ни filter, ни map не запрашивают элементы.
    println!("Конвейер — цепочка создана");
    let selected: Vec<&str> = selection.collect();
    assert_eq!(checked, 3);
    assert_eq!(selected, ["К-101", "К-103"]);
    println!("Конвейер — отобраны {selected:?}");
}

fn search_demo() {
    let mut weights = [120_u16, 80, 150].into_iter().peekable();
    let mut checked = 0;
    let found = weights.find(|&weight| {
        checked += 1;
        weight <= 100
    });
    let preview = weights.peek().copied();
    let next = weights.next();
    // find оставляет курсор после совпадения, peek показывает следующий элемент.
    assert_eq!(
        (checked, found, preview, next),
        (2, Some(80), Some(150), Some(150))
    );
    assert_eq!(weights.next(), None);

    println!(
        "Поиск — проверено {checked}, найдено {found:?}, просмотр {preview:?}, следующий {next:?}"
    );
}

fn summary_demo() {
    let readings = ["120", "н/д", "80", "150"];
    let (count, total) = readings
        .into_iter()
        .filter_map(|reading| reading.parse::<u16>().ok())
        .fold((0, 0_u32), |(count, total), weight| {
            (count + 1, total + u32::from(weight))
        });
    // Мягкий режим пропускает неверный замер, строгий завершает импорт на первой ошибке.
    let mut attempted = 0;
    let strict: Result<Vec<u16>, _> = readings
        .into_iter()
        .inspect(|_| attempted += 1)
        .map(str::parse::<u16>)
        .collect();
    assert_eq!((count, total), (3, 350));
    assert!(strict.is_err());
    assert_eq!(attempted, 2);
    let valid: Result<Vec<u16>, _> = ["120", "80"].into_iter().map(str::parse::<u16>).collect();
    assert_eq!(valid, Ok(vec![120, 80]));
    let strict_status = if strict.is_ok() {
        "принят"
    } else {
        "отклонён"
    };

    println!("Сводка — замеров {count}, общий вес {total} кг, строгий импорт {strict_status}");
}

fn ends_demo() {
    let weights = [120_u16, 80, 150, 110];
    let mut remaining = weights.iter().copied();
    let first = remaining.next();
    let last = remaining.next_back();
    let left = remaining.len();
    let reverse: Vec<u16> = remaining.rev().take(2).collect();
    assert_eq!((first, last, left), (Some(120), Some(110), 2));
    assert_eq!(reverse, [150, 80]);

    println!(
        "Границы — первый {first:?}, последний {last:?}, осталось {left}, с конца {reverse:?}"
    );
}

fn pairs_demo() {
    let labels = ["К-101", "К-102", "К-103"];
    let docks = [1_u8, 2];
    let pairs: Vec<(&str, u8)> = labels.into_iter().zip(docks).collect();
    let unassigned = &labels[pairs.len()..];
    // zip заканчивает работу, когда заканчивается один из входов.
    assert_eq!(pairs, [("К-101", 1), ("К-102", 2)]);
    assert_eq!(unassigned, ["К-103"]);

    println!("Пары — назначены {pairs:?}, без дока {unassigned:?}");
}

fn groups_demo() {
    let arrivals = [("К-101", 2_u8), ("К-102", 1), ("К-103", 2)];
    let mut by_zone = BTreeMap::new();
    for (position, (code, zone)) in arrivals.into_iter().enumerate() {
        by_zone
            .entry(zone)
            .or_insert_with(Vec::new)
            .push((position + 1, code));
    }
    assert_eq!(
        by_zone,
        BTreeMap::from([
            (1, vec![(2, "К-102")]),
            (2, vec![(1, "К-101"), (3, "К-103")])
        ])
    );

    println!("Группы — {by_zone:?}");
}

fn reconciliation_demo() {
    let planned = HashSet::from([101_u16, 102, 103, 104]);
    let loaded = HashSet::from([101_u16, 103, 105]);
    let mut waiting: Vec<u16> = planned.difference(&loaded).copied().collect();
    waiting.sort_unstable();
    assert_eq!(waiting, [102, 104]);
    let unexpected: Vec<u16> = loaded.difference(&planned).copied().collect();
    assert_eq!(unexpected, [105]);

    println!("Сверка — ожидают погрузки {waiting:?}");
}

fn unpack_weights(encoded: &[u16]) -> Result<Vec<u16>, u16> {
    // В каждой паре сначала количество одинаковых коробов, затем их вес.
    let pairs = encoded.chunks_exact(2);
    if let Some(&unpaired) = pairs.remainder().first() {
        return Err(unpaired);
    }

    Ok(pairs
        .flat_map(|pair| repeat_n(pair[1], usize::from(pair[0])))
        .collect())
}

fn unpack_demo() {
    let packed = [2, 410, 0, 999, 3, 120];
    let unpacked = unpack_weights(&packed);
    let incomplete = unpack_weights(&[2, 410, 3]);
    assert_eq!(unpacked, Ok(vec![410, 410, 120, 120, 120]));
    assert_eq!(incomplete, Err(3));
    assert_eq!(unpack_weights(&[]), Ok(vec![]));
    assert_eq!(unpack_weights(&[0, 120]), Ok(vec![]));

    println!("Распаковка — полный пакет {unpacked:?}, обрыв {incomplete:?}");
}

struct LoadBatches {
    remaining: u32,
    capacity: u32,
}

impl LoadBatches {
    fn new(remaining: u32, capacity: u32) -> Option<Self> {
        if capacity == 0 {
            return None;
        }
        Some(Self {
            remaining,
            capacity,
        })
    }
}

impl Iterator for LoadBatches {
    // Каждый вызов next возвращает размер одного рейса и уменьшает остаток.
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let loaded = self.remaining.min(self.capacity);
        self.remaining -= loaded;
        Some(loaded)
    }
}

// После первого None этот итератор продолжает возвращать None.
impl FusedIterator for LoadBatches {}

fn batches_demo() {
    let mut trips = LoadBatches::new(11, 4).expect("У машины есть вместимость");
    // by_ref позволяет collect исчерпать итератор и сохранить переменную для проверки.
    let sizes: Vec<u32> = trips.by_ref().collect();
    let end = trips.next();
    let repeated_end = trips.next();
    let zero_capacity = LoadBatches::new(11, 0).is_none();
    assert_eq!(sizes, [4, 4, 3]);
    assert_eq!((end, repeated_end), (None, None));
    assert!(zero_capacity);
    let empty: Vec<u32> = LoadBatches::new(0, 4)
        .expect("У машины есть вместимость")
        .collect();
    assert!(empty.is_empty());

    println!(
        "Рейсы — короба {sizes:?}, конец {end:?} и {repeated_end:?}, нулевая вместимость отклонена {zero_capacity}"
    );
}

fn main() {
    cursor_demo();
    access_demo();
    pipeline_demo();
    search_demo();
    summary_demo();
    ends_demo();
    pairs_demo();
    groups_demo();
    reconciliation_demo();
    unpack_demo();
    batches_demo();
}
