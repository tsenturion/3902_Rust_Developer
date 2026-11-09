/*
cargo run --offline --bin dyn_trait_any
*/

/*
Терминал склада опрашивает датчики температуры и открытия двери через
общий интерфейс. При обслуживании оператор может задать поправку только
для датчика температуры. Терминал должен проверить тип датчика и принять
поправку от −50 до 50 десятых градуса включительно. При отказе прежние
показания и настройки сохраняются.
*/

use std::any::Any;

#[derive(Debug, PartialEq)]
enum Reading {
    Temperature { tenths: i32 },
    Door { open: bool },
}

// Супертрейт Any позволяет использовать объект Sensor и через интерфейс Any.
trait Sensor: Any {
    fn name(&self) -> &str;
    fn read(&self) -> Reading;
}

struct TemperatureSensor {
    name: String,
    raw_tenths: i16,
    offset_tenths: i16,
}

impl Sensor for TemperatureSensor {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn read(&self) -> Reading {
        Reading::Temperature {
            tenths: i32::from(self.raw_tenths) + i32::from(self.offset_tenths),
        }
    }
}

struct DoorSensor {
    name: String,
    open: bool,
}

impl Sensor for DoorSensor {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn read(&self) -> Reading {
        Reading::Door { open: self.open }
    }
}

fn read_all(sensors: &[&dyn Sensor]) {
    for &sensor in sensors {
        // Обычному опросу достаточно методов Sensor, общих для всех датчиков.
        match sensor.read() {
            Reading::Temperature { tenths } => {
                println!("{} — температура {tenths} десятых °C", sensor.name());
            }
            Reading::Door { open } => {
                let state = if open {
                    "открыта"
                } else {
                    "закрыта"
                };
                println!("{} — дверь {state}", sensor.name());
            }
        }
    }
}

fn current_offset(sensor: &dyn Sensor) -> Option<i16> {
    // Меняем доступный интерфейс ссылки. Объект остаётся на месте.
    let value: &dyn Any = sensor;
    // downcast_ref возвращает Some только для точного конкретного типа.
    let temperature = value.downcast_ref::<TemperatureSensor>()?;
    Some(temperature.offset_tenths)
}

#[derive(Debug, PartialEq)]
enum CalibrationError {
    UnsupportedSensor,
    OffsetOutOfRange { tenths: i16 },
}

fn calibrate(sensor: &mut dyn Sensor, tenths: i16) -> Result<(), CalibrationError> {
    let value: &mut dyn Any = sensor;
    let temperature = value
        .downcast_mut::<TemperatureSensor>()
        .ok_or(CalibrationError::UnsupportedSensor)?;

    if !(-50..=50).contains(&tenths) {
        return Err(CalibrationError::OffsetOutOfRange { tenths });
    }
    temperature.offset_tenths = tenths;
    Ok(())
}

fn main() {
    let mut temperature = TemperatureSensor {
        name: String::from("Камера 2"),
        raw_tenths: 73,
        offset_tenths: 0,
    };
    let mut door = DoorSensor {
        name: String::from("Шлюз 7"),
        open: true,
    };

    {
        let sensors: [&dyn Sensor; 2] = [&temperature, &door];
        assert_eq!(sensors[0].read(), Reading::Temperature { tenths: 73 });
        assert_eq!(sensors[1].read(), Reading::Door { open: true });

        let value: &dyn Any = sensors[0];
        assert!(value.is::<TemperatureSensor>());
        assert!(!value.is::<DoorSensor>());
        assert!(value.downcast_ref::<DoorSensor>().is_none());
        assert_eq!(current_offset(sensors[0]), Some(0));
        assert_eq!(current_offset(sensors[1]), None);

        // Интерфейс Any даёт проверку типа. Метод name объявлен в Sensor.
        // let _ = value.name();

        // Чтобы проверить конкретный тип, используем downcast_ref.
        // let _ = sensors[0] as &TemperatureSensor;
    }

    let target: &mut dyn Sensor = &mut temperature;
    assert_eq!(calibrate(target, -50), Ok(()));
    assert_eq!(current_offset(target), Some(-50));
    assert_eq!(calibrate(target, 50), Ok(()));
    assert_eq!(current_offset(target), Some(50));

    let correction = -4;
    assert_eq!(calibrate(target, correction), Ok(()));
    assert_eq!(
        calibrate(target, 51),
        Err(CalibrationError::OffsetOutOfRange { tenths: 51 })
    );
    assert_eq!(
        calibrate(target, -51),
        Err(CalibrationError::OffsetOutOfRange { tenths: -51 })
    );
    assert_eq!(target.read(), Reading::Temperature { tenths: 69 });
    assert_eq!(current_offset(target), Some(correction));

    assert_eq!(
        calibrate(&mut door, -4),
        Err(CalibrationError::UnsupportedSensor)
    );
    assert_eq!(door.read(), Reading::Door { open: true });

    println!("Применена поправка {correction} десятых °C");
    read_all(&[&temperature, &door]);

    // Any требует 'static у конкретного типа. Сама ссылка &dyn Any может заимствовать локальный объект.
    // let _borrowed: &dyn Any = &temperature.name();
}
