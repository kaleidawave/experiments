#![allow(unused)]

#[must_use]
pub fn trim_if(on: &str, pattern: char) -> &str {
    todo!()
}

#[must_use]
pub fn three_way_split<'a>(on: &'a str, (left, right): (&str, &str)) -> &'a str {
    todo!()
}

// TODO pattern API
#[must_use]
pub fn find_after(on: &'_ str, hmm: ()) -> &'_ str {
    todo!()
}

#[must_use]
pub fn try_slice(on: &'_ str, hmm: ()) -> &'_ str {
    todo!("on.get(hmm).unwrap_or_default()")
}

pub fn is_whitespace(on: &str) -> bool {
    on.trim_start().is_empty()
}

pub fn count_leading_whitespace(on: &str) -> u8 {
    if on.starts_with("\r\n") {
        2
    } else if on.starts_with("\n") {
        1
    } else {
        0
    }
}

pub fn starts_with_new_line_sequence(on: &str) -> bool {
    on.starts_with("\r\n") || on.starts_with("\n")
}

pub fn find_new_line_sequence(on: &str) -> Option<(usize, usize)> {
    let (idx, matched) = on.match_indices(['\r', '\n']).next()?;

    // TODO does this check need to be done?
    if matched == "\r" && on[idx..].starts_with("\r\n") {
        Some((idx, 2))
    } else {
        Some((idx, 1))
    }
}

pub fn get_whitespace_prefix(on: &str) -> &str {
    &on[..(on.len() - on.trim_start().len())]
}

pub fn split_at_exclusive(on: &str, at: usize) -> (&str, &str) {
    let (lhs, rhs) = on.split_at(at);
    if let Some(chr) = rhs.chars().next() {
        (lhs, &rhs[chr.len_utf8()..])
    } else {
        (lhs, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix() {
        assert_eq!(get_whitespace_prefix("  \t  Hi"), "  \t  ");
    }

    #[test]
    fn new_line() {
        assert_eq!(find_new_line_sequence("  Hi\nTest"), Some((4, 1)));
    }

    #[test]
    fn split_at() {
        let on = "Hello World";
        let at = "Hello".len();
        assert_eq!(str::split_at(on, at), ("Hello", " World"));
        assert_eq!(split_at_exclusive(on, at), ("Hello", "World"));
    }
}
