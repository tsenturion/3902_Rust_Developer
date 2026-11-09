/*
cargo run --offline --bin practice_2_15
*/

/*
Задача 1. Поиск посылки в постамате

По номеру ячейки вернуть код посылки, если он есть. Для неизвестной ячейки
и для пустой ячейки поиск кода должен вернуть None. Отдельный поиск ячейки
позволяет оператору различить эти два случая. Нумерация начинается с единицы.
*/
mod lockers {
    struct Locker {
        parcel_code: Option<String>,
    }

    struct Station {
        lockers: [Locker; 3],
    }

    impl Station {
        fn locker(&self, number: usize) -> Option<&Locker> {
            let index = number.checked_sub(1)?;
            self.lockers.get(index)
        }

        fn parcel_code(&self, number: usize) -> Option<&str> {
            let locker = self.locker(number)?;
            locker.parcel_code.as_deref()
        }
    }

    pub fn run() {
        let station = Station {
            lockers: [
                Locker { parcel_code: None },
                Locker {
                    parcel_code: Some(String::from("PK-742")),
                },
                Locker {
                    parcel_code: Some(String::from("PK-810")),
                },
            ],
        };
        assert_eq!(station.parcel_code(2), Some("PK-742"));
        assert_eq!(station.parcel_code(3), Some("PK-810"));
        assert!(station.locker(1).is_some());
        assert_eq!(station.parcel_code(1), None);
        assert!(station.locker(4).is_none());
        assert_eq!(station.parcel_code(4), None);
        assert_eq!(station.parcel_code(0), None);
        println!("Постамат — в ячейке 2 посылка PK-742, ячейка 1 пуста");
    }
}

/*
Задача 2. Проверка IPv4-адреса контроллера

Принять ровно четыре десятичных числа от 0 до 255, разделённых точками.
Пробелы, знак и ведущие нули запрещены. Парсер одной части возвращает Option.
Парсер адреса сообщает причину отказа и номер части, считая с единицы.
*/
mod controller_address {
    #[derive(Debug, PartialEq)]
    enum AddressError {
        MissingPart { part: usize },
        InvalidPart { part: usize, text: String },
        ExtraPart { part: usize },
    }

    fn parse_octet(text: &str) -> Option<u8> {
        if text.is_empty() || text.len() > 3 || (text.len() > 1 && text.starts_with('0')) {
            return None;
        }
        for byte in text.bytes() {
            if !byte.is_ascii_digit() {
                return None;
            }
        }
        text.parse().ok()
    }

    fn parse_address(text: &str) -> Result<[u8; 4], AddressError> {
        let mut parts = text.split('.');
        let mut address = [0; 4];
        let mut part = 1;
        for octet in &mut address {
            let text = parts.next().ok_or(AddressError::MissingPart { part })?;
            *octet = parse_octet(text).ok_or_else(|| AddressError::InvalidPart {
                part,
                text: text.to_owned(),
            })?;
            part += 1;
        }
        if parts.next().is_some() {
            return Err(AddressError::ExtraPart { part });
        }
        Ok(address)
    }

    pub fn run() {
        assert_eq!(parse_address("192.168.0.255"), Ok([192, 168, 0, 255]));
        assert_eq!(
            parse_address("192.168.0"),
            Err(AddressError::MissingPart { part: 4 })
        );
        assert_eq!(
            parse_address("192.168.0.1.5"),
            Err(AddressError::ExtraPart { part: 5 })
        );
        for input in ["256", "01", "", "+1", " 1", "я"] {
            assert_eq!(
                parse_address(&format!("192.168.{input}.1")),
                Err(AddressError::InvalidPart {
                    part: 3,
                    text: input.to_owned()
                })
            );
        }
        println!("Адрес контроллера — 192.168.0.255 принят, 256 в третьей части отклонено");
    }
}

