/*
cargo run --offline --bin pattern_matching
*/

/*
Служба поддержки разбирает очередь заказов. Если заказ ждёт отправки
четыре часа или больше, нужно связаться со складом. Заказ в пути передают
перевозчику на разбор, как только истекает срок доставки. Выданные и
отменённые заказы исключают из активной очереди, но их данные доступны
для ответа клиенту.
*/

enum Handoff {
    CourierSignature { recipient: String },
    LockerPickup { station: u32, compartment: u8 },
}

enum DeliveryState {
    AwaitingDispatch,
    InTransit(u64),
    Delivered { tracking_id: u64, handoff: Handoff },
    Cancelled,
}

struct Ticket {
    order_number: u32,
    state: DeliveryState,
    hours_in_state: u32,
}

#[derive(Debug, PartialEq)]
enum Action {
    Wait,
    ContactWarehouse(u32),
    Track(u64),
    Escalate { tracking_id: u64, hours: u32 },
    Archive,
}

fn next_action(ticket: &Ticket, delivery_limit: u32) -> Action {
    // Rust проверяет ветки сверху вниз. После совпадения шаблона проверяется условие if.
    match (&ticket.state, ticket.hours_in_state) {
        (DeliveryState::AwaitingDispatch, hours @ 4..=u32::MAX) => Action::ContactWarehouse(hours),
        (DeliveryState::AwaitingDispatch, _) => Action::Wait,
        (DeliveryState::InTransit(tracking_id), hours) if hours >= delivery_limit => {
            Action::Escalate {
                tracking_id: *tracking_id,
                hours,
            }
        }
        // Ветка с условием может не сработать. Здесь остаются отправления в сроке.
        (DeliveryState::InTransit(tracking_id), _) => Action::Track(*tracking_id),
        (DeliveryState::Delivered { .. } | DeliveryState::Cancelled, _) => Action::Archive,
    }
}

fn describe_state(state: &DeliveryState) -> String {
    // Вложенный шаблон достаёт данные о выдаче прямо из Delivered.
    // При разборе &DeliveryState поле recipient заимствуется как &String.
    match state {
        DeliveryState::AwaitingDispatch => String::from("ожидает отправки"),
        DeliveryState::InTransit(tracking_id) => format!("отправление {tracking_id} в пути"),
        DeliveryState::Delivered {
            tracking_id,
            handoff: Handoff::CourierSignature { recipient },
        } => format!("отправление {tracking_id} выдано курьером, получатель — {recipient}"),
        DeliveryState::Delivered {
            tracking_id,
            handoff:
                Handoff::LockerPickup {
                    station,
                    compartment,
                },
        } => {
            format!("отправление {tracking_id} выдано из постамата {station}, ячейка {compartment}")
        }
        DeliveryState::Cancelled => String::from("заказ отменён"),
    }
}

fn describe_action(action: &Action) -> String {
    match action {
        Action::Wait => String::from("ожидать отправки"),
        Action::ContactWarehouse(hours) => format!("связаться со складом после {hours} ч ожидания"),
        Action::Track(tracking_id) => format!("отслеживать отправление {tracking_id}"),
        Action::Escalate { tracking_id, hours } => {
            format!("передать перевозчику отправление {tracking_id}, в пути {hours} ч")
        }
        Action::Archive => String::from("перенести в архив"),
    }
}

fn tracking_matches(state: &DeliveryState, scanned_id: u64) -> bool {
    // После let ... else имя tracking_id доступно до конца функции.
    let DeliveryState::InTransit(tracking_id) = state else {
        return false;
    };
    *tracking_id == scanned_id
}

