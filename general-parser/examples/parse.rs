use general_parser::{
	Adjacency, BinaryOperator, Configuration, Expression, ExpressionRepresentation,
	TernaryOperator, UnaryOperator,
};

fn main() {
	let arg = std::env::args().nth(1);

	if let Some("--interactive") = arg.as_deref() {
		run_interactive();
		return;
	}

	if let Some(path) = std::env::args().nth(1) {
		let source = std::fs::read_to_string(&path).unwrap();
		let (configuration, source) = extract_configuration_and_source(&source);

		eprintln!("{configuration:?}");

		let allocator = bumpalo::Bump::new();
		let expression: Expression<&str> =
			Expression::from_string(&source, &configuration, &allocator);

		let expression = ExpressionRepresentation(&expression);
		println!("{source}\n -> {expression}");
	} else {
		let configuration = Configuration {
			binary_operators: vec![
				BinaryOperator { representation: "*", precedence: 4 },
				BinaryOperator { representation: "+", precedence: 3 },
			],
			..Default::default()
		};

		let sources: &[&str] = &["(x (a * b) (d * 2 + e))", "(x (a b) (c d e))"];

		for source in sources {
			let allocator = bumpalo::Bump::new();
			let expression: Expression<&str> =
				Expression::from_string(source, &configuration, &allocator);
			let expression = ExpressionRepresentation(&expression);
			eprintln!("{source}\n -> {expression}");
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

			let (configuration, source) = extract_configuration_and_source(&output);

			// eprintln!("{configuration:?} {source}");

			{
				let allocator = bumpalo::Bump::new();
				let expression: Expression<&str> =
					Expression::from_string(&source, &configuration, &allocator);
				let expression = ExpressionRepresentation(&expression);
				println!("{expression}");
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

fn extract_configuration_and_source(input: &str) -> (Configuration<'_>, &str) {
	if let Some((config, source)) = input.split_once("\n---") {
		let mut configuration = Configuration::default();
		for line in config.lines().map(str::trim_end) {
			let (adjacent, rest) = if let Some(rest) = line.strip_suffix(" (adjacent)") {
				(true, rest)
			} else {
				(false, line)
			};

			let (name, rest) = if let Some((name, rest)) = rest.split_once(':')
				&& name.trim_end().chars().all(is_identifier)
			{
				(name.trim_end(), rest)
			} else {
				("", rest)
			};

			let (syntax, precedence) = rest.split_once('#').unwrap_or((rest, "1"));
			let precedence: u8 = precedence.parse().expect("invalid precedence");
			let parts: Vec<_> = syntax.trim().split('_').map(str::trim).collect();

			match parts.as_slice() {
				["", first, part1, part2] => {
					configuration.postfix_ternary_operators.push(TernaryOperator {
						name,
						parts: (first, part1, part2),
						precedence,
					});
				}
				[first, part1, part2, ""] => {
					configuration.prefix_ternary_operators.push(TernaryOperator {
						name,
						parts: (first, part1, part2),
						precedence,
					});
				}
				["", binary_operator, ""] => {
					let operator = BinaryOperator { representation: binary_operator, precedence };
					if adjacent {
						configuration.adjacency =
							Some(Adjacency { operator, functions: Vec::default() });
					} else {
						configuration.binary_operators.push(operator);
					}
				}
				["", after] => {
					configuration
						.postfix_unary_operators
						.push(UnaryOperator { representation: after, precedence });
				}
				[before, ""] => {
					configuration
						.prefix_unary_operators
						.push(UnaryOperator { representation: before, precedence });
				}
				sequence => panic!("unknown sequence {sequence:?}"),
			}
		}
		(configuration, source.trim())
	} else {
		(Configuration::default(), input.trim())
	}
}

fn is_identifier(chr: char) -> bool {
	chr.is_alphanumeric() || matches!(chr, '_' | '$')
}
