#![warn(clippy::pedantic)]

/// # Panics
/// Panics if `right` is not an inner slice of `whole`
#[must_use]
pub fn offset(whole: &str, right: &str) -> usize {
    let diff = (right.as_ptr() as usize)
        .checked_sub(whole.as_ptr() as usize)
        .expect("slice not in whole");
    debug_assert!(diff < whole.len());
    diff
}

#[must_use]
pub fn whole_line<'a>(whole: &'a str, inner: &'a str) -> &'a str {
    let offset = offset(whole, inner);
    let previous_last_line = whole[..offset]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or_default();
    dbg!(&whole[previous_last_line..])
        .lines()
        .next()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_test() {
        let total = "Hiya, test";
        let slice = &total[4..];
        assert_eq!(offset(total, slice), 4);
    }

    #[test]
    fn whole_line_test() {
        let total = "Start\nThis is a test\n something";
        let offset = total.find("test").unwrap();
        let slice = &total[offset..(offset + 4)];
        assert_eq!(slice, "test");
        assert_eq!(whole_line(total, slice), "This is a test");
    }
}
