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
			if (!(isAlphanumeric || chr === '.')) {
				return current.slice(0, this.#idx - start);
			}
			this.advance();
		}

		return current;
	}

	startsWith(item: string): boolean {
		this.skip();
		const matches = this.current().startsWith(item);

		// fix for or with orpington
		const is_not_actually_operator = matches
			&& Array.from(item).every(c => is_alphanumeric(c.charCodeAt(0)))
			&& is_alphanumeric(this.current().charCodeAt(item.length))

		if (is_not_actually_operator) return is_not_actually_operator;
		return matches;
	}

	startsWithValue(): boolean {
		this.skip();
		let current = this.current();
		return is_alphanumeric(current.charCodeAt(0)) || `"'([`.includes(current[0])
	}

	advance(count: number = 1): void {
		this.#idx += count;
	}
}

// TODO what about 'ą'
function is_alphanumeric(character: number): boolean {
	return (0x30 <= character && character <= 0x39)
		|| (0x41 <= character && character <= 0x5a)
		|| (0x61 <= character && character <= 0x7a)
}

export interface BinaryOperator {
	representation: string,
	precedence: number,
}

export interface UnaryOperator {
	representation: string,
	precedence: number,
}

export interface TernaryOperator {
	name: string,
	parts: [string, string, string],
	precedence: number,
}

export interface Configuration {
	prefix_unary_operators: Array<UnaryOperator>,
	prefix_ternary_operators: Array<TernaryOperator>,
	postfix_unary_operators: Array<UnaryOperator>,
	postfix_ternary_operators: Array<TernaryOperator>,
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
	const expression = parseExpressionFromReader(reader, configuration, 0, null);
	if (!reader.finished()) {
		throw Error(`not finished ${reader.current()}`)
	}
	return expression
}

export function defaultConfiguration(): Configuration {
	return {
		binary_operators: [],
		prefix_unary_operators: [],
		adjacency: null,
		postfix_ternary_operators: [],
		postfix_unary_operators: [],
		prefix_ternary_operators: [],
	};
}

function parseExpressionFromReader(
	reader: Lexer,
	configuration: Configuration,
	precedence: number,
	breakBefore: string | null = null
): Expression {
	let ternary_operator: TernaryOperator | undefined;
	let unary_operator: UnaryOperator | undefined;

	let first: Expression;
	if (reader.startsWith("(")) {
		reader.advance();
		first = parseExpressionFromReader(reader, configuration, 0);
		if (reader.startsWith(")")) {
			reader.advance(1);

			if (configuration.adjacency && reader.startsWithValue()) {
				let identifier = reader.parseIdentifier();
				const rhs = { on: identifier, arguments: [] };
				first = {
					on: configuration.adjacency.operator.representation,
					arguments: [first, rhs]
				};
			}
		} else {
			throw Error(`no close paren ${reader.current()}`);
		}
	} else if (ternary_operator = configuration
		.prefix_ternary_operators
		.find((operator) => reader.startsWith(operator.parts[0]))) {

		reader.advance(ternary_operator.parts[0].length);

		let next = parseExpressionFromReader(
			reader,
			configuration,
			ternary_operator.precedence,
			ternary_operator.parts[1]
		);

		if (reader.startsWith(ternary_operator.parts[1])) {
			reader.advance(ternary_operator.parts[1].length);
		} else {
			throw new Error("Expected " + ternary_operator.parts[1]);
		}

		let lhs = parseExpressionFromReader(
			reader,
			configuration,
			ternary_operator.precedence,
			ternary_operator.parts[2]
		);

		if (reader.startsWith(ternary_operator.parts[2])) {
			reader.advance(ternary_operator.parts[2].length);
		} else {
			throw new Error("Expected " + ternary_operator.parts[2]);
		}

		let rhs = parseExpressionFromReader(
			reader,
			configuration,
			ternary_operator.precedence,
			breakBefore
		);

		first = { on: ternary_operator.name, arguments: [next, lhs, rhs] }
	} else if (unary_operator = configuration
		.prefix_unary_operators
		.find((operator) => reader.startsWith(operator.representation))) {
		reader.advance(unary_operator.representation.length);
		const operand = parseExpressionFromReader(
			reader,
			configuration,
			unary_operator.precedence,
			breakBefore
		);
		first = { on: unary_operator.representation, arguments: [operand] }
	} else {
		const identifier = reader.parseIdentifier();

		if (configuration.adjacency) {
			const parts = splitNumber(identifier);

			if (parts[1]) {
				const isFunction = configuration.adjacency.functions.includes(parts[1]);
				if (isFunction) {
					first = parseExpressionCall(reader, configuration, breakBefore, parts[1]);
				} else {
					first = { on: parts[1][0], arguments: [] };
					for (let i = 1; i < parts[1].length; i++) {
						const rhs: Expression = { on: parts[1][i], arguments: [] };
						first = {
							on: configuration.adjacency.operator.representation,
							arguments: [rhs, first]
						}
					}
				}
				if (parts[0]) {
					const numeric = { on: parts[0], arguments: [] };
					first = {
						on: configuration.adjacency.operator.representation,
						arguments: [numeric, first]
					}
				}
			} else {
				// exclusively number
				first = { on: identifier, arguments: [] }
			}
		} else {
			first = parseExpressionCall(reader, configuration, breakBefore, identifier);
		}
	};

	return parseExpressionFromReaderAfterFirst(
		reader,
		configuration,
		precedence,
		breakBefore,
		first
	)
}

