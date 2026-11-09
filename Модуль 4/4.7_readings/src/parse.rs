#[derive(Debug, PartialEq, Eq)]
pub enum SummaryError {
    EmptyInput,
    InvalidNumber { position: usize, text: String },
}

pub(super) fn read(rows: &[String]) -> Result<Vec<i32>, SummaryError> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            row.parse().map_err(|_| SummaryError::InvalidNumber {
                position: index + 1,
                text: row.clone(),
            })
        })
        .collect()
}
