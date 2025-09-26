pub fn to_binary(value: usize) -> String {
    let mut buf = String::new();
    let start = std::cmp::max(usize::BITS - value.leading_zeros(), 1);
    for i in (0..start).rev() {
        let bit_set = value & (0b1 << i);
        buf.push(if bit_set == 0 { '0' } else { '1' });
    }
    buf
}

pub fn to_hex(value: usize, seperator: &str) -> String {
    let mut buf = String::new();
    let start = std::cmp::max(usize::BITS - value.leading_zeros(), 1).div_ceil(4);
    for i in (0..start).rev() {
        let j = (value >> (i * 4)) & 0b1111;
        buf.push(b"0123456789abcdef"[j] as char);
        if i > 0 && i % 4 == 0 {
            buf.push_str(seperator);
        }
    }
    buf
}

pub fn to_denary(value: usize, seperator: &str) -> String {
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

/// upto 3999
pub fn to_roman_numeral(value: usize) -> String {
    let mut buf = String::new();
    if value == 0 {
        return String::default();
    }

    assert!(value < 4000, "cannot format after 3999");

    const PARTS: &[(&str, &str, &str, &str)] = &[
        ("III", "IV", "VIII", "IX"),
        ("XXX", "XL", "LXXX", "XC"),
        ("CCC", "CD", "DCCC", "CM"),
        ("MMM", "", "", ""),
    ];

    for i in (0..=value.ilog10()).rev() {
        let digit = (value / 10_usize.pow(i)) % 10;
        let (zero_to_three, four, five_to_eight, nine) = PARTS[i as usize];
        let value = match digit {
            0..4 => &zero_to_three[..digit],
            4 => four,
            5..9 => &five_to_eight[..digit - 4],
            9 => nine,
            value => unreachable!("{value}"),
        };
        buf.push_str(value);
    }
    buf
}

pub fn to_english(value: usize) -> String {
    fn under_100_to_str(value: usize) -> &'static str {
        match value {
            0 => "",
            1 => "one",
            2 => "two",
            3 => "three",
            4 => "four",
            5 => "five",
            6 => "six",
            7 => "seven",
            8 => "eight",
            9 => "nine",
            10 => "ten",
            11 => "eleven",
            12 => "twelve",
            13 => "thirteen",
            14 => "fourteen",
            15 => "fifthteen",
            16 => "sixteen",
            17 => "seventeen",
            18 => "eighteen",
            19 => "nineteen",
            20 => "twenty",
            30 => "thirty",
            40 => "fourty",
            50 => "fifty",
            60 => "sixty",
            70 => "seventy",
            80 => "eighty",
            90 => "ninety",
            number => unreachable!("{number}"),
        }
    }

    fn format_part(part: usize, is_last: bool, buf: &mut String) {
        let hundred = part / 100;
        if hundred != 0 {
            if !buf.is_empty() {
                buf.push_str(", ");
            }
            buf.push_str(under_100_to_str(hundred));
            buf.push_str(" hundred");
        }
        let part = part % 100;
        if part != 0 {
            if !buf.is_empty() {
                if is_last || hundred != 0 {
                    buf.push_str(" and ");
                } else {
                    buf.push_str(", ");
                }
            }
            if part <= 20 {
                buf.push_str(under_100_to_str(part));
            } else {
                let digit = part % 10;
                let tens = part - digit;
                buf.push_str(under_100_to_str(tens));
                if digit != 0 {
                    buf.push(' ');
                    buf.push_str(under_100_to_str(digit));
                }
            }
        }
    }

    const PART_WIDTH: u32 = 3;
    const MAGNITUDES: &[&str] = &[
        "",
        " thousand",
        " million",
        " billion",
        " trillion",
        " quadrillion",
        " quintillion",
    ];

    if value == 0 {
        return "zero".to_owned();
    }

    let mut buf = String::new();
    for i in (0..=value.ilog10() / PART_WIDTH).rev() {
        let size = 10_usize.pow(i * PART_WIDTH);
        let left = value / size;
        let part = left % 1000;
        let after = value - (left * size);
        let is_last = after == 0;
        format_part(part, is_last, &mut buf);
        if !buf.is_empty() && part != 0 {
            buf.push_str(MAGNITUDES[i as usize]);
        }
    }
    buf
}
