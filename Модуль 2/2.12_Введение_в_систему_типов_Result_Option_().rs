/*
cargo run --offline --bin type_system_intro
*/

/*
При инвентаризации кладовщик вводит количество коробок по каждой ячейке.
Пустую ячейку нужно отметить нулём. Если подсчёта ещё не было, количество
отсутствует. Ввод с ошибкой, неверный индекс ячейки и превышение её
вместимости отклоняются. После завершения обхода результаты можно сбросить.
*/

const CELL_CAPACITIES: [u16; 4] = [24, 48, 36, 12];

#[derive(Debug, PartialEq)]
enum CountError {
    UnknownCell { index: usize },
    InvalidQuantity,
    CapacityExceeded { quantity: u16, capacity: u16 },
}

struct Stocktake {
    // Option<u16> хранит либо Some с количеством, либо None.
    counts: [Option<u16>; CELL_CAPACITIES.len()],
}

impl Stocktake {
    fn new() -> Self {
        Self {
            // Тип поля даёт компилятору понять, что здесь нужны значения Option<u16>.
            counts: [None; CELL_CAPACITIES.len()],
        }
    }

    fn first_unchecked(&self) -> Option<usize> {
        for index in 0..self.counts.len() {
            if self.counts[index].is_none() {
                return Some(index);
            }
        }
        None
    }

    fn count_at(&self, index: usize) -> Result<Option<u16>, CountError> {
        // get возвращает None для неверного индекса. У найденной ячейки свой Option.
        match self.counts.get(index) {
            Some(&count) => Ok(count),
            None => Err(CountError::UnknownCell { index }),
        }
    }

    fn record(&mut self, index: usize, input: &str) -> Result<(), CountError> {
        let Some(&capacity) = CELL_CAPACITIES.get(index) else {
            return Err(CountError::UnknownCell { index });
        };

        // В Result<u16, _> тип u16 относится к Ok, а тип значения в Err Rust выводит сам.
        let parsed: Result<u16, _> = input.trim().parse();
        let quantity = match parsed {
            Ok(value) => value,
            Err(_) => return Err(CountError::InvalidQuantity),
        };

        if quantity > capacity {
            return Err(CountError::CapacityExceeded { quantity, capacity });
        }

        // Записываем количество после проверок. При успехе дополнительных данных нет.
        self.counts[index] = Some(quantity);
        Ok(())
    }

    // Если возвращаемый тип опущен, функция возвращает (). У этого типа одно значение — ().
    fn reset(&mut self) {
        self.counts = [None; CELL_CAPACITIES.len()];
    }
}

fn main() {
    let mut stocktake = Stocktake::new();

    // Some(0) здесь содержит индекс первой непроверенной ячейки.
    assert_eq!(stocktake.first_unchecked(), Some(0));
    assert_eq!(stocktake.count_at(0), Ok(None));
    assert_eq!(
        stocktake.count_at(4),
        Err(CountError::UnknownCell { index: 4 })
    );

    let recorded: Result<(), CountError> = stocktake.record(0, "0");
    assert_eq!(recorded, Ok(()));
    assert_eq!(stocktake.count_at(0), Ok(Some(0)));
    assert_eq!(stocktake.record(1, " 12 "), Ok(()));

    // Неудачная запись сохраняет результат предыдущего подсчёта.
    assert_eq!(
        stocktake.record(1, "12 шт."),
        Err(CountError::InvalidQuantity)
    );
    assert_eq!(stocktake.record(1, "-1"), Err(CountError::InvalidQuantity));
    assert_eq!(
        stocktake.record(1, "65536"),
        Err(CountError::InvalidQuantity)
    );
    assert_eq!(
        stocktake.record(1, "49"),
        Err(CountError::CapacityExceeded {
            quantity: 49,
            capacity: 48
        })
    );
    assert_eq!(
        stocktake.record(4, "2"),
        Err(CountError::UnknownCell { index: 4 })
    );
    assert_eq!(stocktake.count_at(1), Ok(Some(12)));
    assert_eq!(stocktake.first_unchecked(), Some(2));

    // match различает учтённое количество, непроверенную ячейку и ошибку чтения.
    for index in [0, 1, 2, 4] {
        match stocktake.count_at(index) {
            Ok(Some(quantity)) => println!("Ячейка {index} — учтено {quantity} шт."),
            Ok(None) => println!("Ячейка {index} — ещё не проверена"),
            Err(error) => println!("Ячейка {index} — чтение отклонено ({error:?})"),
        }
    }

    assert_eq!(stocktake.record(2, "36"), Ok(()));
    assert_eq!(stocktake.record(3, "12"), Ok(()));
    assert_eq!(stocktake.first_unchecked(), None);

    let reset_value: () = stocktake.reset();
    assert_eq!(reset_value, ());
    for index in 0..CELL_CAPACITIES.len() {
        assert_eq!(stocktake.count_at(index), Ok(None));
    }
    println!("Учёт сброшен, все ячейки снова ожидают подсчёта");

    // Без контекста Rust не может определить тип значения внутри Option.
    // let _unknown = None;

    // Внешний Result сообщает, допустим ли индекс. Внутренний Option хранит количество.
    // let _quantity: Option<u16> = stocktake.count_at(0);

    // Блок с инструкцией в конце возвращает (). Без точки с запятой здесь было бы число.
    // let _quantity: u16 = { 12_u16; };

    // () и Result<(), CountError> — разные типы.
    // let _status: Result<(), CountError> = stocktake.reset();
}
