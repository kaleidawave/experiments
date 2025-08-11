pub struct Mulberry32 {
	pub seed: u32
}

impl Mulberry32 {
	pub fn next(&mut self) -> u32 {
		self.seed = self.seed.wrapping_add(0x6D2B79F5);
		let mut t = self.seed;
		t = (t ^ t.wrapping_shr(15)).wrapping_mul(t | 1);
		t ^= t.wrapping_add(t ^ t.wrapping_shr(7).wrapping_mul(t | 61));
		t ^ t.wrapping_shr(14)
	}
}