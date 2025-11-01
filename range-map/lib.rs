extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::Range;

#[derive(Debug)]
pub struct RangeMap<T> {
    entries: Vec<(Range<u32>, T)>,
}

impl<T> Default for RangeMap<T> {
    fn default() -> Self {
        Self {
            entries: Vec::default(),
        }
    }
}

impl<T> RangeMap<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::default(),
        }
    }

    pub fn push(&mut self, range: impl Into<Range<u32>>, item: T) {
        self.entries.push((range.into(), item));
    }

    /// Get the top level entry at some point
    #[must_use]
    pub fn get(&self, point: u32) -> Option<&T> {
        fn get_naive<T>(on: &[(Range<u32>, T)], at: u32) -> Option<&T> {
            for (range, value) in on {
                if range.contains(&at) {
                    return Some(value);
                }
            }
            None
        }

        get_naive(&self.entries, point)
    }
}

#[cfg(test)]
mod tests {
    use super::RangeMap;

    fn generate() -> RangeMap<u8> {
        let range: &[(u32, u32, u8)] = &[
            (6, 9, 1),
            (4, 9, 2),
            (15, 18, 3),
            (14, 19, 4),
            (24, 27, 5),
            (23, 28, 6),
            (13, 29, 7),
            (31, 39, 8),
            (41, 44, 9),
            (30, 45, 10),
            (46, 50, 11),
            (45, 54, 12),
            (58, 62, 13),
            (0, 68, 14),
        ];
        let mut map = RangeMap::new();
        for (a, b, value) in range {
            map.push(*a..*b, *value);
        }
        map
    }

    #[test]
    fn get() {
        let map = generate();
        assert_eq!(map.get(7).copied(), Some(1));
    }
}
