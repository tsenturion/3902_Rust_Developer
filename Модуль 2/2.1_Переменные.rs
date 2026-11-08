/*
cargo run --offline --bin variables
*/

const MINUTES_PER_ACADEMIC_HOUR: u32 = 45;

fn main() {
    // Константа вычисляется при компиляции. План занятия хранится в неизменяемой переменной.
    const ACADEMIC_HOURS: u32 = 2;
    const PLAN_MINUTES: u32 = ACADEMIC_HOURS * MINUTES_PER_ACADEMIC_HOUR;
    let planned_minutes = PLAN_MINUTES;

    // Первое присваивание инициализирует переменную, объявленную без значения.
    let review_minutes: u32;
    review_minutes = 10;

    // mut позволяет обновлять остаток времени в той же переменной.
    let mut remaining_minutes = planned_minutes;
    remaining_minutes -= review_minutes;
    assert_eq!(remaining_minutes, 80);

    let explanation_minutes = 25;
    remaining_minutes -= explanation_minutes;
    assert_eq!(remaining_minutes, 55);

    // Второй let создаёт новую привязку с тем же именем.
    let reserve_minutes = 5;
    let available_minutes = remaining_minutes;
    let available_minutes = available_minutes - reserve_minutes;
    assert_eq!(available_minutes, 50);

    {
        // В блоке available_minutes — лимит одного упражнения.
        let available_minutes = 15;
        remaining_minutes -= available_minutes;
        assert_eq!(remaining_minutes, 40);
    }

    // За пределами блока снова доступна внешняя привязка.
    assert_eq!(available_minutes, 50);
    assert_eq!(planned_minutes, 90);

    // Вычисленная ранее величина сохраняется. Новый остаток считаем из обновлённого значения.
    let updated_available_minutes = remaining_minutes - reserve_minutes;
    assert_eq!(updated_available_minutes, 35);

    // При затенении новая привязка может получить другой тип.
    let remaining_duration: u32 = remaining_minutes;
    let remaining_duration: u64 = u64::from(remaining_duration) * 60_000;
    assert_eq!(remaining_duration, 2_400_000);

    let elapsed_minutes = planned_minutes - remaining_minutes;
    println!("План — {planned_minutes} мин, использовано — {elapsed_minutes} мин");
    println!("Остаток — {remaining_minutes} мин, или {remaining_duration} мс");
    println!(
        "Доступно без резерва — было {available_minutes} мин, теперь {updated_available_minutes} мин"
    );

    // Повторное присваивание неизменяемой переменной.
    // let minutes = 90;
    // minutes = 80;

    // Присваивание значения другого типа изменяемой переменной.
    // let mut minutes = 90_u32;
    // minutes = 80_u64;

    // Чтение до инициализации.
    // let minutes: u32;
    // println!("{minutes}");

    // Использование имени после выхода из его блока.
    // { let exercise_minutes = 15; }
    // println!("{exercise_minutes}");

    // Значение локальной переменной в выражении для константы.
    // let minutes = 45;
    // const TOTAL: u32 = minutes;
}
