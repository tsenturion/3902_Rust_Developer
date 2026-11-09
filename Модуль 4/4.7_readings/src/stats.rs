#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub count: usize,
    pub min: i32,
    pub max: i32,
}

pub(super) fn calculate(numbers: &[i32]) -> Option<Summary> {
    let (&first, rest) = numbers.split_first()?;
    let (min, max) = rest.iter().fold((first, first), |(min, max), &number| {
        (min.min(number), max.max(number))
    });
    Some(Summary {
        count: numbers.len(),
        min,
        max,
    })
}
