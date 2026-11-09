/*
cargo run --offline --bin collections_intro
*/

/*
Склад выдаёт водителям талоны для въезда. Обычные заявки вызываются
в порядке поступления. Срочные идут перед обычными, аварийные — перед
срочными. Среди срочных или аварийных талонов одного приоритета первым
вызывается меньший номер.
Повторная регистрация номера в течение смены отклоняется. По номеру
можно найти назначенный док, а в конце смены сверить вызванные талоны
в порядке номеров.
*/

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Regular,
    Urgent,
    Emergency,
}

#[derive(Debug, PartialEq)]
struct Ticket {
    dock: u8,
    priority: Priority,
}

struct Shift {
    regular: VecDeque<u32>,
    urgent: BinaryHeap<(Priority, Reverse<u32>)>,
    tickets: BTreeMap<u32, Ticket>,
    called: BTreeSet<u32>,
}

impl Shift {
    fn new() -> Self {
        Self {
            regular: VecDeque::new(),
            urgent: BinaryHeap::new(),
            tickets: BTreeMap::new(),
            called: BTreeSet::new(),
        }
    }

    fn register(&mut self, number: u32, dock: u8, priority: Priority) -> bool {
        // У каждого номера одна карточка. Повторный талон не меняет карточку или очередь.
        if self.tickets.contains_key(&number) {
            return false;
        }
        self.tickets.insert(number, Ticket { dock, priority });

        match priority {
            Priority::Regular => self.regular.push_back(number),
            Priority::Urgent | Priority::Emergency => {
                // Куча выбирает наибольший ключ. Reverse отдаёт меньшему номеру преимущество
                // при совпадении приоритета.
                self.urgent.push((priority, Reverse(number)));
            }
        }
        true
    }

    fn call_next(&mut self) -> Option<(u32, &Ticket)> {
        let number = match self.urgent.pop() {
            Some((_, Reverse(number))) => number,
            None => self.regular.pop_front()?,
        };
        let ticket = self.tickets.get(&number)?;
        self.called.insert(number);
        Some((number, ticket))
    }
}

fn main() {
    let mut shift = Shift::new();
    assert_eq!(shift.call_next(), None);

    assert!(shift.register(214, 2, Priority::Regular));
    assert!(shift.register(209, 1, Priority::Regular));
    assert!(shift.register(208, 4, Priority::Urgent));
    assert!(shift.register(203, 3, Priority::Urgent));
    assert!(shift.register(219, 2, Priority::Emergency));
    assert!(!shift.register(203, 9, Priority::Emergency));
    assert_eq!(
        shift.tickets.get(&203),
        Some(&Ticket {
            dock: 3,
            priority: Priority::Urgent
        })
    );
    assert_eq!(shift.tickets.get(&999), None);

    // Просмотр первого элемента очереди и вершины кучи ничего не извлекает.
    assert_eq!(shift.regular.front(), Some(&214));
    assert_eq!(
        shift.urgent.peek(),
        Some(&(Priority::Emergency, Reverse(219)))
    );
    assert!(shift.called.is_empty());

    let Some((number, ticket)) = shift.call_next() else {
        panic!("В примере есть аварийная заявка");
    };
    assert_eq!((number, ticket.dock), (219, 2));
    assert!(shift.called.contains(&219));
    assert!(!shift.called.contains(&214));

    let mut order = [number, 0, 0, 0, 0];
    for slot in &mut order[1..] {
        let Some((number, _)) = shift.call_next() else {
            panic!("В примере остались талоны");
        };
        *slot = number;
    }
    assert_eq!(order, [219, 203, 208, 214, 209]);
    assert_eq!(shift.call_next(), None);
    assert!(!shift.register(219, 4, Priority::Regular));
    assert_eq!(shift.tickets.len(), 5);
    println!("Порядок вызова — {order:?}");

    // BTreeSet даёт уникальные номера по возрастанию. По номеру находим карточку в BTreeMap.
    let mut previous = None;
    for &number in &shift.called {
        if let Some(previous) = previous {
            assert!(number > previous);
        }
        let Some(ticket) = shift.tickets.get(&number) else {
            panic!("У каждого вызванного талона должна быть карточка");
        };
        println!("Талон {number} — док {}", ticket.dock);
        previous = Some(number);
    }
    assert_eq!(shift.called, BTreeSet::from([203, 208, 209, 214, 219]));
}
