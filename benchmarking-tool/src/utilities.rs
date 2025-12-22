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

pub struct PairedWriter<W1, W2>(Option<W1>, Option<W2>);

impl<W1, W2> PairedWriter<W1, W2> {
    pub fn new_from_option(first: Option<W1>, second: Option<W2>) -> Option<Self> {
        if first.is_some() || second.is_some() {
            Some(Self(first, second))
        } else {
            None
        }
    }
}

impl<W1, W2> std::io::Write for PairedWriter<W1, W2>
where
    W1: std::io::Write,
    W2: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(first) = self.0.as_mut() {
            let result = std::io::Write::write(first, buf)?;
            if let Some(second) = self.1.as_mut() {
                std::io::Write::write(second, buf)
            } else {
                Ok(result)
            }
        } else {
            std::io::Write::write(self.1.as_mut().unwrap(), buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(first) = self.0.as_mut() {
            std::io::Write::flush(first)?;
        }
        if let Some(second) = self.1.as_mut() {
            std::io::Write::flush(second)?;
        }
        Ok(())
    }
}
