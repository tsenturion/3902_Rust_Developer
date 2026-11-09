/*
cargo run --offline --bin static_dispatch
*/

// В PowerShell из корня курса можно найти обе версии quote_delivery и вызовы методов тарифа.
/*
cargo rustc --offline --bin static_dispatch -- -C opt-level=0 -C codegen-units=1 -C symbol-mangling-version=v0 --emit=asm=target/static_dispatch.s
Select-String -LiteralPath .\target\static_dispatch.s -Pattern '^\S*quote_delivery\S*:$|\b(callq?|bl)\b.*price_kopecks'
*/

/*
Для любой посылки с положительным весом нужно получить расчёт доставки.
Постамат берёт фиксированную плату и принимает посылки до своего предела.
Курьер берёт базовую плату и доплату за каждый начатый килограмм.
Если у клиента есть бюджет, расчёт сверх лимита нужно отклонить.
Все суммы хранятся в копейках, масса — в граммах.
*/

struct Parcel {
    order: u32,
    grams: u32,
}

#[derive(Debug, PartialEq)]
struct Quote {
    order: u32,
    price: u64,
}

#[derive(Debug, PartialEq)]
enum QuoteError {
    ZeroWeight,
    TooHeavy { grams: u32, limit: u32 },
    AboveBudget { price: u64, budget: u64 },
}

// Обобщённой функции нужен только метод расчёта цены. Каждый тариф реализует его сам.
trait DeliveryRate {
    fn price_kopecks(&self, grams: u32) -> Result<u64, QuoteError>;
}

struct LockerRate {
    max_grams: u32,
    flat_fee: u32,
}

impl DeliveryRate for LockerRate {
    // Сохраняем отдельный вызов, чтобы его было видно в ассемблерном листинге.
    #[inline(never)]
    fn price_kopecks(&self, grams: u32) -> Result<u64, QuoteError> {
        if grams > self.max_grams {
            return Err(QuoteError::TooHeavy {
                grams,
                limit: self.max_grams,
            });
        }
        Ok(u64::from(self.flat_fee))
    }
}

struct CourierRate {
    base_fee: u32,
    per_kg: u32,
}

impl DeliveryRate for CourierRate {
    #[inline(never)]
    fn price_kopecks(&self, grams: u32) -> Result<u64, QuoteError> {
        let billed_kg = u64::from(grams.div_ceil(1_000));
        Ok(u64::from(self.base_fee) + billed_kg * u64::from(self.per_kg))
    }
}

// Для использованных T компилятор создаёт конкретные версии функции.
// В каждой версии tariff.price_kopecks вызывает реализацию для этого T.
#[inline(never)]
fn quote_delivery<T: DeliveryRate>(tariff: &T, parcel: &Parcel) -> Result<Quote, QuoteError> {
    if parcel.grams == 0 {
        return Err(QuoteError::ZeroWeight);
    }
    let price = tariff.price_kopecks(parcel.grams)?;
    Ok(Quote {
        order: parcel.order,
        price,
    })
}

// impl DeliveryRate в параметре — краткая запись ограничения, когда имя типа не требуется.
fn quote_with_budget(
    tariff: &impl DeliveryRate,
    parcel: &Parcel,
    budget: u64,
) -> Result<Quote, QuoteError> {
    let quote = quote_delivery(tariff, parcel)?;
    if quote.price > budget {
        return Err(QuoteError::AboveBudget {
            price: quote.price,
            budget,
        });
    }
    Ok(quote)
}

fn main() {
    let locker = LockerRate {
        max_grams: 2_000,
        flat_fee: 15_000,
    };
    let courier = CourierRate {
        base_fee: 12_000,
        per_kg: 3_500,
    };
    let parcel = Parcel {
        order: 781,
        grams: 1_600,
    };

    // Первый вызов выводит T из аргумента, второй указывает его явно.
    let locker_quote = quote_delivery(&locker, &parcel);
    let courier_quote = quote_delivery::<CourierRate>(&courier, &parcel);
    assert_eq!(
        locker_quote,
        Ok(Quote {
            order: 781,
            price: 15_000
        })
    );
    assert_eq!(
        courier_quote,
        Ok(Quote {
            order: 781,
            price: 19_000
        })
    );
    if let (Ok(locker_quote), Ok(courier_quote)) = (locker_quote, courier_quote) {
        println!(
            "Заказ {} — постамат {} коп., курьер {} коп.",
            locker_quote.order, locker_quote.price, courier_quote.price
        );
    }

    let at_limit = Parcel {
        order: 782,
        grams: 2_000,
    };
    assert_eq!(
        quote_delivery(&locker, &at_limit),
        Ok(Quote {
            order: 782,
            price: 15_000
        })
    );
    assert_eq!(
        quote_delivery(&courier, &at_limit),
        Ok(Quote {
            order: 782,
            price: 19_000
        })
    );
    let heavy = Parcel {
        order: 783,
        grams: 2_001,
    };
    assert_eq!(
        quote_delivery(&locker, &heavy),
        Err(QuoteError::TooHeavy {
            grams: 2_001,
            limit: 2_000
        })
    );
    assert_eq!(
        quote_delivery(&courier, &heavy),
        Ok(Quote {
            order: 783,
            price: 22_500
        })
    );

    let empty = Parcel {
        order: 784,
        grams: 0,
    };
    assert_eq!(quote_delivery(&locker, &empty), Err(QuoteError::ZeroWeight));
    assert_eq!(
        quote_delivery(&courier, &empty),
        Err(QuoteError::ZeroWeight)
    );

    assert_eq!(
        quote_with_budget(&locker, &parcel, 15_000),
        Ok(Quote {
            order: 781,
            price: 15_000
        })
    );
    let over_budget = quote_with_budget(&courier, &parcel, 18_000);
    assert_eq!(
        over_budget,
        Err(QuoteError::AboveBudget {
            price: 19_000,
            budget: 18_000
        })
    );
    if let Err(QuoteError::AboveBudget { price, budget }) = over_budget {
        println!("Курьер — цена {price} коп. превышает бюджет {budget} коп.");
    }

    // Явно заданный T должен совпадать с типом переданного тарифа.
    // let _ = quote_delivery::<LockerRate>(&courier, &parcel);

    // Parcel не реализует DeliveryRate, поэтому расчёт с таким T не компилируется.
    // let _ = quote_delivery(&parcel, &parcel);
}
