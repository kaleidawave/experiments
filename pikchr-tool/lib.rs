#[derive(Debug)]
pub enum Shape {
    Box,
    Circle,
    Ellipse,
    Oval,
    Cylinder,
    File,
    Diamond,
}

#[derive(Debug)]
pub enum Connection {
    Line,
    Arrow,
    Spline,
    Arc,
}

#[derive(Debug)]
pub enum Polarity {
	Right
}

#[derive(Debug)]
pub enum Relative {
	Previous
}

struct Size {
	fit: bool
}

#[derive(Debug)]
pub enum NorthSouth {
	North,
	South
}

#[derive(Debug)]
pub enum EastWest {
	East,
	West
}

#[derive(Debug)]
struct RelativeLocation {
	vertical: Option<NorthSouth>,
	horizontal: Option<EastWest>,
}

#[derive(Debug)]
pub enum Direction {
	Down,
	Right,
	Left,
	Up
}

#[derive(Debug)]
pub enum Colour {
	Invisible
}

#[derive(Debug)]
pub enum Command<'a> {
    Label(&'a str),
    Shape {
        shape: Shape,
        text: Option<&'a str>,
        /// TODO at
        after: &'a str,
    },
    Connection {
        connection: Connection,
        from: Option<&'a str>,
    },
    Dot(&'a str),
    Text(&'a str),
    Other {
        kw: &'a str,
        after: &'a str,
    },
	Direction(Direction),
	Move,
}

pub mod utilities {
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

    pub fn find_new_line_sequence(on: &str) -> Option<(usize, usize)> {
        let (idx, matched) = on.match_indices(['\r', '\n']).next()?;

        // TODO does this check need to be done?
        if matched == "\r" && on[idx..].starts_with("\r\n") {
            Some((idx, 2))
        } else {
            Some((idx, 1))
        }
    }

    impl<'a> Iterator for LinesWithContinuation<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<Self::Item> {
            let start = self.last;
            while let Some((idx, seq)) = find_new_line_sequence(&self.on[self.last..]) {
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
}
