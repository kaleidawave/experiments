function generateRange(data, count, index = "") {
	function retrieveKey(object, key) {
		for (const part of key.split("/")) {
			if (part) object = object[part];
		}
		return object
	}

	function minMax(data, key = "") {
		const initial = data[0];
		let min = retrieveKey(initial, key), max = retrieveKey(initial, key);
		for (let i = 1; i < data.length; i++) {
			min = Math.min(min, retrieveKey(data[i], key));
			max = Math.max(max, retrieveKey(data[i], key));
		}
		return [min, max]
	}

	function quadrant(value, scale) {
		return Math.trunc(4 * value / Math.max(scale, 1))
	}

	function ceil10(value) {
		return 10 ** Math.ceil(Math.log10(value))
	}

	function range(start, end, count) {
		const inc = (end - start) / (count - 1);
		return Array.from({ length: count }, (_, idx) => start + idx * inc);
	}

	const [_min, max] = minMax(data, index);
	const c10 = ceil10(max);
	const dataRangeMax = c10 * 0.25 * Math.min(quadrant(max, c10) + 1, 4);

	return range(0, dataRangeMax, count);
}

export function generatePieChart(data) {
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.classList.add("pie-chart");
	svg.setAttribute("viewBox", "-3 -3 206 206");
	svg.setAttribute("xmlns", "http://www.w3.org/2000/svg");

	let sum = 0;
	for (const item of data) sum += item.value;

	const colours = "#2b206c,#1158e3,#d3444a,#098e9f,#e8465c".split(",").reverse()
	let i = 0;

	const radius = 100;

	const TAU = Math.PI * 2;
	const HALF_PI = Math.PI / 2;

	let accPercent = 0;
	let [prevX, prevY] = [100, 0];
	for (const item of data) {
		if (item.value === 0) continue

		const arc = document.createElementNS("http://www.w3.org/2000/svg", "path");

		const percent = item.value / sum;
		accPercent += percent;

		const toX = radius * Math.cos((accPercent * TAU) - HALF_PI) + 100;
		const toY = radius * Math.sin((accPercent * TAU) - HALF_PI) + 100;

		const sweep = percent > 0.5 ? 1 : 0;
		const path = `M ${prevX} ${prevY} A ${radius} ${radius} 0 ${sweep} 1 ${toX} ${toY} L 100 100 Z`;

		arc.setAttribute("d", path);
		arc.style.fill = colours[i++ % colours.length];
		arc.style.setProperty("--accent", arc.style.fill);

		const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
		text.append(item.name);
		text.setAttribute("x", 0.8 * radius * Math.cos(((accPercent - percent / 2) * TAU) - HALF_PI) + 85);
		text.setAttribute("y", 0.8 * radius * Math.sin(((accPercent - percent / 2) * TAU) - HALF_PI) + 100);

		svg.append(arc, text);
		[prevX, prevY] = [toX, toY];
	}
	return svg
}

