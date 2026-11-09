/*
cargo run --offline --bin function_composition
*/

/*
Диспетчер выбирает порядок рейсов по сроку, доле загрузки или номеру.
Время подготовки состоит из постоянной настройки и округлённого вверх
времени загрузки. Один расчёт применяют к нескольким партиям, сохраняя
результаты в общем черновике. После утверждения черновик передаётся
в отчёт целиком.
*/

use std::cmp::Ordering;
use std::num::NonZeroU16;

#[derive(Clone, Copy)]
struct Trip {
    number: u32,
    due_minute: u16,
    cartons: u16,
    capacity: NonZeroU16,
}

type TripOrder = fn(&Trip, &Trip) -> Ordering;

fn by_deadline(left: &Trip, right: &Trip) -> Ordering {
    left.due_minute
        .cmp(&right.due_minute)
        .then_with(|| left.number.cmp(&right.number))
}

fn by_fill_ratio(left: &Trip, right: &Trip) -> Ordering {
    // Сравниваем доли загрузки перекрёстным умножением. Вместимость всегда положительна.
    let left_scaled = u32::from(left.cartons) * u32::from(right.capacity.get());
    let right_scaled = u32::from(right.cartons) * u32::from(left.capacity.get());
    right_scaled
        .cmp(&left_scaled)
        .then_with(|| left.number.cmp(&right.number))
}

fn choose_order(key: &str) -> Option<TripOrder> {
    // Именованные функции и замыкание без захватов приводятся к одному типу fn.
    match key {
        "deadline" => Some(by_deadline),
        "fill" => Some(by_fill_ratio),
        "number" => Some(|left, right| left.number.cmp(&right.number)),
        _ => None,
    }
}

struct Gate {
    number: u16,
    position: (i32, i32),
}

fn nearest_gates(gates: &[Gate], dock: (i32, i32), limit: usize) -> Vec<u16> {
    let mut ordered: Vec<_> = gates.iter().collect();
    ordered.sort_by_key(|gate| {
        // Сначала расширяем координаты, чтобы вычитание и квадраты были точными.
        let dx = i128::from(gate.position.0) - i128::from(dock.0);
        let dy = i128::from(gate.position.1) - i128::from(dock.1);
        (dx * dx + dy * dy, gate.number)
    });
    ordered
        .into_iter()
        .take(limit)
        .map(|gate| gate.number)
        .collect()
}

fn largest_number(numbers: &[u32]) -> String {
    let mut parts: Vec<_> = numbers.iter().map(u32::to_string).collect();
    parts.sort_unstable_by(|left, right| {
        // Сравниваем два порядка склейки, читая байты без промежуточных строк.
        right
            .bytes()
            .chain(left.bytes())
            .cmp(left.bytes().chain(right.bytes()))
    });
    if parts.first().is_some_and(|part| part == "0") {
        return String::from("0");
    }
    parts.concat()
}

fn sorting_exercises() {
    let gates = [
        Gate {
            number: 101,
            position: (5, 1),
        },
        Gate {
            number: 103,
            position: (0, 1),
        },
        Gate {
            number: 102,
            position: (2, 3),
        },
        Gate {
            number: 104,
            position: (-1, 4),
        },
    ];
    let nearest = nearest_gates(&gates, (2, 1), 2);
    assert_eq!(nearest, [102, 103]);
    assert_eq!(nearest_gates(&gates, (5, 1), 1), [101]);
    assert_eq!(nearest_gates(&gates, (2, 1), 9), [102, 103, 101, 104]);
    assert!(nearest_gates(&gates, (2, 1), 0).is_empty());
    assert!(nearest_gates(&[], (2, 1), 2).is_empty());
    let edge = [Gate {
        number: 1,
        position: (i32::MAX, i32::MIN),
    }];
    assert_eq!(nearest_gates(&edge, (i32::MIN, i32::MAX), 1), [1]);
    println!("Ближайшие ворота — к доку (2, 1) {nearest:?}");

    let combined = largest_number(&[3, 30, 34, 5, 9]);
    assert_eq!(combined, "9534330");
    assert_eq!(largest_number(&[12, 121]), "12121");
    assert_eq!(largest_number(&[1212, 12]), "121212");
    assert_eq!(largest_number(&[0, 0]), "0");
    assert_eq!(largest_number(&[0, 10]), "100");
    assert_eq!(largest_number(&[]), "");
    println!("Склейка — максимальное число {combined}");
}

fn compose<A, B, C>(first: impl Fn(A) -> B, second: impl Fn(B) -> C) -> impl Fn(A) -> C {
    // Возвращаемое замыкание владеет обеими операциями и вызывает их в заданном порядке.
    move |value| second(first(value))
}

fn loading_estimator(cartons_per_minute: NonZeroU16, setup_minutes: u32) -> impl Fn(u16) -> u64 {
    let load = move |cartons: u16| cartons.div_ceil(cartons_per_minute.get());
    let add_setup = move |minutes: u16| u64::from(minutes) + u64::from(setup_minutes);
    compose(load, add_setup)
}

