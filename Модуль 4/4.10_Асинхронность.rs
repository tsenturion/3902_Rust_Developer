/*
cargo run --offline --bin async_operations
*/

/*
Терминал склада запрашивает тарифы двух перевозчиков, сверяет комплектацию
заказа и выбирает самый дешёвый тариф. Для расчёта тарифа достаточно веса
из карточки заказа. Сверку товара можно выполнить, пока перевозчики отвечают.

Запросы и ответы передаются через каналы. В блокирующем сценарии перевозчиков
имитирует отдельный поток. В асинхронном сценарии порядок ответов задаётся
вручную в том же потоке, поэтому каждый шаг можно проверить без таймеров.
*/

use std::sync::mpsc::{Receiver, RecvError, SendError, Sender, TryRecvError, channel};
use std::thread;

const PARCEL_WEIGHT_G: u32 = 1_800;
const ORDER_ITEMS: [u32; 3] = [104, 104, 259];
const SCANNED_ITEMS: [u32; 3] = [259, 104, 104];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Carrier {
    Standard,
    Express,
}

#[derive(Debug, PartialEq, Eq)]
struct Quote {
    carrier: Carrier,
    price_rub: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    EmptyParcel,
    Overweight { limit_g: u32 },
}

type QuoteReply = Result<Quote, Refusal>;

impl Carrier {
    fn quote(self, weight_g: u32) -> QuoteReply {
        let (limit_g, base_rub, per_kg_rub) = match self {
            Self::Standard => (20_000, 160, 45),
            Self::Express => (5_000, 270, 60),
        };
        if weight_g == 0 {
            return Err(Refusal::EmptyParcel);
        }
        if weight_g > limit_g {
            return Err(Refusal::Overweight { limit_g });
        }

        Ok(Quote {
            carrier: self,
            price_rub: base_rub + weight_g.div_ceil(1_000) * per_kg_rub,
        })
    }
}

#[derive(Debug)]
struct QuoteRequest {
    carrier: Carrier,
    weight_g: u32,
    reply_to: Sender<QuoteReply>,
}

impl QuoteRequest {
    fn answer(self) -> Result<(), SendError<QuoteReply>> {
        self.reply_to.send(self.carrier.quote(self.weight_g))
    }
}

struct PendingQuote(Receiver<QuoteReply>);

enum QuoteState {
    Pending(PendingQuote),
    Ready(QuoteReply),
    Disconnected,
}

impl PendingQuote {
    fn wait(self) -> Result<QuoteReply, RecvError> {
        // При пустой очереди recv блокирует поток до ответа или отключения.
        self.0.recv()
    }

    fn try_take(self) -> QuoteState {
        // try_recv сразу возвращает результат проверки. При пустом канале
        // возвращаем сам дескриптор, чтобы можно было проверить его позже.
        match self.0.try_recv() {
            Ok(reply) => QuoteState::Ready(reply),
            Err(TryRecvError::Empty) => QuoteState::Pending(self),
            Err(TryRecvError::Disconnected) => QuoteState::Disconnected,
        }
    }
}

fn request_quote(
    outbox: &Sender<QuoteRequest>,
    carrier: Carrier,
    weight_g: u32,
) -> Result<PendingQuote, SendError<QuoteRequest>> {
    // Собственный канал ответа связывает результат с конкретным запросом
    // независимо от порядка завершения операций.
    let (reply_to, reply) = channel();
    outbox.send(QuoteRequest {
        carrier,
        weight_g,
        reply_to,
    })?;
    Ok(PendingQuote(reply))
}

fn picking_is_correct(expected: &[u32], scanned: &[u32]) -> bool {
    let mut expected = expected.to_vec();
    let mut scanned = scanned.to_vec();
    expected.sort_unstable();
    scanned.sort_unstable();
    // Сравнение учитывает количество единиц каждого артикула.
    expected == scanned
}

fn checkout_blocking() -> [Quote; 2] {
    let (outbox, inbox) = channel::<QuoteRequest>();
    let carriers = thread::spawn(move || {
        for _ in 0..2 {
            inbox
                .recv()
                .expect("Ожидался запрос тарифа")
                .answer()
                .expect("Терминал должен принять ответ");
        }
    });

    let quotes = [Carrier::Standard, Carrier::Express].map(|carrier| {
        request_quote(&outbox, carrier, PARCEL_WEIGHT_G)
            .expect("Запрос должен быть отправлен")
            .wait()
            .expect("Перевозчик должен прислать ответ")
            .expect("Вес посылки входит в лимит")
    });
    // Здесь обе операции уже завершились. До этой строки поток терминала
    // ожидал сначала первый тариф, затем второй.
    assert!(picking_is_correct(&ORDER_ITEMS, &SCANNED_ITEMS));
    carriers
        .join()
        .expect("Перевозчики должны обработать запросы");
    println!("Блокирующий сценарий получил оба тарифа до сверки комплектации.");
    quotes
}

