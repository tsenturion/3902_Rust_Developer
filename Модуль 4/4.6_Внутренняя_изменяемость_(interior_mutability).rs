/*
cargo run --offline --bin interior_mutability
cargo run --offline --bin interior_mutability -- borrow-conflict
*/

// Второй запуск показывает ожидаемую панику при конфликте заимствований.

use std::cell::{BorrowError, BorrowMutError, Cell, LazyCell, OnceCell, Ref, RefCell, RefMut};
use std::rc::Rc;

/*
Оператор открывает один и тот же предварительный просмотр. Текст отчёта
нужно собрать при первом обращении. Счётчик обращений можно прочитать и
сбросить отдельно. Оба действия доступны через общую ссылку на отчёт.
*/

struct Preview {
    zones: [String; 2],
    text: OnceCell<String>,
    requests: Cell<usize>,
}

impl Preview {
    fn show(&self) -> &str {
        // Cell возвращает копию числа и позволяет записать новое значение через &self.
        self.requests.set(self.requests.get().saturating_add(1));
        // После первого заполнения OnceCell возвращает ссылку на сохранённый текст.
        self.text.get_or_init(|| self.zones.join(", ")).as_str()
    }

    fn take_requests(&self) -> usize {
        self.requests.replace(0)
    }
}

fn preview_demo() {
    let mut report = Preview {
        zones: [String::from("Приёмка"), String::from("Резерв")],
        text: OnceCell::new(),
        requests: Cell::new(0),
    };
    assert!(report.text.get().is_none());
    let first = report.show();
    let second = report.show();
    assert_eq!(first, "Приёмка, Резерв");
    let same_text = std::ptr::eq(first, second);
    assert!(same_text);
    let requests = report.take_requests();
    assert_eq!(requests, 2);
    let after_reset = report.take_requests();
    assert_eq!(after_reset, 0);
    println!("Просмотр «{first}» открыт {requests} раза");
    println!("Счётчик обращений — до сброса {requests}, после {after_reset}");

    let refused = report
        .text
        .set(String::from("Пробная замена"))
        .expect_err("Сохранённый текст уже существует");
    assert_eq!(refused, "Пробная замена");
    assert_eq!(
        report.text.get().map(String::as_str),
        Some("Приёмка, Резерв")
    );
    println!("Однократный кеш — общий текст {same_text}, отклонено «{refused}»");

    let previous = report.text.take().expect("Текст был подготовлен");
    assert!(report.text.get().is_none());
    report.zones[1] = String::from("Выдача");
    let next = report.show();
    assert_eq!(
        (previous.as_str(), next),
        ("Приёмка, Резерв", "Приёмка, Выдача")
    );
    println!("Сброс кеша — прежний текст «{previous}», новый «{next}»");
}

fn cell_value_demo() {
    let draft = Cell::new(String::from("Черновик"));
    let previous = draft.replace(String::from("Готово"));
    let extracted = draft.take();
    let remaining = draft.into_inner();
    assert_eq!(
        (previous.as_str(), extracted.as_str(), remaining.as_str()),
        ("Черновик", "Готово", "")
    );
    println!(
        "Замена строки — прежняя «{previous}», извлечена «{extracted}», осталось {remaining:?}"
    );
}

fn lazy_demo() {
    let preparations = Cell::new(0);
    let label = LazyCell::new(|| {
        preparations.set(preparations.get() + 1);
        String::from("Отчёт о доках")
    });
    let before = preparations.get();
    let first = LazyCell::force(&label);
    let second = LazyCell::force(&label);
    let after = preparations.get();
    assert_eq!((before, after, first.as_str()), (0, 1, "Отчёт о доках"));
    assert!(std::ptr::eq(first, second));
    println!("Ленивый текст — до чтения {before}, после двух чтений {after}, значение «{first}»");
}

/*
Панель маршрута и редактор используют одно дерево зон. Редактор получает
владельца выбранного узла и может поменять его лимит. Когда у оператора
открыт просмотр, попытка записи возвращает ошибку. После закрытия
просмотра новые данные становятся видны при поиске от корня.
*/

type ZoneLink = Rc<RefCell<Zone>>;

struct Zone {
    number: u8,
    name: String,
    limit: u16,
    children: Vec<ZoneLink>,
}

