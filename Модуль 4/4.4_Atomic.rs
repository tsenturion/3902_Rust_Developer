/*
cargo run --offline --bin atomic_operations
*/

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/*
Сканеры возвращают коды принятых этикеток. None означает, что этикетку
не удалось прочитать. Нужно вести один общий счётчик принятых сканов.
Каждое событие добавляется к счётчику одной атомарной операцией.
*/

fn count_scans() {
    let accepted = AtomicUsize::new(0);
    let batches = [
        [Some(17_u32), None, Some(18)],
        [None, Some(31), Some(32)],
        [Some(40), None, None],
    ];

    thread::scope(|scope| {
        for batch in &batches {
            let counter = &accepted;
            scope.spawn(move || {
                for code in batch {
                    if code.is_some() {
                        // Счётчик независим от других данных. Здесь достаточно Relaxed.
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    });
    // При выходе из scope все прибавления завершены.
    let total = accepted.load(Ordering::Relaxed);
    assert_eq!(total, 5);
    println!("Принято сканов — {total}");
}

fn split_increment_demo() {
    let split = AtomicUsize::new(0);
    // Моделируем порядок двух участников. Оба прочитали число до первой записи.
    let first = split.load(Ordering::Relaxed);
    let second = split.load(Ordering::Relaxed);
    split.store(first + 1, Ordering::Relaxed);
    split.store(second + 1, Ordering::Relaxed);

    let combined = AtomicUsize::new(0);
    assert_eq!(combined.fetch_add(1, Ordering::Relaxed), 0);
    assert_eq!(combined.fetch_add(1, Ordering::Relaxed), 1);
    let split = split.into_inner();
    let combined = combined.into_inner();
    assert_eq!((split, combined), (1, 2));
    println!("Два прибавления — раздельные чтение и запись {split}, fetch_add {combined}");
}

/*
Из общего остатка нужно списать места на обработку партии. При нехватке
вернуть наблюдавшийся остаток. Если другой поток успел изменить число,
повторить проверку с новым значением. Остаток не может уйти ниже нуля.
*/

#[derive(Debug, PartialEq, Eq)]
struct Shortage {
    available: usize,
}

fn try_take(remaining: &AtomicUsize, requested: usize) -> Result<usize, Shortage> {
    let mut available = remaining.load(Ordering::Relaxed);
    loop {
        let after = available
            .checked_sub(requested)
            .ok_or(Shortage { available })?;
        // Запись возможна, только если остаток всё ещё совпадает с проверенным.
        match remaining.compare_exchange_weak(
            available,
            after,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Ok(after),
            Err(observed) => available = observed,
        }
    }
}

fn spend_quota() {
    let remaining = AtomicUsize::new(8);
    let results = thread::scope(|scope| {
        let workers = [3, 3, 3].map(|requested| {
            let quota = &remaining;
            scope.spawn(move || try_take(quota, requested))
        });
        workers.map(|worker| worker.join().expect("Расчёт остатка завершился"))
    });

    let mut after_each = Vec::new();
    let mut rejected = 0;
    for result in results {
        match result {
            Ok(after) => after_each.push(after),
            Err(error) => {
                assert_eq!(error, Shortage { available: 2 });
                rejected += 1;
            }
        }
    }
    after_each.sort_unstable();
    assert_eq!(after_each, [2, 5]);
    assert_eq!(rejected, 1);
    let balance = remaining.load(Ordering::Relaxed);
    assert_eq!(balance, 2);

    // Воспроизводим устаревшее чтение без зависимости от расписания потоков.
    let example = AtomicUsize::new(8);
    let old = example.load(Ordering::Relaxed);
    assert_eq!(try_take(&example, 3), Ok(5));
    let stale = example.compare_exchange(old, 5, Ordering::Relaxed, Ordering::Relaxed);
    assert_eq!(stale, Err(5));
    assert_eq!(try_take(&example, 3), Ok(2));
    let exhausted = try_take(&example, 2).expect("Остатка хватает на точное списание");
    assert_eq!(exhausted, 0);
    assert_eq!(try_take(&example, 1), Err(Shortage { available: 0 }));

    println!(
        "Квота — принято {}, отклонено {rejected}, остаток {balance}",
        after_each.len()
    );
    println!(
        "Устаревшее чтение — ожидали {old}, получено {stale:?}, при точном списании остаток {exhausted}"
    );
}

/*
Один исполнитель один раз публикует сводку по докам. Читатель должен
получить оба значения, когда увидит готовность. После публикации сводку
не меняют, а флаг не сбрасывают.
*/

struct PublishedReport {
    docks: AtomicUsize,
    boxes: AtomicUsize,
    ready: AtomicBool,
}

impl PublishedReport {
    fn new() -> Self {
        Self {
            docks: AtomicUsize::new(0),
            boxes: AtomicUsize::new(0),
            ready: AtomicBool::new(false),
        }
    }

    fn publish(&self, docks: usize, boxes: usize) {
        self.docks.store(docks, Ordering::Relaxed);
        self.boxes.store(boxes, Ordering::Relaxed);
        self.ready.store(true, Ordering::Release);
    }

    fn read_if_ready(&self) -> Option<(usize, usize)> {
        // Acquire, прочитавший true из Release, упорядочивает чтение полей после их записи.
        if self.ready.load(Ordering::Acquire) {
            Some((
                self.docks.load(Ordering::Relaxed),
                self.boxes.load(Ordering::Relaxed),
            ))
        } else {
            None
        }
    }
}

fn publish_summary() {
    let report = PublishedReport::new();
    assert_eq!(report.read_if_ready(), None);

    let (docks, boxes) = thread::scope(|scope| {
        let writer = scope.spawn(|| {
            let cartons = [5, 7, 9];
            report.publish(cartons.len(), cartons.into_iter().sum());
        });
        let started = Instant::now();

        // Читаем сводку до join. Готовность подтверждает именно флаг.
        let summary = loop {
            if let Some(summary) = report.read_if_ready() {
                break summary;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "Сводка не появилась вовремя"
            );
            thread::yield_now();
        };
        writer.join().expect("Публикация завершилась");
        summary
    });
    assert_eq!((docks, boxes), (3, 21));
    println!("Сводка до join — доков {docks}, коробов {boxes}");
}

fn exchange_reports() {
    let last = AtomicUsize::new(0);
    let reports = [AtomicUsize::new(0), AtomicUsize::new(0)];
    let previous = thread::scope(|scope| {
        let workers = [(0, 12), (1, 18)].map(|(index, boxes)| {
            let last = &last;
            let reports = &reports;
            scope.spawn(move || {
                reports[index].store(boxes, Ordering::Relaxed);
                // Публикуем свой номер и одновременно узнаём номер прежней сводки.
                let previous = last.swap(index + 1, Ordering::AcqRel);
                previous
                    .checked_sub(1)
                    .map(|index| reports[index].load(Ordering::Relaxed))
            })
        });
        workers.map(|worker| worker.join().expect("Обмен сводками завершился"))
    });
    assert!(matches!(previous, [None, Some(12)] | [Some(18), None]));
    let readers = previous.into_iter().flatten().count();
    println!("Обмен публикациями — предыдущую сводку прочитал один поток ({readers})");
}

fn ordered_stages() {
    let stage = AtomicUsize::new(0);
    thread::scope(|scope| {
        for (index, label) in ["первый", "второй", "третий"].into_iter().enumerate().rev()
        {
            let stage = &stage;
            scope.spawn(move || {
                let started = Instant::now();
                while stage.load(Ordering::SeqCst) != index {
                    assert!(
                        started.elapsed() < Duration::from_secs(5),
                        "Не дождались очереди этапа"
                    );
                    thread::yield_now();
                }
                println!("Этапы — {label}");
                stage.store(index + 1, Ordering::SeqCst);
            });
        }
    });
    assert_eq!(stage.into_inner(), 3);
}

fn main() {
    count_scans();
    split_increment_demo();
    spend_quota();
    publish_summary();
    exchange_reports();
    ordered_stages();
}
