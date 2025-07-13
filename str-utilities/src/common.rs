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
pub fn find_after<'a>(on: &'a str, hmm: ()) -> &'a str {
    todo!()
}

#[must_use]
pub fn try_slice<'a>(on: &'a str, hmm: ()) -> &'a str {
    todo!("on.get(hmm).unwrap_or_default()")
}

pub fn is_whitespace(on: &str) -> bool {
    on.trim_start().is_empty()
}

pub fn count_leading_whitespace(on: &str) -> u8 {
    todo!()
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
