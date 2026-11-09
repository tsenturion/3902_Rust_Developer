/*
cargo run --offline --bin smart_pointers_raii
*/

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;

/*
У каждого узла маршрута один владелец. При выдаче следующего задания
номер дока и название зоны переходят получателю, остальная цепочка
остаётся у маршрута.
*/

struct Stop {
    dock: u8,
    zone: String,
    next: Option<Box<Stop>>,
}

struct Route {
    head: Option<Box<Stop>>,
}

impl Route {
    fn prepend(&mut self, dock: u8, zone: String) {
        self.head = Some(Box::new(Stop {
            dock,
            zone,
            next: self.head.take(),
        }));
    }

    fn next_dock(&self) -> Option<u8> {
        self.head.as_deref().map(|stop| stop.dock)
    }

    fn take_next(&mut self) -> Option<(u8, String)> {
        let head = self.head.take()?;
        // У Box единственный владелец, поэтому поля можно переместить из *head.
        let Stop { dock, zone, next } = *head;
        self.head = next;
        Some((dock, zone))
    }

    fn merge_sorted(mut left: Self, mut right: Self) -> Self {
        let mut merged = Self { head: None };
        let mut tail = &mut merged.head;
        while let (Some(first), Some(second)) = (&left.head, &right.head) {
            let source = if first.dock <= second.dock {
                &mut left.head
            } else {
                &mut right.head
            };
            let mut stop = source.take().expect("У выбранного маршрута есть узел");
            *source = stop.next.take();
            // Встраиваем существующий Box и переходим к его свободной ссылке.
            tail = &mut tail.insert(stop).next;
        }
        *tail = left.head.or(right.head);
        merged
    }
}

// Box даёт рекурсивному полю фиксированный размер. Вариант ниже имеет бесконечный размер.
// struct InlineStop { next: Option<InlineStop> }

fn route_demo() {
    let mut route = Route { head: None };
    assert_eq!(route.take_next(), None);
    route.prepend(4, String::from("Резерв"));
    route.prepend(2, String::from("Приёмка"));
    assert_eq!(route.next_dock(), Some(2));

    let Some((dock, zone)) = route.take_next() else {
        panic!("В примере должен быть док приёмки");
    };
    assert_eq!((dock, zone.as_str()), (2, "Приёмка"));
    assert_eq!(route.next_dock(), Some(4));
    assert_eq!(route.take_next(), Some((4, String::from("Резерв"))));
    assert_eq!(route.take_next(), None);
    println!("Маршрут — первое задание выдано для зоны «{zone}», док {dock}");
}

fn merge_demo() {
    let mut left = Route { head: None };
    left.prepend(4, String::from("Резерв"));
    left.prepend(2, String::from("Север"));
    let mut right = Route { head: None };
    right.prepend(6, String::from("Выдача"));
    right.prepend(2, String::from("Восток"));

    let first_address = left.head.as_deref().map(std::ptr::from_ref);
    let mut merged = Route::merge_sorted(left, right);
    let same_node = merged.head.as_deref().map(std::ptr::from_ref) == first_address;
    assert!(same_node);
    let mut stops = Vec::new();
    while let Some(stop) = merged.take_next() {
        stops.push(stop);
    }
    assert!(stops.iter().map(|(dock, zone)| (*dock, zone.as_str())).eq([
        (2, "Север"),
        (2, "Восток"),
        (4, "Резерв"),
        (6, "Выдача"),
    ]));
    assert!(
        Route::merge_sorted(Route { head: None }, Route { head: None })
            .head
            .is_none()
    );
    let mut single = Route { head: None };
    single.prepend(7, String::from("Ремонт"));
    assert_eq!(
        Route::merge_sorted(Route { head: None }, single).next_dock(),
        Some(7)
    );
    println!("Слияние — {stops:?}, узел сохранён {same_node}");
}

