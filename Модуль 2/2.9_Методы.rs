/*
cargo run --offline --bin struct_methods
*/

/*
Склад уже зарезервировал часть товара под предыдущие заказы. Новый заказ
можно собрать только после приёмки части ожидаемой поставки. При отгрузке
нужно списать выданные единицы из остатка и резерва, а черновик превратить
в запись с номером отслеживания.
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

impl WarehouseItem {
    // new создаёт позицию, поэтому ему не нужен готовый экземпляр.
    // Это ассоциированная функция. Self здесь означает WarehouseItem.
    fn new(sku: String, on_hand: u32, incoming: u32) -> Self {
        Self {
            sku,
            balance: StockBalance {
                on_hand,
                reserved: 0,
                incoming,
            },
        }
    }

    // Метод возвращает срез строки, которая остаётся внутри позиции.
    fn sku(&self) -> &str {
        self.sku.as_str()
    }

    fn available_units(&self) -> u32 {
        self.balance.on_hand - self.balance.reserved
    }
}

// У одного типа может быть несколько блоков impl. Здесь собраны складские операции.
impl WarehouseItem {
    fn try_receive(&mut self, units: u32) -> bool {
        // Сначала проверяем весь объём, чтобы отказ не изменил остатки.
        if units == 0 || units > self.balance.incoming || units > u32::MAX - self.balance.on_hand {
            return false;
        }

        self.balance.incoming -= units;
        self.balance.on_hand += units;
        true
    }

    fn try_reserve(&mut self, units: u32) -> bool {
        // Из метода с &mut self можно вызвать метод чтения через self.
        if units == 0 || units > self.available_units() {
            return false;
        }

        self.balance.reserved += units;
        true
    }

    fn try_ship(&mut self, units: u32) -> bool {
        if units == 0 || units > self.balance.reserved {
            return false;
        }

        self.balance.reserved -= units;
        self.balance.on_hand -= units;
        true
    }
}

struct ShipmentDraft {
    order_number: String,
    sku: String,
    units: u32,
}

struct DispatchRecord {
    order_number: String,
    sku: String,
    units: u32,
    tracking_code: String,
}

impl ShipmentDraft {
    // self передаёт черновик во владение метода. Строки переходят в запись целиком.
    fn into_record(self, tracking_code: String) -> DispatchRecord {
        DispatchRecord {
            order_number: self.order_number,
            sku: self.sku,
            units: self.units,
            tracking_code,
        }
    }
}

fn main() {
    let mut item = WarehouseItem::new(String::from("CAM-042"), 18, 10);
    // let _ = item.new(String::from("AUD-015"), 0, 0);

    let requested_units = 16;
    assert!(item.try_reserve(8));
    assert!(!item.try_reserve(requested_units));

    // При вызове через точку Rust сам подставляет &item для метода с &self.
    let available = item.available_units();
    assert_eq!(available, WarehouseItem::available_units(&item));
    assert_eq!(available, 10);
    println!(
        "{} — свободно {} из нужных {} ед.",
        item.sku(),
        available,
        requested_units
    );

    let sku = item.sku();
    // item.try_receive(6); // Заимствованный срез sku ещё используется ниже.
    assert_eq!(sku, "CAM-042");

    assert!(!item.try_receive(11));
    assert_eq!(item.balance.on_hand, 18);
    assert_eq!(item.balance.incoming, 10);
    assert!(item.try_receive(6));
    assert_eq!(item.balance.incoming, 4);
    assert_eq!(item.available_units(), requested_units);

    // В форме через имя типа &mut item передаётся явно.
    assert!(WarehouseItem::try_reserve(&mut item, requested_units));
    assert_eq!(item.available_units(), 0);
    println!(
        "{} — после приёмки зарезервировано ещё {} ед.",
        item.sku(),
        requested_units
    );

    let draft = ShipmentDraft {
        order_number: String::from("ORD-781"),
        sku: String::from(item.sku()),
        units: requested_units,
    };
    assert!(!item.try_ship(25));
    assert_eq!(item.balance.on_hand, 24);
    assert_eq!(item.balance.reserved, 24);
    assert!(item.try_ship(draft.units));
    assert_eq!(item.balance.on_hand, 8);
    assert_eq!(item.balance.reserved, 8);

    let record = draft.into_record(String::from("TRK-932"));
    // let _ = draft.into_record(String::from("TRK-933")); // Черновик уже перемещён.
    println!(
        "Заказ {} — отгружено {} ед. {}, номер отслеживания {}",
        record.order_number, record.units, record.sku, record.tracking_code
    );

    assert!(item.try_receive(4));
    assert_eq!(item.balance.incoming, 0);
    assert_eq!(item.available_units(), 4);
    assert!(!item.try_receive(1));
    assert_eq!(item.balance.on_hand, 12);
    assert_eq!(item.balance.reserved, 8);
    assert_eq!(item.balance.incoming, 0);
    println!(
        "Приняты оставшиеся 4 ед. поставки, следующая единица отклонена. Ожидается {}, свободно {}",
        item.balance.incoming,
        item.available_units()
    );
}
