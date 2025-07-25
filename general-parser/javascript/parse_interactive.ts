import { Configuration, parseExpression, printExpression } from "./index";

console.log("start");
let buffer = "";
for await (const line of console) {
	if (line == "close") break;

	if (line == "end") {
		const configuration: Configuration = {
			adjacency: null,
			binary_operators: [],
			postfix_unary_operators: [],
			prefix_unary_operators: [],
		};

		if (buffer.includes("\n---")) {
			const [cfg, source] = buffer.split("\n---");

			for (let line of cfg.split("\n")) {
				let adjacency = false;
				if (line.endsWith(" (adjacent)")) {
					adjacency = true;
					line = line.slice(0, - " (adjacent)".length);
				}

				const [syntax, precedenceString] = line.split(" ");

				const precedence = parseInt(precedenceString);

				const prefix = syntax.startsWith("_");
				const postfix = syntax.endsWith("_");
				if (prefix && postfix) {
					const representation = syntax.slice(1, -1);
					if (adjacency) {
						configuration.adjacency = { operator: { representation, precedence }, functions: [] }
					} else {
						configuration.binary_operators.push({ representation, precedence })
					}
				} else if (prefix) {
					const representation = syntax.slice(1);
					configuration.prefix_unary_operators.push({ representation, precedence })
				} else if (postfix) {
					const representation = syntax.slice(0, -1);
					configuration.postfix_unary_operators.push({ representation, precedence })
				}
			}

			buffer = source;
		}

		const expression = parseExpression(buffer, configuration);
		console.log(printExpression(expression));
		console.log("end");
		buffer = "";
		continue
	}

	buffer += line;
	buffer += "\n";
}

// console.log("finished");