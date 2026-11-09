/*
cargo run --offline --bin practice_3_8
*/

/*
Задача 1. Отбор кодов для ручного терминала

Терминал позволяет набрать только символы с доступных клавиш. Оставить
непустые коды, у которых разрешён каждый символ. Регистр учитывается.
Сохранить порядок поступления и вернуть ссылки на исходные строки.
*/
mod labels {
    use std::collections::HashSet;

    const KEYBOARD_ROWS: [&[u8]; 3] = [b"qwertyuiop", b"asdfghjkl", b"zxcvbnm"];

    fn enterable<'text>(labels: &[&'text str], keys: &HashSet<char>) -> Vec<&'text str> {
        labels
            .iter()
            .copied()
            .filter(|label| {
                !label.is_empty() && label.chars().all(|character| keys.contains(&character))
            })
            .collect()
    }

    fn row_index(key: u8) -> Option<usize> {
        let lowercase = key.to_ascii_lowercase();
        KEYBOARD_ROWS
            .iter()
            .position(|row| row.contains(&lowercase))
    }

    fn single_row(word: &str) -> Option<usize> {
        let mut keys = word.bytes();
        let row = row_index(keys.next()?)?;
        keys.all(|key| row_index(key) == Some(row)).then_some(row)
    }

    pub fn run() {
        let keys = "AB-0123456789".chars().collect();
        let selected = enterable(&["B-21", "C-03", "A-17", "", "А-17", "a-17", "B-21"], &keys);
        assert_eq!(selected, ["B-21", "A-17", "B-21"]);
        assert!(enterable(&[], &keys).is_empty());
        assert!(enterable(&["A-17"], &HashSet::new()).is_empty());
        println!("Терминал принимает {selected:?}");

        let words = ["Type", "Dad", "Mix", "VbN", "Dad", "", "Q1", "Ёлка"];
        let selected: Vec<_> = words
            .into_iter()
            .filter(|word| single_row(word).is_some())
            .collect();
        assert_eq!(selected, ["Type", "Dad", "VbN", "Dad"]);
        assert_eq!(single_row("A"), Some(1));
        assert_eq!(single_row(""), None);
        assert_eq!(single_row("Q1"), None);
        assert_eq!(single_row("Ёлка"), None);
        println!("Один ряд — отобраны {selected:?}");
    }
}

/*
Задача 2. Число конфигураций при фиксации одного этапа

Для каждого этапа известно число допустимых вариантов. После фиксации
этапа число конфигураций равно произведению вариантов остальных этапов.
Посчитать все ответы через произведения слева и справа. Для одного этапа
ответ равен единице. Ноль означает отсутствие вариантов. Вернуть None,
если хотя бы один требуемый результат не помещается в u64.
*/
mod configurations {
    fn without_each(choices: &[u64]) -> Option<Vec<u64>> {
        let zeros = choices.iter().filter(|&&count| count == 0).count();
        if zeros > 1 {
            return Some(vec![0; choices.len()]);
        }
        if zeros == 1 {
            let nonzero_product = choices
                .iter()
                .copied()
                .filter(|&count| count != 0)
                .try_fold(1_u64, u64::checked_mul)?;
            return Some(
                choices
                    .iter()
                    .map(|&count| if count == 0 { nonzero_product } else { 0 })
                    .collect(),
            );
        }

        let mut counts = vec![1; choices.len()];
        // Позиция 0 оставляет единицу. Для остальных добавляем множитель слева.
        let mut left_product = 1_u64;
        for (count, &factor) in counts.iter_mut().skip(1).zip(choices) {
            left_product = left_product.checked_mul(factor)?;
            *count = left_product;
        }

        // Последняя позиция уже готова. Для остальных добавляем множители справа.
        let mut right_product = 1_u64;
        for (count, &factor) in counts.iter_mut().rev().skip(1).zip(choices.iter().rev()) {
            right_product = right_product.checked_mul(factor)?;
            *count = count.checked_mul(right_product)?;
        }
        Some(counts)
    }