/*
Две панели терминала читают один справочник. Наблюдателю достаточно Weak.
Он может получить временного владельца, пока справочник ещё доступен.
*/

struct Layout {
    docks: Vec<u8>,
}

fn layout_demo() {
    // Rc ведёт счётчик владельцев в одном потоке.
    let panel = Rc::new(Layout { docks: vec![2, 4] });
    let second_panel = Rc::clone(&panel);
    let observer: Weak<Layout> = Rc::downgrade(&panel);
    let shared = Rc::ptr_eq(&panel, &second_panel);
    let strong = Rc::strong_count(&panel);
    let weak = Rc::weak_count(&panel);
    assert!(shared);
    assert_eq!((strong, weak), (2, 1));
    println!("Панели — общий справочник {shared}, сильных ссылок {strong}, слабых {weak}");

    // second_panel.docks.push(6);
    drop(panel);
    assert_eq!(Rc::strong_count(&second_panel), 1);
    let strong_during_read = {
        let visible = observer.upgrade().expect("Вторая панель держит справочник");
        assert_eq!(visible.docks, [2, 4]);
        Rc::strong_count(&second_panel)
    };
    assert_eq!(strong_during_read, 2);
    assert_eq!(Rc::strong_count(&second_panel), 1);
    drop(second_panel);
    // Данные справочника освобождены. Weak ещё удерживает блок со счётчиками.
    let visible_after_close = observer.upgrade().is_some();
    assert!(!visible_after_close);
    println!(
        "Наблюдатель — владельцев во время чтения {strong_during_read}, доступ после закрытия {visible_after_close}"
    );
    println!("Справочник — после закрытия обеих панелей наблюдатель не получил данные");
}

struct DockNode {
    dock: u8,
    parent: Weak<DockNode>,
    left: Option<Rc<DockNode>>,
    right: Option<Rc<DockNode>>,
}

impl DockNode {
    fn leaf(dock: u8, parent: Weak<Self>) -> Rc<Self> {
        Rc::new(Self {
            dock,
            parent,
            left: None,
            right: None,
        })
    }
}

fn find_subtree(mut current: Option<&Rc<DockNode>>, dock: u8) -> Option<Rc<DockNode>> {
    while let Some(node) = current {
        current = match dock.cmp(&node.dock) {
            Ordering::Less => node.left.as_ref(),
            Ordering::Greater => node.right.as_ref(),
            Ordering::Equal => return Some(Rc::clone(node)),
        };
    }
    None
}

fn subtree_demo() {
    let tree = Rc::new_cyclic(|root| DockNode {
        dock: 4,
        parent: Weak::new(),
        left: Some(Rc::new_cyclic(|branch| DockNode {
            dock: 2,
            parent: root.clone(),
            left: Some(DockNode::leaf(1, branch.clone())),
            right: None,
        })),
        right: Some(DockNode::leaf(7, root.clone())),
    });
    assert!(find_subtree(None, 2).is_none());
    assert!(find_subtree(Some(&tree), 5).is_none());
    assert_eq!(find_subtree(Some(&tree), 7).map(|node| node.dock), Some(7));
    let found = find_subtree(Some(&tree), 2).expect("В дереве есть док 2");
    let shared = tree
        .left
        .as_ref()
        .is_some_and(|node| Rc::ptr_eq(node, &found));
    assert!(shared);
    let parent_before = found.parent.upgrade().map(|node| node.dock);
    assert_eq!(parent_before, Some(4));
    drop(tree);

    let parent_after = found.parent.upgrade().map(|node| node.dock);
    let child = found.left.as_ref().map(|node| node.dock);
    assert_eq!((found.dock, child, parent_after), (2, Some(1), None));
    println!(
        "Поддерево — корень {}, дочерний док {child:?}, тот же узел {shared}, родитель {parent_before:?} → {parent_after:?}",
        found.dock
    );
}

