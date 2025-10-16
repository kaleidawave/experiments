use date_utilities::Instant;

#[test]
fn format() {
    let date = Instant::new(2025, 1, 1, 11, 0, 0);
    let now = Instant::new(2025, 8, 31, 18, 25, 0);
    let Ok(difference) = now.difference(date) else {
        panic!();
    };
    assert_eq!(difference.format(), "34 weeks ago");
}
