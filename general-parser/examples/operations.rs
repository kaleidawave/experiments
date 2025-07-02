use general_parser::{
	Adjacency, BinaryOperator, Configuration, Expression, lifting::ExpressionKinds,
};

fn main() {
	let multiply_operator = BinaryOperator { representation: "*", precedence: 4 };
	let sources = &["3xy+rx", "3*x*y+r*x", "(x + t)(x + z)"];
	let config = Configuration {
		// unary_operators: Default::default(),
		binary_operators: vec![
			// BinaryOperator { representation: "^", precedence: 5 },
			// BinaryOperator { representation: "*", precedence: 4 },
			multiply_operator,
			BinaryOperator { representation: "+", precedence: 3 },
		],
		adjacency: Some(Adjacency { operator: multiply_operator, functions: Vec::default() }),
		..Default::default()
	};

	for source in sources {
		let expression = Expression::from_string(source, &config);
		let expression = ExpressionKinds::from_expression(&expression, &config);
		eprintln!("{expression:#?}");
	}
}
