// super ведёт к report. crate ведёт к корню этого исполняемого крейта.
use super::model::{Entry, ReportError};
use crate::MAX_DOCK;

fn parse_row(text: &str) -> Option<Entry> {
    let (dock, minutes) = text.split_once(':')?;
    let dock = dock.parse().ok()?;
    let minutes = minutes.parse().ok()?;
    if (1..=MAX_DOCK).contains(&dock) && minutes > 0 {
        Some(Entry { dock, minutes })
    } else {
        None
    }
}

pub(super) fn parse(rows: &[String]) -> Result<Vec<Entry>, ReportError> {
    if rows.is_empty() {
        return Err(ReportError::EmptyInput);
    }
    rows.iter()
        .enumerate()
        .map(|(index, text)| {
            parse_row(text).ok_or_else(|| ReportError::InvalidRecord {
                position: index + 1,
                text: text.clone(),
            })
        })
        .collect()
}
