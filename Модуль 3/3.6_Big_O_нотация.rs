/*
cargo run --offline --bin big_o
*/

/*
В журнале склада n зарегистрированных талонов и q запросов на сверку.
Нужно узнать, какие номера зарегистрированы, и посчитать сравнения ключей.
Сравнить перебор, двоичный поиск и одновременный проход по двум спискам.
Для двоичного поиска журнал отсортирован, для прохода отсортированы оба
списка. Подготовка списков выполняется до начала подсчёта.
*/

use std::cmp::Ordering;
use std::ops::Range;

#[derive(Debug, PartialEq)]
struct Search {
    index: Option<usize>,
    comparisons: usize,
}

fn linear_search(numbers: &[u32], target: u32) -> Search {
    // O(n) сравнений в худшем случае, O(1) при совпадении в первой позиции.
    for index in 0..numbers.len() {
        if numbers[index] == target {
            return Search {
                index: Some(index),
                comparisons: index + 1,
            };
        }
    }
    Search {
        index: None,
        comparisons: numbers.len(),
    }
}

fn binary_search(numbers: &[u32], target: u32) -> Search {
    let mut left = 0;
    let mut right = numbers.len();
    let mut comparisons = 0;
    // За одно сравнение диапазон сужается примерно вдвое. Верхняя оценка — O(log n).
    while left < right {
        let middle = left + (right - left) / 2;
        comparisons += 1;
        match numbers[middle].cmp(&target) {
            Ordering::Less => left = middle + 1,
            Ordering::Greater => right = middle,
            Ordering::Equal => {
                return Search {
                    index: Some(middle),
                    comparisons,
                };
            }
        }
    }
    Search {
        index: None,
        comparisons,
    }
}

#[derive(Debug, PartialEq)]
struct Reconciliation {
    matches: usize,
    comparisons: u128,
}

// Один поиск на запрос даёт O(q × n) для перебора и O(q × log n) для двоичного поиска.
fn reconcile_by_search(
    numbers: &[u32],
    requests: &[u32],
    search: impl Fn(&[u32], u32) -> Search,
) -> Reconciliation {
    let mut result = Reconciliation {
        matches: 0,
        comparisons: 0,
    };
    for &number in requests {
        let probe = search(numbers, number);
        result.matches += usize::from(probe.index.is_some());
        result.comparisons += probe.comparisons as u128;
    }
    result
}

fn reconcile_sorted(numbers: &[u32], requests: &[u32]) -> Reconciliation {
    let mut result = Reconciliation {
        matches: 0,
        comparisons: 0,
    };
    let (mut registered, mut requested) = (0, 0);
    // При каждом сравнении продвигается хотя бы один индекс. Верхняя оценка — O(n + q).
    while registered < numbers.len() && requested < requests.len() {
        result.comparisons += 1;
        match numbers[registered].cmp(&requests[requested]) {
            Ordering::Less => registered += 1,
            Ordering::Greater => requested += 1,
            Ordering::Equal => {
                result.matches += 1;
                requested += 1;
            }
        }
    }
    result
}

fn compare_searches() {
    let sample = [101, 104, 110];
    assert_eq!(
        linear_search(&sample, 101),
        Search {
            index: Some(0),
            comparisons: 1
        }
    );
    assert_eq!(
        binary_search(&sample, 104),
        Search {
            index: Some(1),
            comparisons: 1
        }
    );
    assert_eq!(binary_search(&sample, 110).index, Some(2));
    assert_eq!(binary_search(&sample, 105).index, None);
    assert_eq!(binary_search(&sample, 111).index, None);
    assert_eq!(
        binary_search(&[], 104),
        Search {
            index: None,
            comparisons: 0
        }
    );
    let mixed = [100, 101, 104, 104, 109, 110, 111];
    assert_eq!(
        reconcile_by_search(&sample, &mixed, linear_search).matches,
        4
    );
    assert_eq!(
        reconcile_by_search(&sample, &mixed, binary_search).matches,
        4
    );
    assert_eq!(reconcile_sorted(&sample, &mixed).matches, 4);
    assert_eq!(reconcile_sorted(&sample, &[]).comparisons, 0);

    const MAX_SIZE: usize = 1_024;
    let mut numbers = [0; MAX_SIZE];
    let mut requests = [0; MAX_SIZE];
    for index in 0..MAX_SIZE {
        // Запросы относятся к старым талонам. Их номера меньше любого номера в журнале.
        numbers[index] = (MAX_SIZE + index + 1) as u32;
        requests[index] = (index + 1) as u32;
    }

    // При q = n оценки для пакета становятся O(n²), O(n log n) и O(n).
    // Сама сверка хранит только позиции и счётчики, поэтому ей достаточно O(1) памяти сверх входа.
    println!("Сверка талонов при n = q. В таблице указано число сравнений.");
    println!("    n | перебор | двоичный | пакет перебором | пакет двоичным | два списка");
    for size in [16, 64, 256, MAX_SIZE] {
        let numbers = &numbers[..size];
        let requests = &requests[..size];
        // Когда известна позиция, get обращается к ней за O(1).
        assert_eq!(numbers.get(size - 1), Some(&((MAX_SIZE + size) as u32)));

        let linear = linear_search(numbers, requests[0]);
        let binary = binary_search(numbers, requests[0]);
        let batch_linear = reconcile_by_search(numbers, requests, linear_search);
        let batch_binary = reconcile_by_search(numbers, requests, binary_search);
        let batch_sorted = reconcile_sorted(numbers, requests);
        assert_eq!((linear.index, binary.index), (None, None));
        assert_eq!(
            (
                batch_linear.matches,
                batch_binary.matches,
                batch_sorted.matches
            ),
            (0, 0, 0)
        );
        assert_eq!(batch_linear.comparisons, (size as u128) * (size as u128));
        assert_eq!(
            batch_binary.comparisons,
            (size as u128) * (binary.comparisons as u128)
        );
        assert_eq!(batch_sorted.comparisons, size as u128);
        println!(
            "{size:>5} | {:>7} | {:>8} | {:>15} | {:>14} | {:>10}",
            linear.comparisons,
            binary.comparisons,
            batch_linear.comparisons,
            batch_binary.comparisons,
            batch_sorted.comparisons
        );
    }
}

