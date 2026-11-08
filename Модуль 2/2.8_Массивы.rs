/*
cargo run --offline --bin arrays
*/

const SHIFT_COUNT: usize = 2;
const ZONE_COUNT: usize = 3;

/*
Склад обрабатывает заказы в трёх зонах за две смены. Для каждой зоны
задан лимит на смену. Заказы сверх лимита попадают в отложенную очередь.
Рассчитать очередь по сменам и суммарно по каждой зоне. Сравнить
обычный режим с режимом, в котором лимит второй зоны снижен до 15.
*/
fn main() {
    // Первая координата выбирает смену, вторая — зону.
    let orders: [[u16; ZONE_COUNT]; SHIFT_COUNT] = [[18, 24, 15], [22, 17, 30]];
    assert_eq!(orders.len(), SHIFT_COUNT);
    assert_eq!(orders[0].len(), ZONE_COUNT);
    assert_eq!(orders[1][2], 30);

    let mut regular_capacity: [u16; ZONE_COUNT] = [20; ZONE_COUNT];
    regular_capacity[2] = 25;

    // Копия массива чисел становится отдельным набором лимитов.
    let mut reduced_capacity = regular_capacity;
    reduced_capacity[1] = 15;
    assert_eq!(regular_capacity, [20, 20, 25]);
    assert_eq!(reduced_capacity, [20, 15, 25]);

    let regular_backlog = backlog_by_shift(&orders, &regular_capacity);
    let reduced_backlog = backlog_by_shift(&orders, &reduced_capacity);
    assert_eq!(regular_backlog, [[0, 4, 0], [2, 0, 5]]);
    assert_eq!(reduced_backlog, [[0, 9, 0], [2, 2, 5]]);

    let regular_totals = backlog_by_zone(&regular_backlog);
    let reduced_totals = backlog_by_zone(&reduced_backlog);
    assert_eq!(regular_totals, [2, 4, 5]);
    assert_eq!(reduced_totals, [2, 11, 5]);
    assert_eq!(backlog_by_zone(&[[0; ZONE_COUNT]; SHIFT_COUNT]), [0, 0, 0]);

    let [zone_a, zone_b, zone_c] = regular_totals;
    println!("Обычный режим по сменам — {regular_backlog:?}");
    println!("Обычный режим по зонам — A {zone_a}, B {zone_b}, C {zone_c}");
    println!("Сниженный лимит по зонам — {reduced_totals:?}");

    let additional_zones: [u16; 0] = [];
    assert!(additional_zones.is_empty());

    // Длина входит в тип массива.
    // let _: [u16; ZONE_COUNT] = [20, 25];

    // Все элементы имеют тип u16.
    // let _: [u16; ZONE_COUNT] = [20, true, 25];

    // Длина задаётся константным выражением.
    // let requested_zones = 3;
    // let _ = [20_u16; requested_zones];

    // При повторе этого выражения требуется Copy.
    // let _ = [String::from("зона"); ZONE_COUNT];

    // Для массива из трёх элементов допустимы индексы 0, 1 и 2.
    // println!("{}", regular_capacity[ZONE_COUNT]);

    // Выход за границу с вычисляемым индексом вызывает панику при запуске.
    // let requested_zone = std::hint::black_box(ZONE_COUNT);
    // println!("{}", regular_capacity[requested_zone]);
}

fn backlog_by_shift(
    orders: &[[u16; ZONE_COUNT]; SHIFT_COUNT],
    capacity: &[u16; ZONE_COUNT],
) -> [[u16; ZONE_COUNT]; SHIFT_COUNT] {
    let mut backlog = [[0; ZONE_COUNT]; SHIFT_COUNT];
    for shift in 0..orders.len() {
        for zone in 0..capacity.len() {
            backlog[shift][zone] = orders[shift][zone].saturating_sub(capacity[zone]);
        }
    }
    backlog
}

fn backlog_by_zone(backlog: &[[u16; ZONE_COUNT]; SHIFT_COUNT]) -> [u32; ZONE_COUNT] {
    let mut totals = [0; ZONE_COUNT];
    for shift in 0..backlog.len() {
        for zone in 0..totals.len() {
            totals[zone] += u32::from(backlog[shift][zone]);
        }
    }
    totals
}
