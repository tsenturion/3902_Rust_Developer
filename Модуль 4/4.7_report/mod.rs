mod input;
mod model;
mod totals;

// Типы доступны по пути report::Summary и report::ReportError.
pub use self::model::{ReportError, Summary};

pub fn build(rows: &[String]) -> Result<Summary, ReportError> {
    let entries = input::parse(rows)?;
    Ok(totals::summarize(&entries))
}
