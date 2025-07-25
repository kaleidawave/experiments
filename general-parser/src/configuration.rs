#[derive(Clone, Copy, Debug)]
pub struct BinaryOperator<'a> {
	pub representation: &'a str,
	pub precedence: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct UnaryOperator<'a> {
	pub representation: &'a str,
	pub precedence: u8,
}

#[derive(Default, Debug)]
pub struct Configuration<'a> {
	pub prefix_unary_operators: Vec<UnaryOperator<'a>>,
	pub postfix_unary_operators: Vec<UnaryOperator<'a>>,
	pub binary_operators: Vec<BinaryOperator<'a>>,
	pub adjacency: Option<Adjacency<'a>>,
}

#[derive(Debug)]
pub struct Adjacency<'a> {
	// TODO skip
	pub operator: BinaryOperator<'a>,
	pub functions: Vec<&'a str>,
}
