/*
cargo run --offline --bin closures_intro
*/

/*
На складе готовят рейс тележки. В контейнер помещается 12 единиц товара,
а на тележку — 3 контейнера. Рассчитать потребность по каждой заявке,
поставить срочные заявки первыми, внутри одной срочности упорядочить их
по проходу и номеру. Загружать готовые заявки целиком. Если для заявки
нет места, оставить её на следующий рейс и проверить остальные.
*/

const REQUEST_COUNT: usize = 6;

struct PickRequest {
    order: u32,
    aisle: u8,
    units: u16,
    urgent: bool,
    ready: bool,
}

struct PickLine {
    order: u32,
    aisle: u8,
    totes: u16,
    urgent: bool,
    ready: bool,
}

fn main() {
    let station = String::from("Сборка-4");
    let tote_capacity = 12_u16;
    let cart_capacity = 3_u16;

    // У замыкания без параметров две соседние черты. Оно читает данные из main.
    let print_header = || println!("Пост {station} — вместимость тележки {cart_capacity} конт.");
    print_header();

    // Типы можно записать явно. При вызове tote_capacity передавать не нужно.
    let totes_for = |units: u16| -> u16 { units.div_ceil(tote_capacity) };
    assert_eq!(totes_for(0), 0);
    assert_eq!(totes_for(12), 1);
    assert_eq!(totes_for(13), 2);

    // Замыкание принимает u16, поэтому строка вызовет ошибку типа.
    // let _ = totes_for("12");

    // Обычная функция получает настройки через параметры.
    // fn named_rule(units: u16) -> u16 { units.div_ceil(tote_capacity) }

    let requests: [PickRequest; REQUEST_COUNT] = [
        PickRequest {
            order: 816,
            aisle: 3,
            units: 1,
            urgent: false,
            ready: true,
        },
        PickRequest {
            order: 812,
            aisle: 2,
            units: 25,
            urgent: true,
            ready: false,
        },
        PickRequest {
            order: 813,
            aisle: 3,
            units: 13,
            urgent: false,
            ready: true,
        },
        PickRequest {
            order: 810,
            aisle: 2,
            units: 12,
            urgent: true,
            ready: true,
        },
        PickRequest {
            order: 811,
            aisle: 1,
            units: 37,
            urgent: true,
            ready: true,
        },
        PickRequest {
            order: 815,
            aisle: 1,
            units: 12,
            urgent: false,
            ready: true,
        },
    ];

    // map вызывает замыкание для каждой заявки и собирает новый массив.
    // Тип request выводится из типа элементов requests.
    let mut plan = requests.map(|request| PickLine {
        order: request.order,
        aisle: request.aisle,
        totes: totes_for(request.units),
        urgent: request.urgent,
        ready: request.ready,
    });

    // Ключ сравнивается слева направо. У срочной заявки !urgent даёт false,
    // который при сортировке идёт раньше true.
    plan.sort_by_key(|line| (!line.urgent, line.aisle, line.order));
    // each_ref даёт ссылки на строки, чтобы план остался доступен после map.
    let ordered_ids = plan.each_ref().map(|line| line.order);
    assert_eq!(ordered_ids, [811, 810, 812, 815, 813, 816]);
    assert_eq!(plan.each_ref().map(|line| line.totes), [4, 1, 3, 1, 2, 1]);

    let fits_cart = |line: &PickLine, free: u16| line.ready && line.totes <= free;
    // position проверяет строки по порядку и возвращает индекс первого совпадения.
    let first = plan.iter().position(|line| fits_cart(line, cart_capacity));
    assert_eq!(first, Some(1));
    assert_eq!(plan.iter().position(|line| fits_cart(line, 0)), None);
    if let Some(index) = first {
        println!("Первая подходящая заявка — {}", plan[index].order);
    }

    let mut free_totes = cart_capacity;
    let mut loaded = [false; REQUEST_COUNT];
    {
        // Замыкание заимствует free_totes для изменения. Вызов требует привязки let mut.
        let mut reserve_on_cart = |line: &PickLine| {
            if !fits_cart(line, free_totes) {
                return false;
            }
            free_totes -= line.totes;
            true
        };

        // Заимствование ещё нужно для вызова reserve_on_cart в цикле.
        // let _before_loading = free_totes;

        for index in 0..plan.len() {
            let line = &plan[index];
            loaded[index] = reserve_on_cart(line);
            if loaded[index] {
                println!("Заявка {} — загружено {} конт.", line.order, line.totes);
            }
        }
    }

    assert_eq!(loaded, [false, true, false, true, false, true]);
    assert_eq!(free_totes, 0);
    println!("Свободных мест на тележке — {free_totes}");
}
