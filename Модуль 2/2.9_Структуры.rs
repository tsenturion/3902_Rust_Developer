/*
cargo run --offline --bin struct_models
*/

const UNITS_PER_BOX: u64 = 12;

/*
Склад рассчитывает заказ с учётом физического остатка, резервов
и ожидаемых поставок. После выдачи резерва должно остаться не меньше
установленного минимума. При дефиците добавить к заказу страховой
запас и округлить объём вверх до целых коробов по 12 единиц.
Правила пополнения одинаковы для нескольких товаров.
*/
struct StockBalance {
    on_hand: u32,
    reserved: u32,
    incoming: u32,
}

struct WarehouseItem {
    sku: String,
    balance: StockBalance,
}

struct RestockPolicy {
    minimum_free: u32,
    buffer_units: u32,
}

struct RestockPlan {
    available_now: u32,
    shortage_to_minimum: u64,
    boxes_to_order: u64,
    units_to_order: u64,
}

fn main() {
    let policy = RestockPolicy {
        minimum_free: 20,
        buffer_units: 10,
    };

    let mut item = WarehouseItem {
        sku: String::from("CAM-042"),
        balance: StockBalance {
            on_hand: 30,
            reserved: 18,
            incoming: 4,
        },
    };

    let initial = plan_restock(&item, &policy);
    assert_eq!(initial.available_now, 12);
    assert_eq!(initial.shortage_to_minimum, 4);
    assert_eq!(initial.boxes_to_order, 2);
    assert_eq!(initial.units_to_order, 24);
    print_plan(&item, &initial);

    // Данные о поставке находятся во вложенной структуре balance.
    // let _ = item.incoming;

    // Сигнатура различает складскую позицию и правила заказа.
    // let _ = plan_restock(&policy, &item);

    // У результата именованные поля.
    // let _ = initial.0;

    // После подтверждения ещё четырёх единиц ожидаемый остаток достигнет минимума.
    item.balance.incoming = 8;
    let revised = plan_restock(&item, &policy);
    assert_eq!(revised.shortage_to_minimum, 0);
    assert_eq!(revised.boxes_to_order, 0);
    assert_eq!(revised.units_to_order, 0);
    println!(
        "{} после уточнения поставки — коробов к заказу {}",
        item.sku, revised.boxes_to_order
    );

    // Меняем правило пополнения, сохраняя те же складские данные.
    let minimum_22 = RestockPolicy {
        minimum_free: 22,
        ..policy
    };
    let minimum_23 = RestockPolicy {
        minimum_free: 23,
        ..policy
    };
    let exact_box = plan_restock(&item, &minimum_22);
    let next_box = plan_restock(&item, &minimum_23);
    assert_eq!(
        (exact_box.shortage_to_minimum, exact_box.units_to_order),
        (2, 12)
    );
    assert_eq!(
        (next_box.shortage_to_minimum, next_box.units_to_order),
        (3, 24)
    );
    assert_eq!(item.balance.incoming, 8);
    println!(
        "Те же остатки {} — при минимуме 22 заказ {} ед., при минимуме 23 заказ {} ед.",
        item.sku, exact_box.units_to_order, next_box.units_to_order
    );

    let second_item = WarehouseItem {
        sku: String::from("AUD-015"),
        balance: StockBalance {
            on_hand: 3,
            reserved: 14,
            incoming: 2,
        },
    };
    let second_plan = plan_restock(&second_item, &policy);
    assert_eq!(second_plan.available_now, 0);
    assert_eq!(second_plan.shortage_to_minimum, 29);
    assert_eq!(second_plan.boxes_to_order, 4);
    assert_eq!(second_plan.units_to_order, 48);
    print_plan(&second_item, &second_plan);
}

fn plan_restock(item: &WarehouseItem, policy: &RestockPolicy) -> RestockPlan {
    let balance = &item.balance;
    let available_now = balance.on_hand.saturating_sub(balance.reserved);
    let expected_total = u64::from(balance.on_hand) + u64::from(balance.incoming);
    let reserved_and_minimum = u64::from(balance.reserved) + u64::from(policy.minimum_free);
    let shortage_to_minimum = reserved_and_minimum.saturating_sub(expected_total);
    let units_before_rounding = if shortage_to_minimum > 0 {
        shortage_to_minimum + u64::from(policy.buffer_units)
    } else {
        0
    };
    let boxes_to_order = units_before_rounding.div_ceil(UNITS_PER_BOX);

    RestockPlan {
        available_now,
        shortage_to_minimum,
        boxes_to_order,
        units_to_order: boxes_to_order * UNITS_PER_BOX,
    }
}

fn print_plan(item: &WarehouseItem, plan: &RestockPlan) {
    println!(
        "{} — свободно сейчас {}, дефицит до минимума {}, к заказу {} ед. (коробов — {})",
        item.sku,
        plan.available_now,
        plan.shortage_to_minimum,
        plan.units_to_order,
        plan.boxes_to_order
    );
}
