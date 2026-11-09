/*
cargo run --offline --bin procedural_macros
*/

use course_macros::{EventCode, check_result, checkpoint_mask};

trait EventCode {
    const CODE: u16;
}

#[derive(EventCode)]
#[event_code = 501]
struct TransferAccepted {
    reference: u32,
}

macro_rules! rejection_event {
    ($name:ident, $code:literal, $variant:ident) => {
        #[derive(EventCode)]
        #[event_code = $code]
        enum $name {
            $variant,
        }
    };
}

rejection_event!(TransferRejected, 502, MissingCheckpoint);

const REQUIRED_CHECKPOINTS: u8 = checkpoint_mask!(0, 2, 5,);

#[check_result]
fn all_checkpoints_visited(visited: u8) -> bool {
    visited & REQUIRED_CHECKPOINTS == REQUIRED_CHECKPOINTS
}

#[check_result("Обработайте отказ в перемещении")]
fn accept(reference: u32, visited: u8) -> Result<TransferAccepted, TransferRejected> {
    if all_checkpoints_visited(visited) {
        Ok(TransferAccepted { reference })
    } else {
        Err(TransferRejected::MissingCheckpoint)
    }
}

fn main() {
    let visited = checkpoint_mask!(0, 2, 5, 6);
    let incomplete = checkpoint_mask!(0, 5);
    assert_eq!(REQUIRED_CHECKPOINTS, 37);
    assert_eq!(checkpoint_mask!(), 0);
    assert!(all_checkpoints_visited(visited));
    assert!(!all_checkpoints_visited(incomplete));
    println!(
        "Маски контроля — обязательные: {REQUIRED_CHECKPOINTS:08b}, пройденные: {visited:08b}"
    );

    for (reference, mask) in [(781, visited), (782, incomplete)] {
        match accept(reference, mask) {
            Ok(event) => println!(
                "Перемещение {} — принято, событие {}",
                event.reference,
                <TransferAccepted as EventCode>::CODE
            ),
            Err(TransferRejected::MissingCheckpoint) => println!(
                "Перемещение {reference} — отказ, событие {}",
                <TransferRejected as EventCode>::CODE
            ),
        }
    }
}
