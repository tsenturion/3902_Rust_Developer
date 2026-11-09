/*
cargo run --offline --bin practice_3_5
*/

/*
Задача 1. Минимальная мощность обработчика

Подобрать наименьшее число рабочих мест, способное обработать весь поток.
Проверка мощности передаётся вызывающим кодом и может считать свои вызовы.
Если некоторое число мест справляется, любое большее тоже справляется.
Границы поиска включены. Для пустого диапазона или недостаточной верхней
границы вернуть None.
*/
mod capacity {
    fn first_sufficient(
        mut low: u32,
        mut high: u32,
        mut passes: impl FnMut(u32) -> bool,
    ) -> Option<u32> {
        if low > high || !passes(high) {
            return None;
        }

        while low < high {
            let middle = low + (high - low) / 2;
            if passes(middle) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        Some(low)
    }

    pub fn run() {
        let jobs = 95;
        let mut checks = 0;
        let required = first_sufficient(1, 12, |workers| {
            checks += 1;
            workers * 12 >= jobs
        });
        assert_eq!(required, Some(8));
        assert!(checks > 1);
        assert_eq!(first_sufficient(1, 7, |workers| workers * 12 >= jobs), None);
        assert_eq!(
            first_sufficient(3, 7, |workers| workers * 12 >= 36),
            Some(3)
        );
        assert_eq!(first_sufficient(7, 3, |_| panic!("Пустой диапазон")), None);
        assert_eq!(
            first_sufficient(u32::MAX - 1, u32::MAX, |value| value == u32::MAX),
            Some(u32::MAX)
        );
        if let Some(workers) = required {
            println!("Для {jobs} операций нужны {workers} рабочих мест");
        }
    }
}

/*
Задача 2. Слияние диапазонов обслуживания

Объединить пересекающиеся диапазоны, включая те, у которых совпадает общая
граница. Вход отсортирован по началу. Неверные границы или порядок вернуть
как ошибку с индексом и сохранить вход. Записать объединённые диапазоны
в начало слайса и вернуть длину результата. Границы должны сравниваться
по порядку. Слияние должно работать и для строк без их клонирования.
*/
mod ranges {
    #[derive(Debug, PartialEq)]
    struct Interval<T> {
        start: T,
        end: T,
    }

    #[derive(Debug, PartialEq)]
    enum MergeError {
        Reversed { index: usize },
        OutOfOrder { index: usize },
    }

    fn merge_sorted<T: Ord>(intervals: &mut [Interval<T>]) -> Result<usize, MergeError> {
        for index in 0..intervals.len() {
            if intervals[index].start > intervals[index].end {
                return Err(MergeError::Reversed { index });
            }
            if index > 0 && intervals[index - 1].start > intervals[index].start {
                return Err(MergeError::OutOfOrder { index });
            }
        }
        if intervals.is_empty() {
            return Ok(0);
        }

        let mut last = 0;
        for index in 1..intervals.len() {
            if intervals[index].start <= intervals[last].end {
                if intervals[index].end > intervals[last].end {
                    // Делим слайс, чтобы получить две изменяемые ссылки на разные интервалы.
                    let (prefix, rest) = intervals.split_at_mut(index);
                    std::mem::swap(&mut prefix[last].end, &mut rest[0].end);
                }
            } else {
                last += 1;
                intervals.swap(last, index);
            }
        }
        Ok(last + 1)
    }

    pub fn run() {
        let mut time = [
            Interval {
                start: 60,
                end: 120,
            },
            Interval {
                start: 90,
                end: 100,
            },
            Interval {
                start: 120,
                end: 180,
            },
            Interval {
                start: 200,
                end: 240,
            },
            Interval {
                start: 230,
                end: 250,
            },
        ];
        assert_eq!(merge_sorted(&mut time), Ok(2));
        assert_eq!(
            &time[..2],
            &[
                Interval {
                    start: 60,
                    end: 180
                },
                Interval {
                    start: 200,
                    end: 250
                }
            ]
        );

        let mut sectors = [
            Interval {
                start: String::from("A01"),
                end: String::from("A04"),
            },
            Interval {
                start: String::from("A04"),
                end: String::from("A09"),
            },
        ];
        assert_eq!(merge_sorted(&mut sectors), Ok(1));
        assert_eq!(
            (sectors[0].start.as_str(), sectors[0].end.as_str()),
            ("A01", "A09")
        );
        assert_eq!(merge_sorted::<u16>(&mut []), Ok(0));

        let mut invalid = [
            Interval { start: 1, end: 3 },
            Interval { start: 2, end: 5 },
            Interval { start: 4, end: 3 },
        ];
        assert_eq!(
            merge_sorted(&mut invalid),
            Err(MergeError::Reversed { index: 2 })
        );
        assert_eq!(invalid[0].end, 3);
        let mut unordered = [Interval { start: 4, end: 5 }, Interval { start: 2, end: 3 }];
        assert_eq!(
            merge_sorted(&mut unordered),
            Err(MergeError::OutOfOrder { index: 1 })
        );
        println!(
            "Смежные секторы объединены в {}–{}",
            sectors[0].start, sectors[0].end
        );
    }
}

/*
Задача 3. Таблица назначений в готовом буфере

В слайсе ячеек хранить пары ключей и значений. Ключи в начальном буфере
уникальны. Поиск заимствует значение.
Повторная запись по тому же ключу заменяет значение и возвращает прежнее,
даже если свободных ячеек нет. При переполнении вернуть новую пару
вызывающему коду и сохранить записи. Для ключа достаточно равенства.
*/
mod assignments {
    struct Table<'slots, K, V> {
        slots: &'slots mut [Option<(K, V)>],
    }

