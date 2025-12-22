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