/*
Текущий рейс сохраняет настройки, с которыми начался. Для следующих
рейсов публикуется обновлённая версия с другим лимитом.
*/

#[derive(Clone, Debug, PartialEq)]
struct Settings {
    version: u32,
    carton_limit: u16,
}

fn settings_demo() {
    // Arc использует атомарный счётчик владельцев. clone здесь добавляет владельца снимка.
    let mut published = Arc::new(Settings {
        version: 1,
        carton_limit: 12,
    });
    let active_trip = Arc::clone(&published);
    assert!(Arc::ptr_eq(&published, &active_trip));
    assert_eq!(Arc::strong_count(&published), 2);
    assert!(Arc::get_mut(&mut published).is_none());

    // При двух владельцах make_mut копирует настройки для обновления.
    let updated = Arc::make_mut(&mut published);
    updated.version = 2;
    updated.carton_limit = 16;
    assert!(!Arc::ptr_eq(&published, &active_trip));
    assert_eq!(Arc::strong_count(&published), 1);
    assert_eq!(Arc::strong_count(&active_trip), 1);
    assert_eq!(
        *active_trip,
        Settings {
            version: 1,
            carton_limit: 12
        }
    );
    assert_eq!(
        *published,
        Settings {
            version: 2,
            carton_limit: 16
        }
    );
    assert!(Arc::get_mut(&mut published).is_some());
    println!(
        "Лимит текущего рейса — {}, следующего — {}",
        active_trip.carton_limit, published.carton_limit
    );

    let final_settings =
        Arc::try_unwrap(published).expect("У новых настроек остался один владелец");
    assert_eq!(
        final_settings,
        Settings {
            version: 2,
            carton_limit: 16
        }
    );

    let mut next = Arc::new(final_settings);
    let observer = Arc::downgrade(&next);
    let unique_with_observer = Arc::get_mut(&mut next).is_some();
    assert!(!unique_with_observer);
    Arc::make_mut(&mut next).carton_limit = 18;
    let observer_retained = observer.upgrade().is_some();
    assert!(!observer_retained);
    let new_observer = Arc::downgrade(&next);
    let limit = Arc::try_unwrap(next)
        .expect("Слабая ссылка не удерживает настройки")
        .carton_limit;
    assert_eq!(limit, 18);
    assert!(new_observer.upgrade().is_none());
    println!(
        "Слабый наблюдатель — прямой доступ {unique_with_observer}, связь после обновления {observer_retained}, извлечён лимит {limit}"
    );
}

#[derive(Default)]
struct Dictionary {
    terminal: bool,
    children: BTreeMap<char, Dictionary>,
}

impl Dictionary {
    fn insert(&mut self, word: &str) {
        let mut node = self;
        for character in word.chars() {
            node = node.children.entry(character).or_default();
        }
        node.terminal = true;
    }

    fn follow(&self, prefix: &str) -> Option<&Self> {
        prefix
            .chars()
            .try_fold(self, |node, character| node.children.get(&character))
    }

    fn contains(&self, word: &str) -> bool {
        self.follow(word).is_some_and(|node| node.terminal)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.follow(prefix)
            .is_some_and(|node| node.terminal || !node.children.is_empty())
    }
}

fn dictionary_demo() {
    let mut dictionary = Dictionary::default();
    assert!(!dictionary.starts_with(""));
    for word in ["склад", "складской", "скан", "док"] {
        dictionary.insert(word);
    }
    let published = Arc::new(dictionary);
    let second_panel = Arc::clone(&published);
    let prefix = published.starts_with("скл");
    let exact_prefix = second_panel.contains("скл");
    let exact_word = second_panel.contains("склад");
    let owners = Arc::strong_count(&published);
    assert!(Arc::ptr_eq(&published, &second_panel));
    assert_eq!(
        (prefix, exact_prefix, exact_word, owners),
        (true, false, true, 2)
    );
    assert!(published.starts_with(""));
    assert!(!published.contains(""));
    assert!(!published.starts_with("скло"));
    assert!(!published.contains("Склад"));
    println!(
        "Словарь — префикс «скл» {prefix}, слово «скл» {exact_prefix}, слово «склад» {exact_word}, владельцев {owners}"
    );
}

