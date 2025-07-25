use general_parser::{BinaryOperator, Configuration, Expression, UnaryOperator};

fn main() {
	let arg = std::env::args().nth(1);

	if let Some("--interactive") = arg.as_deref() {
		run_interactive();
		return;
	}

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
		let sources: &[&str] = &["(x (a * b) (d * 2 + e))", "(x (a b) (c d e))"];

		for source in sources {
			let allocator = bumpalo::Bump::new();
			let expression: Expression<&str> =
				Expression::from_string(source, &configuration, &allocator);
			eprintln!("{expression:#?}");
		}
	}
}

fn run_interactive() {
	use std::io::{BufRead, stdin};
	let stdin = stdin();
	let mut buf = Vec::new();

	println!("start");

	for line in stdin.lock().lines().map_while(Result::ok) {
		if line == "close" {
			if !buf.is_empty() {
				eprintln!("no end to message {buf:?}");
			}
			break;
		}

		if line == "end" {
			let output = String::from_utf8_lossy(&buf);

			let (configuration, source) = if let Some((config, source)) = output.split_once("\n---")
			{
				let mut configuration = Configuration::default();
				for line in config.lines() {
					let (adjacent, operation) = if let Some(rest) = line.strip_suffix(" (adjacent)")
					{
						(true, rest)
					} else {
						(false, line)
					};
					let (syntax, precedence) = operation.split_once(' ').unwrap();
					let precedence: u8 = precedence.parse().expect("invalid precedence");
					if let Some(syntax) = syntax.strip_prefix('_') {
						if let Some(representation) = syntax.strip_suffix('_') {
							let operator = BinaryOperator { representation, precedence };
							if adjacent {
								configuration.adjacency = Some(general_parser::Adjacency {
									operator,
									functions: Vec::new(),
								});
							} else {
								configuration.binary_operators.push(operator);
							}
						} else {
							let representation = syntax;
							configuration
								.postfix_unary_operators
								.push(UnaryOperator { representation, precedence });
						}
					} else {
						let representation = syntax.strip_suffix('_').unwrap();
						configuration
							.prefix_unary_operators
							.push(UnaryOperator { representation, precedence });
					}
				}
				(configuration, source)
			} else {
				(Configuration::default(), &*output)
			};

			// eprintln!("{configuration:?} {source}");

			{
				let allocator = bumpalo::Bump::new();
				let expression: Expression<&str> =
					Expression::from_string(&source, &configuration, &allocator);
				println!("{expression:#?}");
			}

			// if let Err(error) = out {
			//     println!("Error: {error:?}");
			// }

			println!("end");
			buf.clear();
			continue;
		}

		buf.extend_from_slice(line.as_bytes());
		buf.push(b'\n');
	}
}
