/*
cargo run --offline --bin generic_programming
*/

/*
Система мониторинга получает серии температур и текстовые коды состояния.
При повторной выгрузке оператору нужен индекс первого отличия от прежней
серии. Для отдельных показаний нужно проверить допустимый диапазон,
сохранив исходное значение и идентификатор источника. Диагностические
события тоже хранятся с идентификатором источника, но диапазон для них
не проверяется. Температура записана в десятых долях градуса, давление — в кПа.
*/

use std::cmp::Ordering;

// Обе серии содержат значения одного типа T. Функции достаточно уметь сравнивать их на равенство.
fn first_change<T>(previous: &[T], current: &[T]) -> Option<usize>
where
    T: PartialEq,
{
    let common_length = previous.len().min(current.len());
    for index in 0..common_length {
        if previous[index] != current[index] {
            return Some(index);
        }
    }

    if previous.len() != current.len() {
        return Some(common_length);
    }
    None
}

struct Reading<Source, T> {
    source: Source,
    value: T,
}

// Source и T могут быть разными типами. Для хранения и извлечения сравнения не нужны.
impl<Source, T> Reading<Source, T> {
    fn new(source: Source, value: T) -> Self {
        Self { source, value }
    }

    fn into_parts(self) -> (Source, T) {
        (self.source, self.value)
    }
}

#[derive(Debug, PartialEq)]
enum LimitCheck<T> {
    Within(T),
    Below(T),
    Above(T),
    Unordered(T),
}

struct Limits<T> {
    lower: T,
    upper: T,
}

impl<T: PartialOrd> Limits<T> {
    fn new(lower: T, upper: T) -> Option<Self> {
        if lower <= upper {
            Some(Self { lower, upper })
        } else {
            None
        }
    }

    fn classify(&self, value: T) -> LimitCheck<T> {
        match (
            value.partial_cmp(&self.lower),
            value.partial_cmp(&self.upper),
        ) {
            (None, _) | (_, None) => LimitCheck::Unordered(value),
            (Some(Ordering::Less), _) => LimitCheck::Below(value),
            (_, Some(Ordering::Greater)) => LimitCheck::Above(value),
            _ => LimitCheck::Within(value),
        }
    }
}

// Ограничение относится только к типу показания. Тип источника остаётся произвольным.
impl<Source, T> Reading<Source, T>
where
    T: PartialOrd,
{
    fn check(self, limits: &Limits<T>) -> Reading<Source, LimitCheck<T>> {
        let (source, value) = self.into_parts();
        Reading::new(source, limits.classify(value))
    }
}

struct ControllerFault {
    code: u16,
}

fn main() {
    let previous = [37_i16, 35, 33];
    let current = [37_i16, 34, 33];
    let temperature_change = first_change(&previous, &current);
    assert_eq!(temperature_change, Some(1));
    assert_eq!(first_change(&previous, &previous), None);
    assert_eq!(first_change(&previous, &current[..1]), Some(1));
    assert_eq!(first_change::<i16>(&[], &[]), None);

    let previous_codes = [String::from("РАБОТА"), String::from("ПАУЗА")];
    let current_codes = [
        String::from("РАБОТА"),
        String::from("ПАУЗА"),
        String::from("СТОП"),
    ];
    let code_change = first_change(&previous_codes, &current_codes);
    assert_eq!(code_change, Some(2));
    if let (Some(temperature_index), Some(code_index)) = (temperature_change, code_change) {
        println!(
            "Первое отличие температур — индекс {temperature_index}, кодов — индекс {code_index}"
        );
    }

    // Тип можно указать у имени, как здесь, или вывести из аргументов, как у пределов давления.
    let Some(cooler_limits) = Limits::<i16>::new(20, 80) else {
        panic!("В примере заданы корректные пределы температуры");
    };
    let Some(pressure_limits) = Limits::new(90.0_f32, 105.0) else {
        panic!("В примере заданы корректные пределы давления");
    };
    assert!(Limits::new(80_i16, 20).is_none());
    assert!(Limits::new(f32::NAN, 105.0).is_none());
    assert_eq!(cooler_limits.classify(20), LimitCheck::Within(20));
    assert_eq!(cooler_limits.classify(80), LimitCheck::Within(80));

    let cooling = Reading::new(String::from("Камера 2"), 19);
    let (source, verdict) = cooling.check(&cooler_limits).into_parts();
    assert_eq!(source, "Камера 2");
    assert_eq!(verdict, LimitCheck::Below(19));
    if let LimitCheck::Below(tenths) = verdict {
        println!(
            "{source} — температура {} °C ниже допустимой",
            f32::from(tenths) / 10.0
        );
    }

    let pressure: Reading<u16, f32> = Reading::new(12, 106.5);
    let (source, verdict) = pressure.check(&pressure_limits).into_parts();
    assert_eq!((source, verdict), (12, LimitCheck::Above(106.5)));
    // NaN не упорядочен относительно пределов. Проверяем вариант, поскольку NaN != NaN.
    assert!(
        matches!(pressure_limits.classify(f32::NAN), LimitCheck::Unordered(value) if value.is_nan())
    );

    let fault = Reading::new("Контроллер 7", ControllerFault { code: 42 });
    // Запись сбоя можно хранить, но для её значения не определено сравнение по порядку.
    // let _ = fault.check(&cooler_limits);
    let (source, event) = fault.into_parts();
    assert_eq!((source, event.code), ("Контроллер 7", 42));
    println!("{source} — зарегистрирован код сбоя {}", event.code);

    // В одном вызове first_change нельзя выбрать два разных типа для T.
    // let _ = first_change(&[1_i16], &[1_u16]);

    // Для поиска изменений T должен поддерживать сравнение на равенство.
    // let _ = first_change(&[ControllerFault { code: 42 }], &[ControllerFault { code: 43 }]);
}
