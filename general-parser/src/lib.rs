pub mod configuration;
pub mod expression;
pub mod lexer;

pub use configuration::{Adjacency, BinaryOperator, Configuration, UnaryOperator};
pub use expression::Expression;
pub use lexer::Lexer;

pub trait Literal<'a> {
	fn from_str(on: &'a str) -> Self;
}

impl<'a> Literal<'a> for &'a str {
	fn from_str(on: &'a str) -> Self {
		on
	}
}
