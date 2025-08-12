pub fn to_binary(value: usize) -> String {
    if value == 0 {
        return "0".to_owned();
    }
    let mut buf = String::new();
    let start = usize::BITS - value.leading_zeros();
    for i in (0..start).rev() {
        let bit_set = value & (0b1 << i);
        buf.push(if bit_set == 0 { '0' } else { '1' });
    }
    buf
}

pub fn to_hex(value: usize) -> String {
    if value == 0 {
        return "0".to_owned();
    }
    let mut buf = String::new();
    let start = (usize::BITS - value.leading_zeros()).div_ceil(4);
    for i in (0..start).rev() {
        let j = (value >> (i * 4)) % 16;
        buf.push("0123456789abcdef".chars().nth(j).unwrap());
    }
    buf
}

pub fn to_base10(value: usize) -> String {
    if value == 0 {
        return "0".to_owned();
    }
    let mut buf = String::new();
    for i in (0..=value.ilog10()).rev() {
        let j = (value / 10i32.pow(i) as usize) % 10;
        buf.push("0123456789".chars().nth(j).unwrap());
    }
    buf
}

/// upto 3999
pub fn to_roman_numeral(value: usize) -> String {
    let mut buf = String::new();
    if value == 0 {
        return String::default();
    }

    for i in (0..=value.ilog10()).rev() {
        let digit = (value / 10_usize.pow(i)) % 10;
        match i {
            3 => {
                buf.push_str(&"MMM"[..digit]);
            }
            2 => {
                let value = match digit {
                    0..4 => &"CCC"[..digit],
                    4 => "CD",
                    5..9 => &"DCCC"[..digit - 4],
                    9 => "CM",
                    value => unreachable!("{value}"),
                };
                buf.push_str(value);
            }
            1 => {
                let value = match digit {
                    0..4 => &"XXX"[..digit],
                    4 => "XL",
                    5..9 => &"LXXX"[..digit - 4],
                    9 => "XC",
                    value => unreachable!("{value}"),
                };
                buf.push_str(value);
            }
            0 => {
                let value = match digit {
                    0..4 => &"III"[..digit],
                    4 => "IV",
                    5..9 => &"VIII"[..digit - 4],
                    9 => "IX",
                    value => unreachable!("{value}"),
                };
                buf.push_str(value);
            }
            a => todo!("{a}"),
        }
    }
    buf
}

pub fn to_english(value: usize) -> String {
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
            number => todo!("{number}"),
        }
    }

    const PART_WIDTH: u32 = 3;

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
        match i {
            5 => {
                format_part(part, is_last, &mut buf);
                if !buf.is_empty() && part != 0 {
                    buf.push_str(" quadrillion");
                }
            }
            4 => {
                format_part(part, is_last, &mut buf);
                if !buf.is_empty() && part != 0 {
                    buf.push_str(" trillion");
                }
            }
            3 => {
                format_part(part, is_last, &mut buf);
                if !buf.is_empty() && part != 0 {
                    buf.push_str(" billion");
                }
            }
            2 => {
                format_part(part, is_last, &mut buf);
                if !buf.is_empty() && part != 0 {
                    buf.push_str(" million");
                }
            }
            1 => {
                format_part(part, is_last, &mut buf);
                if !buf.is_empty() && part != 0 {
                    buf.push_str(" thousand");
                }
            }
            0 => format_part(part, is_last, &mut buf),
            i => todo!("numbers of order {order}", order = i * 3),
        }
    }
    buf
}