/*
Задача 3. Необязательный лимит массы

Если лимит в настройках отсутствует, применить 20 кг. Если поле задано,
принять положительное целое число в диапазоне u16. Пустое поле, ноль и
некорректное число считать ошибками настройки.
*/
mod weight_limit {
    use std::num::ParseIntError;

    const DEFAULT_KG: u16 = 20;

    #[derive(Debug, PartialEq)]
    enum LimitError {
        InvalidNumber(ParseIntError),
        Zero,
    }

    fn parse_limit(text: &str) -> Result<u16, LimitError> {
        let kg = text.trim().parse().map_err(LimitError::InvalidNumber)?;
        if kg == 0 {
            return Err(LimitError::Zero);
        }
        Ok(kg)
    }

    fn effective_limit(raw: Option<&str>) -> Result<u16, LimitError> {
        // map даёт Option<Result<...>>. transpose сохраняет ошибку и выносит Result наружу.
        let configured = raw.map(parse_limit).transpose()?;
        Ok(configured.unwrap_or(DEFAULT_KG))
    }

    pub fn run() {
        assert_eq!(effective_limit(None), Ok(20));
        assert_eq!(effective_limit(Some(" 35 ")), Ok(35));
        assert_eq!(effective_limit(Some("0")), Err(LimitError::Zero));
        for input in ["", "-1", "65536", "35 кг"] {
            assert!(matches!(
                effective_limit(Some(input)),
                Err(LimitError::InvalidNumber(_))
            ));
        }
        println!("Лимит массы — по умолчанию 20 кг, настройка 35 кг принята, ноль отклонён");
    }
}

/*
Задача 4. Списание квоты при загрузке пакета

Пакет расходует два независимых лимита, число запросов и объём данных.
Списать оба значения только при достаточном остатке по каждому лимиту.
При отказе указать, какой лимит превышен и сколько осталось. Использовать
остаток целиком разрешено.
*/
mod upload_quota {
    #[derive(Debug, PartialEq)]
    enum QuotaError {
        Requests { needed: u32, available: u32 },
        Bytes { needed: u64, available: u64 },
    }

    struct Quota {
        requests: u32,
        bytes: u64,
    }

    impl Quota {
        fn debit(&mut self, requests: u32, bytes: u64) -> Result<(), QuotaError> {
            let requests_left =
                self.requests
                    .checked_sub(requests)
                    .ok_or(QuotaError::Requests {
                        needed: requests,
                        available: self.requests,
                    })?;
            let bytes_left = self.bytes.checked_sub(bytes).ok_or(QuotaError::Bytes {
                needed: bytes,
                available: self.bytes,
            })?;
            self.requests = requests_left;
            self.bytes = bytes_left;
            Ok(())
        }
    }

    pub fn run() {
        let mut quota = Quota {
            requests: 3,
            bytes: 1_000,
        };
        assert_eq!(
            quota.debit(4, 500),
            Err(QuotaError::Requests {
                needed: 4,
                available: 3
            })
        );
        assert_eq!((quota.requests, quota.bytes), (3, 1_000));
        assert_eq!(
            quota.debit(1, 1_001),
            Err(QuotaError::Bytes {
                needed: 1_001,
                available: 1_000
            })
        );
        assert_eq!((quota.requests, quota.bytes), (3, 1_000));
        assert_eq!(quota.debit(2, 750), Ok(()));
        assert_eq!((quota.requests, quota.bytes), (1, 250));
        assert_eq!(quota.debit(1, 250), Ok(()));
        assert_eq!((quota.requests, quota.bytes), (0, 0));
        println!("Квота — отказ по объёму сохранил оба остатка, точное списание прошло");
    }
}

/*
Задача 5. Чтение двоичного журнала весов

Кадр с кодом 1 содержит массу в граммах в следующих двух байтах, младший
байт идёт первым. Кадр с кодом 2 обозначает обнуление весов и состоит
из одного байта. Вернуть кадр и смещение следующего кадра. На точной границе
журнала вернуть Ok(None). Неизвестный код, обрыв кадра и смещение за концом
журнала считать ошибками. Смещения считаются с нуля.
*/
mod scale_log {
    #[derive(Debug, PartialEq)]
    enum Frame {
        Weight(u16),
        Tare,
    }

