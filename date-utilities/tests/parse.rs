use date_utilities::{FORMATS::FULL_MINIMAL, Instant};

#[test]
fn parse() {
    let date = Instant::parse_english("22th January 2025").unwrap();
    assert_eq!(date.format(FULL_MINIMAL), "13:00:00 22/01/2025");
}