    impl<K: PartialEq, V> Table<'_, K, V> {
        fn get(&self, key: &K) -> Option<&V> {
            for slot in &*self.slots {
                if let Some((stored, value)) = slot {
                    if stored == key {
                        return Some(value);
                    }
                }
            }
            None
        }

        fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
            for slot in &mut *self.slots {
                if let Some((stored, old_value)) = slot {
                    if *stored == key {
                        return Ok(Some(std::mem::replace(old_value, value)));
                    }
                }
            }

            for slot in &mut *self.slots {
                if slot.is_none() {
                    *slot = Some((key, value));
                    return Ok(None);
                }
            }
            Err((key, value))
        }
    }

    pub fn run() {
        let mut desks = [None, None];
        let mut table = Table { slots: &mut desks };
        assert_eq!(table.insert(12, String::from("Ирина")), Ok(None));
        assert_eq!(table.insert(13, String::from("Павел")), Ok(None));
        assert_eq!(
            table.insert(12, String::from("Анна")),
            Ok(Some(String::from("Ирина")))
        );
        assert_eq!(
            table.insert(14, String::from("Олег")),
            Err((14, String::from("Олег")))
        );
        assert_eq!(table.get(&14), None);
        assert_eq!(table.get(&13).map(String::as_str), Some("Павел"));
        assert_eq!(table.get(&12).map(String::as_str), Some("Анна"));

        let mut versions = [None];
        let mut table_by_name = Table {
            slots: &mut versions,
        };
        assert_eq!(
            table_by_name.insert(String::from("резерв"), 42_u16),
            Ok(None)
        );
        assert_eq!(table_by_name.get(&String::from("резерв")), Some(&42));
        if let Some(person) = table.get(&12) {
            println!("Стойка 12 — назначена {person}");
        }
    }
}

/*
Задача 4. Поиск доступной редакции документа

Найти данные редакции с наибольшим номером. Список запретов приходит
отдельно и может исчезнуть сразу после поиска. Возвращаемая ссылка должна
оставаться пригодной, пока доступен журнал редакций. При одинаковом номере
выбрать первую запись, при отсутствии доступных редакций вернуть None.
*/
mod revisions {
    struct Revision<T> {
        number: u32,
        payload: T,
    }

    fn latest_available<'history, T>(
        history: &'history [Revision<T>],
        blocked: &[u32],
    ) -> Option<&'history T> {
        let mut selected: Option<&Revision<T>> = None;
        for revision in history {
            if blocked.contains(&revision.number) {
                continue;
            }
            match selected {
                Some(current) if current.number >= revision.number => {}
                _ => selected = Some(revision),
            }
        }
        selected.map(|revision| &revision.payload)
    }

    pub fn run() {
        let history = [
            Revision {
                number: 3,
                payload: String::from("отчёт исправлен"),
            },
            Revision {
                number: 1,
                payload: String::from("черновик"),
            },
            Revision {
                number: 3,
                payload: String::from("повторная загрузка"),
            },
            Revision {
                number: 4,
                payload: String::from("на проверке"),
            },
        ];
        let selected = {
            let blocked = [4];
            latest_available(&history, &blocked)
        };
        assert_eq!(selected.map(String::as_str), Some("отчёт исправлен"));
        assert_eq!(latest_available(&history, &[1, 3, 4]), None);
        assert_eq!(latest_available::<String>(&[], &[]), None);
        assert_eq!(
            latest_available(
                &[Revision {
                    number: 1,
                    payload: 300_u32
                }],
                &[]
            ),
            Some(&300)
        );
        if let Some(text) = selected {
            println!("Доступная редакция — {text}");
        }
    }
}

/*
Задача 5. Печать складских этикеток

Один формат создаёт метку шкафа по номеру, другой — этикетку заказа
по его коду. Принтер использует вложенный формат. Перед печатью проверить
длину готовой этикетки в байтах. Граница включена. Одинаковая проверка
длины должна использоваться для обоих форматов.
*/
mod labels {
    trait Label<T> {
        fn render(&self, item: &T) -> String;
    }

