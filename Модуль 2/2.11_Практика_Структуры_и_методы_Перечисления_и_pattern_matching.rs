/*
cargo run --offline --bin practice_2_11
*/

/*
Задача 1. Перевод между счетами

Суммы хранятся в целых минимальных единицах валюты. Разрешить перевод
положительной суммы между счетами в одной валюте. При нехватке средств
указать доступную сумму. Если новый баланс получателя не помещается в u64,
отклонить перевод. При любом отказе сохранить оба баланса.
*/
mod transfers {
    #[derive(PartialEq)]
    pub enum Currency {
        Rub,
        Kzt,
    }

    #[derive(Debug, PartialEq)]
    pub enum Transfer {
        Applied { amount: u64 },
        ZeroAmount,
        DifferentCurrencies,
        InsufficientFunds { available: u64 },
        RecipientBalanceLimit,
    }

    pub struct Account {
        currency: Currency,
        balance: u64,
    }

    impl Account {
        pub fn new(currency: Currency, balance: u64) -> Self {
            Self { currency, balance }
        }

        pub fn balance(&self) -> u64 {
            self.balance
        }

        pub fn transfer_to(&mut self, recipient: &mut Self, amount: u64) -> Transfer {
            match amount {
                0 => Transfer::ZeroAmount,
                _ if self.currency != recipient.currency => Transfer::DifferentCurrencies,
                _ if amount > self.balance => Transfer::InsufficientFunds {
                    available: self.balance,
                },
                _ if amount > u64::MAX - recipient.balance => Transfer::RecipientBalanceLimit,
                _ => {
                    self.balance -= amount;
                    recipient.balance += amount;
                    Transfer::Applied { amount }
                }
            }
        }
    }
}

fn run_transfers() {
    use transfers::{Account, Currency, Transfer};

    let mut sender = Account::new(Currency::Rub, 50_000);
    let mut recipient = Account::new(Currency::Rub, 2_000);
    assert_eq!(sender.transfer_to(&mut recipient, 0), Transfer::ZeroAmount);
    assert_eq!(
        sender.transfer_to(&mut recipient, 50_001),
        Transfer::InsufficientFunds { available: 50_000 }
    );
    assert_eq!((sender.balance(), recipient.balance()), (50_000, 2_000));

    let mut foreign = Account::new(Currency::Kzt, 3_000);
    assert_eq!(
        sender.transfer_to(&mut foreign, 1),
        Transfer::DifferentCurrencies
    );
    assert_eq!((sender.balance(), foreign.balance()), (50_000, 3_000));

    let mut near_limit = Account::new(Currency::Rub, u64::MAX - 10);
    assert_eq!(
        sender.transfer_to(&mut near_limit, 11),
        Transfer::RecipientBalanceLimit
    );
    assert_eq!(
        (sender.balance(), near_limit.balance()),
        (50_000, u64::MAX - 10)
    );
    assert_eq!(
        sender.transfer_to(&mut near_limit, 10),
        Transfer::Applied { amount: 10 }
    );
    assert_eq!(near_limit.balance(), u64::MAX);

    let transfer = sender.transfer_to(&mut recipient, 12_500);
    let Transfer::Applied { amount } = transfer else {
        panic!("Ожидался успешный перевод");
    };
    assert_eq!(amount, 12_500);
    assert_eq!((sender.balance(), recipient.balance()), (37_490, 14_500));
    println!(
        "Перевод {amount} коп. — остатки {} и {} коп.",
        sender.balance(),
        recipient.balance()
    );
}

/*
Задача 2. Заявка на ремонт оборудования

Новой заявке можно назначить техника. Закрыть её может только назначенный
техник, указав ненулевое время работы. При повторной поломке закрытую заявку
открывают заново, а техника назначают повторно. Повторные и недопустимые
команды должны оставить прежнее состояние.
*/
mod repairs {
    #[derive(Debug, PartialEq)]
    enum State {
        Open,
        Assigned(u32),
        Resolved { technician: u32, minutes: u16 },
    }

    enum Command {
        Assign(u32),
        Resolve { technician: u32, minutes: u16 },
        Reopen,
    }

