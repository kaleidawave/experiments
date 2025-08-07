fn main() {
	let s = if let Some(item) = std::env::args().nth(1) {
		std::borrow::Cow::Owned(item)
	} else {
		std::borrow::Cow::Borrowed("XXXIX")
	};

	let mut acc = 0;
	for chr in s.chars() {
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
		if value != 1 {
			acc -= 2 * (acc % value);
		}
	}

	eprintln!("{s} = {acc}");
}