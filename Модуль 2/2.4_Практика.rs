/*
cargo run --offline --bin practice_2_4
*/

/*
Задача 1. Ёмкость для резервных копий

Каждый день сервис сохраняет резервную копию известного размера в МиБ.
Нужно хранить заданное число дневных копий и добавить процент запаса
к их суммарному объёму. Вернуть требуемую ёмкость в целых ГиБ, округлив вверх.
Для 2400 МиБ в день, 14 дней и запаса 15% требуется 38 ГиБ.
*/
fn backup_capacity_gib(daily_mib: u64, days: u16, reserve_percent: u8) -> u128 {
    const MIB_PER_GIB: u128 = 1_024;
    const WHOLE_PERCENT: u128 = 100;

    let stored_mib = u128::from(daily_mib) * u128::from(days);
    let percent_with_reserve = WHOLE_PERCENT + u128::from(reserve_percent);
    (stored_mib * percent_with_reserve).div_ceil(WHOLE_PERCENT * MIB_PER_GIB)
}

/*
Задача 2. Символ на складской метке

Код секции склада допускает латинскую заглавную букву или цифру ASCII.
Латинскую строчную букву нужно привести к верхнему регистру.
Вернуть символ после обработки и признак его допустимости.
Кириллическую В следует отклонить, даже если она похожа на латинскую B.
*/
fn rack_symbol(input: char) -> (char, bool) {
    let normalized = input.to_ascii_uppercase();
    let accepted = normalized.is_ascii_uppercase() || normalized.is_ascii_digit();
    (normalized, accepted)
}

/*
Задача 3. Показание датчика в серверной

Датчик передаёт температуру, поправку к ней и признак связи.
Для этой серверной настроен диапазон от 18 до 27 °C включительно.
Сначала проверить связь и конечность скорректированного значения.
Затем определить, попадает ли температура в диапазон, ниже она или выше.
*/
struct TemperatureReading {
    celsius: f64,
    correction: f64,
    online: bool,
}

#[derive(Debug, PartialEq)]
enum TemperatureStatus {
    Unavailable,
    BelowLimit,
    InRange,
    AboveLimit,
}

fn server_room_status(reading: TemperatureReading) -> TemperatureStatus {
    const MIN_CELSIUS: f64 = 18.0;
    const MAX_CELSIUS: f64 = 27.0;

    let corrected = reading.celsius + reading.correction;
    if !reading.online || !corrected.is_finite() {
        TemperatureStatus::Unavailable
    } else if corrected < MIN_CELSIUS {
        TemperatureStatus::BelowLimit
    } else if corrected > MAX_CELSIUS {
        TemperatureStatus::AboveLimit
    } else {
        TemperatureStatus::InRange
    }
}

/*
Задача 4. Расчёт платы за парковку

Первые 15 минут визита бесплатны. За каждый начатый час после них
начисляется 80 рублей. Каждые 24 часа оплачиваемого времени стоят
не более 600 рублей. Тот же предел действует для остатка неполных суток.
Вернуть сумму в целых рублях.
*/
fn parking_charge(visit_minutes: u32) -> u64 {
    const FREE_MINUTES: u32 = 15;
    const MINUTES_PER_HOUR: u32 = 60;
    const MINUTES_PER_DAY: u32 = 24 * MINUTES_PER_HOUR;
    const HOURLY_RATE: u64 = 80;
    const DAILY_LIMIT: u64 = 600;

    if visit_minutes <= FREE_MINUTES {
        return 0;
    }

    let paid_minutes = visit_minutes - FREE_MINUTES;
    let full_days = paid_minutes / MINUTES_PER_DAY;
    let last_day_minutes = paid_minutes % MINUTES_PER_DAY;
    let last_day_hours = last_day_minutes.div_ceil(MINUTES_PER_HOUR);
    let last_day_charge = u64::from(last_day_hours) * HOURLY_RATE;
    let last_day_charge = if last_day_charge > DAILY_LIMIT {
        DAILY_LIMIT
    } else {
        last_day_charge
    };
    u64::from(full_days) * DAILY_LIMIT + last_day_charge
}

/*
Задача 5. Пропуски в счётчике телеметрии

Устройство передаёт номер пакета от 0 до 255. После 255 следует 0.
Приёмник вычисляет расстояние вперёд по кругу от последнего принятого номера.
Расстояние 0 означает повтор, 1 означает следующий пакет, от 2 до 127
означает пропуск. При расстоянии от 128 до 255 пакет считается старым
или неоднозначным. Для пропуска вернуть число пропущенных номеров.
*/
#[derive(Debug, PartialEq)]
enum PacketStatus {
    Duplicate,
    Next,
    Gap { missing: u8 },
    OldOrAmbiguous,
}

