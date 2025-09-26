#![allow(clippy::result_unit_err)]

pub fn parse_binary(on: &str) -> Result<usize, ()> {
    let mut value = 0usize;
    for byte in on.bytes() {
        value <<= 1;
        if let b'0' | b'1' = byte {
            value |= (byte == b'1') as usize;
        } else {
            return Err(());
        }
    }
    Ok(value)
}

pub fn parse_denary(on: &str) -> Result<usize, ()> {
    let mut value = 0usize;
    for byte in on.bytes() {
        value *= 10;
        if let b'0'..=b'9' = byte {
            value += usize::from(byte - b'0')
        } else {
            return Err(());
        }
    }
    Ok(value)
}

pub fn parse_hex(on: &str) -> Result<usize, ()> {
    let mut value = 0usize;
    for byte in on.bytes() {
        value <<= 4; // log2(16) = 4
        let code = match byte {
            b'0'..=b'9' => usize::from(byte - b'0'),
            b'a'..=b'f' => usize::from(byte - b'a') + 10,
            // also include uppercase
            b'A'..=b'F' => usize::from(byte - b'A') + 10,
            _byte => {
                return Err(());
            }
        };
        value |= code;
    }
    Ok(value)
}

pub fn parse_english(on: &str) -> Result<usize, &str> {
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
                return Err(number);
            }
        })
    }

    let mut acc: usize = 0;
    for block in on.split(',') {
        let block = block.trim();
        let item = parse_item(block)?;
        acc += item;
    }
    Ok(acc)
}

pub fn parse_roman_numeral(on: &str) -> Result<usize, ()> {
    let mut acc = 0;
    for chr in on.chars() {
        let letter = match chr.to_ascii_uppercase() {
            'M' => 1000,
            'D' => 500,
            'C' => 100,
            'L' => 50,
            'X' => 10,
            'V' => 5,
            'I' => 1,
            _chr => {
                return Err(());
            }
        };

        acc += letter;
        acc -= 2 * (acc % letter);
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_binary() {
        assert_eq!(super::parse_binary("0"), Ok(0));
        assert_eq!(super::parse_binary("1"), Ok(1));
        assert_eq!(super::parse_binary("001"), Ok(1));
        assert_eq!(super::parse_binary("101"), Ok(5));
        assert_eq!(super::parse_binary("110101"), Ok(1 + 4 + 16 + 32));
    }

    #[test]
    fn parse_denary() {
        assert_eq!(super::parse_denary("0"), Ok(0));
        assert_eq!(super::parse_denary("5"), Ok(5));
        assert_eq!(super::parse_denary("17"), Ok(17));
        assert_eq!(super::parse_denary("160"), Ok(160));
        assert_eq!(super::parse_denary("999999"), Ok(999999));
    }

    #[test]
    fn parse_hex() {
        assert_eq!(super::parse_hex("2"), Ok(2));
        assert_eq!(super::parse_hex("a3"), Ok(163));
        assert_eq!(super::parse_hex("1b4"), Ok(436));
    }
}
