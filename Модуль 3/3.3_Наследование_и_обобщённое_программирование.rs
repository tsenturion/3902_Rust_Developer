/*
cargo run --offline --bin trait_composition
*/

/*
Импортёр читает записи из архива или принимает одиночный входящий пакет.
Записи передаются в буфер по порядку. Размер буфера и лимит партии могут
отличаться. Для архива нужен повторный импорт с начала, в том числе когда
он обёрнут ограничителем партии. Входящий пакет передаётся во владение
получателю при первом чтении.
*/

trait Source<T> {
    fn next_record(&mut self) -> Option<T>;
    fn remaining(&self) -> usize;

    // Все источники могут использовать одну реализацию через remaining.
    fn is_empty(&self) -> bool {
        self.remaining() == 0
    }
}

// Source<T> — супертрейт. Реализация Replayable<T> требует реализации Source<T>.
trait Replayable<T>: Source<T> {
    fn rewind(&mut self);
}

struct Archive<'data, T> {
    records: &'data [T],
    position: usize,
}

impl<'data, T> Archive<'data, T> {
    fn new(records: &'data [T]) -> Self {
        Self {
            records,
            position: 0,
        }
    }
}

impl<T: Clone> Source<T> for Archive<'_, T> {
    fn next_record(&mut self) -> Option<T> {
        // Архив сохраняет исходные записи для повторного чтения.
        let record = self.records.get(self.position)?.clone();
        self.position += 1;
        Some(record)
    }

    fn remaining(&self) -> usize {
        self.records.len() - self.position
    }
}

impl<T: Clone> Replayable<T> for Archive<'_, T> {
    fn rewind(&mut self) {
        self.position = 0;
    }
}

struct Pending<T> {
    record: Option<T>,
}

impl<T> Source<T> for Pending<T> {
    fn next_record(&mut self) -> Option<T> {
        // take перемещает пакет получателю и оставляет None.
        self.record.take()
    }

    fn remaining(&self) -> usize {
        usize::from(self.record.is_some())
    }
}

// Ограничитель хранит источник в поле и делегирует ему операции чтения.
struct Limited<S> {
    inner: S,
    limit: usize,
    taken: usize,
}

impl<S> Limited<S> {
    fn new(inner: S, limit: usize) -> Self {
        Self {
            inner,
            limit,
            taken: 0,
        }
    }
}

impl<T, S: Source<T>> Source<T> for Limited<S> {
    fn next_record(&mut self) -> Option<T> {
        if self.taken == self.limit {
            return None;
        }

        let record = self.inner.next_record()?;
        self.taken += 1;
        Some(record)
    }

    fn remaining(&self) -> usize {
        self.inner.remaining().min(self.limit - self.taken)
    }
}

// Обёртка получает расширенный контракт, когда его поддерживает вложенный источник.
impl<T, S: Replayable<T>> Replayable<T> for Limited<S> {
    fn rewind(&mut self) {
        self.inner.rewind();
        self.taken = 0;
    }
}

// Заполняем только первые записанные позиции. Хвост буфера остаётся прежним.
fn import_batch<T, S: Source<T>>(source: &mut S, output: &mut [Option<T>]) -> usize {
    let mut written = 0;
    for slot in output {
        let Some(record) = source.next_record() else {
            break;
        };
        *slot = Some(record);
        written += 1;
    }
    written
}

// Ограничение Replayable<T> даёт доступ к rewind и позволяет вызвать import_batch.
fn replay_batch<T, S: Replayable<T>>(source: &mut S, output: &mut [Option<T>]) -> usize {
    source.rewind();
    import_batch(source, output)
}

struct IncomingPacket {
    code: String,
}

fn main() {
    let codes = [17_u16, 24, 31];
    let mut archive = Limited::new(Archive::new(&codes), 2);
    let mut codes_out = [None; 3];

    assert_eq!(archive.remaining(), 2);
    assert!(!archive.is_empty());
    assert_eq!(import_batch(&mut archive, &mut codes_out[..1]), 1);
    assert_eq!(codes_out, [Some(17), None, None]);
    assert_eq!(archive.remaining(), 1);
    assert_eq!(import_batch(&mut archive, &mut codes_out[1..]), 1);
    assert_eq!(codes_out, [Some(17), Some(24), None]);
    assert!(archive.is_empty());
    assert_eq!(archive.next_record(), None);

    let mut repeated = [None; 3];
    let replayed = replay_batch(&mut archive, &mut repeated);
    assert_eq!(replayed, 2);
    assert_eq!(repeated, codes_out);
    assert!(archive.is_empty());
    println!("Архив — повторно импортировано {replayed} шт.");

    let mut disabled = Limited::new(Archive::new(&codes), 0);
    assert_eq!(replay_batch(&mut disabled, &mut repeated), 0);
    assert_eq!(repeated, codes_out);

    let pending = Pending {
        record: Some(IncomingPacket {
            code: String::from("А-41"),
        }),
    };
    let mut inbox = Limited::new(pending, 2);
    let mut packets_out = [None, None];
    assert_eq!(import_batch(&mut inbox, &mut packets_out[..0]), 0);
    assert_eq!(inbox.remaining(), 1);
    assert_eq!(import_batch(&mut inbox, &mut packets_out), 1);
    assert_eq!(
        packets_out[0].as_ref().map(|packet| packet.code.as_str()),
        Some("А-41")
    );
    assert!(packets_out[1].is_none());
    assert!(inbox.is_empty());
    assert_eq!(import_batch(&mut inbox, &mut packets_out[1..]), 0);
    if let Some(packet) = &packets_out[0] {
        println!("Входящие данные — получен пакет {}", packet.code);
    }

    // Pending реализует Source. Этого достаточно для import_batch.
    // let _ = replay_batch(&mut inbox, &mut packets_out);
}

// Для вызова rewind ограничение должно включать Replayable<T>.
// fn rewind_source<T, S: Source<T>>(source: &mut S) { source.rewind(); }

// Даже с реализацией rewind компилятор потребует Source<u16> для Unwired.
/*
struct Unwired;

impl Replayable<u16> for Unwired {
    fn rewind(&mut self) {}
}
*/