fn packet_status(last: u8, incoming: u8) -> PacketStatus {
    let forward_distance = incoming.wrapping_sub(last);
    if forward_distance == 0 {
        PacketStatus::Duplicate
    } else if forward_distance == 1 {
        PacketStatus::Next
    } else if forward_distance < 128 {
        PacketStatus::Gap {
            missing: forward_distance - 1,
        }
    } else {
        PacketStatus::OldOrAmbiguous
    }
}

/*
Задача 6. Очередь заявок в окно обслуживания

В начале каждой минуты в очередь добавляется заданное число заявок.
Затем обработчик закрывает до заданного числа заявок за минуту.
Расчёт останавливается, когда очередь пуста или окно обслуживания закончилось.
Вернуть число прошедших минут и остаток заявок.
*/
fn drain_queue(
    initial: u32,
    arrivals_per_minute: u16,
    processed_per_minute: u16,
    window_minutes: u16,
) -> (u16, u64) {
    let mut minutes = 0;
    let mut remaining = u64::from(initial);
    while remaining > 0 && minutes < window_minutes {
        remaining += u64::from(arrivals_per_minute);
        remaining = remaining.saturating_sub(u64::from(processed_per_minute));
        minutes += 1;
    }
    (minutes, remaining)
}

/*
Задача 7. Бюджет повторных запросов

После ошибки сервис повторяет запрос с паузой. Перед каждым следующим
повтором пауза удваивается до установленного предела. Сумма пауз должна
уложиться в общий бюджет, число повторов ограничено. Нулевая пауза
отключает повторы. Вернуть число запланированных повторов и сумму пауз.
*/
fn retry_budget(initial_ms: u32, max_ms: u32, budget_ms: u64, limit: u16) -> (u16, u64) {
    let max_ms = u64::from(max_ms);
    let mut delay_ms = u64::from(initial_ms).min(max_ms);
    let mut retries = 0;
    let mut elapsed_ms = 0;
    loop {
        if delay_ms == 0 || retries == limit || delay_ms > budget_ms - elapsed_ms {
            break (retries, elapsed_ms);
        }
        elapsed_ms += delay_ms;
        retries += 1;
        delay_ms = (delay_ms * 2).min(max_ms);
    }
}

/*
Задача 8. Сводка доступности по часам

Есть число часов наблюдения, окно плановых работ и интервал сбоя.
У интервалов начало включено, конец исключён. Из отчёта исключаются
часы плановых работ, включая их пересечение со сбоем.
Вернуть число учтённых часов и число часов недоступности.
*/
struct HourWindow {
    start: u8,
    end: u8,
}

fn availability_hours(hours: u8, maintenance: HourWindow, outage: HourWindow) -> (u8, u8) {
    let mut measured = 0;
    let mut unavailable = 0;
    for hour in 0..hours {
        if hour >= maintenance.start && hour < maintenance.end {
            continue;
        }
        measured += 1;
        if hour >= outage.start && hour < outage.end {
            unavailable += 1;
        }
    }
    (measured, unavailable)
}

/*
Задача 9. Ближайшее окно для экспорта

Рабочее окно открыто с 08:00 до 18:00, запускать экспорт можно в начале часа.
Экспорт должен завершиться в тот же день. В один из дней действует
интервал обслуживания с включённым началом и исключённым концом.
Найти самый ранний день и час, когда экспорт целиком помещается в окно
и не пересекается с обслуживанием. День 1 — первый день поиска.
*/
#[derive(Debug, PartialEq)]
enum ExportSlot {
    Found { day: u8, hour: u8 },
    Unavailable,
}

fn earliest_export_slot(
    days: u8,
    duration_hours: u8,
    maintenance_day: u8,
    maintenance: HourWindow,
) -> ExportSlot {
    const OPEN_HOUR: u8 = 8;
    const CLOSE_HOUR: u8 = 18;

    if duration_hours == 0 {
        return ExportSlot::Unavailable;
    }

    let mut result = ExportSlot::Unavailable;
    'days: for day in 1..=days {
        for hour in OPEN_HOUR..CLOSE_HOUR {
            let end = u16::from(hour) + u16::from(duration_hours);
            if end > u16::from(CLOSE_HOUR) {
                continue 'days;
            }
            if day == maintenance_day
                && maintenance.start < maintenance.end
                && hour < maintenance.end
                && end > u16::from(maintenance.start)
            {
                continue;
            }
            result = ExportSlot::Found { day, hour };
            break 'days;
        }
    }
    result
}

