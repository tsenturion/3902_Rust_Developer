/*
cargo run --offline --bin move_and_arc
*/

/*
Склад рассчитывает потребность в коробах для нескольких партий. Для каждой
строки число коробов округляется вверх. Справочник вместимости общий
для исполнителей, данные партии принадлежат своему
заданию. При неизвестном артикуле расчёт партии отклоняется. Исправленную
партию можно обработать с той же версией справочника.
*/

use std::collections::BTreeMap;
use std::io;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

struct PackingRules {
    version: u32,
    units_per_carton: BTreeMap<u32, NonZeroU16>,
}

struct Line {
    sku: u32,
    units: u16,
}

struct Batch {
    number: u32,
    lines: Vec<Line>,
}

#[derive(Debug, PartialEq, Eq)]
struct Plan {
    batch: u32,
    rules_version: u32,
    cartons: u128,
}

#[derive(Debug, PartialEq, Eq)]
enum PlanError {
    UnknownSku { sku: u32 },
}

type PlanResult = Result<Plan, PlanError>;

fn plan_batch(rules: &PackingRules, batch: &Batch) -> PlanResult {
    let mut cartons = 0;
    for line in &batch.lines {
        let capacity = rules
            .units_per_carton
            .get(&line.sku)
            .ok_or(PlanError::UnknownSku { sku: line.sku })?;
        cartons += u128::from(line.units.div_ceil(capacity.get()));
    }
    Ok(Plan {
        batch: batch.number,
        rules_version: rules.version,
        cartons,
    })
}

fn start_batch(rules: &Arc<PackingRules>, batch: Batch) -> io::Result<JoinHandle<PlanResult>> {
    // Клонируем владельца до запуска. Обе ссылки указывают на один справочник.
    let rules_for_worker = Arc::clone(rules);
    let shared = Arc::ptr_eq(rules, &rules_for_worker);
    assert!(shared);
    println!(
        "Справочник партии {} — версия {}, то же выделение {shared}",
        batch.number, rules.version
    );

    thread::Builder::new()
        .name(format!("партия-{}", batch.number))
        // move передаёт замыканию его Arc и Batch. Внутри данные можно заимствовать.
        .spawn(move || plan_batch(&rules_for_worker, &batch))
}

fn shared_rule_demo() {
    let limit = 4_u16;
    let fits: Arc<dyn Fn(u16) -> bool + Send + Sync> = Arc::new(move |cartons| cartons <= limit);
    let first_rule = Arc::clone(&fits);
    let second_rule = Arc::clone(&fits);
    let first = thread::spawn(move || first_rule(4));
    let second = thread::spawn(move || second_rule(5));
    let first = first.join();
    let second = second.join();
    let accepted = first.expect("Сбой проверки четырёх коробов");
    let rejected = second.expect("Сбой проверки пяти коробов");
    assert_eq!((accepted, rejected), (true, false));
    println!("Общее правило — 4 короба {accepted}, 5 коробов {rejected}");
}

fn last_owner_demo(rules: Arc<PackingRules>) {
    let observer = Arc::downgrade(&rules);
    let task = move || rules.as_ref().version;
    let owners_before_launch = observer.strong_count();
    let version = thread::spawn(task).join().expect("Сбой чтения версии");
    let available_after_join = observer.upgrade().is_some();
    assert_eq!(
        (owners_before_launch, version, available_after_join),
        (1, 7, false)
    );
    println!(
        "Последний владелец — до запуска {owners_before_launch}, прочитана версия {version}, доступ после join {available_after_join}"
    );
}

fn main() -> io::Result<()> {
    let rules = Arc::new(PackingRules {
        version: 7,
        units_per_carton: BTreeMap::from([
            (101, NonZeroU16::new(12).expect("Вместимость задана")),
            (205, NonZeroU16::new(8).expect("Вместимость задана")),
        ]),
    });

    // Если клонировать только внутри move-замыкания, оно сначала заберёт исходный Arc.
    // let _ = thread::spawn(move || Arc::clone(&rules));

    // move перенесёт саму ссылку local_rules, а её время жизни останется прежним.
    /*
    let local_rules = rules.as_ref();
    let _ = thread::spawn(move || local_rules.version);
    */

    let first = start_batch(
        &rules,
        Batch {
            number: 41,
            lines: vec![
                Line {
                    sku: 101,
                    units: 13,
                },
                Line {
                    sku: 205,
                    units: 10,
                },
            ],
        },
    )?;
    let second = start_batch(
        &rules,
        Batch {
            number: 42,
            lines: vec![
                Line {
                    sku: 101,
                    units: 12,
                },
                Line { sku: 999, units: 8 },
            ],
        },
    );

    // Оригинальный Arc доступен главному потоку и после запуска.
    assert_eq!(rules.version, 7);
    let first_result = first.join();
    let second_result = second?.join();
    let first_plan = first_result.expect("Сбой исполнителя партии 41");
    let second_plan = second_result.expect("Сбой исполнителя партии 42");
    assert_eq!(
        first_plan,
        Ok(Plan {
            batch: 41,
            rules_version: 7,
            cartons: 4
        })
    );
    assert_eq!(second_plan, Err(PlanError::UnknownSku { sku: 999 }));
    if let Err(PlanError::UnknownSku { sku }) = second_plan {
        println!("Отказ упаковки — партия 42, неизвестный артикул {sku}");
    }

    // Проверяем число владельцев после join, когда оба замыкания уже завершились.
    let owners = Arc::strong_count(&rules);
    assert_eq!(owners, 1);
    println!("Владельцы справочника — после двух заданий {owners}");
    let retry = start_batch(
        &rules,
        Batch {
            number: 42,
            lines: vec![
                Line {
                    sku: 101,
                    units: 12,
                },
                Line { sku: 205, units: 8 },
            ],
        },
    )?;
    let corrected = retry.join().expect("Сбой повторного расчёта партии 42");
    assert_eq!(
        corrected,
        Ok(Plan {
            batch: 42,
            rules_version: 7,
            cartons: 2
        })
    );
    assert_eq!(Arc::strong_count(&rules), 1);

    if let (Ok(first), Ok(corrected)) = (first_plan, corrected) {
        println!(
            "Версия справочника {}. Коробов для партии {} — {}",
            first.rules_version, first.batch, first.cartons
        );
        println!(
            "Коробов для партии {} после исправления — {}",
            corrected.batch, corrected.cartons
        );
    }
    shared_rule_demo();
    last_owner_demo(rules);
    Ok(())
}
