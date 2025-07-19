#[derive(Clone, Copy, Debug)]
pub struct BinaryOperator {
	pub representation: &'static str,
	pub precedence: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct UnaryOperator {
	pub representation: &'static str,
	pub precedence: u8,
}

#[derive(Default)]
pub struct Configuration {
	pub prefix_unary_operators: Vec<UnaryOperator>,
	pub postfix_unary_operators: Vec<UnaryOperator>,
	pub binary_operators: Vec<BinaryOperator>,
	pub identifier_prefixes: Vec<char>,
	pub adjacency: Option<Adjacency>,
}

pub struct Adjacency {
	// TODO skip
	pub operator: BinaryOperator,
	pub functions: Vec<&'static str>,
}
