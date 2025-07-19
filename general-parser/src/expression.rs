use crate::{Configuration, Lexer, Literal};

#[cfg(not(feature = "nightly"))]
use allocator_api2::{vec::Vec};

#[cfg(feature = "nightly")]
use std::alloc::Allocator as AllocatorTrait;

pub type Allocator = bumpalo::Bump;

#[derive(Debug)]
pub struct Expression<'a, T> {
	pub on: T,
	pub arguments: Vec<Expression<'a, T>, &'a Allocator>,
}

impl<'a, T> Expression<'a, T>
where
	T: Literal<'a>,
{
	pub fn from_string(source: &'a str, config: &Configuration, allocator: &'a Allocator) -> Self {
		let mut reader = Lexer::new(source);
		let this = Self::from_reader(&mut reader, config, allocator);
		if !reader.finished() {
			panic!("not finished {}", reader.current());
		}
		this
	}

	pub fn from_reader(
		reader: &mut Lexer<'a>,
		config: &Configuration,
		allocator: &'a Allocator,
	) -> Self {
		Self::from_reader_with_precedence(reader, config, allocator, 0)
	}

	pub(crate) fn from_reader_with_precedence(
		reader: &mut Lexer<'a>,
		config: &Configuration,
		allocator: &'a Allocator,
		precedence: u8,
	) -> Self {
		let operator = config
			.prefix_unary_operators
			.iter()
			.find(|operator| reader.starts_with(operator.representation));

		let top = if let Some(operator) = operator {
			// hmm
			// if precedence > operator.precedence {
			//     return top;
			// }
			reader.advance(operator.representation.len());
			let operand =
				Self::from_reader_with_precedence(reader, config, allocator, operator.precedence);
			let mut arguments: Vec<Expression<_>, &Allocator> = Vec::new_in(allocator);
			arguments.push(operand);
			Expression { on: T::from_str(operator.representation), arguments }
		} else if reader.starts_with("(") {
			reader.advance(1);
			let value = Expression::from_reader(reader, config, allocator);
			if reader.starts_with(")") {
				reader.advance(1);
			} else {
				panic!("no close paren {current:?}", current=reader.current());
			};

			value
		} else {
			let identifier = reader.parse_identifier();
			
			if let Some(ref adjacency) = config.adjacency && !adjacency.functions.contains(&identifier) {
				let mut chars = identifier.char_indices();
				
				let first: Self = {
					let (idx, chr) = chars.next().unwrap();
					let identifier = &identifier[idx..(idx + chr.len_utf8())];
					Expression { on: T::from_str(identifier), arguments: Vec::new_in(allocator) }
				};
				
				let mut top = first;

				for (idx, chr) in chars {
					let identifier = &identifier[idx..(idx + chr.len_utf8())];
					let rhs = Expression { on: T::from_str(identifier), arguments: Vec::new_in(allocator) };

					let mut arguments = Vec::new_in(allocator);
					arguments.push(top);
					arguments.push(rhs);
					top = Expression { on: T::from_str(adjacency.operator.representation), arguments };
				}

				top
			} else {
				let on = T::from_str(identifier);
	
				let mut arguments = Vec::new_in(allocator);
				// TODO WIP
				if precedence == 0 {
					while reader.starts_with_value() {
						let expression =
							Self::from_reader_with_precedence(reader, config, allocator, 1);
						arguments.push(expression);
					}
				}
	
				Expression { on, arguments }
			}
		};
		Self::append_operators(reader, config, allocator, precedence, top)
	}

	fn append_operators(
		reader: &mut Lexer<'a>,
		config: &Configuration,
		allocator: &'a Allocator,
		return_precedence: u8,
		mut top: Self,
	) -> Self {
		while !reader.finished() {
			let binary_operator = config
				.binary_operators
				.iter()
				.find(|operator| reader.starts_with(operator.representation));

			if let Some(operator) = binary_operator {
				if return_precedence > operator.precedence {
					return top;
				}

				reader.advance(operator.representation.len());
				let lhs = top;
				let rhs =
					Self::from_reader_with_precedence(reader, config, allocator, operator.precedence);

				let mut arguments = Vec::new_in(allocator);
				arguments.push(lhs);
				arguments.push(rhs);

				top = Expression { on: T::from_str(operator.representation), arguments };
			} else {
				let unary_operator = config
					.postfix_unary_operators
					.iter()
					.find(|operator| reader.starts_with(operator.representation));

				if let Some(operator) = unary_operator {
					if return_precedence > operator.precedence {
						return top;
					}
					reader.advance(operator.representation.len());
					let mut arguments = Vec::new_in(allocator);
					arguments.push(top);

					top = Expression { on: T::from_str(operator.representation), arguments };
				} else if reader
					.current()
					.starts_with(|chr: char| matches!(chr, '(')) && let Some(adjacency) = &config.adjacency {
					let operator = adjacency.operator;
					if return_precedence > operator.precedence {
						return top;
					}

					let rhs = Self::from_reader_with_precedence(reader, config, allocator, operator.precedence);
					let mut arguments = Vec::new_in(allocator);
					arguments.push(top);
					arguments.push(rhs);
					top = Expression { on: T::from_str(operator.representation), arguments };
				} else {
					break;
				}
			}
		}
		top
	}
}
