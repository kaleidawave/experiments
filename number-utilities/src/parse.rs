pub fn parse_binary(on: &str) -> usize {
    let mut value = 0usize;
    for byte in on.bytes() {
        value <<= 1;
        if let b'0' | b'1' = byte {
            value |= (byte == b'1') as usize;
        } else {
            panic!("bad char {byte:?}!");
        }
    }
    value
}

pub fn parse_base10(on: &str) -> usize {
    let mut value = 0usize;
    for byte in on.bytes() {
        value *= 10;
        let code = if let b'0'..=b'9' = byte {
            usize::from(byte - b'0')
        } else {
            panic!("bad char {byte:?}!");
        };
        value += code;
    }
    value
}

pub fn parse_hex(on: &str) -> usize {
    let mut value = 0usize;
    for byte in on.bytes() {
        value <<= 4; // log2(16) = 4
        let code = match byte {
            b'0'..=b'9' => usize::from(byte - b'0'),
            b'a'..=b'f' => usize::from(byte - b'a') + 10,
            // also include uppercase
            b'A'..=b'F' => usize::from(byte - b'A') + 10,
            byte => {
                panic!("bad char {byte:?}!");
            }
        };
        value |= code;
    }
    value
}

pub fn parse_english(on: &str) -> usize {
    fn parse_item(item: &str) -> Result<usize, &str> {
        let prefix_magnitude = item
            .rsplit_once(" ")
            .and_then(|(lhs, rhs)| parse_magnitude(rhs).map(|rhs| (lhs, rhs)));

        if let Some((left, right)) = item.rsplit_once(" and ")
            && (left.contains(" and ") || prefix_magnitude.is_none())
        {
            let left = parse_item(left)?;
            let right = parse_item(right)?;
            Ok(left + right)
        } else if let Some((prefix, magnitude)) = prefix_magnitude {
            let prefix = if let Some((left, right)) = prefix.split_once(" and ") {
                if left.ends_with(" hundred") {
                    let left = parse_possible_pair(left)?;
                    let right = parse_possible_pair(right)?;
                    left + right
                } else {
                    let left = parse_item(left)?;
                    let right = parse_item(right)?;
                    let value = left + right * magnitude;
                    return Ok(value);
                }
            } else {
                parse_possible_pair(prefix)?
            };
            Ok(prefix * magnitude)
        } else {
            parse_possible_pair(item)
        }
    }

    fn parse_magnitude(item: &str) -> Option<usize> {
        match item {
            "quadrillion" => Some(1_000_000_000_000_000),
            "trillion" => Some(1_000_000_000_000),
            "billion" => Some(1_000_000_000),
            "million" => Some(1_000_000),
            "thousand" => Some(1_000),
            "hundred" => Some(100),
            _ => None,
        }
    }

    fn parse_possible_pair(item: &str) -> Result<usize, &str> {
        if let Some((left, right)) = item.rsplit_once(" ") {
            parse_pair(left, right)
        } else {
            under_one_hundred_to_usize(item)
        }
    }

    fn parse_pair<'a>(left: &'a str, right: &'a str) -> Result<usize, &'a str> {
        let left = under_one_hundred_to_usize(left)?;
        if let Some(magnitude) = parse_magnitude(right) {
            Ok(left * magnitude)
        } else {
            let right = under_one_hundred_to_usize(right)?;
            Ok(left + right)
        }
    }

    fn under_one_hundred_to_usize(item: &str) -> Result<usize, &str> {
        Ok(match item {
            "zero" => 0,
            "one" => 1,
            "two" => 2,
            "three" => 3,
            "four" => 4,
            "five" => 5,
            "six" => 6,
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            "ten" => 10,
            "eleven" => 11,
            "twelve" => 12,
            "thirteen" => 13,
            "fourteen" => 14,
            "fifthteen" => 15,
            "sixteen" => 16,
            "seventeen" => 17,
            "eighteen" => 18,
            "nineteen" => 19,
            "twenty" => 20,
            "thirty" => 30,
            "fourty" => 40,
            "fifty" => 50,
            "sixty" => 60,
            "seventy" => 70,
            "eighty" => 80,
            "ninety" => 90,
            number => {
                dbg!(number);
                return Err(number);
            }
        })
    }

    // mod utilities {
    //     pub struct ListIterator<'a> {
    //         on: &'a str,
    //         last_and: usize,
    //         idx: usize,
    //     }

    //     impl<'a> ListIterator<'a> {
    //         pub fn new(on: &'a str) -> Self {
    //             let mut last_and = on.len();
    //             // TODO faster?, skip parenthesises etc, account for whitespace
    //             for (idx, _) in on.char_indices().rev() {
    //                 let after = &on[idx..];
    //                 if after.starts_with(",") {
    //                     break;
    //                 } else if after.starts_with(" and ") {
    //                     last_and = idx + 1;
    //                     break;
    //                 }
    //             }
    //             Self {
    //                 on,
    //                 // last_and,
    //                 idx: 0,
    //             }
    //         }
    //     }

    //     impl<'a> Iterator for ListIterator<'a> {
    //         type Item = &'a str;

    //         fn next(&mut self) -> Option<Self::Item> {
    //             let start = self.idx;
    //             if start < self.on.len() {
    //                 let items = self.on[self.idx..].match_indices(&[',', '&', '(', 'a']);
    //                 for (i, matched) in items {
    //                     match matched {
    //                         "," => {
    //                             self.idx += i + 1;
    //                             return Some(self.on[start..][..i].trim());
    //                         }
    //                         "(" => todo!(),
    //                         "a" => {
    //                             if self.idx + i == self.last_and {
    //                                 self.idx += i + 4;
    //                                 return Some(self.on[start..][..i].trim());
    //                             }
    //                         }
    //                         item => unreachable!("{item:?}"),
    //                     }
    //                 }
    //                 self.idx = self.on.len();
    //                 Some(self.on[start..].trim())
    //             } else {
    //                 None
    //             }
    //         }
    //     }
    // }

    let mut acc: usize = 0;
    for block in on.split(',') {
        let block = block.trim();
        let Ok(item) = parse_item(block) else {
            panic!("cannot parse {block}");
        };
        acc += item;
    }
    acc
}

pub fn parse_roman_numeral(on: &str) -> usize {
    let mut acc = 0;
    for chr in on.chars() {
        let value = match chr.to_ascii_uppercase() {
            'M' => 1000,
            'D' => 500,
            'C' => 100,
            'L' => 50,
            'X' => 10,
            'V' => 5,
            'I' => 1,
            chr => {
                eprintln!("unknown char '{chr}'");
                continue;
            }
        };

        acc += value;
        acc -= 2 * (acc % value);
        // if value != 1 { }
    }
    acc
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_binary() {
        assert_eq!(super::parse_binary("0"), 0);
        assert_eq!(super::parse_binary("1"), 1);
        assert_eq!(super::parse_binary("001"), 1);
        assert_eq!(super::parse_binary("101"), 5);
        assert_eq!(super::parse_binary("110101"), 1 + 4 + 16 + 32);
    }

    #[test]
    fn parse_base10() {
        assert_eq!(super::parse_base10("0"), 0);
        assert_eq!(super::parse_base10("5"), 5);
        assert_eq!(super::parse_base10("17"), 17);
        assert_eq!(super::parse_base10("160"), 160);
        assert_eq!(super::parse_base10("999999"), 999999);
    }

    #[test]
    fn parse_hex() {
        assert_eq!(super::parse_hex("2"), 2);
        assert_eq!(super::parse_hex("a3"), 163);
        assert_eq!(super::parse_hex("1b4"), 436);
    }
}