/*
Для отчёта о загрузке склада нужно быстро суммировать число коробок за
произвольные часы. Один запрос можно посчитать обходом его диапазона.
При многих запросах заранее сохранить накопленные суммы. Это добавляет
проход по всему входу и n + 1 ячеек памяти. После подготовки каждый
запрос читает две накопленные суммы. Правая граница диапазона исключена.
*/

#[derive(Debug, PartialEq)]
struct Sum {
    boxes: u128,
    reads: usize,
}

fn sum_by_scan(counts: &[u16], hours: Range<usize>) -> Option<Sum> {
    let segment = counts.get(hours)?;
    let mut boxes = 0;
    for &count in segment {
        boxes += u128::from(count);
    }
    // O(k) чтений для k часов. Дополнительная память O(1).
    Some(Sum {
        boxes,
        reads: segment.len(),
    })
}

struct Prefix<'work> {
    totals: &'work [u128],
}

impl<'work> Prefix<'work> {
    fn build(counts: &[u16], workspace: &'work mut [u128]) -> Option<Self> {
        let (zero, tail) = workspace.split_first_mut()?;
        if tail.len() != counts.len() {
            return None;
        }

        *zero = 0;
        let mut boxes = 0;
        for index in 0..counts.len() {
            boxes += u128::from(counts[index]);
            tail[index] = boxes;
        }
        Some(Self { totals: workspace })
    }

    fn sum(&self, hours: Range<usize>) -> Option<Sum> {
        if hours.start > hours.end {
            return None;
        }
        let before = self.totals.get(hours.start)?;
        let after = self.totals.get(hours.end)?;
        Some(Sum {
            boxes: after - before,
            reads: 2,
        })
    }
}

fn compare_sums() {
    let counts = [3, 1, 7, 0, 2, 6, 1, 4];
    let mut workspace = [0; 9];
    assert!(Prefix::build(&counts, &mut workspace[..8]).is_none());
    let Some(prefix) = Prefix::build(&counts, &mut workspace) else {
        panic!("Для примера нужны девять ячеек накопленных сумм");
    };
    assert_eq!(prefix.totals.len(), counts.len() + 1);
    let direct = sum_by_scan(&counts, 1..7);
    let prepared = prefix.sum(1..7);
    assert_eq!(
        direct,
        Some(Sum {
            boxes: 17,
            reads: 6
        })
    );
    assert_eq!(
        prepared,
        Some(Sum {
            boxes: 17,
            reads: 2
        })
    );
    assert_eq!(prefix.sum(8..8), Some(Sum { boxes: 0, reads: 2 }));
    assert_eq!(prefix.sum(1..9), None);
    assert_eq!(prefix.sum(Range { start: 3, end: 2 }), None);
    if let (Some(direct), Some(prepared)) = (direct, prepared) {
        println!(
            "Часы 1..7 — {} коробок. Чтений при обходе {}, после подготовки {}.",
            direct.boxes, direct.reads, prepared.reads
        );
    }
}

fn main() {
    // Big O задаёт верхнюю оценку роста работы. Счётчик показывает работу на конкретных данных.
    compare_searches();
    compare_sums();
}