    pub fn run() {
        let counts = without_each(&[2, 3, 5]).expect("Произведения помещаются в u64");
        assert_eq!(counts, [15, 10, 6]);
        let one_zero = without_each(&[2, 0, 5]).expect("Произведения помещаются в u64");
        let two_zeros = without_each(&[0, u64::MAX, u64::MAX, 0]);
        let overflow = without_each(&[0, u64::MAX, 2]);
        assert_eq!(without_each(&[]), Some(vec![]));
        assert_eq!(without_each(&[0]), Some(vec![1]));
        assert_eq!(one_zero, [0, 10, 0]);
        assert_eq!(two_zeros, Some(vec![0, 0, 0, 0]));
        assert_eq!(without_each(&[u64::MAX, 2]), Some(vec![2, u64::MAX]));
        assert_eq!(without_each(&[u64::MAX, 2, 1]), None);
        assert_eq!(overflow, None);
        println!("Варианты после фиксации этапов — {counts:?}");
        println!("Нулевые этапы — один {one_zero:?}, два {two_zeros:?}, переполнение {overflow:?}");
    }
}

/*
Задача 3. Частые коды ошибок в журнале

Подсчитать появления каждого кода и выбрать до limit самых частых.
При одинаковой частоте первым идёт меньший код по лексикографическому
порядку. Нулевой лимит и пустой журнал дают пустой список.
*/
mod errors {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap};

    fn count_codes<'text>(codes: &[&'text str]) -> HashMap<&'text str, usize> {
        codes
            .iter()
            .copied()
            .fold(HashMap::new(), |mut counts, code| {
                *counts.entry(code).or_insert(0) += 1;
                counts
            })
    }

    fn ranked_entries<'text>(
        counts: &HashMap<&'text str, usize>,
    ) -> impl Iterator<Item = (Reverse<usize>, &'text str)> {
        counts.iter().map(|(&code, &count)| (Reverse(count), code))
    }

    fn top_codes<'text>(
        counts: &HashMap<&'text str, usize>,
        limit: usize,
    ) -> Vec<(&'text str, usize)> {
        let mut ranked: Vec<_> = ranked_entries(counts).collect();
        ranked.sort_unstable();
        ranked
            .into_iter()
            .take(limit)
            .map(|(Reverse(count), code)| (code, count))
            .collect()
    }

    fn top_with_heap<'text>(
        counts: &HashMap<&'text str, usize>,
        limit: usize,
    ) -> Vec<(&'text str, usize)> {
        let limit = limit.min(counts.len());
        if limit == 0 {
            return Vec::new();
        }

        let mut candidates = BinaryHeap::with_capacity(limit);
        for entry in ranked_entries(counts) {
            if candidates.len() < limit {
                candidates.push(entry);
            } else if candidates.peek().is_some_and(|worst| entry < *worst) {
                // На вершине находится худший из оставленных кандидатов.
                candidates.pop();
                candidates.push(entry);
            }
        }

        candidates
            .into_sorted_vec()
            .into_iter()
            .map(|(Reverse(count), code)| (code, count))
            .collect()
    }

    pub fn run() {
        let codes = ["E42", "E10", "E42", "E10", "E42", "E07", "E07", "E99"];
        let counts = count_codes(&codes);
        let ranked = top_codes(&counts, 2);
        assert_eq!(ranked, [("E42", 3), ("E07", 2)]);
        assert_eq!(
            top_codes(&counts, 10),
            [("E42", 3), ("E07", 2), ("E10", 2), ("E99", 1)]
        );
        assert!(top_codes(&counts, 0).is_empty());
        assert!(top_codes(&HashMap::new(), 3).is_empty());
        println!("Частые ошибки — {ranked:?}");

        let from_heap = top_with_heap(&counts, 2);
        for limit in 0..=counts.len() + 1 {
            assert_eq!(top_with_heap(&counts, limit), top_codes(&counts, limit));
        }
        assert!(top_with_heap(&HashMap::new(), 3).is_empty());
        println!("Рейтинг из кучи — {from_heap:?}");
    }
}

