import { defaultConfiguration, parseExpression } from "./index.ts";
import type { Configuration, Expression } from "./index.ts";

const configuration: Configuration = {
	...defaultConfiguration(),
	binary_operators: [
		{ precedence: 1, representation: "+" },
		{ precedence: 1, representation: "-" },
		{ precedence: 2, representation: "*" },
		{ precedence: 2, representation: "/" },
	],
	prefix_unary_operators: [
		{ precedence: 1, representation: "-" },
	],
};

const examples = [
	"x * y",
	"x * y + 2",
	"x + y * 2",
	"(x * y) + 5 + (x - y)",
	"x + sin(y)"
];

const x = 5, y = -2;

console.log({ x, y })

for (const example of examples) {
	console.log(`--- ${example} ---`)
	const expression = parseExpression(example, configuration);
	// console.dir(expression, { depth: Number.POSITIVE_INFINITY });
	console.log(evaluate(expression));
}

function evaluate(expression: Expression): number {
	if (expression.arguments.length == 0) {
		if (expression.on === "x") return x;
		if (expression.on === "y") return y;
		return parseFloat(expression.on)
	} else if (expression.arguments.length == 1) {
		const operand = evaluate(expression.arguments[0]);
		if (expression.on === "-") return -operand;
		if (expression.on === "sin") return Math.sin(operand);

		throw new Error("Unknown operator " + expression.on);
	} else if (expression.arguments.length == 2) {
		const lhs = evaluate(expression.arguments[0]), rhs = evaluate(expression.arguments[1]);
		if (expression.on === "+") return lhs + rhs;
		if (expression.on === "-") return lhs - rhs;
		if (expression.on === "*") return lhs * rhs;
		if (expression.on === "/") return lhs / rhs;

		throw new Error("Unknown operator " + expression.on);
	} else {
		throw new Error("Unknown expression");
	}
}
