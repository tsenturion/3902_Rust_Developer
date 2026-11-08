/*
cargo run --offline --bin structs
*/

struct Resolution {
    width: u16,
    height: u16,
}

struct RecordingProfile {
    resolution: Resolution,
    frames_per_second: u16,
    captions: bool,
}

// Каждая кортежная структура задаёт отдельный тип.
struct Width(u16);
struct Height(u16);

// Структура без полей может служить меткой.
struct PreviewPreset;

fn main() {
    let mut recording = recording_profile(Width(1_920), Height(1_080), 30);
    assert!(!recording.captions);
    recording.captions = true;

    // Поля перечисляются по именам. ..recording заполняет оставшееся поле captions.
    let preview = RecordingProfile {
        resolution: Resolution {
            height: 360,
            width: 640,
        },
        frames_per_second: 15,
        ..recording
    };
    assert_eq!(preview.resolution.width, 640);
    assert_eq!(preview.resolution.height, 360);
    assert_eq!(preview.captions, recording.captions);
    assert_eq!(recording.frames_per_second, 30);

    // Для эскиза меняем разрешение, а частоту кадров и субтитры берём из предпросмотра.
    let thumbnail = RecordingProfile {
        resolution: Resolution {
            width: 320,
            height: 180,
        },
        ..preview
    };
    assert_eq!(thumbnail.frames_per_second, preview.frames_per_second);
    assert_eq!(thumbnail.captions, preview.captions);
    assert_eq!(preview.resolution.width, 640);

    let preset = PreviewPreset;
    let preset_size = std::mem::size_of_val(&preset);
    assert_eq!(preset_size, 0);

    println!(
        "Запись — {}×{}, {} кадров/с",
        recording.resolution.width, recording.resolution.height, recording.frames_per_second
    );
    println!(
        "Предпросмотр — {}×{}, {} кадров/с, субтитры — {}",
        preview.resolution.width,
        preview.resolution.height,
        preview.frames_per_second,
        preview.captions
    );
    println!(
        "Эскиз — {}×{}, {} кадров/с, субтитры — {}",
        thumbnail.resolution.width,
        thumbnail.resolution.height,
        thumbnail.frames_per_second,
        thumbnail.captions
    );
    println!("Метка режима предпросмотра — {preset_size} байт");

    // В литерале требуется значение для каждого поля.
    // let _ = RecordingProfile {
    //     resolution: Resolution { width: 640, height: 360 },
    //     captions: true,
    // };

    // Поле выбирается по объявленному имени.
    // let _ = recording.fps;

    // Компилятор различает Width и Height.
    // let _ = recording_profile(Height(1_920), Width(1_080), 30);

    // Присваивание полю требует изменяемого экземпляра.
    // preview.captions = false;

    let recording_rate = pixels_per_second(recording);
    let preview_rate = pixels_per_second(preview);
    let thumbnail_rate = pixels_per_second(thumbnail);
    assert_eq!(recording_rate, 62_208_000);
    assert_eq!(preview_rate, 3_456_000);
    assert_eq!(thumbnail_rate, 864_000);
    println!("Нагрузка — {recording_rate} и {preview_rate} пикселей/с");
    println!("Нагрузка эскиза — {thumbnail_rate} пикселей/с");
}

fn recording_profile(width: Width, height: Height, frames_per_second: u16) -> RecordingProfile {
    RecordingProfile {
        resolution: Resolution {
            width: width.0,
            height: height.0,
        },
        // Имя параметра совпадает с именем поля.
        frames_per_second,
        captions: false,
    }
}

fn pixels_per_second(profile: RecordingProfile) -> u64 {
    let Resolution { width, height } = profile.resolution;
    u64::from(width) * u64::from(height) * u64::from(profile.frames_per_second)
}