    #[derive(Debug, PartialEq)]
    enum FrameError {
        OffsetOutside { offset: usize },
        Truncated { offset: usize },
        UnknownCode { offset: usize, code: u8 },
    }

    fn read_frame(input: &[u8], offset: usize) -> Result<Option<(Frame, usize)>, FrameError> {
        let rest = input
            .get(offset..)
            .ok_or(FrameError::OffsetOutside { offset })?;
        let Some((&code, payload)) = rest.split_first() else {
            return Ok(None);
        };

        match code {
            1 => {
                let [low, high, ..] = payload else {
                    return Err(FrameError::Truncated { offset });
                };
                let weight = u16::from_le_bytes([*low, *high]);
                Ok(Some((Frame::Weight(weight), offset + 3)))
            }
            2 => Ok(Some((Frame::Tare, offset + 1))),
            code => Err(FrameError::UnknownCode { offset, code }),
        }
    }

    fn count_frames(input: &[u8]) -> Result<usize, FrameError> {
        let mut offset = 0;
        let mut count = 0;
        // ? останавливается на Err, while let завершает цикл при None.
        while let Some((_, next_offset)) = read_frame(input, offset)? {
            offset = next_offset;
            count += 1;
        }
        Ok(count)
    }

    pub fn run() {
        let input = [2, 1, 232, 3];
        assert_eq!(read_frame(&input, 0), Ok(Some((Frame::Tare, 1))));
        assert_eq!(read_frame(&input, 1), Ok(Some((Frame::Weight(1_000), 4))));
        assert_eq!(read_frame(&input, 4), Ok(None));
        assert_eq!(
            read_frame(&input, 5),
            Err(FrameError::OffsetOutside { offset: 5 })
        );
        assert_eq!(count_frames(&[]), Ok(0));
        assert_eq!(count_frames(&input), Ok(2));
        assert_eq!(
            count_frames(&[2, 1]),
            Err(FrameError::Truncated { offset: 1 })
        );
        assert_eq!(
            count_frames(&[2, 1, 232]),
            Err(FrameError::Truncated { offset: 1 })
        );
        assert_eq!(
            count_frames(&[2, 9]),
            Err(FrameError::UnknownCode { offset: 1, code: 9 })
        );
        println!("Журнал весов — прочитаны обнуление и 1000 г, обрыв второго кадра обнаружен");
    }
}

/*
Задача 6. Резервный датчик влажности

Сначала запросить основной датчик. Если он исправен, но нового показания
ещё нет, попробовать резервный. Сообщать об ошибке использованного датчика.
Если ни у одного показания нет, вернуть Ok(None). Нулевое показание допустимо.
*/
mod humidity_feeds {
    enum Feed {
        Reading(u8),
        Pending,
        Offline,
    }

    #[derive(Debug, PartialEq)]
    enum ReadError {
        PrimaryOffline,
        BackupOffline,
    }

    struct Disconnected;

    impl Feed {
        fn read(&self) -> Result<Option<u8>, Disconnected> {
            match self {
                Self::Reading(value) => Ok(Some(*value)),
                Self::Pending => Ok(None),
                Self::Offline => Err(Disconnected),
            }
        }
    }

    fn latest(primary: &Feed, backup: &Feed) -> Result<Option<u8>, ReadError> {
        match primary.read().map_err(|_| ReadError::PrimaryOffline)? {
            Some(value) => Ok(Some(value)),
            None => backup.read().map_err(|_| ReadError::BackupOffline),
        }
    }

