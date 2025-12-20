#[derive(Clone, Debug)]
pub struct Sorting {
    pub field: String,
    pub direction: Direction,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
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
        if self.state.is_empty() {
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
        if !self.items.is_empty() {
            write!(f, "{prefix}", prefix = self.prefix)?;
        }
        for value in self.items.iter() {
            write!(f, " {value}")?;
        }
        Ok(())
    }
}

/// Used for printing numbers in SDE
pub fn count_with_seperator(value: usize) -> String {
    const NON_BREAKING_SPACE: &str = "\u{00A0}";

    to_denary(value, NON_BREAKING_SPACE)
}

fn to_denary(value: usize, seperator: &str) -> String {
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
