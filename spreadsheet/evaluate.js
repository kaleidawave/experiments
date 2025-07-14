import { parseExpression } from "@bengineering/mathematics-parser";

const configuration = {
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

export function evaluate(command, table) {
	const expression = parseExpression(command, configuration)
	return evaluateExpression(expression, table)
}

function evaluateExpression(expression, table) {
	if ("name" in expression) {
		const name = expression.name;
		if ("ABCD".includes(name.slice(0, 1))) {
			const column = name.charCodeAt(0) - "A".charCodeAt(0);
			// TODO more bounds checking
			const row = Math.max(parseInt(name.slice(1)) - 1, 0);
			const item = table[row][column] ?? "";
			const value = parseFloat(item);
			if (!Number.isNaN(value)) {
				return value;
			} else {
				return 0;
			}
		} else {
			return parseFloat(name)
		}
	} else if ("values" in expression) {
		return evaluateExpression(expression.values[0], table)
	} else if ("function" in expression) {
		if ("name" in expression.function) {
			const func = expression.function.name;
			const operand = evaluateExpression(expression.argument, table);
			if (func === "+") return +operand;
			else if (func === "-") return -operand;
			else if (func === "sin") return Math.sin(operand);
			throw new Error(`Unknown operator ${func}`);
		} else if ("function" in expression.function) {
			if ("name" in expression.function.function) {
				const name = expression.function.function.name;
				const lhs = evaluateExpression(expression.function.argument, table);
				const rhs = evaluateExpression(expression.argument, table);
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