fn compose_steps<'a>(stages: &'a [&'a dyn Fn(u64) -> u64]) -> impl Fn(u64) -> u64 + 'a {
    // В записи композиции сначала применяется крайняя правая операция.
    move |value| stages.iter().rev().fold(value, |input, stage| stage(input))
}

fn steps_demo() {
    let setup = 5;
    let factor = 2;
    let add_setup = |minutes| minutes + setup;
    let scale = |minutes| minutes * factor;
    let stages: [&dyn Fn(u64) -> u64; 2] = [&add_setup, &scale];
    let pipeline = compose_steps(&stages);
    let identity = compose_steps(&[]);
    assert_eq!(pipeline(4), 13);
    assert_eq!(pipeline(6), 17);
    assert_eq!(compose_steps(&[&scale, &add_setup])(4), 18);
    assert_eq!(identity(4), 4);
    println!(
        "Цепочка операций — для 4 минут {}, для 6 минут {}, без операций {}",
        pipeline(4),
        pipeline(6),
        identity(4)
    );
}

fn report_batch(
    trips: &[Trip],
    estimate: &impl Fn(u16) -> u64,
    record: &mut impl FnMut(u32, u64),
) -> u128 {
    let mut total = 0;
    for trip in trips {
        let minutes = estimate(trip.cartons);
        record(trip.number, minutes);
        total += u128::from(minutes);
    }
    total
}

#[derive(Debug, PartialEq)]
struct Manifest {
    rows: Vec<(u32, u64)>,
    total_minutes: u128,
}

fn approve_export(approved: bool, finish: impl FnOnce() -> Manifest) -> Option<Manifest> {
    // finish может забрать черновик из своего окружения, поэтому достаточно одного вызова.
    if approved { Some(finish()) } else { None }
}

fn main() {
    let trips = [
        Trip {
            number: 812,
            due_minute: 9,
            cartons: 4,
            capacity: NonZeroU16::new(8).expect("Вместимость задана"),
        },
        Trip {
            number: 813,
            due_minute: 9,
            cartons: 4,
            capacity: NonZeroU16::new(5).expect("Вместимость задана"),
        },
        Trip {
            number: 810,
            due_minute: 11,
            cartons: 3,
            capacity: NonZeroU16::new(6).expect("Вместимость задана"),
        },
    ];

    for (key, label, expected) in [
        ("deadline", "по сроку", [812, 813, 810]),
        ("number", "по номеру", [810, 812, 813]),
    ] {
        let mut checked = trips;
        let compare = choose_order(key).expect("В примере указан известный порядок");
        checked.sort_by(compare);
        let numbers = checked.map(|trip| trip.number);
        assert_eq!(numbers, expected);
        println!("Выбор порядка — {label} {numbers:?}");
    }
    assert!(choose_order("unknown").is_none());

    // Это замыкание использует trips из окружения. Указатель fn хранит только вызываемый код.
    // let _pointer: TripOrder = |left, right| (left.number == trips[0].number).cmp(&(right.number == trips[0].number));

    let mut scheduled = trips;
    let compare = choose_order("fill").expect("В примере задан порядок по загрузке");
    scheduled.sort_by(compare);
    let numbers = scheduled.map(|trip| trip.number);
    assert_eq!(numbers, [813, 810, 812]);
    println!("Рейсы по доле загрузки — {numbers:?}");
    sorting_exercises();

    let estimate = loading_estimator(NonZeroU16::new(3).expect("Скорость задана"), 5);
    let express_estimate = loading_estimator(NonZeroU16::new(4).expect("Скорость задана"), 2);
    assert_eq!((estimate(0), estimate(3), estimate(4)), (5, 6, 7));
    assert_eq!((express_estimate(4), express_estimate(5)), (3, 4));
    println!(
        "Время подготовки — для 4 коробов обычное {} мин, ускоренное {} мин",
        estimate(4),
        express_estimate(4)
    );
    steps_demo();

    let mut rows = Vec::new();
    let total_minutes = {
        // Одна привязка FnMut сохраняет общий черновик между двумя партиями.
        let mut record = |number, minutes| rows.push((number, minutes));
        // let _read_only: &dyn Fn(u32, u64) = &record;
        let first_batch = report_batch(&scheduled[..2], &estimate, &mut record);
        let second_batch = report_batch(&scheduled[2..], &estimate, &mut record);
        assert_eq!((first_batch, second_batch), (13, 7));
        println!("Партии — первая {first_batch} мин, вторая {second_batch} мин");
        first_batch + second_batch
    };
    assert_eq!(rows, [(813, 7), (810, 6), (812, 7)]);

    let skipped = approve_export(false, || panic!("Экспорт не утверждён"));
    assert!(skipped.is_none());
    println!("Экспорт отклонён — отчёт {skipped:?}");
    // finish переносит Vec в Manifest. Из-за этого замыкание можно вызвать только один раз.
    let finish = || Manifest {
        rows,
        total_minutes,
    };
    let report = approve_export(true, finish);
    // let _ = finish();
    assert_eq!(
        report,
        Some(Manifest {
            rows: vec![(813, 7), (810, 6), (812, 7)],
            total_minutes: 20
        })
    );
    if let Some(report) = report {
        println!(
            "Отчёт утверждён — рейсов {}, минут подготовки {}",
            report.rows.len(),
            report.total_minutes
        );
    }
}
