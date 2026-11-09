/*
cargo run --offline --bin advanced_syntax
*/

use std::collections::VecDeque;

/*
Диспетчер ищет маршрут для тележки. Кандидаты поступают группами
в порядке приоритета. Нужно проверить стартовую клетку и каждый шаг.
После первого подходящего маршрута поиск заканчивается, а в отчёт
попадает число проверенных кандидатов. Координаты выдаются в формате
панели оператора и в формате терминала. Заметку можно добавить в черновик.
*/

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    row: usize,
    column: usize,
}

#[derive(Clone, Copy)]
struct Route {
    start: Point,
    step: (isize, isize),
    count: u8,
}

// Размеры сетки входят в тип. Компилятор может вывести их из массива cells.
struct Grid<const ROWS: usize, const COLUMNS: usize> {
    cells: [[bool; COLUMNS]; ROWS],
}

impl<const ROWS: usize, const COLUMNS: usize> Grid<ROWS, COLUMNS> {
    fn dimensions(&self) -> (usize, usize) {
        (ROWS, COLUMNS)
    }

    fn can_enter(&self, row: usize, column: usize) -> bool {
        self.cells.get(row).and_then(|line| line.get(column)) == Some(&true)
    }

    fn follow(
        &self,
        Route {
            start: Point {
                mut row,
                mut column,
            },
            step: (down, right),
            count,
        }: Route,
    ) -> Option<Point> {
        if !self.can_enter(row, column) {
            return None;
        }

        for _ in 0..count {
            // Следующее условие выполняется только при успехе предыдущего.
            if let Some(next_row) = row.checked_add_signed(down)
                && let Some(next_column) = column.checked_add_signed(right)
                && self.can_enter(next_row, next_column)
            {
                // Присваиваем пару в уже существующие переменные.
                (row, column) = (next_row, next_column);
            } else {
                return None;
            }
        }
        Some(Point { row, column })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Selection {
    endpoint: Option<Point>,
    checked: usize,
}

fn select<const ROWS: usize, const COLUMNS: usize>(
    grid: &Grid<ROWS, COLUMNS>,
    groups: &[&[Route]],
) -> Selection {
    let mut checked = 0;
    // break выходит из блока с меткой и задаёт его значение. После блока формируем отчёт.
    let endpoint = 'search: {
        for &group in groups {
            for &route in group {
                checked += 1;
                if let Some(endpoint) = grid.follow(route) {
                    break 'search Some(endpoint);
                }
            }
        }
        None
    };
    Selection { endpoint, checked }
}

trait PanelLabel {
    fn label(&self) -> String;
}

trait TerminalLabel {
    fn label(&self) -> String;
}

impl PanelLabel for Point {
    fn label(&self) -> String {
        format!(
            "ряд {}, место {}",
            self.row as u128 + 1,
            self.column as u128 + 1
        )
    }
}

impl TerminalLabel for Point {
    fn label(&self) -> String {
        format!("R{:02}-C{:02}", self.row, self.column)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Dispatch {
    Draft { target: Point, note: String },
    Confirmed { target: Point },
}

fn add_note(dispatch: &mut Dispatch, text: &str) -> bool {
    // Сопоставляем значение *dispatch. ref mut заимствует String внутри варианта.
    // При match dispatch привязка note уже получила бы &mut String по умолчанию.
    match *dispatch {
        Dispatch::Draft { ref mut note, .. } => {
            note.push_str(text);
            true
        }
        Dispatch::Confirmed { .. } => false,
    }
}

fn demonstrate_routes() {
    let grid = Grid {
        cells: [
            [true, false, true, true],
            [true, true, false, true],
            [false, true, true, true],
        ],
    };
    let compact = Grid::<1, { 1 + 1 }> {
        cells: [[true, false]],
    };
    assert_eq!(compact.dimensions(), (1, 2));
    println!(
        "Размеры сеток — {:?} и {:?}",
        grid.dimensions(),
        compact.dimensions()
    );

    let start = Point { row: 0, column: 0 };
    let blocked = Route {
        start,
        step: (0, 1),
        count: 2,
    };
    let outside = Route {
        start,
        step: (-1, -1),
        count: 1,
    };
    let diagonal = Route {
        start,
        step: (1, 1),
        count: 2,
    };
    let stationary = Route {
        start,
        step: (0, 0),
        count: 0,
    };
    assert_eq!(grid.follow(blocked), None);
    assert_eq!(grid.follow(outside), None);
    assert_eq!(grid.follow(stationary), Some(start));
    assert_eq!(
        grid.follow(Route {
            start: Point { row: 2, column: 3 },
            step: (1, 0),
            count: 1
        }),
        None
    );
    assert_eq!(
        grid.follow(Route {
            start: Point { row: 0, column: 1 },
            ..stationary
        }),
        None
    );

    let priority = [blocked, outside];
    let fallback = [diagonal, stationary];
    println!(
        "Проверка шагов — преграда: {:?}, выход за границу: {:?}, диагональ: {:?}",
        grid.follow(blocked),
        grid.follow(outside),
        grid.follow(diagonal),
    );
    assert_eq!(
        select(&grid, &[]),
        Selection {
            endpoint: None,
            checked: 0
        }
    );
    let rejected = select(&grid, &[&priority]);
    assert_eq!(
        rejected,
        Selection {
            endpoint: None,
            checked: 2
        }
    );
    let selection = select(&grid, &[&priority, &fallback]);
    let endpoint = Point { row: 2, column: 2 };
    assert_eq!(
        selection,
        Selection {
            endpoint: Some(endpoint),
            checked: 3
        }
    );
    println!(
        "Поиск маршрута — с результатом: {:?} после {} проверок, без результата: {:?} после {} проверок",
        selection.endpoint, selection.checked, rejected.endpoint, rejected.checked,
    );
    let Some(target) = selection.endpoint else {
        panic!("Диагональный маршрут доступен в примере");
    };

    let panel = <Point as PanelLabel>::label(&target);
    let terminal = <Point as TerminalLabel>::label(&target);
    assert_eq!(panel, "ряд 3, место 3");
    assert_eq!(terminal, "R02-C02");

    let mut draft = Dispatch::Draft {
        target,
        note: String::from("Маршрут проверен"),
    };
    assert!(add_note(&mut draft, ". Тележка готова"));
    assert_eq!(
        draft,
        Dispatch::Draft {
            target,
            note: String::from("Маршрут проверен. Тележка готова")
        }
    );
    let mut confirmed = Dispatch::Confirmed { target };
    let confirmed_changed = add_note(&mut confirmed, "Изменить маршрут");
    assert!(!confirmed_changed);
    assert_eq!(confirmed, Dispatch::Confirmed { target });

    println!(
        "Проверено маршрутов {}. Точка назначения — {panel}, код {terminal}",
        selection.checked
    );
    if let Dispatch::Draft { note, .. } = draft {
        println!("Черновик — {note}");
    }
    println!("Подтверждённая заявка изменена — {confirmed_changed}");
}

#[derive(Debug, PartialEq, Eq)]
enum SudokuIssue {
    DigitOutOfRange,
    RepeatedInRow,
    RepeatedInColumn,
    RepeatedInBox,
}

fn sudoku_conflict(board: &[[Option<u8>; 9]; 9]) -> Option<(Point, SudokuIssue)> {
    let mut rows = [[false; 9]; 9];
    let mut columns = [[false; 9]; 9];
    let mut boxes = [[false; 9]; 9];

    for (row, line) in board.iter().enumerate() {
        for (column, &cell) in line.iter().enumerate() {
            let Some(digit @ 1..=9) = cell else {
                if cell.is_some() {
                    return Some((Point { row, column }, SudokuIssue::DigitOutOfRange));
                }
                continue;
            };

            let digit_index = usize::from(digit - 1);
            let box_index = (row / 3) * 3 + column / 3;
            let [seen_in_row, seen_in_column, seen_in_box] = [
                &mut rows[row][digit_index],
                &mut columns[column][digit_index],
                &mut boxes[box_index][digit_index],
            ];
            let issue = match (*seen_in_row, *seen_in_column, *seen_in_box) {
                (true, _, _) => SudokuIssue::RepeatedInRow,
                (_, true, _) => SudokuIssue::RepeatedInColumn,
                (_, _, true) => SudokuIssue::RepeatedInBox,
                (false, false, false) => {
                    (*seen_in_row, *seen_in_column, *seen_in_box) = (true, true, true);
                    continue;
                }
            };
            return Some((Point { row, column }, issue));
        }
    }
    None
}

fn demonstrate_sudoku() {
    let mut valid = [[None; 9]; 9];
    valid[0][0] = Some(5);
    valid[3][3] = Some(5);
    let result = sudoku_conflict(&valid);
    assert_eq!(result, None);
    assert_eq!(sudoku_conflict(&[[None; 9]; 9]), None);

    let mut repeated = valid;
    repeated[1][1] = Some(5);
    let conflict = sudoku_conflict(&repeated);
    assert_eq!(
        conflict,
        Some((Point { row: 1, column: 1 }, SudokuIssue::RepeatedInBox))
    );
    println!("Судоку — допустимое поле: {result:?}, повтор: {conflict:?}");

    for (row, column, digit, issue) in [
        (0, 4, 5, SudokuIssue::RepeatedInRow),
        (4, 0, 5, SudokuIssue::RepeatedInColumn),
        (1, 1, 0, SudokuIssue::DigitOutOfRange),
        (1, 1, 10, SudokuIssue::DigitOutOfRange),
    ] {
        let mut board = valid;
        board[row][column] = Some(digit);
        assert_eq!(
            sudoku_conflict(&board),
            Some((Point { row, column }, issue))
        );
    }
}

struct Node<'a> {
    id: u16,
    children: [Option<&'a Node<'a>>; 2],
}

fn nodes_by_level(root: Option<&Node<'_>>) -> Vec<Vec<u16>> {
    let Some(root) = root else {
        return Vec::new();
    };
    let mut pending = VecDeque::from([(root, 0)]);
    let mut levels = Vec::new();

    while let Some((node, depth)) = pending.pop_front() {
        if depth == levels.len() {
            levels.push(Vec::new());
        }
        levels[depth].push(node.id);
        for child in node.children.into_iter().flatten() {
            pending.push_back((child, depth + 1));
        }
    }
    levels
}

fn demonstrate_tree() {
    let leaf = Node {
        id: 40,
        children: [None, None],
    };
    let left = Node {
        id: 20,
        children: [None, Some(&leaf)],
    };
    let right = Node {
        id: 30,
        children: [None, None],
    };
    let root = Node {
        id: 10,
        children: [Some(&left), Some(&right)],
    };

    let levels = nodes_by_level(Some(&root));
    let empty = nodes_by_level(None);
    assert_eq!(levels, [vec![10], vec![20, 30], vec![40]]);
    assert_eq!(nodes_by_level(Some(&leaf)), [vec![40]]);
    assert!(empty.is_empty());
    println!("Обход дерева — по уровням: {levels:?}, пустое дерево: {empty:?}");
}

fn diagonal_order<const ROWS: usize, const COLUMNS: usize>(
    matrix: &[[u16; COLUMNS]; ROWS],
) -> Vec<u16> {
    if ROWS == 0 || COLUMNS == 0 {
        return Vec::new();
    }

    let (last_row, last_column) = (ROWS - 1, COLUMNS - 1);
    let (mut row, mut column, mut up) = (0, 0, true);
    let mut result = Vec::with_capacity(ROWS * COLUMNS);

    loop {
        result.push(matrix[row][column]);
        if (row, column) == (last_row, last_column) {
            break;
        }

        // При совпадении двух границ угловой переход имеет приоритет.
        (row, column, up) = match (up, row, column) {
            (true, r, c) if c == last_column => (r + 1, c, false),
            (true, 0, c) => (0, c + 1, false),
            (true, r, c) => (r - 1, c + 1, true),
            (false, r, c) if r == last_row => (r, c + 1, true),
            (false, r, 0) => (r + 1, 0, true),
            (false, r, c) => (r + 1, c - 1, false),
        };
    }
    result
}

fn demonstrate_diagonals() {
    let rectangular = diagonal_order(&[[1, 2, 3], [4, 5, 6]]);
    let column = diagonal_order(&[[7], [8], [9]]);
    assert_eq!(rectangular, [1, 2, 4, 5, 3, 6]);
    assert_eq!(column, [7, 8, 9]);
    assert_eq!(diagonal_order(&[[7, 8, 9]]), [7, 8, 9]);
    assert_eq!(
        diagonal_order(&[[1, 2], [3, 4], [5, 6]]),
        [1, 2, 3, 5, 4, 6]
    );
    assert_eq!(diagonal_order(&[[7]]), [7]);
    assert!(diagonal_order::<0, 3>(&[]).is_empty());
    assert!(diagonal_order(&[[]]).is_empty());
    println!("Диагональный обход — матрица 2 × 3: {rectangular:?}, один столбец: {column:?}");
}

fn main() {
    demonstrate_routes();
    demonstrate_sudoku();
    demonstrate_tree();
    demonstrate_diagonals();
}
