/*
cargo run --offline --bin async_await
*/

/*
Для расчёта заказа нужны цена из каталога и остаток на складе. Эти запросы
независимы. Ответы подставляет учебный сервис, который приостанавливается
на один опрос. По журналу видно, когда начался и завершился каждый запрос.

Вызов async fn возвращает Future. Его тело начнёт выполняться при первом
опросе. Если значение в точке .await ещё не готово, выполнение
приостановится. При возобновлении код продолжится с той же точки
и с теми же локальными данными. Уже готовое значение доступно сразу.
*/

use std::cell::RefCell;
use std::future::{Future, poll_fn, ready};
use std::pin::pin;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

const SKU: &str = "FD-41";
const PRICE_RUB: u32 = 1_250;
const STOCK: u16 = 4;

#[derive(Debug, PartialEq, Eq)]
enum Step {
    PriceRequested,
    PriceReceived,
    StockRequested,
    StockReceived,
}

#[derive(Debug, PartialEq, Eq)]
enum QuoteError {
    UnknownSku,
    EmptyOrder,
    InsufficientStock { available: u16 },
}

#[derive(Debug, PartialEq, Eq)]
struct Quote {
    quantity: u16,
    total_rub: u64,
}

#[derive(Default)]
struct Services {
    steps: RefCell<Vec<Step>>,
}

impl Services {
    async fn price(&self, sku: &str) -> Result<u32, QuoteError> {
        self.exchange(Step::PriceRequested, Step::PriceReceived)
            .await;
        if sku == SKU {
            Ok(PRICE_RUB)
        } else {
            Err(QuoteError::UnknownSku)
        }
    }

    async fn stock(&self, sku: &str) -> Result<u16, QuoteError> {
        self.exchange(Step::StockRequested, Step::StockReceived)
            .await;
        if sku == SKU {
            Ok(STOCK)
        } else {
            Err(QuoteError::UnknownSku)
        }
    }

    async fn exchange(&self, sent: Step, received: Step) {
        self.steps.borrow_mut().push(sent);

        let mut yielded = false;
        poll_fn(|context| {
            if yielded {
                return Poll::Ready(());
            }
            yielded = true;
            // Учебный ответ будет готов к следующему опросу. Будим
            // исполнителя, чтобы он запланировал ещё один опрос.
            context.waker().wake_by_ref();
            Poll::Pending
        })
        .await;

        self.steps.borrow_mut().push(received);
    }
}

async fn quote_sequentially(
    services: &Services,
    sku: &str,
    quantity: u16,
) -> Result<Quote, QuoteError> {
    // Второй запрос начнётся после успешного ответа на первый.
    let price = services.price(sku).await?;
    let stock = services.stock(sku).await?;
    build_quote(price, stock, quantity)
}

async fn quote_concurrently(
    services: &Services,
    sku: &str,
    quantity: u16,
) -> Result<Quote, QuoteError> {
    // Сначала создаём оба Future. Совместное ожидание продвигает их
    // в рамках одной задачи. join_two собирает оба результата, включая Err.
    let price = services.price(sku);
    let stock = services.stock(sku);
    let (price, stock) = join_two(price, stock).await;
    build_quote(price?, stock?, quantity)
}

fn build_quote(price: u32, stock: u16, quantity: u16) -> Result<Quote, QuoteError> {
    if quantity == 0 {
        return Err(QuoteError::EmptyOrder);
    }
    if quantity > stock {
        return Err(QuoteError::InsufficientStock { available: stock });
    }
    Ok(Quote {
        quantity,
        total_rub: u64::from(price) * u64::from(quantity),
    })
}

fn inspect_suspension_and_drop() {
    let services = Services::default();
    let price = services.price(SKU);
    assert!(services.steps.borrow().is_empty());

    // Здесь опросы вызываются вручную, чтобы проверить состояние между ними.
    let mut context = Context::from_waker(Waker::noop());
    let mut price = pin!(price);
    assert_eq!(price.as_mut().poll(&mut context), Poll::Pending);
    assert_eq!(services.steps.borrow().as_slice(), &[Step::PriceRequested]);
    assert_eq!(
        price.as_mut().poll(&mut context),
        Poll::Ready(Ok(PRICE_RUB))
    );
    println!("Ручной опрос — сначала Pending, затем Ready с ценой {PRICE_RUB} руб.");

    {
        let mut stock = pin!(services.stock(SKU));
        assert_eq!(stock.as_mut().poll(&mut context), Poll::Pending);
        // При выходе из блока Future уничтожается в точке ожидания.
        // Уже начатый запрос виден в журнале, продолжение не выполняется.
    }
    assert_eq!(
        services.steps.borrow().as_slice(),
        &[
            Step::PriceRequested,
            Step::PriceReceived,
            Step::StockRequested
        ]
    );
    println!("Удаление Future — запрос остатков начался, обработка ответа не выполнялась.");
}

