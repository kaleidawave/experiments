import { WebSocketServer } from 'ws';
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { evaluate } from './evaluate.js';

const server = createServer();

const wss = new WebSocketServer({ server });

const connections = new Set;

const columnCount = 4;
const rowCount = 10;

const tableValues = Array.from({ length: rowCount }, () => Array.from({ length: columnCount }))
const tableExpressions = new Map();

wss.on('connection', function connection(ws) {
	ws.on('error', console.error);

	ws.on("close", () => {
		connections.delete(ws);
	})

	ws.on('message', function message(data) {
		// TODO sanitisation. What if message contains `:` etc
		const [command, position, message] = data.toString().split(":");

		const [xs, ys] = position.split(",");
		const x = parseInt(xs);
		const y = parseInt(ys);

		if (command === "update") {
			let expression = null;
			let out = message;
			if (message.startsWith("=")) {
				expression = message.slice(1);
				// Evaluate it non-locally
				out = evaluate(expression, tableValues).toString();
				broadcastMessage(`evaluation:${position}:${expression}=${out}`);
			} else {
				tableExpressions.delete(position);
				broadcastMessage(`update:${position}:${message}`, ws);
			}

			tableValues[y][x] = out;

			for (const [position, expression] of tableExpressions) {
				// TODO recursive
				const newEvaluation = evaluate(expression, tableValues).toString();
				broadcastMessage(`evaluation:${position}:${expression}=${newEvaluation}`);
			}

			if (expression) tableExpressions.set(position, expression);
		}

		console.log({ command, position, message, tableValues, tableExpressions });
	});

	connections.add(ws)
});

function broadcastMessage(message, thisConnection = null) {
	for (const connection of connections) {
		if (connection == thisConnection) {
			continue;
		}
		connection.send(message);
	}
}

server.on("request", (_req, res) => {
	res.writeHead(200, { 'Content-Type': 'text/html' });
	res.end(readFileSync("./index.html"));
});

const port = 8080;
server.listen(port);
console.log(`live on http://localhost:${port}`)