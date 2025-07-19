use general_parser::{Configuration, BinaryOperator, Expression};

fn main() {
	let configuration = Configuration {
		binary_operators: vec![
			BinaryOperator { representation: "*", precedence: 4 },
			BinaryOperator { representation: "+", precedence: 3 },
		],
		..Default::default()
	};

	if let Some(source) = std::env::args().nth(1) {
		let allocator = bumpalo::Bump::new();
		let expression: Expression<&str> =
			Expression::from_string(&source, &configuration, &allocator);
		eprintln!("{expression:#?}");
	} else {
		let sources: &[&str] = &[
			"(x (a * b) (d * 2 + e))",
			"(x (a b) (c d e))"
		];
	
		for source in sources {
			let allocator = bumpalo::Bump::new();
			let expression: Expression<&str> =
				Expression::from_string(source, &configuration, &allocator);
			eprintln!("{expression:#?}");
		}
	}
}
