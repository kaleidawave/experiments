use number_utilities::{format, parse};
use std::time::SystemTime;

pub struct Mulberry32 {
    pub seed: u32,
}

impl Mulberry32 {
    pub fn next_value(&mut self) -> u32 {
        self.seed = self.seed.wrapping_add(0x6D2B79F5);
        let mut t = self.seed;
        t = (t ^ t.wrapping_shr(15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add(t ^ t.wrapping_shr(7).wrapping_mul(t | 61));
        t ^ t.wrapping_shr(14)
    }

    pub fn new_from_sys_time() -> Self {
        let elapsed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let value = (elapsed.as_micros() % 1000) as u32;
        Self { seed: value }
    }
}

#[test]
fn fuzz_binary() {
    let mut random = Mulberry32::new_from_sys_time();

    for _ in 0..100_000 {
        let random_value = random.next_value();
        let pow: u32 = 10_u32.pow((random_value.rotate_left(16) % 8) + 1);
        let value = (random_value % pow) as usize;

        let base_format = format::to_binary(value);
        let out_value = parse::parse_binary(&base_format);

        assert_eq!(value, out_value, "{base_format} parsed as {out_value}");
    }
}

// TODO fuzz octal

#[test]
fn fuzz_decinary() {
    let mut random = Mulberry32::new_from_sys_time();

    for _ in 0..100_000 {
        let random_value = random.next_value();
        let pow: u32 = 10_u32.pow((random_value.rotate_left(16) % 8) + 1);
        let value = (random_value % pow) as usize;

        let base_format = format::to_base10(value);
        let out_value = parse::parse_base10(&base_format);

        assert_eq!(value, out_value, "{base_format} parsed as {out_value}");
    }
}

#[test]
fn fuzz_hex() {
    let mut random = Mulberry32::new_from_sys_time();

    for _ in 0..100_000 {
        let random_value = random.next_value();
        let pow: u32 = 10_u32.pow((random_value.rotate_left(16) % 8) + 1);
        let value = (random_value % pow) as usize;

        let base_format = format::to_hex(value);
        let out_value = parse::parse_hex(&base_format);

        assert_eq!(value, out_value, "{base_format} parsed as {out_value}");
    }
}

// TODO fuzz base64