fn print_open_queue(mut pending: &[Ticket], delivery_limit: u32) -> (usize, usize) {
    let mut active = 0;
    let mut archived = 0;

    // У непустого слайса есть первый элемент и хвост. На пустом слайсе цикл закончится.
    while let [ticket, remaining @ ..] = pending {
        pending = remaining;

        let action = next_action(ticket, delivery_limit);
        // matches! даёт булев ответ о совпадении с шаблоном.
        if matches!(action, Action::Archive) {
            archived += 1;
            continue;
        }

        // Такой шаблон структуры подходит любому Ticket, поэтому достаточно let.
        let Ticket {
            order_number,
            state,
            ..
        } = ticket;
        println!(
            "Заказ {order_number} — {}. Действие — {}",
            describe_state(state),
            describe_action(&action)
        );
        active += 1;
    }

    (active, archived)
}

fn main() {
    let tickets = [
        Ticket {
            order_number: 781,
            state: DeliveryState::AwaitingDispatch,
            hours_in_state: 3,
        },
        Ticket {
            order_number: 782,
            state: DeliveryState::AwaitingDispatch,
            hours_in_state: 4,
        },
        Ticket {
            order_number: 783,
            state: DeliveryState::InTransit(793_203),
            hours_in_state: 47,
        },
        Ticket {
            order_number: 784,
            state: DeliveryState::InTransit(793_204),
            hours_in_state: 48,
        },
        Ticket {
            order_number: 785,
            state: DeliveryState::Delivered {
                tracking_id: 793_205,
                handoff: Handoff::CourierSignature {
                    recipient: String::from("Ирина"),
                },
            },
            hours_in_state: 2,
        },
        Ticket {
            order_number: 786,
            state: DeliveryState::Delivered {
                tracking_id: 793_206,
                handoff: Handoff::LockerPickup {
                    station: 24,
                    compartment: 7,
                },
            },
            hours_in_state: 1,
        },
        Ticket {
            order_number: 787,
            state: DeliveryState::Cancelled,
            hours_in_state: 1,
        },
    ];

    let delivery_limit = 48;
    assert_eq!(next_action(&tickets[0], delivery_limit), Action::Wait);
    assert_eq!(
        next_action(&tickets[1], delivery_limit),
        Action::ContactWarehouse(4)
    );
    assert_eq!(
        next_action(&tickets[2], delivery_limit),
        Action::Track(793_203)
    );
    assert_eq!(
        next_action(&tickets[3], delivery_limit),
        Action::Escalate {
            tracking_id: 793_204,
            hours: 48
        }
    );
    assert!(tracking_matches(&tickets[2].state, 793_203));
    assert!(!tracking_matches(&tickets[2].state, 793_204));
    assert!(!tracking_matches(&tickets[4].state, 793_205));

    let (active, archived) = print_open_queue(&tickets, delivery_limit);
    assert_eq!((active, archived), (4, 3));
    assert_eq!(print_open_queue(&[], delivery_limit), (0, 0));
    println!("Активных заказов — {active}, архивных — {archived}");

    // Номер ячейки нужен только для справки о постамате. Прочие варианты пропускаем.
    let locker_ticket = &tickets[5];
    if let DeliveryState::Delivered {
        handoff: Handoff::LockerPickup {
            station,
            compartment,
        },
        ..
    } = &locker_ticket.state
    {
        assert_eq!((*station, *compartment), (24, 7));
        println!(
            "Справка по заказу {} — постамат {station}, ячейка {compartment}",
            locker_ticket.order_number
        );
    }
    assert_eq!(
        describe_state(&locker_ticket.state),
        "отправление 793206 выдано из постамата 24, ячейка 7"
    );
    assert_eq!(
        describe_state(&tickets[4].state),
        "отправление 793205 выдано курьером, получатель — Ирина"
    );
    assert_eq!(describe_state(&tickets[6].state), "заказ отменён");

    // Обычный let требует шаблон, который подходит любому значению данного типа.
    // let DeliveryState::InTransit(_tracking_id) = &tickets[2].state;

    // Ветки match должны вместе покрывать все возможные варианты.
    // let _ = match &tickets[2].state { DeliveryState::InTransit(_) => "в пути" };
}