    struct WorkOrder {
        equipment_id: u32,
        state: State,
    }

    impl WorkOrder {
        fn new(equipment_id: u32) -> Self {
            Self {
                equipment_id,
                state: State::Open,
            }
        }

        fn apply(&mut self, command: Command) -> bool {
            let next = match (&self.state, command) {
                (State::Open, Command::Assign(technician)) => State::Assigned(technician),
                (
                    State::Assigned(assigned),
                    Command::Resolve {
                        technician,
                        minutes,
                    },
                ) if *assigned == technician && minutes > 0 => State::Resolved {
                    technician,
                    minutes,
                },
                (State::Resolved { .. }, Command::Reopen) => State::Open,
                _ => return false,
            };
            self.state = next;
            true
        }

        fn requires_work(&self) -> bool {
            matches!(&self.state, State::Open | State::Assigned(_))
        }
    }

    pub fn run() {
        let mut order = WorkOrder::new(402);
        let steps = [
            (Command::Reopen, false, State::Open),
            (Command::Assign(19), true, State::Assigned(19)),
            (Command::Assign(27), false, State::Assigned(19)),
            (
                Command::Resolve {
                    technician: 27,
                    minutes: 30,
                },
                false,
                State::Assigned(19),
            ),
            (
                Command::Resolve {
                    technician: 19,
                    minutes: 0,
                },
                false,
                State::Assigned(19),
            ),
            (
                Command::Resolve {
                    technician: 19,
                    minutes: 35,
                },
                true,
                State::Resolved {
                    technician: 19,
                    minutes: 35,
                },
            ),
            (Command::Reopen, true, State::Open),
            (Command::Assign(27), true, State::Assigned(27)),
        ];

        for (command, accepted, expected_state) in steps {
            assert_eq!(order.apply(command), accepted);
            assert_eq!(order.state, expected_state);
        }
        assert!(order.requires_work());
        println!(
            "Оборудование {} — повторная заявка назначена технику 27",
            order.equipment_id
        );
    }
}

/*
Задача 3. Контроль холодильной камеры

Температура поступает в десятых долях градуса. Рабочий диапазон от 2,0 до 8,0 °C
включительно. При возрасте показания больше 120 секунд запросить свежее
измерение независимо от температуры. Коды 101 и 102 означают потерю связи.
Остальные коды отправить на диагностику, а плановое обслуживание пропустить.
*/
mod cold_room {
    enum Event {
        Sample { tenths: i16, age_seconds: u32 },
        Fault(u16),
        Maintenance,
    }

    #[derive(Debug, PartialEq)]
    enum Action {
        ContinueMonitoring,
        ReadAgain { age_seconds: u32 },
        TooCold(i16),
        TooWarm(i16),
        RestoreLink(u16),
        Diagnose(u16),
        SkipMaintenance,
    }

    fn classify(event: Event) -> Action {
        match event {
            Event::Sample { age_seconds, .. } if age_seconds > 120 => {
                Action::ReadAgain { age_seconds }
            }
            Event::Sample {
                tenths: 20..=80, ..
            } => Action::ContinueMonitoring,
            Event::Sample {
                tenths: value @ i16::MIN..=19,
                ..
            } => Action::TooCold(value),
            Event::Sample { tenths, .. } => Action::TooWarm(tenths),
            Event::Fault(code @ (101 | 102)) => Action::RestoreLink(code),
            Event::Fault(code) => Action::Diagnose(code),
            Event::Maintenance => Action::SkipMaintenance,
        }
    }

    pub fn run() {
        let samples = [
            (19, Action::TooCold(19)),
            (20, Action::ContinueMonitoring),
            (80, Action::ContinueMonitoring),
            (81, Action::TooWarm(81)),
        ];
        for (tenths, action) in samples {
            assert_eq!(
                classify(Event::Sample {
                    tenths,
                    age_seconds: 120
                }),
                action
            );
        }
        assert_eq!(
            classify(Event::Sample {
                tenths: 81,
                age_seconds: 121
            }),
            Action::ReadAgain { age_seconds: 121 }
        );
        assert_eq!(classify(Event::Fault(101)), Action::RestoreLink(101));
        assert_eq!(classify(Event::Fault(102)), Action::RestoreLink(102));
        assert_eq!(classify(Event::Fault(301)), Action::Diagnose(301));
        assert_eq!(classify(Event::Maintenance), Action::SkipMaintenance);
        println!(
            "Холодильная камера — для показания 8,1 °C давностью 121 с нужно повторить измерение"
        );
    }
}

