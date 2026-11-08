// Сборка и запуск из корня курса в PowerShell.
/*
cargo build --offline --bin architecture
.\target\debug\architecture.exe
*/

// Создание ассемблерного и объектного файлов.
/*
cargo rustc --offline --bin architecture -- '--emit=asm=target/architecture.s,obj=target/architecture.obj'
*/

fn main() {
    // ОС запускает исполняемый файл как процесс.
    println!(
        "Процесс {}, целевая архитектура {}",
        std::process::id(),
        std::env::consts::ARCH
    );

    // Размер адреса определяется платформой, размер u32 задан в битах.
    println!(
        "Размеры в байтах — указатель {}, число u32 {}",
        std::mem::size_of::<*const u8>(),
        std::mem::size_of::<u32>()
    );

    // Один набор байтов можно записать в разных порядках.
    println!(
        "Порядок байтов машины — {:02X?}",
        0x1234_5678_u32.to_ne_bytes()
    );
    println!(
        "Порядок от младшего байта — {:02X?}",
        0x1234_5678_u32.to_le_bytes()
    );
    println!(
        "Порядок от старшего байта — {:02X?}",
        0x1234_5678_u32.to_be_bytes()
    );

    // У текста есть адрес в процессе и отдельное байтовое представление.
    println!(
        "Текст «Я Rust» — виртуальный адрес {:p}, байты {:02X?}",
        "Я Rust".as_ptr(),
        "Я Rust".as_bytes()
    );

    // black_box помогает сохранить операции в ассемблерном файле.
    println!("19 + 23 = {}", std::hint::black_box(19_u32) + 23);
    println!(
        "1010 И 1100 = {:04b}",
        std::hint::black_box(0b1010_u32) & 0b1100
    );
}
