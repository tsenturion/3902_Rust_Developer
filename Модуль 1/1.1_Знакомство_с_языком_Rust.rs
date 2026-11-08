// Команда для запуска из корня курса.
// cargo run --offline --bin hello_rust

fn main() {
    println!("Привет, Rust!");
    println!("Результат сложения — {}.", 2 + 3);
    assert_eq!(2 + 3, 5);
    println!("Проверка 2 + 3 = 5 пройдена.");
}
