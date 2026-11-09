/*
cargo run --offline --bin collections_reconciliation
*/

/*
После инвентаризации несколько сканеров передают пакеты показаний.
Позиция определяется парой из зоны и номера метки. Повтор позиции
в ведомости считается одной ожидаемой позицией. Первый скан позиции
попадает в подтверждённые или в лишние, каждый последующий — в повторы.
В этих списках сохраняется порядок поступления. Недостача группируется
по возрастанию зон, внутри зоны сохраняется порядок ведомости.
*/

use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    zone: u8,
    tag: u32,
}

impl Position {
    fn new(zone: u8, tag: u32) -> Self {
        Self { zone, tag }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Report {
    confirmed: Vec<Position>,
    unexpected: Vec<Position>,
    repeated: Vec<Position>,
    missing_by_zone: BTreeMap<u8, Vec<u32>>,
}

fn reconcile(plan: &[Position], batches: &[&[Position]]) -> Report {
    let mut missing = HashSet::with_capacity(plan.len());
    for &position in plan {
        missing.insert(position);
    }

    let mut seen = HashSet::new();
    let mut report = Report::default();
    for &batch in batches {
        for &position in batch {
            // Множество сохраняется между пакетами. insert возвращает false для повтора.
            if !seen.insert(position) {
                report.repeated.push(position);
            } else if missing.remove(&position) {
                // Первый скан ожидаемой позиции закрывает её в рабочем множестве недостачи.
                report.confirmed.push(position);
            } else {
                report.unexpected.push(position);
            }
        }
    }

    // Идём по ведомости, чтобы сохранить её порядок внутри зоны.
    // remove одновременно проверяет недостачу и исключает повтор из ведомости.
    for &position in plan {
        if missing.remove(&position) {
            // BTreeMap упорядочивает зоны, а Vec добавляет метки в порядке ведомости.
            report
                .missing_by_zone
                .entry(position.zone)
                .or_default()
                .push(position.tag);
        }
    }
    report
}

fn main() {
    let tag_17_in_zone_1 = Position::new(1, 17);
    let tag_17_in_zone_2 = Position::new(2, 17);
    let tag_17_in_zone_3 = Position::new(3, 17);
    let tag_22 = Position::new(1, 22);
    let tag_33 = Position::new(1, 33);
    let tag_28 = Position::new(1, 28);
    let tag_99 = Position::new(2, 99);

    let plan = [
        tag_17_in_zone_2,
        tag_33,
        tag_17_in_zone_1,
        tag_28,
        tag_22,
        tag_33,
    ];
    let scanner_a = [tag_22, tag_17_in_zone_3, tag_22];
    let scanner_b = [tag_17_in_zone_1, tag_99, tag_17_in_zone_3, tag_22];
    let report = reconcile(&plan, &[&scanner_a, &[], &scanner_b]);

    assert_eq!(report.confirmed, [tag_22, tag_17_in_zone_1]);
    assert_eq!(report.unexpected, [tag_17_in_zone_3, tag_99]);
    assert_eq!(report.repeated, [tag_22, tag_17_in_zone_3, tag_22]);
    assert_eq!(
        report.missing_by_zone,
        BTreeMap::from([(1, vec![33, 28]), (2, vec![17])])
    );

    assert_eq!(reconcile(&[], &[]), Report::default());
    let without_scans = reconcile(&[tag_33, tag_33], &[]);
    assert_eq!(
        without_scans.missing_by_zone,
        BTreeMap::from([(1, vec![33])])
    );
    let all_confirmed = reconcile(&[tag_22], &[&[tag_22]]);
    assert_eq!(all_confirmed.confirmed, [tag_22]);
    assert!(all_confirmed.missing_by_zone.is_empty());
    let without_plan = reconcile(&[], &[&[tag_99, tag_99]]);
    assert_eq!(without_plan.unexpected, [tag_99]);
    assert_eq!(without_plan.repeated, [tag_99]);

    println!(
        "Подтверждено {}, лишних позиций {}, повторных сканов {}",
        report.confirmed.len(),
        report.unexpected.len(),
        report.repeated.len()
    );
    for (zone, tags) in &report.missing_by_zone {
        println!("Зона {zone} — недостающие метки {tags:?}");
    }
}