    pub fn run() {
        assert_eq!(latest(&Feed::Reading(0), &Feed::Offline), Ok(Some(0)));
        assert_eq!(latest(&Feed::Pending, &Feed::Reading(43)), Ok(Some(43)));
        assert_eq!(latest(&Feed::Pending, &Feed::Pending), Ok(None));
        assert_eq!(
            latest(&Feed::Offline, &Feed::Reading(43)),
            Err(ReadError::PrimaryOffline)
        );
        assert_eq!(
            latest(&Feed::Pending, &Feed::Offline),
            Err(ReadError::BackupOffline)
        );
        println!("Датчик — резервное показание 43 % используется при штатном отсутствии основного");
    }
}

/*
Задача 7. Проверка строк расписания

Перед импортом проверить список времён в формате ЧЧ:ММ. Допустимы
00:00–23:59, нужны ровно две цифры в каждой части. Успешный разбор возвращает
число минут с полуночи. Ошибка одной строки не останавливает проверку
остальных. В отчёте указать число принятых и отклонённых строк, а также
номер и причину первой ошибки, если она есть.
*/
mod schedule {
    #[derive(Debug, PartialEq)]
    enum TimeError {
        Format,
        Hour(u8),
        Minute(u8),
    }

    #[derive(Debug, PartialEq)]
    struct LineError {
        line: usize,
        cause: TimeError,
    }

    #[derive(Debug, PartialEq)]
    struct Report {
        accepted: usize,
        rejected: usize,
        first_error: Option<LineError>,
    }

    fn parse_time(text: &str) -> Result<u16, TimeError> {
        let [h1, h2, b':', m1, m2] = text.as_bytes() else {
            return Err(TimeError::Format);
        };
        for digit in [h1, h2, m1, m2] {
            if !digit.is_ascii_digit() {
                return Err(TimeError::Format);
            }
        }

        let hour = (*h1 - b'0') * 10 + (*h2 - b'0');
        let minute = (*m1 - b'0') * 10 + (*m2 - b'0');
        if hour > 23 {
            return Err(TimeError::Hour(hour));
        }
        if minute > 59 {
            return Err(TimeError::Minute(minute));
        }
        Ok(u16::from(hour) * 60 + u16::from(minute))
    }

    fn validate(lines: &[&str]) -> Report {
        let mut report = Report {
            accepted: 0,
            rejected: 0,
            first_error: None,
        };
        let mut line = 1;
        for text in lines {
            let result = parse_time(text).map_err(|cause| LineError { line, cause });
            match result {
                Ok(_) => report.accepted += 1,
                Err(error) => {
                    report.rejected += 1;
                    if report.first_error.is_none() {
                        report.first_error = Some(error);
                    }
                }
            }
            line += 1;
        }
        report
    }

    pub fn run() {
        assert_eq!(parse_time("00:00"), Ok(0));
        assert_eq!(parse_time("23:59"), Ok(1_439));
        assert_eq!(parse_time("24:00"), Err(TimeError::Hour(24)));
        assert_eq!(parse_time("12:60"), Err(TimeError::Minute(60)));
        for input in ["9:15", "09-15", "0x:15", "09:1я", ""] {
            assert_eq!(parse_time(input), Err(TimeError::Format));
        }
        assert_eq!(
            validate(&["09:00"]),
            Report {
                accepted: 1,
                rejected: 0,
                first_error: None
            }
        );
        let report = validate(&["09:00", "10:60", "24:00", "17:30"]);
        assert_eq!(report.accepted, 2);
        assert_eq!(report.rejected, 2);
        assert_eq!(
            report.first_error,
            Some(LineError {
                line: 2,
                cause: TimeError::Minute(60)
            })
        );
        if let Some(error) = report.first_error {
            println!(
                "Расписание — принято строк {}, отклонено {}, первая ошибка в строке {}",
                report.accepted, report.rejected, error.line
            );
        }
    }
}

fn main() {
    lockers::run();
    controller_address::run();
    weight_limit::run();
    upload_quota::run();
    scale_log::run();
    humidity_feeds::run();
    schedule::run();
}