export function generateScatterplot(data) {
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.classList.add("scatterplot");
	svg.setAttribute("viewBox", "-30 0 275 275");
	svg.setAttribute("xmlns", "http://www.w3.org/2000/svg");

	const ticks = 6;
	const xTicks = generateRange(data, ticks, "x");
	const yTicks = generateRange(data, ticks, "y");

	const xScale = 200 / xTicks.at(-1);
	const yScale = 200 / yTicks.at(-1);
	const xShift = data.some(data => data.x === 0) ? 20 : 0;
	const yShift = data.some(data => data.y === 0) ? 20 : 0;

	const yOffset = 250;

	const tickScale = 40;

	const mainColour = "#e8465c", pointColour = "#d3444a", tickColour = "#dfdfdf";

	// xAxis
	{
		const line = generateLine(0, 200 + xShift, yOffset, yOffset);
		line.style.stroke = mainColour;
		svg.append(line);
	}

	// xTicks
	for (let i = 0; i < ticks; i++) {
		{
			const line = generateLine(xShift + i * tickScale, xShift + i * tickScale, yOffset, yOffset + 3);
			line.style.stroke = tickColour;

			svg.append(line);
		}
		{
			const label = generateText(xShift + i * tickScale, yOffset + 14, xTicks[i]);
			label.style.color = tickColour;
			label.style.textAnchor = "middle";
			svg.append(label);
		}
	}

	// yAxis
	{
		const line = generateLine(0, 0, yOffset, yOffset - 200 - yShift);
		line.style.stroke = mainColour;
		svg.append(line);
	}

	// yTicks
	for (let i = 0; i < ticks; i++) {
		const y = yOffset - i * tickScale - yShift;
		{
			const line = generateLine(-3, 0, y, y);
			line.style.stroke = tickColour;
			svg.append(line);
		}
		{
			const label = generateText(-5, y + 3, yTicks[i]);
			label.style.color = tickColour;
			label.style.textAnchor = "end";
			svg.append(label);
		}
	}

	for (const item of data) {
		const px = item.x * xScale + xShift, py = (yOffset - item.y * yScale) - yShift;
		const line1 = generateLine(
			px, px, py + 2, py - 2
		);
		line1.style.stroke = pointColour;
		line1.style.strokeWidth = "1px";

		const line2 = generateLine(
			px - 2, px + 2, py, py
		);
		line2.style.stroke = pointColour;
		line2.style.strokeWidth = "1px";

		const label = generateText(px + 2, py - 2, item.name.toString())
		label.classList.add("point-label");
		label.style.color = tickColour;

		svg.append(line1, line2, label);
	}

	return svg
}

export function generateHistogram(data) {
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.classList.add("histogram");
	svg.setAttribute("viewBox", "-40 0 300 275");
	svg.setAttribute("xmlns", "http://www.w3.org/2000/svg");

	const ticks = 6;
	const xTicks = generateRange(data, ticks, "value");

	const bins = Array.from({ length: ticks }, () => 0);
	for (const item of data) {
		const idx = Math.trunc(5 * item.value / xTicks.at(-1));
		++bins[idx];
	}

	const yTicks = generateRange(bins, ticks);
	const yOffset = 240;

	const tickScale = 40;

	const mainColour = "#e8465c", tickColour = "#dfdfdf";

	// xAxis
	{

		const line = generateLine(0, 200, yOffset, yOffset);
		line.style.stroke = mainColour;
		svg.append(line);
	}

	// xTicks
	for (let i = 0; i < ticks; i++) {
		{
			const line = generateLine(i * tickScale, i * tickScale, yOffset, yOffset + 3);
			line.style.stroke = tickColour;

			svg.append(line);
		}
		{
			const label = generateText(i * tickScale, yOffset + 14, xTicks[i]);
			label.style.color = tickColour;
			label.style.textAnchor = "middle";
			svg.append(label);
		}
	}

	// yAxis
	{
		const line = generateLine(0, 0, yOffset, yOffset - 200);
		line.style.stroke = mainColour;
		svg.append(line);
	}

	// yTicks
	for (let i = 0; i < ticks; i++) {
		const y = yOffset - i * tickScale;
		{
			const line = generateLine(-3, 0, y, y);
			line.style.stroke = tickColour;
			svg.append(line);
		}
		{
			const label = generateText(-5, y + 3, yTicks[i]);
			label.style.color = tickColour;
			label.style.textAnchor = "end";
			svg.append(label);
		}
	}

	for (let i = 0; i < ticks; i++) {
		const height = 200 * (bins[i] / yTicks.at(-1));
		const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
		rect.setAttribute("x", i * tickScale);
		rect.setAttribute("y", yOffset - height);
		rect.setAttribute("width", tickScale);
		rect.setAttribute("height", height);
		rect.style.fill = mainColour;
		svg.append(rect);
	}

	return svg
}

function generateText(x, y, name) {
	const label = document.createElementNS("http://www.w3.org/2000/svg", "text");
	label.innerHTML = name;
	label.setAttribute("x", x);
	label.setAttribute("y", y);
	label.style.fontFamily = "Inter";
	label.style.fontSize = "8px";
	return label
}

function generateLine(x1, x2, y1, y2) {
	const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
	line.setAttribute("x1", x1);
	line.setAttribute("x2", x2);
	line.setAttribute("y1", y1);
	line.setAttribute("y2", y2);
	line.style.strokeWidth = "1px";
	return line
}
