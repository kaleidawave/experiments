use general_parser::{Adjacency, BinaryOperator, Configuration, Expression, UnaryOperator};

fn main() {
	let multiply_operator = BinaryOperator { representation: "*", precedence: 4 };
	let sources = &["3xy+rx", "3*x*y+r*x", "(x + t)(x + z)", "sin(-x)"];
	let configuration = Configuration {
		prefix_unary_operators: vec![UnaryOperator { representation: "-", precedence: 2 }],
		binary_operators: vec![
			BinaryOperator { representation: "^", precedence: 5 },
			BinaryOperator { representation: "*", precedence: 4 },
			multiply_operator,
			BinaryOperator { representation: "+", precedence: 3 },
		],
		adjacency: Some(Adjacency { operator: multiply_operator, functions: vec!["sin"] }),
		..Default::default()
	};

	for source in sources {
		let allocator = bumpalo::Bump::new();
		let expression: Expression<&str> =
			Expression::from_string(source, &configuration, &allocator);
		eprintln!("{expression:#?}");
	}
}
