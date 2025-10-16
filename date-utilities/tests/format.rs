use date_utilities::{FORMATS::FULL_MINIMAL, Instant};

#[test]
fn format() {
    let date = Instant::new(2025, 1, 1, 11, 0, 0);
    assert_eq!(date.format(FULL_MINIMAL), "12:00:00 01/01/2025");
}
