import { BinaryOperator, Configuration, defaultConfiguration, parseExpression, parseExpressionPartial, printExpression } from "./index";
import { readFile } from "node:fs/promises";

if (process.argv.includes("--interactive")) {
	console.log("start");
	let buffer = "";
	for await (const line of console) {
		if (line == "close") break;

		if (line == "end") {
			const [partial, configuration, input] = extractConfigurationAndSource(buffer);
			if (partial) {
				const [expression, parsed] = parseExpressionPartial(input, configuration, partial.before);
				// TODO conversion here
				console.log(`parsed ${parsed} bytes`);
				console.log(printExpression(expression));
			} else {
				const expression = parseExpression(input, configuration);
				console.log(printExpression(expression));
			}
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
		const [partial, configuration, input] = extractConfigurationAndSource(buffer.toString());
		console.log(configuration);
		if (partial) {
			const [expression, parsed] = parseExpressionPartial(input, configuration, partial.before);
			// TODO conversion here
			console.log(`parsed ${parsed} bytes`);
			console.log(printExpression(expression));
		} else {
			const expression = parseExpression(input, configuration);
			console.log(printExpression(expression));
		}
	} else {
		const buffer = "(x (a b) (c d))";
		const [_partial, configuration, input] = extractConfigurationAndSource(buffer);
		const expression = parseExpression(input, configuration);
		console.log(printExpression(expression));
	}
}

type Partial = { before: string | null } | null;

function extractConfigurationAndSource(input: string): [Partial, Configuration, string] {
	const configuration: Configuration = defaultConfiguration();

	if (input.includes("\n---")) {
		const [cfg, source] = input.split("\n---");
		let partial: Partial = null;

		for (let line of cfg.split("\n")) {
			let adjacency = false;

			if (line.startsWith("partial")) {
				const next = line.slice("partial".length).trim();
				const before = next.startsWith("upto ") ? next.slice("upto ".length) : null;
				partial = { before };
				continue;
			}

			if (line.endsWith(" (function)")) {
				const func = line.slice(0, - " (function)".length);
				if (!configuration.adjacency) throw new Error("Adjacency needed to register functions");
				configuration.adjacency.functions.push(func);
				continue;
			}

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

		return [partial, configuration, source.trim()];
	} else {
		return [null, configuration, input.trim()];
	}
}
