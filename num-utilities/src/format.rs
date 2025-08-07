pub fn to_roman_numeral(item: usize) -> String {
    let mut buf = String::new();
	if item == 0 {
		return String::default();
	}

    for i in (0..=item.ilog10()).rev() {
		let digit = (item / 10_usize.pow(i)) % 10;
        match i {
            3 => {
                // TODO more
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

pub fn to_english(item: usize) -> String {
    fn format_part(part: usize, buf: &mut String) {
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
                buf.push_str(" and ");
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

    fn under_100_to_str(item: usize) -> &'static str {
        match item {
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
            18 => "eightteen",
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

	if item == 0 {
		return "zero".to_owned();
	}

    let mut buf = String::new();
    for i in (0..=item.ilog10() / PART_WIDTH).rev() {
        let i2 = i * PART_WIDTH;
        let part = item / (10_usize.pow(i2)) % 1000;
        // dbg!(i, item);
        match i {
            3 => {
                format_part(part, &mut buf);
                if !buf.is_empty() {
                    buf.push_str(" trillion");
                }
            }
            2 => {
                format_part(part, &mut buf);
                if !buf.is_empty() {
                    buf.push_str(" million");
                }
            }
            1 => {
                format_part(part, &mut buf);
                if !buf.is_empty() {
                    buf.push_str(" thousand");
                }
            }
            0 => format_part(part, &mut buf),
            i => todo!("{i}"),
        }
    }
    buf
}