/*
Задача 4. Сверка двух коробок перед отправкой

В накладной указаны два разных кода. Сканер должен передать ровно два
показания. Разрешить отправку, когда оба кода совпадают с накладной
в любом порядке. Различать лишние или недостающие показания,
непрочитанную метку, повторный скан одной коробки и чужой код.
*/
mod manifest {
    enum Scan {
        Read(u32),
        Unreadable,
    }

    #[derive(Debug, PartialEq)]
    enum Check {
        Ready,
        WrongCount(usize),
        Rescan,
        Duplicate(u32),
        CodeMismatch,
    }

    struct Manifest {
        labels: [u32; 2],
    }

    impl Manifest {
        fn check(&self, scans: &[Scan]) -> Check {
            let [first, second] = scans else {
                return Check::WrongCount(scans.len());
            };
            let [left, right] = self.labels;
            match (first, second) {
                (Scan::Unreadable, _) | (_, Scan::Unreadable) => Check::Rescan,
                (Scan::Read(a), Scan::Read(b)) if a == b => Check::Duplicate(*a),
                (Scan::Read(a), Scan::Read(b))
                    if (*a, *b) == (left, right) || (*a, *b) == (right, left) =>
                {
                    Check::Ready
                }
                (Scan::Read(_), Scan::Read(_)) => Check::CodeMismatch,
            }
        }
    }

    pub fn run() {
        let manifest = Manifest { labels: [510, 511] };
        use Scan::{Read, Unreadable};
        assert_eq!(manifest.check(&[Read(510), Read(511)]), Check::Ready);
        assert_eq!(manifest.check(&[Read(511), Read(510)]), Check::Ready);
        assert_eq!(manifest.check(&[]), Check::WrongCount(0));
        assert_eq!(manifest.check(&[Read(510)]), Check::WrongCount(1));
        assert_eq!(
            manifest.check(&[Read(510), Read(511), Read(512)]),
            Check::WrongCount(3)
        );
        assert_eq!(manifest.check(&[Unreadable, Read(511)]), Check::Rescan);
        assert_eq!(manifest.check(&[Read(510), Unreadable]), Check::Rescan);
        assert_eq!(
            manifest.check(&[Read(510), Read(510)]),
            Check::Duplicate(510)
        );
        assert_eq!(manifest.check(&[Read(510), Read(512)]), Check::CodeMismatch);
        println!("Накладная — коробки 511 и 510 готовы к отправке после сверки в обратном порядке");
    }
}

/*
Задача 5. Очередь талонов на стойке выдачи

Терминал хранит до трёх ожидающих талонов. Добавлять их в конец, вызывать
с начала, а при просмотре первого сохранять очередь. После выдачи можно
снова использовать освободившуюся ячейку массива. При переполнении или
пустой очереди вернуть соответствующий ответ и сохранить состояние.
*/
mod service_queue {
    #[derive(Clone, Copy)]
    enum Command {
        Enqueue(u32),
        Peek,
        Call,
    }

    #[derive(Debug, PartialEq)]
    enum Reply {
        Queued,
        Full,
        Empty,
        Next(u32),
        Called(u32),
    }

    struct TicketQueue {
        slots: [u32; 3],
        head: usize,
        len: usize,
    }

    impl TicketQueue {
        fn new() -> Self {
            Self {
                slots: [0; 3],
                head: 0,
                len: 0,
            }
        }

