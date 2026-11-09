mod parse;
mod stats;

pub use self::parse::SummaryError;
pub use self::stats::Summary;

pub fn summarize(rows: &[String]) -> Result<Summary, SummaryError> {
    let numbers = parse::read(rows)?;
    stats::calculate(&numbers).ok_or(SummaryError::EmptyInput)
}
