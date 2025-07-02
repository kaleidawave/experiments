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
		for (idx, chr) in self.current().char_indices() {
			if !chr.is_whitespace() {
				self.advance(idx);
				break;
			}
		}
	}

	pub(crate) fn parse_identifier(&mut self) -> &'a str {
		self.skip();
		let current = self.current();
		for (idx, chr) in current.char_indices() {
			if !chr.is_alphanumeric() {
				self.advance(idx);
				return &current[..idx];
			}
		}
		self.advance(current.len());
		// TODO assert non empty
		current
	}

	pub(crate) fn starts_with(&mut self, slice: &str) -> bool {
		self.skip();
		let current = self.current();
		let matches = current.starts_with(slice);
		let is_not_actually_operator = matches
			&& slice.chars().all(char::is_alphanumeric)
			&& current[slice.len()..].chars().next().is_some_and(char::is_alphanumeric);

		if is_not_actually_operator { false } else { matches }
	}

	pub(crate) fn starts_with_chr(&mut self, chr: char) -> bool {
		self.skip();
		self.current().starts_with(chr)
	}

	pub(crate) fn advance(&mut self, idx: usize) {
		self.idx += idx;
	}
}