        fn apply(&mut self, command: Command) -> Reply {
            match command {
                Command::Enqueue(_) if self.len == self.slots.len() => Reply::Full,
                Command::Enqueue(number) => {
                    let tail = (self.head + self.len) % self.slots.len();
                    self.slots[tail] = number;
                    self.len += 1;
                    Reply::Queued
                }
                Command::Peek | Command::Call if self.len == 0 => Reply::Empty,
                Command::Peek => Reply::Next(self.slots[self.head]),
                Command::Call => {
                    let number = self.slots[self.head];
                    self.head = (self.head + 1) % self.slots.len();
                    self.len -= 1;
                    Reply::Called(number)
                }
            }
        }
    }

    pub fn run() {
        let mut queue = TicketQueue::new();
        let steps = [
            (Command::Call, Reply::Empty),
            (Command::Enqueue(121), Reply::Queued),
            (Command::Enqueue(122), Reply::Queued),
            (Command::Enqueue(123), Reply::Queued),
            (Command::Enqueue(124), Reply::Full),
            (Command::Peek, Reply::Next(121)),
            (Command::Call, Reply::Called(121)),
            (Command::Enqueue(124), Reply::Queued),
            (Command::Peek, Reply::Next(122)),
            (Command::Call, Reply::Called(122)),
            (Command::Call, Reply::Called(123)),
            (Command::Call, Reply::Called(124)),
            (Command::Peek, Reply::Empty),
        ];

        let mut remaining = &steps[..];
        while let [(command, expected), rest @ ..] = remaining {
            assert_eq!(&queue.apply(*command), expected);
            remaining = rest;
        }
        println!("Очередь — талоны 121, 122, 123 и 124 вызваны по порядку");
    }
}

/*
Задача 6. Подготовка уведомления к отправке

Черновик содержит текст и адрес назначения. Адрес электронной почты и номер
для СМС хранятся в разных вариантах. Разрешить обновление номера только
для СМС и отклонять пустое значение. Для очереди отправки подготовить пакет,
переместив в него контакт и текст из черновика без клонирования строк.
*/
mod notices {
    enum Destination {
        Email { address: String },
        Sms { phone: String },
    }

    #[derive(Debug, PartialEq)]
    enum Channel {
        Email,
        Sms,
    }

    struct Draft {
        destination: Destination,
        message: String,
    }

    struct Packet {
        channel: Channel,
        contact: String,
        message: String,
    }

    impl Draft {
        fn change_phone(&mut self, new_phone: String) -> bool {
            if new_phone.trim().is_empty() {
                return false;
            }
            if let Destination::Sms { phone } = &mut self.destination {
                *phone = new_phone;
                true
            } else {
                false
            }
        }

        fn into_packet(self) -> Packet {
            let Self {
                destination,
                message,
            } = self;
            let (channel, contact) = match destination {
                Destination::Email { address } => (Channel::Email, address),
                Destination::Sms { phone } => (Channel::Sms, phone),
            };
            Packet {
                channel,
                contact,
                message,
            }
        }
    }

    pub fn run() {
        let mut sms = Draft {
            destination: Destination::Sms {
                phone: String::from("+70000000001"),
            },
            message: String::from("Заказ готов к выдаче"),
        };
        assert!(sms.change_phone(String::from("+70000000002")));
        assert!(!sms.change_phone(String::from("  ")));
        let packet = sms.into_packet();
        assert_eq!(packet.channel, Channel::Sms);
        assert_eq!(packet.contact, "+70000000002");
        assert_eq!(packet.message, "Заказ готов к выдаче");

        let mut email = Draft {
            destination: Destination::Email {
                address: String::from("customer@example.com"),
            },
            message: String::from("Квитанция готова"),
        };
        assert!(!email.change_phone(String::from("+70000000003")));
        let packet = email.into_packet();
        assert_eq!(packet.channel, Channel::Email);
        assert_eq!(packet.contact, "customer@example.com");
        assert_eq!(packet.message, "Квитанция готова");
        println!(
            "Уведомления — подготовлены пакет СМС с новым номером и пакет письма с исходным адресом"
        );
    }
}

fn main() {
    run_transfers();
    repairs::run();
    cold_room::run();
    manifest::run();
    service_queue::run();
    notices::run();
}
