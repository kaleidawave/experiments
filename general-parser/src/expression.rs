use crate::{Allocator, Configuration, Lexer, Literal};

#[cfg(not(feature = "stable"))]
use std::alloc::Allocator as AllocatorTrait;

#[cfg(feature = "stable")]
use allocator_api2::vec::Vec;

#[derive(Debug)]
pub struct Expression<'a, T> {
	pub on: T,
	pub arguments: Vec<Expression<'a, T>, &'a Allocator>,
}

impl<'a, T> Expression<'a, T>
where
	T: Literal<'a>,
{
	pub fn from_string(
		source: &'a str,
		configuration: &'a Configuration,
		allocator: &'a Allocator,
	) -> Self {
		let mut reader = Lexer::new(source);
		let this = Self::from_reader(&mut reader, configuration, allocator);
		reader.skip();
		if !reader.finished() {
			panic!("not finished {:?}", reader.current());
		}
		this
	}

	pub fn from_reader(
		reader: &mut Lexer<'a>,
		configuration: &'a Configuration,
		allocator: &'a Allocator,
	) -> Self {
		Self::from_reader_with_precedence(reader, configuration, allocator, 0, None)
	}

	pub(crate) fn from_reader_with_precedence(
		reader: &mut Lexer<'a>,
		configuration: &'a Configuration,
		allocator: &'a Allocator,
		precedence: u8,
		break_before: Option<&'a str>,
	) -> Self {
		if reader.starts_with("(") {
			reader.advance(1);
			let value = Expression::from_reader(reader, configuration, allocator);
			if reader.starts_with(")") {
				reader.advance(1);
				value
			} else {
				panic!("no close paren {current:?}", current = reader.current());
			}
		} else {
			let top = if let Some(prefix) = configuration
				.prefix_ternary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.parts.0))
			{
				// hmm
				// if precedence.order > operator.precedence {
				//     return top;
				// }

				reader.advance(prefix.parts.0.len());

				let next = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					prefix.precedence,
					Some(prefix.parts.1),
				);
				if reader.starts_with(prefix.parts.1) {
					reader.advance(prefix.parts.1.len());
				} else {
					// TODO check prefix operators
					panic!("{:?}", (reader.current(), prefix.parts.1));
				}

				let lhs = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					prefix.precedence,
					Some(prefix.parts.2),
				);
				if reader.starts_with(prefix.parts.2) {
					reader.advance(prefix.parts.2.len());
				} else {
					panic!()
				}
				let rhs = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					prefix.precedence,
					break_before,
				);

				let mut arguments: Vec<Expression<_>, &Allocator> = Vec::new_in(allocator);
				arguments.push(next);
				arguments.push(lhs);
				arguments.push(rhs);

				let on = T::from_str(prefix.name);
				Expression { on, arguments }
			} else if let Some(operator) = configuration
				.prefix_unary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.representation))
			{
				// hmm
				// if precedence > operator.precedence {
				//     return top;
				// }

				reader.advance(operator.representation.len());
				let operand = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					operator.precedence,
					break_before,
				);

				let mut arguments: Vec<Expression<_>, &Allocator> = Vec::new_in(allocator);
				arguments.push(operand);
				Expression { on: T::from_str(operator.representation), arguments }
			} else {
				let identifier = reader.parse_identifier();

				if let Some(ref adjacency) = configuration.adjacency
					&& !adjacency.functions.contains(&identifier)
				{
					// TODO slice iterator
					let mut chars = identifier.char_indices();

					let first: Self = {
						let (idx, chr) = chars.next().unwrap();
						let identifier = &identifier[idx..(idx + chr.len_utf8())];
						Expression {
							on: T::from_str(identifier),
							arguments: Vec::new_in(allocator),
						}
					};

					let mut top = first;

					for (idx, chr) in chars {
						let identifier = &identifier[idx..(idx + chr.len_utf8())];
						let rhs = Expression {
							on: T::from_str(identifier),
							arguments: Vec::new_in(allocator),
						};

						let mut arguments = Vec::new_in(allocator);
						arguments.push(top);
						arguments.push(rhs);
						top = Expression {
							on: T::from_str(adjacency.operator.representation),
							arguments,
						};
					}

					top
				} else {
					let on = T::from_str(identifier);

					let mut arguments = Vec::new_in(allocator);

					// TODO WIP check
					while reader.starts_with_value() {
						let Configuration {
							binary_operators,
							postfix_ternary_operators,
							postfix_unary_operators,
							..
						} = &configuration;

						let should_break = binary_operators
							.iter()
							.any(|operator| reader.starts_with(operator.representation))
							|| postfix_ternary_operators
								.iter()
								.any(|operator| reader.starts_with(operator.parts.0))
							|| postfix_unary_operators
								.iter()
								.any(|operator| reader.starts_with(operator.representation))
							|| break_before
								.is_some_and(|break_before| reader.starts_with(break_before));

						if should_break {
							break;
						}

						let expression = Self::from_reader_with_precedence(
							reader,
							configuration,
							allocator,
							1,
							break_before,
						);
						arguments.push(expression);
					}

					Expression { on, arguments }
				}
			};

			Self::append_operators(reader, configuration, allocator, precedence, break_before, top)
		}
	}

	fn append_operators(
		reader: &mut Lexer<'a>,
		configuration: &'a Configuration,
		allocator: &'a Allocator,
		return_precedence: u8,
		break_before: Option<&'a str>,
		mut top: Self,
	) -> Self {
		while !reader.finished() {
			if break_before.is_some_and(|break_before| reader.starts_with(break_before)) {
				break;
			}

			top = if let Some(postfix) = configuration
				.postfix_ternary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.parts.0))
			{
				if return_precedence > postfix.precedence {
					return top;
				}

				reader.advance(postfix.parts.0.len());
				let lhs = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					postfix.precedence,
					Some(postfix.parts.1),
				);

				if reader.starts_with(postfix.parts.1) {
					reader.advance(postfix.parts.1.len());
					let rhs = Self::from_reader_with_precedence(
						reader,
						configuration,
						allocator,
						postfix.precedence,
						Some(postfix.parts.2),
					);

					if reader.starts_with(postfix.parts.2) {
						reader.advance(postfix.parts.2.len());
					} else {
						panic!()
					}

					let mut arguments: Vec<Expression<_>, &Allocator> = Vec::new_in(allocator);
					arguments.push(top);
					arguments.push(lhs);
					arguments.push(rhs);

					let on = T::from_str(postfix.name);
					Expression { on, arguments }
				} else {
					let substitute_binary_operator = configuration
						.binary_operators
						.iter()
						.find(|operator| operator.representation == postfix.parts.0);

					if let Some(operator) = substitute_binary_operator {
						debug_assert!(
							return_precedence <= operator.precedence,
							"binary operator {bop_prec} not equal to ternary {tern_prec} ({return_precedence})",
							bop_prec = operator.precedence,
							tern_prec = postfix.precedence
						);

						let mut arguments = Vec::new_in(allocator);
						arguments.push(top);
						arguments.push(lhs);

						let on = T::from_str(operator.representation);
						Expression { on, arguments }
					} else {
						panic!(
							"Expected {part} found {found}",
							part = postfix.parts.1,
							found = reader.current()
						);
					}
				}
			} else if let Some(operator) = configuration
				.binary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.representation))
			{
				if return_precedence > operator.precedence {
					return top;
				}

				reader.advance(operator.representation.len());
				let lhs = top;
				let rhs = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					operator.precedence,
					break_before,
				);

				let mut arguments = Vec::new_in(allocator);
				arguments.push(lhs);
				arguments.push(rhs);

				let on = T::from_str(operator.representation);
				Expression { on, arguments }
			} else if let Some(operator) = configuration
				.postfix_unary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.representation))
			{
				if return_precedence > operator.precedence {
					return top;
				}
				reader.advance(operator.representation.len());
				let mut arguments = Vec::new_in(allocator);
				arguments.push(top);

				let on = T::from_str(operator.representation);
				Expression { on, arguments }
			} else if reader.current().starts_with('(')
				&& let Some(adjacency) = &configuration.adjacency
			{
				let operator = adjacency.operator;
				if return_precedence > operator.precedence {
					return top;
				}

				let rhs = Self::from_reader_with_precedence(
					reader,
					configuration,
					allocator,
					operator.precedence,
					break_before,
				);
				let mut arguments = Vec::new_in(allocator);
				arguments.push(top);
				arguments.push(rhs);
				Expression { on: T::from_str(operator.representation), arguments }
			} else {
				break;
			};
		}
		top
	}
}

pub struct ExpressionRepresentation<'a, T>(pub &'a Expression<'a, T>);

impl<'a, T> std::fmt::Display for ExpressionRepresentation<'a, T>
where
	T: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		if self.0.arguments.is_empty() {
			write!(f, "{on}", on = self.0.on)
		} else {
			write!(f, "({on}", on = self.0.on)?;
			for arg in &self.0.arguments {
				write!(f, " {on}", on = ExpressionRepresentation(arg))?;
			}
			write!(f, ")")
		}
	}
}