/*
Задача 4. Строгий импорт загрузки доков

Строка «док;короба» содержит ненулевой номер дока и число коробов.
Пробелы вокруг полей допустимы. Найти первую некорректную строку и вернуть
её номер с единицы и причину. Если все строки корректны, сложить значения
по докам и выдать сводку в порядке номеров.
*/
mod import {
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq)]
    enum Cause {
        Format,
        Dock,
        Boxes,
    }

    #[derive(Debug, PartialEq)]
    struct ImportError {
        line: usize,
        cause: Cause,
    }

    fn parse_load(line: &str) -> Result<(u8, u16), Cause> {
        let (dock, boxes) = line.split_once(';').ok_or(Cause::Format)?;
        let dock = dock.trim().parse().map_err(|_| Cause::Dock)?;
        if dock == 0 {
            return Err(Cause::Dock);
        }
        let boxes = boxes.trim().parse().map_err(|_| Cause::Boxes)?;
        Ok((dock, boxes))
    }

    fn summarize(lines: &[&str]) -> Result<BTreeMap<u8, u128>, ImportError> {
        lines
            .iter()
            .enumerate()
            .try_fold(BTreeMap::new(), |mut totals, (index, line)| {
                let (dock, boxes) = parse_load(line).map_err(|cause| ImportError {
                    line: index + 1,
                    cause,
                })?;
                *totals.entry(dock).or_insert(0) += u128::from(boxes);
                Ok(totals)
            })
    }

    pub fn run() {
        let summary = summarize(&["2;12", "1; 5", " 2 ; 8 ", "1;0"]);
        assert_eq!(summary, Ok(BTreeMap::from([(1, 5), (2, 20)])));
        assert_eq!(summarize(&[]), Ok(BTreeMap::new()));
        let rejected = summarize(&["2;12", "2;много", "0;1"])
            .expect_err("Строка с текстовым количеством должна быть отклонена");
        assert_eq!(
            rejected,
            ImportError {
                line: 2,
                cause: Cause::Boxes
            }
        );
        assert_eq!(
            summarize(&["0;1"]),
            Err(ImportError {
                line: 1,
                cause: Cause::Dock
            })
        );
        assert_eq!(
            summarize(&["без разделителя"]),
            Err(ImportError {
                line: 1,
                cause: Cause::Format
            })
        );
        if let Ok(totals) = summary {
            println!("Сумма коробов по докам — {totals:?}");
        }
        println!(
            "Ошибка импорта — строка {}, причина {:?}",
            rejected.line, rejected.cause
        );
    }
}

/*
Задача 5. Сверка перечня допусков перед сменой

По действующему и требуемому перечням найти допуски для выдачи, отзыва
и сохранения. Повторы в исходных списках относятся к одному допуску.
Каждый список результата вывести в лексикографическом порядке.
*/
mod permissions {
    use std::collections::BTreeSet;

    #[derive(Debug, PartialEq)]
    struct Changes<'text> {
        grant: Vec<&'text str>,
        revoke: Vec<&'text str>,
        keep: Vec<&'text str>,
    }

    fn compare<'text>(current: &[&'text str], required: &[&'text str]) -> Changes<'text> {
        let current: BTreeSet<_> = current.iter().copied().collect();
        let required: BTreeSet<_> = required.iter().copied().collect();
        Changes {
            grant: required.difference(&current).copied().collect(),
            revoke: current.difference(&required).copied().collect(),
            keep: current.intersection(&required).copied().collect(),
        }
    }

    pub fn run() {
        let current = ["склад", "офис", "док", "офис"];
        let required = ["склад", "касса", "архив", "архив"];
        let changes = compare(&current, &required);
        assert_eq!(changes.grant, ["архив", "касса"]);
        assert_eq!(changes.revoke, ["док", "офис"]);
        assert_eq!(changes.keep, ["склад"]);

        let mut state: BTreeSet<_> = current.into_iter().collect();
        for &entry in &changes.revoke {
            state.remove(entry);
        }
        state.extend(changes.grant.iter().copied());
        assert_eq!(state, BTreeSet::from(required));
        let unchanged = compare(&["склад"], &["склад", "склад"]);
        assert!(unchanged.grant.is_empty() && unchanged.revoke.is_empty());
        assert_eq!(unchanged.keep, ["склад"]);
        println!(
            "Допуски к выдаче — {:?}, к отзыву — {:?}, сохранить — {:?}",
            changes.grant, changes.revoke, changes.keep
        );
    }
}

