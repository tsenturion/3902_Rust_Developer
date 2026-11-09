/*
cargo run --offline --bin declarative_macros
*/

/*
Перед отгрузкой терминал проверяет этикетку, количество коробов и доступность
дока. Применяется первое нарушение в этом порядке. У каждого отказа есть
числовой код для терминала и описание для оператора. По коду можно восстановить
вариант из каталога.
*/

macro_rules! define_denials {
    // vis — видимость, ident — имя, ty — тип, literal — литерал.
    ($visibility:vis enum $name:ident, $code_type:ty {
        $($variant:ident => ($code:literal, $message:literal)),+ $(,)?
    }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $visibility enum $name {
            $(#[doc = $message] $variant),+
        }

        impl $name {
            $visibility const ALL: &'static [Self] = &[$(Self::$variant),+];

            $visibility fn code(self) -> $code_type {
                match self {
                    $(Self::$variant => $code),+
                }
            }

            $visibility fn message(self) -> &'static str {
                match self {
                    $(Self::$variant => $message),+
                }
            }

            // Повторяющийся код дал бы недостижимую ветку и ошибку компиляции.
            #[deny(unreachable_patterns)]
            $visibility fn from_code(code: $code_type) -> ::core::option::Option<Self> {
                match code {
                    $($code => ::core::option::Option::Some(Self::$variant),)+
                    _ => ::core::option::Option::None,
                }
            }
        }
    };
    ($($tokens:tt)*) => {
        compile_error!("Каталогу нужен enum, тип кода и хотя бы один вариант с парой из кода и сообщения");
    };
}

mod catalog {
    // + требует хотя бы один вариант. $(,)? допускает последнюю запятую.
    define_denials! {
        pub(super) enum Denial, u16 {
            MissingLabel => (101, "нет этикетки"),
            InvalidCount => (102, "недопустимое количество коробов"),
            DockUnavailable => (103, "док недоступен"),
        }
    }
}

macro_rules! gate {
    () => {
        // $crate ведёт к каталогу в крейте, где объявлен макрос.
        ::core::result::Result::<(), $crate::catalog::Denial>::Ok(())
    };
    ($($condition:expr => $error:expr),+ $(,)?) => {
        'gate: {
            $(
                // Каждое выражение условия подставляется один раз.
                let passed = $condition;
                if !passed {
                    break 'gate ::core::result::Result::Err($error);
                }
            )+
            gate!()
        }
    };
    ($($tokens:tt)*) => {
        compile_error!("В gate ожидаются пары условие => отказ, разделённые запятыми")
    };
}

struct Shipment<'label> {
    number: u32,
    label: Option<&'label str>,
    cartons: u16,
    dock_ready: bool,
}

mod dispatch {
    use super::Shipment;
    use super::catalog::Denial;

    pub(super) fn check(shipment: &Shipment<'_>) -> Result<(), Denial> {
        gate!(
            shipment.label.is_some_and(|label| !label.trim().is_empty()) => Denial::MissingLabel,
            (1..=20).contains(&shipment.cartons) => Denial::InvalidCount,
            shipment.dock_ready => Denial::DockUnavailable,
        )
    }
}

fn check_evaluation() {
    use catalog::Denial;

    let passed = false;
    let mut checked = Vec::new();
    let mut failures_built = 0;
    let result = gate!(
        { checked.push("этикетка"); !passed } => {
            failures_built += 1;
            Denial::MissingLabel
        },
        { checked.push("количество"); passed } => {
            failures_built += 1;
            Denial::InvalidCount
        },
        { checked.push("док"); true } => Denial::DockUnavailable,
    );
    // В выражениях вызова passed — переменная вызывающего кода.
    assert_eq!(result, Err(Denial::InvalidCount));
    assert_eq!(checked, ["этикетка", "количество"]);
    assert_eq!(failures_built, 1);
    assert_eq!(gate!(), Ok(()));
    assert_eq!(gate!(true => Denial::MissingLabel), Ok(()));
    println!(
        "Порядок проверки — {}, выражений отказа: {failures_built}",
        checked.join(", ")
    );
}

fn main() {
    use catalog::Denial;

    check_evaluation();
    let shipments = [
        Shipment {
            number: 781,
            label: Some("K-781"),
            cartons: 20,
            dock_ready: true,
        },
        Shipment {
            number: 782,
            label: None,
            cartons: 21,
            dock_ready: false,
        },
        Shipment {
            number: 783,
            label: Some("K-783"),
            cartons: 21,
            dock_ready: false,
        },
        Shipment {
            number: 784,
            label: Some("K-784"),
            cartons: 1,
            dock_ready: false,
        },
    ];
    let decisions = shipments.each_ref().map(dispatch::check);
    assert_eq!(
        decisions,
        [
            Ok(()),
            Err(Denial::MissingLabel),
            Err(Denial::InvalidCount),
            Err(Denial::DockUnavailable)
        ]
    );
    for (shipment, decision) in shipments.iter().zip(decisions) {
        match decision {
            Ok(()) => println!("Заявка {} — допущена", shipment.number),
            Err(denial) => {
                let code = denial.code();
                println!(
                    "Заявка {} — отказ {code}, {}",
                    shipment.number,
                    denial.message()
                );
            }
        }
    }

    for &denial in Denial::ALL {
        assert_eq!(Denial::from_code(denial.code()), Some(denial));
    }
    let codes: Vec<_> = Denial::ALL.iter().map(|denial| denial.code()).collect();
    let unknown = Denial::from_code(999);
    assert_eq!(unknown, None);
    println!("Каталог отказов — коды {codes:?}, код 999: {unknown:?}");
}
