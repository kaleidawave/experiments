use std::borrow::Cow;

pub struct ArgumentIter<'a> {
    on: &'a str,
    last: usize,
}

impl<'a> ArgumentIter<'a> {
    pub fn new(on: &'a str) -> Self {
        Self {
            on: on.trim(),
            last: 0,
        }
    }
}

impl<'a> Iterator for ArgumentIter<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.last;
        if let Some((idx, matched)) = self.on[self.last..]
            .match_indices(&[' ', '\'', '"', ',', '\n'])
            .next()
        {
            let value = match matched {
                " " => {
                    let end = self.last + idx;
                    self.last += idx + matched.len();
                    Some(Cow::Borrowed(self.on[start..end].trim()))
                }
                // TODO wip
                "," | "\n" => {
                    if idx == 0 {
                        self.last += idx + 1;
                        Some(Cow::Borrowed(","))
                    } else {
                        let end = self.last + idx;
                        self.last += idx;
                        Some(Cow::Borrowed(self.on[start..end].trim()))
                    }
                }
                "\"" | "\'" => {
                    let rest = &self.on[self.last..][1..];
                    let (idx2, _) = rest
                        .match_indices(matched)
                        .filter(|(idx, _)| !rest[..*idx].ends_with('\\'))
                        .next()
                        .expect("no end to quoted item");

                    self.last += idx + idx2 + 2;
                    let content = &rest[..idx2];
                    if content.contains('\\') {
                        Some(Cow::Owned(content.replace('\\', "")))
                    } else {
                        Some(Cow::Borrowed(content))
                    }
                }
                item => unreachable!("{item}"),
            };
            if let Some(rest) = self.on.get(self.last..) {
                let spaces = rest
                    .find(|chr: char| matches!(chr, ' ' | '\t' | '\r'))
                    .unwrap_or_default();
                self.last += spaces;
            }
            value
        } else if start < self.on.len() {
            self.last = self.on.len();
            Some(Cow::Borrowed(&self.on[start..]))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Sorting<'a> {
    pub field: &'a str,
    pub direction: Direction,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Direction {
    Ascending,
    Descending,
}

impl Direction {
    pub fn compare<T: std::cmp::Ord>(self, a: &T, b: &T) -> std::cmp::Ordering {
        let order = a.cmp(b);
        if let Self::Ascending = self {
            order.reverse()
        } else {
            order
        }
    }
}

pub(crate) fn to_denary(value: usize, seperator: &str) -> String {
    if value == 0 {
        return "0".to_owned();
    }
    let mut buf = String::new();
    for i in (0..=value.ilog10()).rev() {
        let j = (value / 10i32.pow(i) as usize) % 10;
        buf.push(b"0123456789"[j] as char);
        if i > 0 && i % 3 == 0 {
            buf.push_str(seperator);
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let on = "this is a test! 'with' \"things in quotes\" see";
        assert_eq!(
            ArgumentIter::new(on).collect::<Vec<_>>(),
            vec![
                "this",
                "is",
                "a",
                "test!",
                "with",
                "things in quotes",
                "see"
            ]
        );
    }

    #[test]
    fn escaping() {
        let on = "testing 'escaping \\'' \"with \\\" quote\"";
        assert_eq!(
            ArgumentIter::new(on).collect::<Vec<_>>(),
            vec!["testing", "escaping '", "with \" quote"]
        );
    }
}

pub struct Permutations<T> {
    choices: Vec<Vec<T>>,
    state: Vec<usize>,
}

impl<T> Permutations<T> {
    pub fn new(choices: Vec<Vec<T>>) -> Self {
        let state = vec![0; choices.len()];
        Self { choices, state }
    }
}

impl<T> Iterator for Permutations<T>
where
    T: Clone,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.len() == 0 {
            return None;
        }
        let last = self.state.len() - 1;
        if self.state[last] == self.choices[last].len() {
            return None;
        }
        let mut items: Vec<T> = Vec::with_capacity(self.state.len());
        for (i, choices_i) in self.choices.iter().enumerate() {
            let item = choices_i[self.state[i]].clone();
            items.push(item);
        }
        for (i, counter) in self.state.iter_mut().enumerate() {
            if i == last || *counter + 1 < self.choices[i].len() {
                *counter += 1;
                break;
            } else {
                *counter = 0;
            }
        }
        Some(items)
    }
}

pub struct List<'a, T> {
    items: &'a [T],
    prefix: &'a str,
}

impl<'a, T> List<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self { items, prefix: "" }
    }

    pub fn with_prefix(&mut self, prefix: &'a str) -> &mut Self {
        self.prefix = prefix;
        self
    }
}

impl<'a, T> std::fmt::Display for List<'a, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if self.items.len() > 0 {
            write!(f, "{prefix}", prefix = self.prefix)?;
        }
        for value in self.items.iter() {
            write!(f, " {value}")?;
        }
        Ok(())
    }
}
