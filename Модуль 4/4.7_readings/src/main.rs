use readings::{SummaryError, summarize};
use std::process::ExitCode;

fn main() -> ExitCode {
    let rows: Vec<_> = std::env::args().skip(1).collect();
    match summarize(&rows) {
        Ok(summary) => {
            println!(
                "count={} min={} max={}",
                summary.count, summary.min, summary.max
            );
            ExitCode::SUCCESS
        }
        Err(SummaryError::EmptyInput) => {
            eprintln!("Передайте хотя бы одно целое показание");
            ExitCode::FAILURE
        }
        Err(SummaryError::InvalidNumber { position, text }) => {
            eprintln!("Показание {position} «{text}» не является целым числом в диапазоне i32");
            ExitCode::FAILURE
        }
    }
}