async fn compare_await_order() {
    let sequential = Services::default();
    let first = quote_sequentially(&sequential, SKU, 3)
        .await
        .expect("Хватает товара для заказа");
    assert_eq!(
        sequential.steps.borrow().as_slice(),
        &[
            Step::PriceRequested,
            Step::PriceReceived,
            Step::StockRequested,
            Step::StockReceived
        ]
    );

    let concurrent = Services::default();
    let services = &concurrent;
    let task = {
        let sku = String::from(SKU);
        // async move забирает String в Future. Ссылка на сервис
        // тоже становится частью Future и остаётся действительной.
        async move { quote_concurrently(services, &sku, 3).await }
    };
    assert!(concurrent.steps.borrow().is_empty());
    let second = task.await.expect("Хватает товара для заказа");
    assert_eq!(
        concurrent.steps.borrow().as_slice(),
        &[
            Step::PriceRequested,
            Step::StockRequested,
            Step::PriceReceived,
            Step::StockReceived
        ]
    );

    assert_eq!(first, second);
    assert_eq!(
        second,
        Quote {
            quantity: 3,
            total_rub: 3_750
        }
    );
    println!("Последовательные .await запустили второй запрос после первого ответа.");
    println!(
        "Совместное ожидание запустило оба запроса до первого ответа. Сумма — {} руб.",
        second.total_rub
    );
}

async fn check_error_paths() {
    let missing = Services::default();
    assert_eq!(
        quote_sequentially(&missing, "UNKNOWN", 3).await,
        Err(QuoteError::UnknownSku)
    );
    // В записи .await? ожидание даёт Result, а ? возвращает ошибку.
    assert_eq!(
        missing.steps.borrow().as_slice(),
        &[Step::PriceRequested, Step::PriceReceived]
    );
    let parallel = Services::default();
    let (price, stock) = join_two(parallel.price("UNKNOWN"), parallel.stock(SKU)).await;
    assert_eq!(price, Err(QuoteError::UnknownSku));
    let stock = stock.expect("Запрос остатков должен завершиться");
    println!(
        "Ошибка цены — последовательный расчёт пропустил остатки, совместное ожидание получило {stock} шт."
    );

    let services = Services::default();
    assert_eq!(
        quote_concurrently(&services, SKU, STOCK + 1).await,
        Err(QuoteError::InsufficientStock { available: STOCK })
    );
    // Первый Future готов сразу, пока второй приостанавливается.
    // Готовый результат сохраняется, повторный опрос ему уже не нужен.
    let (label, stock) = join_two(ready("склад"), services.stock(SKU)).await;
    assert_eq!((label, stock), ("склад", Ok(STOCK)));
}

// Учебная реализация совместного ожидания двух Future.
async fn join_two<A: Future, B: Future>(first: A, second: B) -> (A::Output, B::Output) {
    let mut first = pin!(first);
    let mut second = pin!(second);
    let mut first_output = None;
    let mut second_output = None;

    poll_fn(|context| {
        if first_output.is_none()
            && let Poll::Ready(value) = first.as_mut().poll(context)
        {
            first_output = Some(value);
        }
        if second_output.is_none()
            && let Poll::Ready(value) = second.as_mut().poll(context)
        {
            second_output = Some(value);
        }
        if first_output.is_some() && second_output.is_some() {
            Poll::Ready((
                first_output.take().expect("Первый результат готов"),
                second_output.take().expect("Второй результат готов"),
            ))
        } else {
            Poll::Pending
        }
    })
    .await
}

struct ThreadWake(Thread);

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

// Синхронная точка входа ждёт один корневой Future. Между опросами
// park ждёт сигнал, а unpark сохраняет сигнал и до начала ожидания.
fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => thread::park(),
        }
    }
}

#[derive(Default)]
struct WorkerReply {
    stock: Option<u16>,
    waker: Option<Waker>,
}

fn inspect_wakeup_from_worker() {
    let reply = Arc::new(Mutex::new(WorkerReply::default()));
    let worker_reply = Arc::clone(&reply);
    let (start, wait_for_start) = channel();
    let worker = thread::spawn(move || {
        wait_for_start
            .recv()
            .expect("Задача должна начать ожидание");
        let waker = {
            let mut reply = worker_reply.lock().expect("Состояние доступно");
            reply.stock = Some(STOCK);
            reply
                .waker
                .take()
                .expect("Задача зарегистрировала пробуждение")
        };
        waker.wake();
    });

    let mut start = Some(start);
    let stock = block_on(poll_fn(|context| {
        let mut reply = reply.lock().expect("Состояние доступно");
        if let Some(stock) = reply.stock.take() {
            return Poll::Ready(stock);
        }
        reply.waker = Some(context.waker().clone());
        if let Some(start) = start.take() {
            start.send(()).expect("Рабочий поток готов");
        }
        Poll::Pending
    }));

    worker.join().expect("Рабочий поток должен завершиться");
    assert_eq!(stock, STOCK);
    println!("Пробуждение задачи — рабочий поток передал остаток: {stock} шт.");
}

fn main() {
    inspect_suspension_and_drop();
    inspect_wakeup_from_worker();
    block_on(async {
        compare_await_order().await;
        check_error_paths().await;
    });
}
