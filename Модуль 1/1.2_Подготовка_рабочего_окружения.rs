// Проверка, сборка и запуск из корня курса в PowerShell.
/*
cargo check --offline --bin environment
cargo build --offline --bin environment
.\target\debug\environment.exe
*/

// Сборка и запуск напрямую через rustc.
/*
New-Item -ItemType Directory -Force -Path '.\target' | Out-Null
rustc --edition=2024 --crate-name environment_standalone -o '.\target\environment_standalone.exe' '.\Модуль 1\1.2_Подготовка_рабочего_окружения.rs'
if ($LASTEXITCODE -eq 0) {
    .\target\environment_standalone.exe
}
*/

// Вывод форматирования в терминал.
/*
rustfmt --edition 2024 --config max_width=50 --emit stdout '.\Модуль 1\1.2_Подготовка_рабочего_окружения.rs'
*/

fn main() {
    println!("Исходник — {}", file!());
    println!("Имя сборки — {}", module_path!());
    println!(
        "Версия пакета — {}",
        option_env!("CARGO_PKG_VERSION").unwrap_or("локальная сборка")
    );
    println!(
        "Запущенный файл — {}",
        std::env::current_exe()
            .expect("Не удалось определить путь к программе")
            .display()
    );
    println!(
        "Рабочий каталог — {}",
        std::env::current_dir()
            .expect("Не удалось определить рабочий каталог")
            .display()
    );
}
