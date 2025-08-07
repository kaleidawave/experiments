import { BinaryOperator, Configuration, parseExpression, printExpression } from "./index";
import { readFile } from "node:fs/promises";

if (process.argv.includes("--interactive")) {
	console.log("start");
	let buffer = "";
	for await (const line of console) {
		if (line == "close") break;

		if (line == "end") {
			const [configuration, input] = extractConfigurationAndSource(buffer);
			const expression = parseExpression(input, configuration);
			console.log(printExpression(expression));
			console.log("end");
			buffer = "";
			continue
		}

		buffer += line;
		buffer += "\n";
	}

	// console.log("finished");
} else {
	const firstArgument = process.argv[2];
	if (firstArgument) {
		const buffer = await readFile(firstArgument);
		const [configuration, input] = extractConfigurationAndSource(buffer.toString());
		const expression = parseExpression(input, configuration);
		console.log(configuration);
		console.log(printExpression(expression));
	} else {
		const buffer = "(x (a b) (c d))";
		const [configuration, input] = extractConfigurationAndSource(buffer);
		const expression = parseExpression(input, configuration);
		console.log(printExpression(expression));
	}
}

function extractConfigurationAndSource(input: string): [Configuration, string] {
	const configuration: Configuration = {
		adjacency: null,
		binary_operators: [],
		prefix_ternary_operators: [],
		postfix_ternary_operators: [],
		postfix_unary_operators: [],
		prefix_unary_operators: [],
	};

	if (input.includes("\n---")) {
		const [cfg, source] = input.split("\n---");

		for (let line of cfg.split("\n")) {
			let adjacency = false;
			if (line.endsWith(" (adjacent)")) {
				adjacency = true;
				line = line.slice(0, - " (adjacent)".length);
			}

			let name = "", precedence = 1;
			{
				const parts = line.split(":");
				if (parts.length > 1 && parts[0].trimEnd().match(/^([a-zA-Z0-9-_]+)$/m)) {
					name = parts[0].trimEnd()
					line = line.slice(parts[0].length + 1);
				}
			}

			{
				const parts = line.split("#");
				if (parts.length > 1) {
					// @ts-ignore
					const item: string = parts.at(-1);
					precedence = parseInt(item);
					line = line.slice(0, -(item.length + 1))
				}
			}

			const parts = line.split("_").map(part => part.trim());

			if (parts.length === 4) {
				if (parts[0] === "") {
					// @ts-ignore
					const items: [string, string, string] = parts.slice(1, 4);
					configuration.postfix_ternary_operators.push({
						name,
						parts: items,
						precedence
					});
				} else {
					// @ts-ignore
					const items: [string, string, string] = parts.slice(0, 3);
					configuration.prefix_ternary_operators.push({
						name,
						parts: items,
						precedence
					});
				}
			} else if (parts.length === 3) {
				const representation = parts[1];
				const operator: BinaryOperator = { representation, precedence };
				if (adjacency) configuration.adjacency = { operator, functions: [] };
				else configuration.binary_operators.push(operator);
			} else if (parts.length === 2) {
				if (parts[0] === "") {
					const representation = parts[1];
					configuration.postfix_unary_operators.push({
						precedence,
						representation
					});
				} else {
					const representation = parts[0];
					configuration.prefix_unary_operators.push({
						precedence,
						representation
					});
				}
			} else {
				throw new Error("Unknown operator");
			}
		}

		return [configuration, source.trim()];
	} else {
		return [configuration, input.trim()];
	}
}