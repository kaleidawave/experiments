use std::ops::Range;

pub type SliceRange = Range<usize>;
pub type ChangesSlice<'a> = &'a [(SliceRange, String)];
pub type ChangesVec = Vec<(SliceRange, String)>;

pub fn apply_changes<W: std::io::Write>(to: &mut W, on: &str, changes: ChangesSlice) {
    let mut cur = 0;
    for (range, item) in changes {
        let (lhs, rhs) = (range.start, range.end);
        debug_assert!(cur <= lhs, "cur > lhs");
        write!(to, "{item}", item = &on[cur..lhs]).unwrap();
        write!(to, "{item}").unwrap();
        cur = rhs;
    }
    write!(to, "{item}", item = &on[cur..]).unwrap();
}

pub fn apply_changes_to_string(on: &str, changes: ChangesSlice) -> String {
    let mut buf = Vec::new();
    apply_changes(&mut buf, on, changes);
    // TODO on debug check
    unsafe { String::from_utf8_unchecked(buf) }
}

pub fn invert_changes(on: &str, changes: ChangesSlice) -> ChangesVec {
    let mut inverted = Vec::new();
    for (range, item) in changes {
        let (lhs, rhs) = (range.start, range.end);
        let existing = &on[lhs..rhs];
        inverted.push((lhs..(lhs + item.len()), existing.to_owned()));
    }
    inverted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes() {
        let input = "Hello Ben! This is a test";
        let range = 6..9;
        assert_eq!(&input[range.clone()], "Ben");

        let changes = vec![(range, "World".to_owned())];
        let out = apply_changes_to_string(input, &changes);
        assert_eq!(out, "Hello World! This is a test");
    }

    #[test]
    fn invert() {
        let input = "Hello Ben! This is a test";
        let range = 6..9;

        let changes = vec![(range, "World".to_owned())];
        let inverted = invert_changes(input, &changes);
        assert_eq!(inverted, &[(6..11, "Ben".to_owned())]);
    }
}
