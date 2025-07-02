use crate::{Configuration, Lexer};

#[derive(Debug, Clone)]
pub enum Expression<'a> {
	Identifier {
		prefix: Option<char>,
		name: &'a str,
	},
	/// TODO more config
	Grouped {
		values: Vec<Self>,
	},
	Application {
		on: Box<Self>,
		argument: Box<Self>,
	},
}

// TODO results her
impl<'a> Expression<'a> {
	pub fn from_string(source: &'a str, config: &Configuration) -> Self {
		let mut reader = Lexer::new(source);
		let this = Self::from_reader_precedence(&mut reader, config, 0);
		if !reader.finished() {
			panic!("not finished {}", reader.current());
		}
		this
	}

	pub fn from_reader(reader: &mut Lexer<'a>, config: &Configuration) -> Self {
		Self::from_reader_precedence(reader, config, 0)
	}

	pub(crate) fn from_reader_precedence(
		reader: &mut Lexer<'a>,
		config: &Configuration,
		precedence: u8,
	) -> Self {
		// TODO numbers, strings
		let operator = config
			.unary_operators
			.iter()
			.find(|operator| operator.prefix && reader.starts_with(operator.representation));

		let acc = if let Some(operator) = operator {
			// hmm
			// if precedence > operator.precedence {
			//     return top;
			// }
			reader.advance(operator.representation.len());
			let operand = Self::from_reader_precedence(reader, config, operator.precedence);
			Expression::new_unary_operator(operator.representation, operand)
		} else if reader.starts_with("(") {
			reader.advance(1);
			let mut values = Vec::new();
			let mut expr = None;
			while !reader.finished() {
				if reader.starts_with(")") {
					reader.advance(1);
					expr = Some(Self::Grouped { values });
					break;
				}
				if !values.is_empty() {
					if reader.starts_with(",") {
						reader.advance(1);
					} else {
						panic!("no comma")
					}
				}
				let value = Expression::from_reader(reader, config);
				values.push(value);
			}
			if let Some(expr) = expr { expr } else { panic!("no close paren") }
		} else {
			// TODO number
			let prefix = config
				.identifier_prefixes
				.iter()
				.find(|chr| reader.starts_with_chr(**chr))
				.copied();

			if let Some(ref prefix) = prefix {
				reader.advance(prefix.len_utf8());
			}

			let name = reader.parse_identifier();

			if let Some(ref adjacency) = config.adjacency {
				// TODO skip if in thingy
				// TODO split by prefix
				// We reverse to preserve ltr associativity
				let mut chars = name.char_indices().rev();
				let (idx, chr) = chars.next().unwrap();
				let mut top = Self::Identifier { prefix, name: &name[idx..(idx + chr.len_utf8())] };
				for (idx, chr) in chars {
					let rhs = Self::Identifier { prefix, name: &name[idx..(idx + chr.len_utf8())] };
					top = Expression::new_binary_operator(
						adjacency.operator.representation,
						rhs,
						top,
					);
				}
				top
			} else {
				Self::Identifier { prefix, name }
			}
		};
		Self::from_reader_first(reader, config, precedence, acc)
	}

	pub fn new_unary_operator(operation: &'static str, operand: Expression<'a>) -> Self {
		let on = Expression::Identifier { prefix: None, name: operation };
		Expression::Application { on: Box::new(on), argument: Box::new(operand) }
	}

	pub fn new_binary_operator(
		operation: &'static str,
		lhs: Expression<'a>,
		rhs: Expression<'a>,
	) -> Self {
		let on = Expression::Identifier { prefix: None, name: operation };
		let lhs = Expression::Application { on: Box::new(on), argument: Box::new(lhs) };
		Expression::Application { on: Box::new(lhs), argument: Box::new(rhs) }
	}

	// TODO result
	pub(crate) fn from_reader_first(
		reader: &mut Lexer<'a>,
		config: &Configuration,
		return_precedence: u8,
		mut top: Self,
	) -> Self {
		// TODO postfix operators
		while !reader.finished() {
			reader.skip();
			let binary_operator = config
				.binary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.representation));

			let unary_operator = config
				.unary_operators
				.iter()
				.find(|operator| !operator.prefix && reader.starts_with(operator.representation));

			if let Some(operator) = binary_operator {
				if return_precedence > operator.precedence {
					return top;
				}

				reader.advance(operator.representation.len());
				let lhs = top;
				let rhs = Self::from_reader_precedence(reader, config, operator.precedence);
				top = Expression::new_binary_operator(operator.representation, lhs, rhs);
			} else if let Some(operator) = unary_operator {
				if return_precedence > operator.precedence {
					return top;
				}
				reader.advance(operator.representation.len());
				top = Expression::new_unary_operator(operator.representation, top);
			} else if reader
				.current()
				.starts_with(|chr: char| chr.is_ascii_alphanumeric() || matches!(chr, '('))
			{
				if let Some(crate::configuration::Adjacency { operator, functions: _ }) =
					config.adjacency
				{
					// TODO functions
					// TODO backwards ...?
					if return_precedence > operator.precedence {
						return top;
					}

					let lhs = top;
					let rhs = Self::from_reader_precedence(reader, config, operator.precedence);
					top = Expression::new_binary_operator(operator.representation, lhs, rhs);
				} else {
					let on = top;
					let argument = Self::from_reader_precedence(reader, config, 1);
					top = Expression::Application { on: Box::new(on), argument: Box::new(argument) }
				}
			} else {
				break;
			}
		}
		top
	}

	#[cfg(feature = "to_string")]
	pub fn to_string(&self) -> String {
		let mut buf = String::new();
		self.to_string2(&mut buf);
		buf
	}

	#[cfg(feature = "to_string")]
	pub(crate) fn to_string2(&self, buf: &mut String) {
		use std::fmt::Write;
		match self {
			Self::Identifier { prefix, name } => {
				if let Some(prefix) = prefix {
					write!(buf, "{prefix}").unwrap();
				}
				write!(buf, "{name}").unwrap();
			}
			// TODO more config
			Self::Grouped { values } => {
				write!(buf, "(").unwrap();
				values.iter().for_each(|value| value.to_string2(buf));
				write!(buf, ")").unwrap();
			}
			Self::Application { on, argument } => {
				on.to_string2(buf);
				if !matches!(&**argument, Self::Grouped { .. }) {
					write!(buf, " ").unwrap();
				}
				argument.to_string2(buf);
			}
		}
	}
}