/*
До ответа диспетчера места на рейс удерживает временная бронь.
При отказе или выходе из блока места возвращаются. После подтверждения
бронь становится постоянной, а места остаются занятыми.
*/

#[derive(Debug, PartialEq)]
enum BookingError {
    NoCapacity { requested: usize, free: usize },
    Declined,
}

struct Slots {
    free: usize,
}

struct Hold<'slots> {
    slots: &'slots mut Slots,
    count: usize,
    confirmed: bool,
}

impl Slots {
    fn hold(&mut self, count: usize) -> Result<Hold<'_>, BookingError> {
        let remaining = self
            .free
            .checked_sub(count)
            .ok_or(BookingError::NoCapacity {
                requested: count,
                free: self.free,
            })?;
        self.free = remaining;
        // С этого момента за возврат мест отвечает Hold.
        Ok(Hold {
            slots: self,
            count,
            confirmed: false,
        })
    }
}

impl Hold<'_> {
    fn free_after_hold(&self) -> usize {
        self.slots.free
    }

    fn confirm(mut self) {
        self.confirmed = true;
    }
}

// Время удержания мест связано с владельцем Hold. Drop выполняет откат при его уничтожении.
impl Drop for Hold<'_> {
    fn drop(&mut self) {
        if !self.confirmed {
            self.slots.free += self.count;
        }
    }
}

fn book(
    slots: &mut Slots,
    count: usize,
    authorize: impl FnOnce(usize) -> Result<(), BookingError>,
) -> Result<(), BookingError> {
    let hold = slots.hold(count)?;
    authorize(hold.free_after_hold())?;
    hold.confirm();
    Ok(())
}

fn booking_demo() {
    let mut slots = Slots { free: 8 };
    let free_during_hold = {
        let hold = slots.hold(2).expect("В примере есть свободные места");
        hold.free_after_hold()
    };
    assert_eq!(free_during_hold, 6);
    assert_eq!(slots.free, 8);
    println!(
        "Бронь в блоке — свободно {free_during_hold}, после выхода {}",
        slots.free
    );

    assert_eq!(
        book(&mut slots, 3, |free| {
            assert_eq!(free, 5);
            Err(BookingError::Declined)
        }),
        Err(BookingError::Declined)
    );
    assert_eq!(slots.free, 8);
    println!("Отказ диспетчера — свободных мест снова {}", slots.free);
    assert_eq!(
        book(&mut slots, 9, |_| panic!("Резерв не создан")),
        Err(BookingError::NoCapacity {
            requested: 9,
            free: 8
        })
    );
    assert_eq!(slots.free, 8);

    assert_eq!(
        book(&mut slots, 3, |free| {
            assert_eq!(free, 5);
            Ok(())
        }),
        Ok(())
    );
    assert_eq!(slots.free, 5);
    println!("Свободных мест после подтверждения — {}", slots.free);
}

struct DropTrace(&'static str);

impl Drop for DropTrace {
    fn drop(&mut self) {
        println!("Уничтожение — {}", self.0);
    }
}

struct ExportResources {
    _output: DropTrace,
    _reservation: DropTrace,
}

fn drop_order_demo() {
    let _terminal = DropTrace("терминал");
    let _export = ExportResources {
        _output: DropTrace("файл отчёта"),
        _reservation: DropTrace("бронь каталога"),
    };
    let _view = DropTrace("просмотр");
}

fn main() {
    route_demo();
    merge_demo();
    layout_demo();
    subtree_demo();
    settings_demo();
    dictionary_demo();
    booking_demo();
    drop_order_demo();
}
