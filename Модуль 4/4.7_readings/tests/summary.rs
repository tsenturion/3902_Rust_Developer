use readings::{Summary, SummaryError, summarize};

#[test]
fn summarizes_several_readings() {
    assert_eq!(
        summarize(&["8", "-2", "5"].map(String::from)),
        Ok(Summary {
            count: 3,
            min: -2,
            max: 8
        })
    );
}

#[test]
fn uses_a_single_reading_for_both_bounds() {
    assert_eq!(
        summarize(&[String::from("-2147483648")]),
        Ok(Summary {
            count: 1,
            min: i32::MIN,
            max: i32::MIN
        })
    );
}

#[test]
fn rejects_empty_input() {
    assert_eq!(summarize(&[]), Err(SummaryError::EmptyInput));
}

#[test]
fn reports_the_first_invalid_reading() {
    assert_eq!(
        summarize(&["8", "x", "2147483648"].map(String::from)),
        Err(SummaryError::InvalidNumber {
            position: 2,
            text: String::from("x")
        })
    );
}
