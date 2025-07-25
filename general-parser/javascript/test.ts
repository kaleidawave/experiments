import { parseExpression } from "./index.ts";
import type { Configuration, Expression } from "./index.ts";

const configuration: Configuration = {
	binary_operators: [
		{ precedence: 1, representation: "+" },
		{ precedence: 1, representation: "-" },
		{ precedence: 2, representation: "*" },
		{ precedence: 2, representation: "/" },
	],
	unary_operators: [
		{ precedence: 1, representation: "-", prefix: true },
	],
	identifier_prefixes: []
};

const examples = [
	"x * y",
	"x * y + 2",
	"x + y * 2",
	"(x * y) + 5 + (x - y)",
	"x + sin(y)"
];

for (const example of examples) {
	console.log(`--- ${example} ---`)
	const expression = parseExpression(example, configuration);
	// console.dir(expression, { depth: Number.POSITIVE_INFINITY });
	console.log(evaluate(expression));
}

function evaluate(expression: Expression): number {
	if ("name" in expression) {
		if (expression.name === "x") {
			return 5
		} else if (expression.name === "y") {
			return -2
		} else {
			return parseFloat(expression.name)
		}
	} else if ("values" in expression) {
		return evaluate(expression.values[0])
	} else if ("function" in expression) {
		if ("name" in expression.function) {
			const func = expression.function.name;
			const operand = evaluate(expression.argument);
			if (func === "+") return +operand;
			else if (func === "-") return -operand;
			else if (func === "sin") return Math.sin(operand);
			throw new Error(`Unknown operator ${func}`);
		} else if ("function" in expression.function) {
			if ("name" in expression.function.function) {
				const name = expression.function.function.name;
				const lhs = evaluate(expression.function.argument);
				const rhs = evaluate(expression.argument);
				if (name === "+") return lhs + rhs;
				else if (name === "-") return lhs - rhs;
				else if (name === "*") return lhs * rhs;
				else if (name === "/") return lhs / rhs;
				throw new Error(`Unknown operator ${name}`);
			} else {
				throw new Error("Unimplemented calling non-named function");
			}
		} else {
			throw new Error("Unimplemented calling non-named function");
		}
	} else {
		throw new Error("Unreachable");
	}
}