fn checkout_asynchronously() -> [Quote; 2] {
    let (outbox, inbox) = channel();
    let standard = request_quote(&outbox, Carrier::Standard, PARCEL_WEIGHT_G)
        .expect("Обычный запрос должен быть отправлен");
    let express = request_quote(&outbox, Carrier::Express, PARCEL_WEIGHT_G)
        .expect("Экспресс-запрос должен быть отправлен");

    let QuoteState::Pending(standard) = standard.try_take() else {
        panic!("Ожидался обычный запрос без ответа");
    };
    let QuoteState::Pending(express) = express.try_take() else {
        panic!("Ожидался экспресс-запрос без ответа");
    };
    println!("Состояние запросов — оба отправлены и ожидают ответа.");
    assert!(picking_is_correct(&ORDER_ITEMS, &SCANNED_ITEMS));
    println!("Асинхронный сценарий сверил комплектацию до получения тарифов.");

    // Оба запроса уже в очереди. Ответ на второй отправляем первым.
    let standard_request = inbox.try_recv().expect("Ожидался обычный запрос");
    let express_request = inbox.try_recv().expect("Ожидался экспресс-запрос");
    express_request
        .answer()
        .expect("Терминал ждёт экспресс-тариф");

    let QuoteState::Ready(Ok(express)) = express.try_take() else {
        panic!("Экспресс-тариф должен быть готов");
    };
    let QuoteState::Pending(standard) = standard.try_take() else {
        panic!("После ответа на второй запрос первый должен оставаться в ожидании");
    };
    assert_eq!(express.carrier, Carrier::Express);
    println!(
        "Экспресс-тариф пришёл первым и стоит {} руб.",
        express.price_rub
    );

    standard_request
        .answer()
        .expect("Терминал ждёт обычный тариф");
    let QuoteState::Ready(Ok(standard)) = standard.try_take() else {
        panic!("Обычный тариф должен быть готов");
    };
    assert_eq!(standard.carrier, Carrier::Standard);
    [standard, express]
}

fn check_failures_and_discard() {
    let (outbox, inbox) = channel();

    // Перевозчик ответил и объяснил, почему расчёт не выполнен.
    let overweight =
        request_quote(&outbox, Carrier::Express, 5_001).expect("Запрос должен быть отправлен");
    inbox
        .try_recv()
        .expect("Ожидался запрос")
        .answer()
        .expect("Терминал должен получить отказ");
    let QuoteState::Ready(Err(Refusal::Overweight { limit_g })) = overweight.try_take() else {
        panic!("Ожидался ответ с отказом по весу");
    };
    assert_eq!(limit_g, 5_000);
    println!("Отказ перевозчика — допустимый вес до {limit_g} г.");

    // Отправитель ответа исчез. Ожидание такого ответа уже не имеет смысла.
    let lost = request_quote(&outbox, Carrier::Standard, PARCEL_WEIGHT_G)
        .expect("Запрос должен быть отправлен");
    drop(inbox.try_recv().expect("Ожидался запрос"));
    assert!(matches!(lost.try_take(), QuoteState::Disconnected));
    println!("Канал ответа — закрыт до получения тарифа.");

    // Если комплектация не сошлась, терминалу больше не нужен тариф.
    // Освобождение дескриптора прекращает приём ответа. Запрос уже в очереди,
    // поэтому перевозчик ещё может обработать его и попытаться ответить.
    let discarded = request_quote(&outbox, Carrier::Standard, PARCEL_WEIGHT_G)
        .expect("Запрос должен быть отправлен");
    assert!(!picking_is_correct(&ORDER_ITEMS, &[259, 104, 259]));
    drop(discarded);
    let late_reply = inbox.try_recv().expect("Запрос остался в очереди").answer();
    assert!(late_reply.is_err());
    println!("Отмена ожидания — перевозчик обработал запрос, но получатель ответа уже закрыт.");

    drop(inbox);
    assert!(request_quote(&outbox, Carrier::Standard, PARCEL_WEIGHT_G).is_err());
}

fn main() {
    let sequential_quotes = checkout_blocking();
    let overlapping_quotes = checkout_asynchronously();
    assert_eq!(sequential_quotes, overlapping_quotes);

    let best_quote = overlapping_quotes
        .into_iter()
        .min_by_key(|quote| quote.price_rub)
        .expect("Есть два предложения");
    assert_eq!(
        best_quote,
        Quote {
            carrier: Carrier::Standard,
            price_rub: 250
        }
    );
    println!(
        "После получения обоих ответов выбран тариф за {} руб.",
        best_quote.price_rub
    );
    check_failures_and_discard();
}
