/*
cargo run --offline --bin struct_encapsulation
*/

/*
Для заказа зарезервировали 12 камер. Покупатель отменил три, поэтому склад
вернул их в свободный остаток и отгрузил девять. После отгрузки пришла новая
поставка. Программа для заказов может смотреть остатки и запрашивать операции,
а склад сам проверяет, достаточно ли единиц для каждой операции.
*/

// Граница модуля позволяет показать видимость полей в одном файле.
mod warehouse {
    // Тип доступен за пределами warehouse. Его поля закрыты.
    pub struct WarehouseItem {
        sku: String,
        available: u32,
        reserved: u32,
    }

    // Снимок содержит собственные числа. Его поля доступны для отчёта.
    pub struct StockSnapshot {
        pub on_hand: u32,
        pub available: u32,
        pub reserved: u32,
    }

    impl WarehouseItem {
        // При создании все принятые единицы свободны.
        pub fn new(sku: &str, on_hand: u32) -> Self {
            Self {
                sku: String::from(sku),
                available: on_hand,
                reserved: 0,
            }
        }

        pub fn sku(&self) -> &str {
            self.sku.as_str()
        }

        pub fn snapshot(&self) -> StockSnapshot {
            StockSnapshot {
                on_hand: self.total_on_hand(),
                available: self.available,
                reserved: self.reserved,
            }
        }

        pub fn try_reserve(&mut self, units: u32) -> bool {
            // При отказе обе части остатка сохраняют прежние значения.
            if units == 0 || units > self.available {
                return false;
            }

            self.available -= units;
            self.reserved += units;
            true
        }

        pub fn try_cancel_reservation(&mut self, units: u32) -> bool {
            if units == 0 || units > self.reserved {
                return false;
            }

            self.reserved -= units;
            self.available += units;
            true
        }

        pub fn try_ship(&mut self, units: u32) -> bool {
            if units == 0 || units > self.reserved {
                return false;
            }

            self.reserved -= units;
            true
        }

        pub fn try_receive(&mut self, units: u32) -> bool {
            // Проверяем место для поставки в общем остатке, включая резерв.
            if units == 0 || units > u32::MAX - self.total_on_hand() {
                return false;
            }

            self.available += units;
            true
        }

        // Склад хранит две части остатка и вычисляет общее количество по ним.
        // Этот вспомогательный метод доступен внутри warehouse.
        fn total_on_hand(&self) -> u32 {
            self.available + self.reserved
        }
    }
}

fn main() {
    let mut item = warehouse::WarehouseItem::new("CAM-042", 18);

    // Каждая строка отдельно показывает проверку видимости компилятором.
    // item.reserved = 100;
    // let _ = warehouse::WarehouseItem { sku: String::from("CAM"), available: 0, reserved: 1 };
    // let _ = item.total_on_hand();

    assert!(item.try_reserve(12));
    let after_reserve = item.snapshot();
    assert_eq!(after_reserve.on_hand, 18);
    assert_eq!(after_reserve.available, 6);
    assert_eq!(after_reserve.reserved, 12);
    println!(
        "{} — свободно {}, в резерве {}",
        item.sku(),
        after_reserve.available,
        after_reserve.reserved
    );

    assert!(!item.try_reserve(7));
    assert!(!item.try_cancel_reservation(13));
    assert!(!item.try_ship(13));
    let after_rejections = item.snapshot();
    assert_eq!(after_rejections.on_hand, 18);
    assert_eq!(after_rejections.available, 6);
    assert_eq!(after_rejections.reserved, 12);

    assert!(item.try_cancel_reservation(3));
    let after_cancel = item.snapshot();
    assert_eq!(after_cancel.on_hand, 18);
    assert_eq!(after_cancel.available, 9);
    assert_eq!(after_cancel.reserved, 9);

    assert!(item.try_ship(9));
    let after_shipment = item.snapshot();
    assert_eq!(after_shipment.on_hand, 9);
    assert_eq!(after_shipment.available, 9);
    assert_eq!(after_shipment.reserved, 0);

    assert!(item.try_receive(6));
    let current = item.snapshot();
    assert_eq!(current.on_hand, 15);
    assert_eq!(current.available, 15);
    assert_eq!(current.reserved, 0);

    // Прежний снимок по-прежнему показывает состояние на момент резерва.
    assert_eq!(after_reserve.reserved, 12);
    println!(
        "{} — после отгрузки и приёмки на складе {}, свободно {}, в резерве {}",
        item.sku(),
        current.on_hand,
        current.available,
        current.reserved
    );
}
