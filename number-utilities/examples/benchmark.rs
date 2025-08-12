fn main() {
    const COUNT: usize = 100_000;

    let mut numbers: Vec<(u32, String)> = Vec::with_capacity(COUNT);
    let mut random = Mulberry32 { seed: 123 };

    let now = std::time::Instant::now();
    for _ in 0..COUNT {
        // assume random.next_value is neglible
        let value = random.next_value();
        numbers.push((value, number_utilities::format::to_english(value as usize)));
    }
    let format_elapsed = now.elapsed();

    let now = std::time::Instant::now();
    for i in 0..COUNT {
        let _value = number_utilities::parse::parse_english(&numbers[i].1);
    }
    let parse_elapsed = now.elapsed();

    eprintln!("format {format_elapsed:?}. parse {parse_elapsed:?}");
}

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
