class Lexer {
	#on: string
	#idx: number

	constructor(on: string) {
		this.#on = on;
		this.#idx = 0;
	}

	current(): string {
		return this.#on.slice(this.#idx)
	}

	finished(): boolean {
		return this.current() == ""
	}

	skip(): void {
		for (const chr of this.current()) {
			const isWhitespace = chr == " " || chr == "\t" || chr == "\n" || chr == "\r";
			if (!isWhitespace) {
				break
			}
			this.advance();
		}
	}

	parseIdentifier(): string {
		this.skip();
		const current = this.current();
		const start = this.#idx;
		for (const chr of current) {
			const code = chr.charCodeAt(0);
			const isDigit = 0x30 <= code && code <= 0x39;
			const isLowercase = 0x61 <= code && code <= 0x7a;
			const isUppercase = 0x41 <= code && code <= 0x5a;
			const isAlphanumeric = isDigit || isLowercase || isUppercase;
			if (!isAlphanumeric) {
				return current.slice(0, this.#idx - start);
			}
			this.advance();
		}

		return current;
	}

	startsWith(item: string): boolean {
		/*
		pub(crate) fn starts_with(&mut self, slice: &str) -> bool {
		self.skip();
		let current = self.current();
		let matches = current.starts_with(slice);
		let is_not_actually_operator = matches
			&& slice.chars().all(char::is_alphanumeric)
			&& current[slice.len()..].chars().next().is_some_and(char::is_alphanumeric);

		if is_not_actually_operator { false } else { matches }
	}
		*/
		this.skip();
		return this.current().startsWith(item);
	}

	startsWithValue(): boolean {
		this.skip();
		let current = this.current();
		const character = current.charCodeAt(0);
		return (0x30 <= character && character <= 0x39)
			|| (0x41 <= character && character <= 0x5a)
			|| (0x61 <= character && character <= 0x7a)
			|| `"'([`.includes(current[0])
	}

	advance(count: number = 1): void {
		this.#idx += count;
	}
}

export interface BinaryOperator {
	representation: string,
	precedence: number,
}

export interface UnaryOperator {
	representation: string,
	precedence: number,
}

export interface Configuration {
	prefix_unary_operators: Array<UnaryOperator>,
	postfix_unary_operators: Array<UnaryOperator>,
	binary_operators: Array<BinaryOperator>,
	adjacency: Adjacency | null,
}

export interface Adjacency {
	operator: BinaryOperator,
	functions: Array<string>
}

export interface Expression {
	on: string,
	arguments: Array<Expression>
}

export function parseExpression(
	source: string,
	configuration: Configuration,
): Expression {
	const reader = new Lexer(source);
	const expression = parseExpressionFromReader(reader, configuration, 0);
	if (!reader.finished()) {
		throw Error(`not finished ${reader.current()}`)
	}
	return expression
}

function parseExpressionFromReader(
	reader: Lexer,
	configuration: Configuration,
	precedence: number
): Expression {
	let unary_operator = configuration
		.prefix_unary_operators
		.find((operator) => reader.startsWith(operator.representation));

	let first: Expression;
	if (unary_operator) {
		reader.advance(unary_operator.representation.length);
		const operand = parseExpressionFromReader(reader, configuration, unary_operator.precedence)
		first = { on: unary_operator.representation, arguments: [operand] }
	} else if (reader.startsWith("(")) {
		reader.advance();
		first = parseExpressionFromReader(reader, configuration, 0);
		if (reader.startsWith(")")) {
			reader.advance(1);
		} else {
			throw Error(`no close paren ${reader.current()}`);
		}
	} else {
		const identifier = reader.parseIdentifier();

		if (configuration.adjacency && !configuration.adjacency.functions.includes(identifier)) {
			first = { on: identifier[0], arguments: [] };

			for (let i = 1; i < identifier.length; i++) {
				const rhs: Expression = { on: identifier[i], arguments: [] };
				first = {
					on: configuration.adjacency.operator.representation,
					arguments: [first, rhs]
				}
			}
		} else {
			const args: Array<Expression> = [];
			if (precedence == 0) {
				while (reader.startsWithValue()) {
					const expression: Expression = parseExpressionFromReader(reader, configuration, 1);
					args.push(expression);
				}
			}

			first = { on: identifier, arguments: args };
		}

	};

	return parseExpressionFromReaderAfterFirst(reader, configuration, precedence, first)
}

function parseExpressionFromReaderAfterFirst(
	reader: Lexer,
	configuration: Configuration,
	returnPrecedence: number,
	top: Expression
): Expression {
	// TODO postfix operators
	while (!reader.finished()) {
		reader.skip();
		const binary_operator = configuration
			.binary_operators
			.find((operator) => reader.startsWith(operator.representation));

		if (binary_operator) {
			if (returnPrecedence > binary_operator.precedence) {
				return top;
			}

			reader.advance(binary_operator.representation.length);
			const lhs = top;
			const rhs = parseExpressionFromReader(reader, configuration, binary_operator.precedence);
			top = {
				on: binary_operator.representation,
				arguments: [lhs, rhs]
			};
		} else {
			const unary_operator = configuration
				.postfix_unary_operators
				.find((operator) => reader.startsWith(operator.representation));

			if (unary_operator) {
				if (returnPrecedence > unary_operator.precedence) {
					return top;
				}

				reader.advance(unary_operator.representation.length);
				top = { on: unary_operator.representation, arguments: [top] }
			} else if (reader.startsWith("(") && configuration.adjacency) {
				const operator = configuration.adjacency.operator;

				if (returnPrecedence > operator.precedence) return top;

				const rhs = parseExpressionFromReader(reader, configuration, operator.precedence);

				top = {
					on: operator.representation,
					arguments: [top, rhs]
				};
			} else {
				break;
			}
		}
	}
	return top
}