fn main() {
    let capacity = backup_capacity_gib(2_400, 14, 15);
    assert_eq!(capacity, 38);
    assert_eq!(backup_capacity_gib(1_024, 1, 0), 1);
    assert_eq!(backup_capacity_gib(1_024, 1, 1), 2);
    assert_eq!(backup_capacity_gib(2_400, 0, 15), 0);
    println!("1. Хранилище резервных копий — {capacity} ГиБ");

    let (normalized, accepted) = rack_symbol('b');
    let (cyrillic, cyrillic_accepted) = rack_symbol('В');
    assert_eq!((normalized, accepted), ('B', true));
    assert_eq!(rack_symbol('7'), ('7', true));
    assert_eq!((cyrillic, cyrillic_accepted), ('В', false));
    println!(
        "2. Метка {normalized} принята, {cyrillic} (U+{:04X}) отклонена",
        u32::from(cyrillic)
    );

    let reading = TemperatureReading {
        celsius: 28.25,
        correction: -0.25,
        online: true,
    };
    assert_eq!(
        server_room_status(TemperatureReading {
            celsius: 27.25,
            ..reading
        }),
        TemperatureStatus::InRange
    );
    assert_eq!(
        server_room_status(TemperatureReading {
            celsius: 18.0,
            ..reading
        }),
        TemperatureStatus::BelowLimit
    );
    assert_eq!(
        server_room_status(TemperatureReading {
            celsius: f64::NAN,
            ..reading
        }),
        TemperatureStatus::Unavailable
    );
    assert_eq!(
        server_room_status(TemperatureReading {
            online: false,
            ..reading
        }),
        TemperatureStatus::Unavailable
    );
    let temperature_status = server_room_status(reading);
    assert_eq!(temperature_status, TemperatureStatus::AboveLimit);
    println!("3. Состояние датчика серверной — {temperature_status:?}");

    let charge = parking_charge(15 + 1_440 + 61);
    assert_eq!(charge, 760);
    assert_eq!(parking_charge(15), 0);
    assert_eq!(parking_charge(75), 80);
    assert_eq!(parking_charge(76), 160);
    assert_eq!(parking_charge(15 + 8 * 60), 600);
    assert_eq!(parking_charge(15 + 1_440), 600);
    println!("4. Парковка за 25 ч 16 мин — {charge} руб.");

    let sequence = packet_status(254, 1);
    assert_eq!(sequence, PacketStatus::Gap { missing: 2 });
    assert_eq!(packet_status(255, 0), PacketStatus::Next);
    assert_eq!(packet_status(10, 10), PacketStatus::Duplicate);
    assert_eq!(packet_status(0, 127), PacketStatus::Gap { missing: 126 });
    assert_eq!(packet_status(0, 128), PacketStatus::OldOrAmbiguous);
    assert_eq!(packet_status(10, 9), PacketStatus::OldOrAmbiguous);
    println!("5. Пакет 1 после пакета 254 — {sequence:?}");

    let (minutes, remaining) = drain_queue(950, 50, 200, 10);
    assert_eq!((minutes, remaining), (7, 0));
    assert_eq!(drain_queue(950, 50, 200, 6), (6, 50));
    assert_eq!(drain_queue(100, 200, 100, 3), (3, 400));
    assert_eq!(drain_queue(0, 50, 200, 10), (0, 0));
    println!("6. Очередь — {minutes} мин обработки, остаток {remaining}");

    let (retries, elapsed_ms) = retry_budget(250, 2_000, 3_750, 10);
    assert_eq!((retries, elapsed_ms), (4, 3_750));
    assert_eq!(retry_budget(500, 600, 1_700, 2), (2, 1_100));
    assert_eq!(retry_budget(500, 600, 499, 10), (0, 0));
    assert_eq!(retry_budget(0, 600, 1_700, 10), (0, 0));
    println!("7. Повторов — {retries}, суммарная пауза {elapsed_ms} мс");

    let (measured, unavailable) = availability_hours(
        24,
        HourWindow { start: 2, end: 4 },
        HourWindow { start: 3, end: 6 },
    );
    assert_eq!((measured, unavailable), (22, 2));
    assert_eq!(
        availability_hours(
            24,
            HourWindow { start: 0, end: 24 },
            HourWindow { start: 3, end: 6 }
        ),
        (0, 0)
    );
    println!("8. Мониторинг — {measured} ч учтено, {unavailable} ч недоступности");

    let slot = earliest_export_slot(3, 3, 1, HourWindow { start: 8, end: 18 });
    assert_eq!(slot, ExportSlot::Found { day: 2, hour: 8 });
    assert_eq!(
        earliest_export_slot(1, 3, 1, HourWindow { start: 10, end: 13 }),
        ExportSlot::Found { day: 1, hour: 13 }
    );
    assert_eq!(
        earliest_export_slot(1, 10, 1, HourWindow { start: 10, end: 10 }),
        ExportSlot::Found { day: 1, hour: 8 }
    );
    assert_eq!(
        earliest_export_slot(3, 11, 1, HourWindow { start: 8, end: 18 }),
        ExportSlot::Unavailable
    );
    assert_eq!(
        earliest_export_slot(3, 0, 1, HourWindow { start: 8, end: 18 }),
        ExportSlot::Unavailable
    );
    println!("9. Ближайшее окно экспорта — {slot:?}");
}