function parseExpressionCall(
	reader: Lexer,
	configuration: Configuration,
	breakBefore: string | null,
	on: string
): Expression {
	const args: Array<Expression> = [];
	
	while (reader.startsWithValue()) {
		let shouldBreak = configuration.binary_operators.some(op => reader.startsWith(op.representation));
		shouldBreak ||= configuration.postfix_ternary_operators.some(op => reader.startsWith(op.parts[0]));
		shouldBreak ||= configuration.postfix_unary_operators.some(op => reader.startsWith(op.representation));
		shouldBreak ||= breakBefore !== null && reader.startsWith(breakBefore);

		if (shouldBreak) break;

		const expression: Expression = parseExpressionFromReader(
			reader,
			configuration,
			1,
			breakBefore
		);
		args.push(expression);
	}
	
	return { on, arguments: args }
}

function parseExpressionFromReaderAfterFirst(
	reader: Lexer,
	configuration: Configuration,
	returnPrecedence: number,
	breakBefore: string | null,
	top: Expression
): Expression {
	while (!reader.finished()) {
		reader.skip();

		if (breakBefore && reader.startsWith(breakBefore)) {
			break;
		}

		let postfix_ternary_operator: TernaryOperator | undefined;
		let binary_operator: BinaryOperator | undefined;
		let unary_operator: UnaryOperator | undefined;

		if (postfix_ternary_operator =
			configuration.postfix_ternary_operators.find(operator => reader.startsWith(operator.parts[0]))
		) {
			if (returnPrecedence > postfix_ternary_operator.precedence) {
				return top;
			}

			reader.advance(postfix_ternary_operator.parts[0].length);
			const lhs = parseExpressionFromReader(
				reader,
				configuration,
				postfix_ternary_operator.precedence,
				postfix_ternary_operator.parts[1]
			);

			if (reader.startsWith(postfix_ternary_operator.parts[1])) {
				reader.advance(postfix_ternary_operator.parts[1].length);

				const rhs = parseExpressionFromReader(
					reader,
					configuration,
					postfix_ternary_operator.precedence,
					postfix_ternary_operator.parts[2]
				);

				if (reader.startsWith(postfix_ternary_operator.parts[2])) {
					reader.advance(postfix_ternary_operator.parts[2].length);
				} else {
					throw new Error("expected " + postfix_ternary_operator.parts[2]);
				}

				top = {
					on: postfix_ternary_operator.name,
					arguments: [top, lhs, rhs]
				};
			} else {
				const substituteBinaryOperator: BinaryOperator | undefined = configuration
					.binary_operators
					.find(operator => operator.representation === postfix_ternary_operator!.parts[0]);

				if (substituteBinaryOperator) {
					if (returnPrecedence > substituteBinaryOperator.precedence) {
						throw new Error("binary operator greater than ternary");
					}

					top = { on: substituteBinaryOperator.representation, arguments: [top, lhs] }
				} else {
					throw new Error("Expected " + postfix_ternary_operator.parts[1]);
				}
			}
		} else if (binary_operator = configuration
			.binary_operators
			.find((operator) => reader.startsWith(operator.representation))) {
			if (returnPrecedence > binary_operator.precedence) {
				return top;
			}

			reader.advance(binary_operator.representation.length);
			const lhs = top;
			const rhs = parseExpressionFromReader(
				reader,
				configuration,
				binary_operator.precedence,
				breakBefore
			);
			top = {
				on: binary_operator.representation,
				arguments: [lhs, rhs]
			};
		} else if (unary_operator = configuration
			.postfix_unary_operators
			.find((operator) => reader.startsWith(operator.representation))) {
			if (returnPrecedence > unary_operator.precedence) {
				return top;
			}

			reader.advance(unary_operator.representation.length);
			top = { on: unary_operator.representation, arguments: [top] }
		} else if (reader.startsWith("(") && configuration.adjacency) {
			const operator = configuration.adjacency.operator;

			if (returnPrecedence > operator.precedence) return top;

			const rhs = parseExpressionFromReader(
				reader,
				configuration,
				operator.precedence,
				breakBefore
			);

			top = {
				on: operator.representation,
				arguments: [top, rhs]
			};
		} else {
			break;
		}
	}
	return top
}

function splitNumber(on: string): [string, string] {
	for (let i = 0; i < on.length; i++) {
		const code = on.charCodeAt(i);
		let numberLike = '0'.charCodeAt(0) <= code && code <= '9'.charCodeAt(0);  
		numberLike ||= code === '.'.charCodeAt(0);
		if (!numberLike) return [on.slice(0, i), on.slice(i)] 
	}
	return [on, ""]
}

export function printExpression(expression: Expression): string {
	if (expression.arguments.length) {
		let s = "(";
		s += expression.on;
		for (const argument of expression.arguments) {
			s += " ";
			s += printExpression(argument);
		}
		s += ")";
		return s
	} else {
		return expression.on
	}
}
