use super::model::{Entry, Summary};
use std::collections::BTreeMap;

pub(super) fn summarize(entries: &[Entry]) -> Summary {
    let mut total_minutes = 0;
    let mut by_dock = BTreeMap::new();
    for entry in entries {
        let minutes = u128::from(entry.minutes);
        total_minutes += minutes;
        *by_dock.entry(entry.dock).or_insert(0) += minutes;
    }
    Summary {
        events: entries.len(),
        total_minutes,
        by_dock,
    }
}