/*
Задача 6. Очередь заявок на текущую смену

Переместить заявки в очередь. Срочные поставить первыми, внутри каждой
группы сохранить порядок поступления. Передать на смену до limit первых
заявок, оставив остальные в очереди. Код каждой заявки передаётся
во владение получателю.
*/
mod dispatch {
    use std::collections::VecDeque;

    #[derive(Debug, PartialEq)]
    struct Request {
        code: String,
        urgent: bool,
    }

    fn schedule(requests: Vec<Request>) -> VecDeque<Request> {
        let (mut urgent, regular): (VecDeque<_>, VecDeque<_>) =
            requests.into_iter().partition(|request| request.urgent);
        urgent.extend(regular);
        urgent
    }

    fn take_shift(queue: &mut VecDeque<Request>, limit: usize) -> Vec<Request> {
        let end = limit.min(queue.len());
        queue.drain(..end).collect()
    }

    pub fn run() {
        let mut queue = schedule(vec![
            Request {
                code: String::from("A-10"),
                urgent: false,
            },
            Request {
                code: String::from("A-11"),
                urgent: true,
            },
            Request {
                code: String::from("A-12"),
                urgent: false,
            },
            Request {
                code: String::from("A-13"),
                urgent: true,
            },
        ]);

        assert!(take_shift(&mut queue, 0).is_empty());
        assert_eq!(queue.len(), 4);
        let shift = take_shift(&mut queue, 3);
        let codes: Vec<_> = shift.iter().map(|request| request.code.as_str()).collect();
        assert_eq!(codes, ["A-11", "A-13", "A-10"]);
        assert_eq!(
            queue.front().map(|request| request.code.as_str()),
            Some("A-12")
        );
        let remaining: Vec<_> = take_shift(&mut queue, 10)
            .into_iter()
            .map(|request| request.code)
            .collect();
        assert_eq!(remaining, ["A-12"]);
        assert!(take_shift(&mut queue, 10).is_empty());
        println!("Заявки текущей смены — {codes:?}, остаток очереди {remaining:?}");
    }
}

/*
Задача 7. Изменения показаний счётчика

Сравнить соседние показания в журнале. Рост означает расход за интервал,
уменьшение означает сброс. Равные значения пропустить. Сохранить индекс
второго показания пары, считая с нуля, и порядок событий.
*/
mod meter {
    use std::cmp::Ordering;

    #[derive(Debug, PartialEq)]
    enum Change {
        Used {
            index: usize,
            amount: u32,
        },
        Reset {
            index: usize,
            before: u32,
            after: u32,
        },
    }

    fn changes(readings: &[u32]) -> Vec<Change> {
        readings
            .windows(2)
            .enumerate()
            .filter_map(|(index, window)| {
                let (before, after) = (window[0], window[1]);
                let index = index + 1;
                match after.cmp(&before) {
                    Ordering::Greater => Some(Change::Used {
                        index,
                        amount: after - before,
                    }),
                    Ordering::Less => Some(Change::Reset {
                        index,
                        before,
                        after,
                    }),
                    Ordering::Equal => None,
                }
            })
            .collect()
    }

    pub fn run() {
        let report = changes(&[100, 106, 106, 2, 5]);
        assert_eq!(
            report,
            [
                Change::Used {
                    index: 1,
                    amount: 6
                },
                Change::Reset {
                    index: 3,
                    before: 106,
                    after: 2
                },
                Change::Used {
                    index: 4,
                    amount: 3
                }
            ]
        );
        assert!(changes(&[]).is_empty());
        assert!(changes(&[100]).is_empty());
        assert!(changes(&[100, 100]).is_empty());
        let used: u64 = report
            .iter()
            .filter_map(|change| match change {
                Change::Used { amount, .. } => Some(u64::from(*amount)),
                Change::Reset { .. } => None,
            })
            .sum();
        assert_eq!(used, 9);
        println!("Расход между замерами — {used} ед., события {report:?}");
    }
}

fn main() {
    labels::run();
    configurations::run();
    errors::run();
    import::run();
    permissions::run();
    dispatch::run();
    meter::run();
}
