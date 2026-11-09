/*
cargo run --offline --bin practice_2_7
*/

use std::fs::{self, File};
use std::io::Write;

/*
Задача 1. Проверка заявки перед экспортом

Одна проверка оценивает объём файла по длительности и скорости потока.
Вторая проверяет, что в заявке указан непустой путь назначения.
Обе проверки получают заявку через общие ссылки. После проверок
заявка нужна для редактирования.
*/
struct ExportJob {
    id: u32,
    destination: String,
    duration_seconds: u32,
    bytes_per_second: u32,
}

fn estimated_export_bytes(job: &ExportJob) -> u64 {
    u64::from(job.duration_seconds) * u64::from(job.bytes_per_second)
}

fn has_export_destination(job: &ExportJob) -> bool {
    !job.destination.is_empty()
}

/*
Задача 2. Перенаправление экспорта

Архив перенесли в другую папку. Записать новый путь в существующую
заявку и вернуть прежний для сверки с журналом. Обе строки нужно
передать между владельцами без создания дополнительных копий.
*/
fn replace_destination(job: &mut ExportJob, destination: String) -> String {
    // Поле остаётся заполненным во время замены.
    // let _previous = job.destination;
    std::mem::replace(&mut job.destination, destination)
}

/*
Задача 3. Передача заявки в очередь

Элемент очереди должен хранить заявку и оценку объёма. Рассчитать
объём через общую ссылку, затем передать саму заявку в элемент очереди.
С этого момента исходный путь и параметры принадлежат элементу очереди.
*/
struct QueuedExport {
    job: ExportJob,
    estimated_bytes: u64,
}

fn enqueue_export(job: ExportJob) -> QueuedExport {
    let estimated_bytes = estimated_export_bytes(&job);
    QueuedExport {
        job,
        estimated_bytes,
    }
}

/*
Задача 4. Резервирование места

Квота хранит лимит и объём уже зарезервированного места.
Принять квоту через &mut и увеличить резерв только при наличии места.
При отказе сохранить прежний резерв. Если исходный резерв превышает
лимит, отклонить заявку до вычисления остатка.
*/
struct StorageQuota {
    limit_bytes: u64,
    reserved_bytes: u64,
}

fn reserve_bytes(quota: &mut StorageQuota, bytes: u64) -> bool {
    if quota.reserved_bytes > quota.limit_bytes || bytes > quota.limit_bytes - quota.reserved_bytes
    {
        return false;
    }

    quota.reserved_bytes += bytes;
    true
}

/*
Задача 5. Уведомление диспетчеру

Номер инцидента и текст уведомления лежат в разных полях.
Взять номер через общую ссылку, а текст — через изменяемую.
Сформировать сообщение из номера. Повторное формирование должно
заменить прежний текст, чтобы номер не дописывался повторно.
*/
#[derive(Clone)]
struct DispatchNotice {
    incident_code: String,
    message: String,
    priority: u8,
}

fn prepare_notice(notice: &mut DispatchNotice) {
    let incident_code = &notice.incident_code;
    let message = &mut notice.message;

    message.clear();
    message.push_str("Проверить инцидент ");
    message.push_str(incident_code);
}

/*
Задача 6. Имя архива после выхода из блока

Внутренний блок формирует основу имени без расширения.
Функция принимает её во владение, добавляет .tar.gz при включённом
сжатии или .tar в остальных случаях и возвращает готовую строку.
Итоговое имя должно быть доступно после завершения блока.
*/
fn archive_name(mut stem: String, compressed: bool) -> String {
    let extension = if compressed { ".tar.gz" } else { ".tar" };
    stem.push_str(extension);
    stem
}

/*
Задача 7. Отдельное уведомление для тестовой рассылки

Оператор готовит рабочее уведомление и его тестовую версию.
Тестовая версия получает пометку в тексте и приоритет на единицу ниже.
Нижняя граница приоритета — 0. Сохранить приоритет рабочего сообщения
как числовой снимок и создать отдельное уведомление через clone.
*/
fn make_test_notice(notice: &DispatchNotice) -> DispatchNotice {
    let mut test_notice: DispatchNotice = notice.clone();
    test_notice.message.push_str(" [тест]");
    test_notice.priority = test_notice.priority.saturating_sub(1);
    test_notice
}

