/*
cargo test --offline --doc
*/

/*!
Складской терминал принимает заявку на перемещение и передаёт её следующему
участку. При обмене системам нужны коды событий и маска уже пройденных
контрольных точек. Результат решения по заявке должен обработать вызывающий код.

`rustdoc` компилирует примеры ниже как потребителей библиотеки `course_macros`.
В Cargo.toml эта библиотека объявлена как `proc-macro`. Компилятор вызывает
её функции во время сборки, передаёт им токены исходника и подставляет
полученный код в крейт, где вызван макрос.

```
use course_macros::{EventCode, checkpoint_mask, check_result};

// Трейт и одноимённый derive доступны в разных пространствах имён.
trait EventCode {
    const CODE: u16;
}

// Derive добавляет impl EventCode for TransferAccepted после объявления типа.
#[derive(EventCode)]
#[event_code = 501]
pub(crate) struct TransferAccepted {
    reference: u32,
}

#[derive(EventCode)]
#[event_code = 502]
enum TransferRejected {
    MissingCheckpoint,
}

// Вызов с ! заменяется одним литералом u8, поэтому годится для const.
const REQUIRED_CHECKPOINTS: u8 = checkpoint_mask!(0, 2, 5,);

// Атрибут возвращает исходную функцию вместе с новым атрибутом must_use.
#[check_result("Обработайте отказ в перемещении")]
fn accept(
    reference: u32,
    visited: u8,
) -> Result<TransferAccepted, TransferRejected> {
    if visited & REQUIRED_CHECKPOINTS == REQUIRED_CHECKPOINTS {
        Ok(TransferAccepted { reference })
    } else {
        Err(TransferRejected::MissingCheckpoint)
    }
}

let visited = checkpoint_mask!(0, 2, 5, 6);
match accept(781, visited) {
    Ok(event) => assert_eq!((event.reference, TransferAccepted::CODE), (781, 501)),
    Err(_) => panic!("Заявка прошла все обязательные контрольные точки"),
}

let visited = checkpoint_mask!(0, 5);
match accept(782, visited) {
    Ok(_) => panic!("Заявка пропустила обязательную точку 2"),
    Err(TransferRejected::MissingCheckpoint) => assert_eq!(TransferRejected::CODE, 502),
}

assert_eq!(REQUIRED_CHECKPOINTS, 0b0010_0101);
assert_eq!(checkpoint_mask!(), 0u8);
assert_eq!(checkpoint_mask!(7), 0b1000_0000);
```

В следующих примерах сборка должна завершиться ошибкой. Макрос указывает
на проблемный токен. В последнем примере срабатывает диагностика Rust,
которую включает сгенерированный `must_use`.

```compile_fail
use course_macros::checkpoint_mask;

const INVALID: u8 = checkpoint_mask!(0, 8);
```

```compile_fail
use course_macros::checkpoint_mask;

const INVALID: u8 = checkpoint_mask!(2, 2);
```

```compile_fail
use course_macros::EventCode;

trait EventCode {
    const CODE: u16;
}

#[derive(EventCode)]
struct UnregisteredTransfer;
```

```compile_fail
use course_macros::EventCode;

trait EventCode {
    const CODE: u16;
}

#[derive(EventCode)]
#[event_code = 503]
struct GenericTransfer<T>(T);
```

```compile_fail
use course_macros::check_result;

#[check_result(17)]
fn has_space() -> bool {
    false
}
```

```compile_fail
#![deny(unused_must_use)]

use course_macros::check_result;

#[check_result("Проверьте доступность места")]
fn has_space() -> bool {
    false
}

has_space();
```

```compile_fail
#![deny(unused_must_use)]

use course_macros::check_result;

#[check_result]
fn can_dispatch() -> bool {
    true
}

can_dispatch();
```
*/

extern crate proc_macro;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

