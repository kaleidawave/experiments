pub fn parse_hex(on: &str) -> u32 {
    let mut value = 0u32;
    for byte in on.bytes() {
        value <<= 4; // log2(16) = 4
        let code = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'a'..=b'f' => u32::from(byte - b'a') + 10,
            b'A'..=b'F' => u32::from(byte - b'A') + 10,
            byte => {
                panic!("bad char {byte:?}!");
            }
        };
        value |= code;
    }
    value
}

pub fn parse_binary(on: &str) -> u32 {
    let mut value = 0u32;
    for byte in on.bytes() {
        value <<= 1;
        if let b'0' | b'1' = byte {
            value |= (byte == b'1') as u32;
        } else {
            panic!("bad char {byte:?}!");
        }
    }
    value
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_hex() {
        assert_eq!(super::parse_hex("2"), 2);
        assert_eq!(super::parse_hex("a3"), 163);
        assert_eq!(super::parse_hex("1b4"), 436);
    }

    #[test]
    fn parse_binary() {
        assert_eq!(super::parse_binary("0"), 0);
        assert_eq!(super::parse_binary("1"), 1);
        assert_eq!(super::parse_binary("001"), 1);
        assert_eq!(super::parse_binary("101"), 5);
        assert_eq!(super::parse_binary("110101"), 1 + 4 + 16 + 32);
    }
}