/*
Задача 8. Запись квитанции экспорта

Передать открытый файл и элемент очереди функции записи.
Записать номер заявки, путь назначения и оценку объёма.
После возврата функции прочитать квитанцию по пути и сверить содержимое.
Файл закрывается при выходе из функции записи.
*/
fn write_export_receipt(mut file: File, queued: QueuedExport) {
    writeln!(
        file,
        "Заявка — {}\nПуть — {}\nОценка — {} байт",
        queued.job.id, queued.job.destination, queued.estimated_bytes
    )
    .expect("Не удалось записать квитанцию");
}

fn main() {
    let mut job = ExportJob {
        id: 17,
        destination: String::from("exports/standard/lesson-02.mp4"),
        duration_seconds: 90,
        bytes_per_second: 100_000,
    };
    let size_view = &job;
    let destination_view = &job;
    let estimated_bytes = estimated_export_bytes(size_view);
    // Пока destination_view нужна для проверки, заявка доступна через общую ссылку.
    // let _editor = &mut job;
    assert!(has_export_destination(destination_view));
    assert_eq!(estimated_bytes, 9_000_000);
    println!("1. Заявка проверена, оценка — {estimated_bytes} байт");

    let mut additional_job = ExportJob {
        id: 18,
        destination: String::new(),
        duration_seconds: 30,
        bytes_per_second: 100_000,
    };
    let additional_bytes = estimated_export_bytes(&additional_job);
    let has_destination = has_export_destination(&additional_job);
    assert_eq!(additional_bytes, 3_000_000);
    assert!(!has_destination);
    println!(
        "1. Заявка {} — оценка {additional_bytes} байт, путь указан {has_destination}",
        additional_job.id
    );

    let previous = replace_destination(&mut job, String::from("exports/archive/lesson-02.mp4"));
    assert_eq!(previous, "exports/standard/lesson-02.mp4");
    assert_eq!(job.destination, "exports/archive/lesson-02.mp4");
    println!("2. Путь изменён с {previous} на {}", job.destination);

    let previous_empty = replace_destination(
        &mut additional_job,
        String::from("exports/archive/lesson-03.mp4"),
    );
    assert!(previous_empty.is_empty());
    assert!(has_export_destination(&additional_job));
    println!(
        "2. Первый путь заявки {} — {}, прежний путь пустой — {}",
        additional_job.id,
        additional_job.destination,
        previous_empty.is_empty()
    );

    let queued = enqueue_export(job);
    assert_eq!(queued.job.id, 17);
    assert_eq!(queued.estimated_bytes, estimated_bytes);
    println!("3. В очереди заявка {}", queued.job.id);
    // Владение заявкой передано в queued.
    // println!("{}", job.destination);

    let additional_queued = enqueue_export(additional_job);
    assert_eq!(additional_queued.job.id, 18);
    assert_eq!(additional_queued.estimated_bytes, additional_bytes);
    assert_eq!(
        additional_queued.job.destination,
        "exports/archive/lesson-03.mp4"
    );
    println!(
        "3. В очереди заявка {} — {} байт для {}",
        additional_queued.job.id,
        additional_queued.estimated_bytes,
        additional_queued.job.destination
    );

    let mut quota = StorageQuota {
        limit_bytes: 10_000_000,
        reserved_bytes: 500_000,
    };
    let editor = &mut quota;
    // Ссылка editor используется при резервировании.
    // let second_editor = &mut quota;
    // reserve_bytes(second_editor, 1);
    assert!(reserve_bytes(editor, queued.estimated_bytes));
    assert!(!reserve_bytes(editor, 1_000_000));
    assert_eq!(quota.reserved_bytes, 9_500_000);

    let mut invalid_quota = StorageQuota {
        limit_bytes: 1,
        reserved_bytes: 2,
    };
    assert!(!reserve_bytes(&mut invalid_quota, 0));
    assert_eq!(invalid_quota.reserved_bytes, 2);
    println!("4. В квоте зарезервировано {} байт", quota.reserved_bytes);

    let exact_reserve = reserve_bytes(&mut quota, 500_000);
    let extra_byte = reserve_bytes(&mut quota, 1);
    assert!(exact_reserve);
    assert!(!extra_byte);
    assert_eq!(quota.reserved_bytes, quota.limit_bytes);
    println!(
        "4. Ровно до лимита — {exact_reserve}, ещё один байт — {extra_byte}, резерв — {} байт",
        quota.reserved_bytes
    );

    let mut notice = DispatchNotice {
        incident_code: String::from("INC-42"),
        message: String::new(),
        priority: 3,
    };
    prepare_notice(&mut notice);
    prepare_notice(&mut notice);
    assert_eq!(notice.incident_code, "INC-42");
    assert_eq!(notice.message, "Проверить инцидент INC-42");
    println!("5. Уведомление — {}", notice.message);

    let mut revised_notice = notice.clone();
    revised_notice.incident_code = String::from("INC-43");
    revised_notice.priority = 0;
    prepare_notice(&mut revised_notice);
    assert_eq!(revised_notice.message, "Проверить инцидент INC-43");
    println!("5. После смены номера — {}", revised_notice.message);

    let (compressed_name, plain_name) = {
        let stem = String::from("course-lesson-02");
        let compressed = archive_name(stem.clone(), true);
        let plain = archive_name(stem, false);
        (compressed, plain)
    };
    assert_eq!(compressed_name, "course-lesson-02.tar.gz");
    assert_eq!(plain_name, "course-lesson-02.tar");
    println!("6. Имя архива — {compressed_name}");
    println!("6. Имя без сжатия из того же блока — {plain_name}");
    // Временная строка освобождается при выходе из блока.
    // Обращение к ссылке после блока компилятор отклоняет.
    // let invalid_name = {
    //     let temporary = String::from("course-lesson-02");
    //     &temporary
    // };
    // println!("{invalid_name}");

    let priority_before = notice.priority;
    let test_notice = make_test_notice(&notice);
    assert_eq!(notice.priority, priority_before);
    assert_eq!(notice.message, "Проверить инцидент INC-42");
    assert_eq!(test_notice.message, "Проверить инцидент INC-42 [тест]");
    assert_eq!(test_notice.priority, 2);
    println!(
        "7. Приоритеты рабочего и тестового уведомлений — {priority_before} и {}",
        test_notice.priority
    );

    notice.priority = 4;
    let zero_priority_test = make_test_notice(&revised_notice);
    assert_eq!(priority_before, 3);
    assert_eq!(test_notice.priority, 2);
    assert_eq!(zero_priority_test.priority, 0);
    println!(
        "7. После повышения приоритета — снимок {priority_before}, рабочий {}, тестовый {}, нижняя граница {}",
        notice.priority, test_notice.priority, zero_priority_test.priority
    );

    fs::create_dir_all("target/ownership_practice").expect("Не удалось создать папку квитанций");
    let process_id = std::process::id();
    let receipt_path = format!("target/ownership_practice/receipt-{process_id}.txt");
    let file = File::create(&receipt_path).expect("Не удалось создать квитанцию");
    write_export_receipt(file, queued);
    // Открытый файл передан функции записи.
    // drop(file);
    let receipt = fs::read_to_string(&receipt_path).expect("Не удалось прочитать квитанцию");
    assert_eq!(
        receipt,
        "Заявка — 17\nПуть — exports/archive/lesson-02.mp4\nОценка — 9000000 байт\n"
    );
    println!("8. Квитанция записана в {receipt_path}");

    let additional_receipt_path = format!(
        "target/ownership_practice/receipt-{process_id}-{}.txt",
        additional_queued.job.id
    );
    let additional_file =
        File::create(&additional_receipt_path).expect("Не удалось создать вторую квитанцию");
    write_export_receipt(additional_file, additional_queued);
    let additional_receipt = fs::read_to_string(&additional_receipt_path)
        .expect("Не удалось прочитать вторую квитанцию");
    assert_eq!(
        additional_receipt,
        "Заявка — 18\nПуть — exports/archive/lesson-03.mp4\nОценка — 3000000 байт\n"
    );
    println!("8. Вторая квитанция записана в {additional_receipt_path}");
}
