pub fn match_indicies_after<'a>(
    on: &'a str,
    pattern: &str,
    after: usize,
) -> impl Iterator<Item = (usize, &'a str)> {
    on.get(after..)
        .unwrap_or_default()
        .match_indices(pattern)
        .map(move |(idx, matched)| (idx + after, matched))
}

pub struct EdibleLines<'a> {
    start: usize,
    last: usize,
    on: &'a str,
}

impl<'a> EdibleLines<'a> {
    pub fn new(on: &'a str) -> Self {
        EdibleLines {
            on,
            start: 0,
            last: 0,
        }
    }

    pub fn moving_on(&mut self) {
        self.start = self.last;
    }

    pub fn peek_line(&self) -> Option<&'a str> {
        self.on
            .get(self.last..)
            .map(|rest| rest.lines().next().unwrap_or(rest))
    }

    pub fn on(&self) -> &'a str {
        self.on
    }

    pub fn is_at_start(&self) -> bool {
        self.start == 0
    }

    pub fn skip_next(&mut self) {
        if let Some((idx, len)) = super::common::find_new_line_sequence(&self.on[self.last..]) {
            self.last += idx + len;
        } else {
            self.last = self.on.len();
        }
    }
}

impl<'a> Iterator for EdibleLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((idx, len)) = super::common::find_new_line_sequence(&self.on[self.last..]) {
            let slice = &self.on[self.start..(self.last + idx)];
            self.last += idx + len;
            Some(slice)
        } else if self.last < self.on.len() {
            self.last = self.on.len();
            Some(&self.on[self.start..])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod edible_lines {
    use super::*;

    #[test]
    fn successive_lines() {
        let lines = "line one
line two
line three";
        let iter = EdibleLines::new(lines);
        assert_eq!(
            iter.collect::<Vec<_>>(),
            vec![
                "line one",
                "line one\nline two",
                "line one\nline two\nline three"
            ]
        )
    }

    #[test]
    fn successive_lines_with_moving_on() {
        let lines = "line one
line two
line three";
        let mut iter = EdibleLines::new(lines);
        assert_eq!(iter.next(), Some("line one"));
        assert_eq!(iter.next(), Some("line one\nline two"));
        iter.moving_on();
        assert_eq!(iter.next(), Some("line three"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn peek_line() {
        let lines = "line one
line two
line three";
        let mut iter = EdibleLines::new(lines);
        let _ = iter.next();
        assert_eq!(iter.peek_line(), Some("line two"));
    }

    #[test]
    fn edge_cases() {
        let empty_iter = EdibleLines::new("");
        assert!(empty_iter.collect::<Vec<_>>().is_empty());

        let single_line = EdibleLines::new("\n");
        assert_eq!(single_line.collect::<Vec<_>>(), vec![""]);

        let carriage_return_line = EdibleLines::new("\r\n");
        assert_eq!(carriage_return_line.collect::<Vec<_>>(), vec![""]);
    }
}

pub struct LinesWithContinuation<'a> {
    matches: &'static [&'static str],
    on: &'a str,
    last: usize,
}

impl<'a> LinesWithContinuation<'a> {
    pub fn new(on: &'a str, matches: &'static [&'static str]) -> Self {
        LinesWithContinuation {
            on,
            matches,
            last: 0,
        }
    }
}

impl<'a> Iterator for LinesWithContinuation<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.last;
        while let Some((idx, seq)) = super::common::find_new_line_sequence(&self.on[self.last..]) {
            self.last += idx + seq;
            let last = self.on[..self.last].trim_end();
            if self.matches.iter().any(|matcher| last.ends_with(matcher)) {
                continue;
            } else {
                return Some(self.on[start..self.last].trim_end());
            }
        }
        if start < self.on.len() {
            self.last = self.on.len();
            Some(self.on[start..].trim_end())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod lines_with_continuation {
    use super::*;

    #[test]
    fn basic() {
        let on = "
hello
world
on two \\
lines";
        assert_eq!(
            LinesWithContinuation::new(on, &["\\"]).collect::<Vec<_>>(),
            vec!["", "hello", "world", "on two \\\nlines"]
        );
    }
}

pub struct Words<'a> {
    on: &'a str,
    last: usize,
}

impl<'a> Words<'a> {
    pub fn new(on: &'a str) -> Self {
        Self { on, last: 0 }
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.last;
        if let Some((idx, matched)) = self.on[self.last..]
            .match_indices(&[' ', '\n', '.', '?', '!', ','])
            .next()
        {
            let end = self.last + idx;
            self.last += idx + matched.len();
            Some(self.on[start..end].trim())
        } else if start < self.on.len() {
            self.last = self.on.len();
            Some(&self.on[start..])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod words {
    use super::*;

    #[test]
    fn basic() {
        let on = "this is a test!";
        assert_eq!(
            Words::new(on).collect::<Vec<_>>(),
            vec!["this", "is", "a", "test"]
        );
    }
}
