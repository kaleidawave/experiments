use number_utilities::format::to_english;
use number_utilities::parse::parse_english;

#[test]
fn fuzz_english() {
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
    }

    let elapsed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let value = (elapsed.as_micros() % 1000) as u32;
    let mut random = Mulberry32 { seed: value };

    for _ in 0..100_000 {
        let random_value = random.next_value();
        let pow: u32 = 10_u32.pow((random_value.rotate_left(16) % 8) + 1);
        let value = (random_value % pow) as usize;

        let value_english = to_english(value);
        let out_value = parse_english(&value_english);

        assert_eq!(value, out_value, "{value_english} parsed as {out_value}");
    }
}

#[test]
fn format_english() {
    assert_eq!(to_english(39), "thirty nine");
    assert_eq!(to_english(246), "two hundred and fourty six");
    assert_eq!(to_english(789), "seven hundred and eighty nine");
    assert_eq!(
        to_english(2421),
        "two thousand, four hundred and twenty one"
    );

    assert_eq!(to_english(160), "one hundred and sixty");
    assert_eq!(to_english(207), "two hundred and seven");
    assert_eq!(to_english(1009), "one thousand and nine");
    assert_eq!(to_english(1066), "one thousand and sixty six");

    assert_eq!(
        to_english(1776),
        "one thousand, seven hundred and seventy six"
    );
    assert_eq!(to_english(1918), "one thousand, nine hundred and eighteen");
    assert_eq!(
        to_english(1944),
        "one thousand, nine hundred and fourty four"
    );
    assert_eq!(to_english(2025), "two thousand and twenty five");

    assert_eq!(
        to_english(3999),
        "three thousand, nine hundred and ninety nine"
    );

    assert_eq!(to_english(1_000_000), "one million");
    assert_eq!(to_english(1_000_002), "one million and two");
    assert_eq!(to_english(1_000_002_000), "one billion and two thousand");
}
