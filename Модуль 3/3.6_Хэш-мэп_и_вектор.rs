/*
cargo run --offline --bin hash_map_and_vector
*/

/*
Склад хранит талоны в порядке регистрации. По коду талона оператор
находит его док, может переназначить док или отменить талон. Повторная
регистрация активного кода отклоняется. Отмена сохраняет порядок оставшихся
талонов. Для сводки нужно посчитать число активных талонов на каждый док.
*/

use std::collections::HashMap;

// В ключе сводки участвуют и зона, и номер дока.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Dock {
    zone: u8,
    number: u8,
}

#[derive(Debug, PartialEq)]
struct Ticket {
    code: String,
    dock: Dock,
}

struct Registry {
    tickets: Vec<Ticket>,
    by_code: HashMap<String, usize>,
}

impl Registry {
    fn with_capacity(expected: usize) -> Self {
        // Резервируем место. Длина обеих коллекций пока равна нулю.
        Self {
            tickets: Vec::with_capacity(expected),
            by_code: HashMap::with_capacity(expected),
        }
    }

    fn register(&mut self, code: &str, dock: Dock) -> bool {
        if self.by_code.contains_key(code) {
            return false;
        }

        let code = code.to_owned();
        // Вектор владеет записью, хеш-таблица владеет своей копией ключа.
        self.by_code.insert(code.clone(), self.tickets.len());
        // Для push амортизированная стоимость — O(1) на элемент.
        self.tickets.push(Ticket { code, dock });
        true
    }

    fn get(&self, code: &str) -> Option<&Ticket> {
        // Ключи хранятся как String, а для поиска достаточно &str.
        // По числу записей поиск ожидаемо O(1). Для строк учитываем и длину ключа.
        let &position = self.by_code.get(code)?;
        self.tickets.get(position)
    }

    fn reassign(&mut self, code: &str, dock: Dock) -> Option<Dock> {
        let &position = self.by_code.get(code)?;
        let ticket = self.tickets.get_mut(position)?;
        Some(std::mem::replace(&mut ticket.dock, dock))
    }

    fn remove(&mut self, code: &str) -> Option<Ticket> {
        let position = self.by_code.remove(code)?;
        let removed = self.tickets.remove(position);

        // remove сдвигает хвост вектора за O(n) в худшем случае.
        // Обновляем позиции сдвинутых записей в индексе.
        for position in position..self.tickets.len() {
            let code = self.tickets[position].code.as_str();
            let indexed = self
                .by_code
                .get_mut(code)
                .expect("У талона должна быть запись в индексе");
            *indexed = position;
        }
        Some(removed)
    }

    fn undo_last(&mut self) -> Option<Ticket> {
        // После pop позиции остальных элементов сохраняются.
        let ticket = self.tickets.pop()?;
        self.by_code.remove(ticket.code.as_str());
        Some(ticket)
    }

    fn tickets(&self) -> &[Ticket] {
        self.tickets.as_slice()
    }
}

fn count_by_dock(tickets: &[Ticket]) -> HashMap<Dock, usize> {
    let mut counts = HashMap::new();
    for ticket in tickets {
        // entry даёт значение по ключу. Если ключа ещё нет, начинаем с нуля.
        *counts.entry(ticket.dock).or_insert(0) += 1;
    }
    counts
}

fn check_unordered_pool() {
    // Для списка свободных мест важен только состав. swap_remove кладёт
    // последний элемент на место удалённого за O(1).
    let mut free_places = vec![101, 103, 105, 107];
    assert_eq!(free_places.swap_remove(1), 103);
    assert_eq!(free_places.as_slice(), &[101, 107, 105]);
    free_places.reserve(4);
    assert!(free_places.capacity() >= free_places.len() + 4);
    assert_eq!(free_places.get(3), None);
    free_places.push(109);
    assert_eq!(free_places.get(3), Some(&109));
}

fn main() {
    let north = Dock { zone: 1, number: 2 };
    let south = Dock { zone: 2, number: 2 };
    let mut registry = Registry::with_capacity(2);
    assert!(registry.tickets().is_empty());
    assert!(registry.tickets.capacity() >= 2);
    assert!(registry.by_code.is_empty());
    assert_eq!(registry.undo_last(), None);

    assert!(registry.register("К-101", north));
    assert!(registry.register("К-207", south));
    assert!(registry.register("К-305", north));
    assert!(registry.register("К-110", north));
    assert_eq!(registry.tickets().len(), 4);
    assert!(!registry.register("К-207", north));
    assert_eq!(registry.get("К-207").map(|ticket| ticket.dock), Some(south));
    assert_eq!(registry.get("К-999"), None);

    assert_eq!(registry.reassign("К-305", south), Some(north));
    assert_eq!(registry.reassign("К-999", south), None);
    assert_eq!(
        registry.remove("К-207"),
        Some(Ticket {
            code: String::from("К-207"),
            dock: south
        })
    );
    assert_eq!(registry.remove("К-207"), None);
    assert_eq!(registry.get("К-207"), None);
    assert_eq!(registry.by_code.get("К-305"), Some(&1));
    assert_eq!(registry.by_code.get("К-110"), Some(&2));
    assert_eq!(registry.get("К-305").map(|ticket| ticket.dock), Some(south));

    assert!(registry.register("К-406", south));
    assert_eq!(
        registry.undo_last().map(|ticket| ticket.code),
        Some(String::from("К-406"))
    );
    assert_eq!(registry.get("К-406"), None);
    let [first, second, third] = registry.tickets() else {
        panic!("В реестре примера должны остаться три талона");
    };
    assert_eq!(
        [
            first.code.as_str(),
            second.code.as_str(),
            third.code.as_str()
        ],
        ["К-101", "К-305", "К-110"]
    );
    println!(
        "Талоны в порядке поступления — {}, {}, {}",
        first.code, second.code, third.code
    );

    let counts = count_by_dock(registry.tickets());
    // Порядок обхода HashMap не задан. Для отчёта обращаемся к нужным ключам.
    assert_eq!(counts.len(), 2);
    assert_eq!(counts.get(&north), Some(&2));
    assert_eq!(counts.get(&south), Some(&1));
    let north_count = counts.get(&north).copied().unwrap_or(0);
    let south_count = counts.get(&south).copied().unwrap_or(0);
    println!("Док 1/2 — {north_count} талона, док 2/2 — {south_count} талон");

    assert_eq!(
        registry.remove("К-101").map(|ticket| ticket.code),
        Some(String::from("К-101"))
    );
    assert_eq!(registry.by_code.get("К-305"), Some(&0));
    assert_eq!(registry.by_code.get("К-110"), Some(&1));
    assert!(registry.register("К-101", south));
    assert_eq!(registry.by_code.get("К-101"), Some(&2));
    assert_eq!(registry.get("К-101").map(|ticket| ticket.dock), Some(south));
    check_unordered_pool();
}