fn find(root: &ZoneLink, number: u8) -> Result<Option<ZoneLink>, BorrowError> {
    let zone = root.try_borrow()?;
    if zone.number == number {
        // Клонируем Rc на узел. Guard остаётся локальным для этого вызова.
        return Ok(Some(Rc::clone(root)));
    }
    for child in &zone.children {
        if let Some(found) = find(child, number)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn change_limit(zone: &ZoneLink, limit: u16) -> Result<u16, BorrowMutError> {
    let mut data = zone.try_borrow_mut()?;
    Ok(std::mem::replace(&mut data.limit, limit))
}

fn zone_demo() {
    let intake = Rc::new(RefCell::new(Zone {
        number: 2,
        name: String::from("Приёмка"),
        limit: 20,
        children: Vec::new(),
    }));
    let reserve = Rc::new(RefCell::new(Zone {
        number: 4,
        name: String::from("Резерв"),
        limit: 12,
        children: Vec::new(),
    }));
    let root = Rc::new(RefCell::new(Zone {
        number: 1,
        name: String::from("Склад"),
        limit: 0,
        children: vec![Rc::clone(&intake), Rc::clone(&reserve)],
    }));
    let selected = find(&root, 4).expect("Дерево доступно").expect("Зона есть");
    assert!(Rc::ptr_eq(&selected, &reserve));
    assert!(find(&root, 99).expect("Дерево доступно").is_none());

    {
        // Ref::map оставляет только поле, а заимствование узла продолжается.
        let first_view = Ref::map(reserve.borrow(), |zone| &zone.limit);
        let second_view = selected.borrow();
        assert_eq!((*first_view, second_view.limit), (12, 12));
        let changed = change_limit(&selected, 16).is_ok();
        assert!(!changed);
        assert_eq!(*first_view, 12);
        // Другой RefCell можно изменить, пока открыт просмотр резерва.
        assert_eq!(change_limit(&intake, 24).expect("Приёмка доступна"), 20);
        let neighbor = intake.borrow().limit;
        println!(
            "Общий просмотр — лимиты {} и {}, запись {changed}, соседний лимит {neighbor}",
            *first_view, second_view.limit
        );
    }
    // Оба guard чтения уничтожены. Теперь доступна запись.
    assert_eq!(
        change_limit(&selected, 16).expect("Редактирование доступно"),
        12
    );

    {
        let (mut name, limit) = RefMut::map_split(selected.borrow_mut(), |zone| {
            (&mut zone.name, &mut zone.limit)
        });
        name.push_str(" северный");
        assert_eq!(*limit, 16);
        // Пока действует RefMut, даже поиск от корня не может прочитать этот узел.
        assert!(find(&root, 4).is_err());
        assert!(reserve.try_borrow().is_err());
        assert!(reserve.try_borrow_mut().is_err());
        drop(name);
        let readable = reserve.try_borrow().is_ok();
        assert!(!readable);
        println!(
            "Разделённая запись — после освобождения имени узел доступен для чтения {readable}"
        );
    }

    let refreshed = find(&root, 4).expect("Дерево доступно").expect("Зона есть");
    let zone = refreshed.borrow();
    assert_eq!(zone.name, "Резерв северный");
    assert_eq!(zone.limit, 16);
    println!("Зона «{}» — новый лимит {}", zone.name, zone.limit);
}

mod binary_tree {
    use std::cell::{RefCell, RefMut};
    use std::cmp::Ordering;
    use std::rc::Rc;

    type Link = Rc<RefCell<Node>>;
    type Tree = Option<Link>;

    struct Node {
        key: i32,
        children: [Tree; 2],
    }

    impl Node {
        fn shared(key: i32) -> Link {
            Rc::new(RefCell::new(Self {
                key,
                children: [None, None],
            }))
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum TreeError {
        ReadConflict,
        WriteConflict,
        Overflow,
    }

    fn insert(root: &mut Tree, key: i32) -> Result<bool, TreeError> {
        let Some(mut current) = root.clone() else {
            *root = Some(Node::shared(key));
            return Ok(true);
        };

        loop {
            let (branch, next) = {
                let node = current.try_borrow().map_err(|_| TreeError::ReadConflict)?;
                let branch = match key.cmp(&node.key) {
                    Ordering::Less => 0,
                    Ordering::Greater => 1,
                    Ordering::Equal => return Ok(false),
                };
                (branch, node.children[branch].clone())
            };
            if let Some(child) = next {
                current = child;
            } else {
                let mut parent = current
                    .try_borrow_mut()
                    .map_err(|_| TreeError::WriteConflict)?;
                parent.children[branch] = Some(Node::shared(key));
                return Ok(true);
            }
        }
    }

    fn reverse_order(root: &Tree) -> Result<Vec<Link>, TreeError> {
        let mut ordered = Vec::new();
        let mut ancestors = Vec::new();
        let mut current = root.clone();
        loop {
            while let Some(node) = current {
                current = node
                    .try_borrow()
                    .map_err(|_| TreeError::ReadConflict)?
                    .children[1]
                    .clone();
                ancestors.push(node);
            }
            let Some(node) = ancestors.pop() else {
                return Ok(ordered);
            };
            current = node
                .try_borrow()
                .map_err(|_| TreeError::ReadConflict)?
                .children[0]
                .clone();
            ordered.push(node);
        }
    }

    fn borrow_nodes(nodes: &[Link]) -> Result<Vec<RefMut<'_, Node>>, TreeError> {
        nodes
            .iter()
            .map(|node| node.try_borrow_mut().map_err(|_| TreeError::WriteConflict))
            .collect()
    }

    fn invert(root: &Tree) -> Result<(), TreeError> {
        let nodes = reverse_order(root)?;
        // Начинаем изменения после того, как получены все необходимые защиты.
        for mut node in borrow_nodes(&nodes)? {
            node.children.swap(0, 1);
        }
        Ok(())
    }

    fn make_greater(root: &Tree) -> Result<(), TreeError> {
        let nodes = reverse_order(root)?;
        let writable = borrow_nodes(&nodes)?;
        let mut sums = Vec::with_capacity(writable.len());
        let mut total = 0_i32;
        for node in &writable {
            total = total.checked_add(node.key).ok_or(TreeError::Overflow)?;
            sums.push(total);
        }
        for (mut node, sum) in writable.into_iter().zip(sums) {
            node.key = sum;
        }
        Ok(())
    }

    fn build(keys: &[i32]) -> Tree {
        let mut tree = None;
        for &key in keys {
            assert_eq!(insert(&mut tree, key), Ok(true));
        }
        tree
    }

    fn keys(tree: &Tree) -> Vec<i32> {
        reverse_order(tree)
            .expect("Узлы доступны для чтения")
            .into_iter()
            .rev()
            .map(|node| node.borrow().key)
            .collect()
    }

    pub fn run() {
        let mut tree = build(&[4, 2, 7, 1, 3]);
        let right = tree.as_ref().expect("Есть корень").borrow().children[1]
            .clone()
            .expect("Есть правая ветвь");
        let active_view = right.borrow();
        assert_eq!(insert(&mut tree, 5), Err(TreeError::WriteConflict));
        assert_eq!(keys(&tree), [1, 2, 3, 4, 7]);
        drop(active_view);
        assert_eq!(insert(&mut tree, 5), Ok(true));
        assert_eq!(insert(&mut tree, 5), Ok(false));
        let inserted = keys(&tree);
        assert_eq!(inserted, [1, 2, 3, 4, 5, 7]);
        assert_eq!(
            right.borrow().children[0]
                .as_ref()
                .map(|node| node.borrow().key),
            Some(5)
        );
        println!("Вставка в дерево — {inserted:?}");

        let left = tree.as_ref().expect("Есть корень").borrow().children[0]
            .clone()
            .expect("Есть левая ветвь");
        let active_view = left.borrow();
        let conflict = invert(&tree);
        assert_eq!(conflict, Err(TreeError::WriteConflict));
        let preserved = keys(&tree) == inserted;
        assert!(preserved);
        drop(active_view);
        assert_eq!(invert(&tree), Ok(()));
        let inverted = keys(&tree);
        assert_eq!(inverted, [7, 5, 4, 3, 2, 1]);
        let root = tree.as_ref().expect("Есть корень").borrow();
        assert!(Rc::ptr_eq(
            root.children[1].as_ref().expect("Ветка переставлена"),
            &left
        ));
        assert_eq!(invert(&None), Ok(()));
        println!(
            "Отражение дерева — при конфликте сохранено {preserved}, после освобождения {inverted:?}"
        );

        let greater = build(&[4, 2, 7, 1, 3]);
        assert_eq!(make_greater(&greater), Ok(()));
        let sums = keys(&greater);
        assert_eq!(sums, [17, 16, 14, 11, 7]);
        assert_eq!(make_greater(&None), Ok(()));
        println!("Суммы ключей — {sums:?}");

        let boundary = build(&[1, i32::MAX]);
        let failure = make_greater(&boundary);
        let unchanged = keys(&boundary);
        assert_eq!(failure, Err(TreeError::Overflow));
        assert_eq!(unchanged, [1, i32::MAX]);
        println!("Граница суммы — {failure:?}, дерево {unchanged:?}");
    }
}

fn borrow_conflict() {
    let limit = RefCell::new(12);
    let _view = limit.borrow();
    // borrow_mut проверяет действующие заимствования при выполнении.
    *limit.borrow_mut() = 16;
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("borrow-conflict") {
        borrow_conflict();
        return;
    }
    preview_demo();
    cell_value_demo();
    lazy_demo();
    zone_demo();
    binary_tree::run();
}
