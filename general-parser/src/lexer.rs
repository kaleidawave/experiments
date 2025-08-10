pub struct Lexer<'a> {
	on: &'a str,
	idx: usize,
}

impl<'a> Lexer<'a> {
	pub fn new(on: &'a str) -> Self {
		Self { on, idx: 0 }
	}

	pub(crate) fn current(&self) -> &'a str {
		&self.on[self.idx..]
	}

	pub(crate) fn finished(&self) -> bool {
		self.current().is_empty()
	}

	pub(crate) fn skip(&mut self) {
		let current = self.current();
		let idx = current.find(|chr: char| !chr.is_whitespace()).unwrap_or(current.len());
		self.advance(idx);
	}

	// TODO some of these temp
	pub(crate) fn parse_identifier(&mut self) -> &'a str {
		self.skip();
		let current = self.current();
		if let Some(rest) = current.strip_prefix('"') {
			let next = rest.find('"').unwrap();
			self.advance(next + 2);
			&current[..(next + 2)]
		} else {
			for (idx, chr) in current.char_indices() {
				if !(chr.is_alphanumeric() || matches!(chr, '.')) {
					self.advance(idx);
					return &current[..idx];
				}
			}
			self.advance(current.len());
			assert!(!current.is_empty(), "empty identifier");
			current
		}
	}

	pub(crate) fn starts_with(&mut self, slice: &str) -> bool {
		self.skip();
		let current = self.current();
		let matches = current.starts_with(slice);
		// fix for or with orpington
		let is_not_actually_operator = matches
			&& slice.chars().all(char::is_alphanumeric)
			&& current[slice.len()..].chars().next().is_some_and(char::is_alphanumeric);

		if is_not_actually_operator { false } else { matches }
	}

	pub(crate) fn starts_with_value(&mut self) -> bool {
		self.skip();
		let current = self.current();
		current
			.starts_with(|chr: char| chr.is_alphanumeric() || matches!(chr, '"' | '\'' | '(' | '['))
	}

	pub(crate) fn advance(&mut self, idx: usize) {
		self.idx += idx;
	}

	pub(crate) fn bytes_parsed(&self) -> usize {
		self.idx
	}
}