type MacroResult<T> = Result<T, (Span, &'static str)>;

#[proc_macro]
pub fn checkpoint_mask(input: TokenStream) -> TokenStream {
    emit(expand_checkpoint_mask(input))
}

fn expand_checkpoint_mask(input: TokenStream) -> MacroResult<TokenStream> {
    let mut tokens = input.into_iter();
    let mut mask = 0u8;

    while let Some(token) = tokens.next() {
        let span = token.span();
        let position = decimal_u16(token)?;
        if u32::from(position) >= u8::BITS {
            return Err((span, "Номер контрольной точки должен быть от 0 до 7"));
        }

        let bit = 1u8 << position;
        if mask & bit != 0 {
            return Err((span, "Контрольная точка уже указана"));
        }
        mask |= bit;

        if let Some(separator) = tokens.next()
            && !matches!(&separator, TokenTree::Punct(punct) if punct.as_char() == ',')
        {
            return Err((separator.span(), "Между контрольными точками нужна запятая"));
        }
    }

    // Возвращаем токен 37u8 для входа 0, 2, 5. Цикл выполнился при сборке.
    Ok(TokenTree::Literal(Literal::u8_suffixed(mask)).into())
}

#[proc_macro_derive(EventCode, attributes(event_code))]
pub fn derive_event_code(input: TokenStream) -> TokenStream {
    emit(expand_event_code(input))
}

fn expand_event_code(input: TokenStream) -> MacroResult<TokenStream> {
    let mut tokens = input.into_iter().peekable();
    let mut code = None;

    // В derive приходят атрибуты и объявление целиком. Читаем только
    // заголовок, поэтому типы полей и значения вариантов разбирает сам Rust.
    while matches!(tokens.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '#') {
        tokens.next();
        let Some(TokenTree::Group(attribute)) = tokens.next() else {
            return Err((Span::call_site(), "Ожидался атрибут в квадратных скобках"));
        };
        if let Some((span, value)) = read_event_code(attribute)?
            && code.replace(value).is_some()
        {
            return Err((span, "Атрибут event_code можно указать один раз"));
        }
    }

    // Варианты заголовка включают struct, pub struct и pub(crate) struct.
    if matches!(tokens.peek(), Some(TokenTree::Ident(ident)) if ident.to_string() == "pub") {
        tokens.next();
        if matches!(tokens.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis)
        {
            tokens.next();
        }
    }
    let kind = tokens.next();
    if !matches!(&kind, Some(TokenTree::Ident(ident)) if matches!(ident.to_string().as_str(), "struct" | "enum"))
    {
        let span = kind.map_or_else(Span::call_site, |token| token.span());
        return Err((span, "EventCode поддерживает структуры и перечисления"));
    }
    let Some(TokenTree::Ident(name)) = tokens.next() else {
        return Err((Span::call_site(), "Ожидалось имя типа"));
    };
    if let Some(TokenTree::Punct(punct)) = tokens.peek()
        && punct.as_char() == '<'
    {
        return Err((
            punct.span(),
            "EventCode поддерживает типы без обобщённых параметров",
        ));
    }
    let code = code.ok_or((name.span(), "Добавьте к типу атрибут #[event_code = число]"))?;

    // Подставляем только проверенный идентификатор и готовое число.
    // Трейт EventCode разрешается в месте применения derive.
    let generated = format!("impl EventCode for {name} {{ const CODE: u16 = {code}; }}");
    generated
        .parse()
        .map_err(|_| (name.span(), "Не удалось сформировать impl EventCode"))
}

fn read_event_code(attribute: Group) -> MacroResult<Option<(Span, u16)>> {
    let mut tokens = attribute.stream().into_iter();
    let Some(TokenTree::Ident(name)) = tokens.next() else {
        return Ok(None);
    };
    if name.to_string() != "event_code" {
        return Ok(None);
    }

    match (tokens.next(), tokens.next(), tokens.next()) {
        (Some(TokenTree::Punct(equals)), Some(value), None) if equals.as_char() == '=' => {
            Ok(Some((name.span(), decimal_u16(value)?)))
        }
        _ => Err((
            name.span(),
            "Ожидался атрибут в форме #[event_code = число]",
        )),
    }
}

#[proc_macro_attribute]
pub fn check_result(arguments: TokenStream, item: TokenStream) -> TokenStream {
    emit(expand_check_result(arguments, item))
}

fn expand_check_result(arguments: TokenStream, item: TokenStream) -> MacroResult<TokenStream> {
    let mut tokens = arguments.into_iter();
    let message = match tokens.next().map(unwrap_invisible_group) {
        None => Literal::string("Обработайте результат операции"),
        Some(TokenTree::Literal(message)) => message,
        Some(token) => {
            return Err((token.span(), "Сообщение check_result должно быть строкой"));
        }
    };
    if let Some(extra) = tokens.next() {
        return Err((extra.span(), "check_result принимает одно сообщение"));
    }
    let text = message.to_string();
    if !(text.starts_with('"') || text.starts_with("r\"") || text.starts_with("r#")) {
        return Err((message.span(), "Сообщение check_result должно быть строкой"));
    }

    let span = Span::call_site();
    let contents = [
        TokenTree::Ident(Ident::new("must_use", span)),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(message),
    ]
    .into_iter()
    .collect();
    let mut output: TokenStream = [
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, contents)),
    ]
    .into_iter()
    .collect();
    // Возвращённый поток заменяет весь элемент. Сохраняем его исходные токены.
    output.extend(item);
    Ok(output)
}

fn decimal_u16(token: TokenTree) -> MacroResult<u16> {
    let token = unwrap_invisible_group(token);
    let span = token.span();
    let TokenTree::Literal(literal) = token else {
        return Err((span, "Ожидалось целое число из десятичных цифр"));
    };
    let digits = literal.to_string();
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err((
            span,
            "Ожидались десятичные цифры без суффикса и разделителей",
        ));
    }
    digits
        .parse()
        .map_err(|_| (span, "Число должно помещаться в u16"))
}

fn unwrap_invisible_group(mut token: TokenTree) -> TokenTree {
    // При передаче фрагмента macro_rules! компилятор может обернуть его в группу.
    while let TokenTree::Group(group) = &token {
        if group.delimiter() != Delimiter::None {
            break;
        }
        let mut inner = group.stream().into_iter();
        let (Some(only_token), None) = (inner.next(), inner.next()) else {
            break;
        };
        token = only_token;
    }
    token
}

fn emit(result: MacroResult<TokenStream>) -> TokenStream {
    match result {
        Ok(tokens) => tokens,
        Err((span, message)) => {
            // Привязываем compile_error! к исходному токену, который вызвал ошибку.
            let mut bang = Punct::new('!', Spacing::Alone);
            bang.set_span(span);
            let mut literal = Literal::string(message);
            literal.set_span(span);
            let mut argument = Group::new(Delimiter::Brace, TokenTree::Literal(literal).into());
            argument.set_span(span);
            [
                TokenTree::Ident(Ident::new("compile_error", span)),
                TokenTree::Punct(bang),
                TokenTree::Group(argument),
            ]
            .into_iter()
            .collect()
        }
    }
}
