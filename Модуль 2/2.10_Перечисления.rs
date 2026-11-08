/*
cargo run --offline --bin delivery_enums
*/

/*
Склад передаёт заказ в доставку, присваивая ему номер отправления. Подтверждение
выдачи принимается, только если заказ находится в пути и номер совпадает.
Для курьерской выдачи сохраняется имя получателя, для постамата — станция
и ячейка. Отменить заказ можно до передачи в доставку.
*/

mod delivery {
    #[derive(Debug, PartialEq)]
    pub enum Handoff {
        CourierSignature { recipient: String },
        LockerPickup { station: u32, compartment: u8 },
    }

    #[derive(Debug, PartialEq)]
    pub enum DeliveryState {
        AwaitingDispatch,
        InTransit(u64),
        Delivered { tracking_id: u64, handoff: Handoff },
        Cancelled,
    }

    impl DeliveryState {
        fn is_in_transit_with(&self, tracking_id: u64) -> bool {
            // PartialEq сравнивает и вариант, и данные внутри него.
            self == &Self::InTransit(tracking_id)
        }
    }

    pub struct Shipment {
        order_number: u32,
        // У заказа всегда ровно одно состояние. Номер заказа общий для всех состояний.
        state: DeliveryState,
    }

    impl Shipment {
        pub fn new(order_number: u32) -> Self {
            Self {
                order_number,
                state: DeliveryState::AwaitingDispatch,
            }
        }

        pub fn order_number(&self) -> u32 {
            self.order_number
        }

        pub fn state(&self) -> &DeliveryState {
            &self.state
        }

        pub fn try_dispatch(&mut self, tracking_id: u64) -> bool {
            // Нулевой номер зарезервирован системой сканирования для пустого значения.
            if self.state != DeliveryState::AwaitingDispatch || tracking_id == 0 {
                return false;
            }

            self.state = DeliveryState::InTransit(tracking_id);
            true
        }

        pub fn try_confirm_delivery(&mut self, tracking_id: u64, handoff: Handoff) -> bool {
            if !self.state.is_in_transit_with(tracking_id) {
                return false;
            }

            // При подтверждении новый вариант получает номер отправления и данные о выдаче.
            self.state = DeliveryState::Delivered {
                tracking_id,
                handoff,
            };
            true
        }

        pub fn try_cancel(&mut self) -> bool {
            if self.state != DeliveryState::AwaitingDispatch {
                return false;
            }

            self.state = DeliveryState::Cancelled;
            true
        }
    }
}

use delivery::{DeliveryState, Handoff, Shipment};

fn main() {
    // let _: DeliveryState = 793_201; // Номер сам по себе не является состоянием.
    // let _ = DeliveryState::Delivered { tracking_id: 793_201 }; // Нужны данные о выдаче.
    // let _ = DeliveryState::AwaitingDispatch as u64; // У перечисления есть варианты с данными.

    let mut courier_order = Shipment::new(781);
    let courier_tracking = 793_201;
    let another_tracking = 793_202;
    assert_eq!(courier_order.state(), &DeliveryState::AwaitingDispatch);
    assert!(!courier_order.try_dispatch(0));
    assert!(courier_order.try_dispatch(courier_tracking));
    assert!(!courier_order.try_dispatch(another_tracking));
    assert!(!courier_order.try_cancel());

    // Вариант InTransit совпадает, но подтверждение пришло с чужим номером.
    assert!(!courier_order.try_confirm_delivery(
        another_tracking,
        Handoff::CourierSignature {
            recipient: String::from("Ирина"),
        },
    ));
    assert_eq!(
        courier_order.state(),
        &DeliveryState::InTransit(courier_tracking)
    );

    assert!(courier_order.try_confirm_delivery(
        courier_tracking,
        Handoff::CourierSignature {
            recipient: String::from("Ирина"),
        },
    ));
    assert_eq!(
        courier_order.state(),
        &DeliveryState::Delivered {
            tracking_id: courier_tracking,
            handoff: Handoff::CourierSignature {
                recipient: String::from("Ирина"),
            },
        },
    );
    println!(
        "Заказ {} — выдача курьером подтверждена",
        courier_order.order_number()
    );

    let mut locker_order = Shipment::new(782);
    let locker_pickup = Handoff::LockerPickup {
        station: 24,
        compartment: 7,
    };
    assert!(locker_order.try_dispatch(another_tracking));
    assert!(locker_order.try_confirm_delivery(another_tracking, locker_pickup));
    assert_eq!(
        locker_order.state(),
        &DeliveryState::Delivered {
            tracking_id: another_tracking,
            handoff: Handoff::LockerPickup {
                station: 24,
                compartment: 7,
            },
        },
    );
    println!(
        "Заказ {} — выдача из постамата подтверждена",
        locker_order.order_number()
    );

    let mut cancelled_order = Shipment::new(783);
    assert!(cancelled_order.try_cancel());
    assert!(!cancelled_order.try_dispatch(793_203));
    assert_eq!(cancelled_order.state(), &DeliveryState::Cancelled);
    println!(
        "Заказ {} — отменён до отправки",
        cancelled_order.order_number()
    );
}