    #[derive(Debug, PartialEq)]
    struct TooLong {
        bytes: usize,
        limit: usize,
    }

    trait PrintableLabel<T>: Label<T> {
        fn limit(&self) -> usize;

        fn render_checked(&self, item: &T) -> Result<String, TooLong> {
            let text = self.render(item);
            let limit = self.limit();
            if text.len() > limit {
                return Err(TooLong {
                    bytes: text.len(),
                    limit,
                });
            }
            Ok(text)
        }
    }

    struct CabinetLabel {
        zone: char,
    }

    impl Label<u16> for CabinetLabel {
        fn render(&self, number: &u16) -> String {
            format!("{}-{number:04}", self.zone)
        }
    }

    struct OrderLabel;

    impl Label<String> for OrderLabel {
        fn render(&self, code: &String) -> String {
            format!("З-{code}")
        }
    }

    struct Printer<F> {
        format: F,
        max_bytes: usize,
    }

    impl<T, F: Label<T>> Label<T> for Printer<F> {
        fn render(&self, item: &T) -> String {
            self.format.render(item)
        }
    }

    impl<T, F: Label<T>> PrintableLabel<T> for Printer<F> {
        fn limit(&self) -> usize {
            self.max_bytes
        }
    }

    pub fn run() {
        let cabinets = Printer {
            format: CabinetLabel { zone: 'A' },
            max_bytes: 6,
        };
        let cabinet = cabinets.render_checked(&27);
        assert_eq!(cabinet, Ok(String::from("A-0027")));
        assert_eq!(
            cabinets.render_checked(&10_000),
            Err(TooLong { bytes: 7, limit: 6 })
        );

        let orders = Printer {
            format: OrderLabel,
            max_bytes: 6,
        };
        assert_eq!(
            orders.render_checked(&String::from("781")),
            Ok(String::from("З-781"))
        );
        assert_eq!(
            orders.render_checked(&String::from("А781")),
            Err(TooLong { bytes: 8, limit: 6 })
        );
        if let Ok(text) = cabinet {
            println!("Этикетка шкафа — {text}");
        }
    }
}

/*
Задача 6. Проверка партии перед упаковкой

Проверить значение по списку правил в заданном порядке. Правила могут
иметь разные реализации и принимают один тип данных. При отказе указать
позицию первого не пройденного правила. Число товаров должно быть чётным
для упаковки парами и укладываться в лимит. Для объёма файла используется
тот же алгоритм проверки, но только правило верхнего предела.
*/
mod rules {
    #[derive(Debug, PartialEq)]
    enum Violation {
        AboveMaximum,
        NeedsPairs,
    }

    trait Rule<T> {
        fn check(&self, value: &T) -> Result<(), Violation>;
    }

    struct AtMost<T> {
        max: T,
    }

    impl<T: Ord> Rule<T> for AtMost<T> {
        fn check(&self, value: &T) -> Result<(), Violation> {
            if value <= &self.max {
                Ok(())
            } else {
                Err(Violation::AboveMaximum)
            }
        }
    }

    struct Pairs;

    impl Rule<u16> for Pairs {
        fn check(&self, value: &u16) -> Result<(), Violation> {
            if *value % 2 == 0 {
                Ok(())
            } else {
                Err(Violation::NeedsPairs)
            }
        }
    }

    #[derive(Debug, PartialEq)]
    struct Rejected {
        index: usize,
        cause: Violation,
    }

    fn validate<T>(value: &T, rules: &[&dyn Rule<T>]) -> Result<(), Rejected> {
        for index in 0..rules.len() {
            rules[index]
                .check(value)
                .map_err(|cause| Rejected { index, cause })?;
        }
        Ok(())
    }

    pub fn run() {
        let capacity = AtMost { max: 20_u16 };
        let pairs = Pairs;
        let packing: [&dyn Rule<u16>; 2] = [&capacity, &pairs];
        assert_eq!(validate(&20, &packing), Ok(()));
        assert_eq!(
            validate(&21, &packing),
            Err(Rejected {
                index: 0,
                cause: Violation::AboveMaximum
            })
        );
        assert_eq!(
            validate(&19, &packing),
            Err(Rejected {
                index: 1,
                cause: Violation::NeedsPairs
            })
        );

        let file_limit = AtMost { max: 4_000_000_u64 };
        assert_eq!(validate(&4_000_000, &[&file_limit]), Ok(()));
        assert_eq!(
            validate(&4_000_001, &[&file_limit]),
            Err(Rejected {
                index: 0,
                cause: Violation::AboveMaximum
            })
        );
        assert_eq!(validate(&19_u16, &[]), Ok(()));
        println!("Партия из 20 товаров прошла проверку вместимости и упаковки парами");
    }
}

fn main() {
    capacity::run();
    ranges::run();
    assignments::run();
    revisions::run();
    labels::run();
    rules::run();
}
