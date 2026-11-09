use std::collections::BTreeMap;

// Записи нужны только внутри report и его дочерних модулей.
pub(super) struct Entry {
    pub(super) dock: u8,
    pub(super) minutes: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub events: usize,
    pub total_minutes: u128,
    pub by_dock: BTreeMap<u8, u128>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReportError {
    EmptyInput,
    InvalidRecord { position: usize, text: String },
}
