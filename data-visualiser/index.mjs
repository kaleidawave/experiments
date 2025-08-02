document.addEventListener("click", (ev) => {
	if (ev.target.hasAttribute("data-upload")) {
		openJSONFile().then(file => registerFile(file))
	}
});

document.body.addEventListener("dragover", (ev) => {
	if (ev.target.closest("#upload")) {
		// prevent default to allow file drop
		ev.preventDefault();
	}
});

document.body.addEventListener("drop", (ev) => {
	if (ev.target.closest("#upload")) {
		ev.preventDefault();
		const files = ev.dataTransfer.files;
		if (files.length) {
			registerFile(files.item(0))
		} else {
			alert("no files in drop")
		}
	}
});

addEventListener("paste", (ev) => {
	const files = ev.clipboardData.files;
	if (files.length) {
		registerFile(files.item(0))
	} else {
		ev.clipboardData.items[0].getAsString(registerContent)
	}
});

addEventListener("hashchange", (ev) => {
	if (document.body.firstElementChild.getAttribute("data-view") === "dashboard") {
		document.querySelector("#inner").setAttribute("data-view", new URL(ev.newURL).hash.slice(1));
	}
});

document.body.addEventListener("change", specifierChange);

// ---

/** Returns a `Promise` that resolves to a  */
function openJSONFile() {
	const input = document.createElement("input");
	input.type = "file";
	input.setAttribute("accept", "application/json");
	const result = new Promise((res, _rej) => input.addEventListener("change", () => {
		const [file] = input.files;
		res(file);
	}));
	input.click();
	return result
}

function registerFile(file) {
	const name = file.name;
	const text = file.text();
	text.then(text => registerContent(text, name))
}

function registerContent(text, fileName = "") {
	function appendKeys(object, keys, prefix = "") {
		for (const key in object) {
			if (typeof object[key] === "object") {
				appendKeys(object[key], keys, prefix + key + "/")
			} else {
				keys.push(prefix + key)
			}
		}
	}

	data = JSON.parse(text);
	keys = [];

	// TODO assumes structure of data
	appendKeys(Object.values(data)[0], keys);

	console.log(data, keys)

	{
		document.body.querySelector("[data-content='file-name']").innerText = fileName;
		document.body.querySelectorAll("select[name$='key']").forEach(select => {
			for (const key of keys) {
				const option = document.createElement("option");
				option.value = key;
				option.innerText = key;
				select.append(option);
			}
		});
	}

	// Switch to dashboard view
	document.body.firstElementChild.setAttribute("data-view", "dashboard")
}

// ...

let data = null;
let keys = null;

function retrieveKey(object, key) {
	for (const part of key.split("/")) {
		if (part) object = object[part];
	}
	return object
}

function specifierChange(ev) {
	const form = ev.target.closest("form");
	const slot = form.nextElementSibling;
	console.log("here", slot)
	slot.innerHTML = "";
	switch (document.querySelector("#inner").getAttribute("data-view")) {
		case "pie-chart": {
			const index = form.elements["key"].value;
			const selection = Object.entries(data).map(([name, object]) => ({
				name, value: retrieveKey(object, index)
			}));
			const chart = generatePieChart(selection);
			slot.append(chart);
			break;
		}
		case "scatterplot":
			throw Error("TODO")
		case "histogram":
			throw Error("TODO")
	}
}

// ---

function generatePieChart(data) {
	const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
	svg.classList.add("pie-chart");
	svg.setAttribute("viewBox", "-3 -3 206 206");
	svg.setAttribute("xmlns", "http://www.w3.org/2000/svg");

	let sum = 0;
	for (const item of data) sum += item.value;

	const colours = "#2b206c,#1158e3,#d3444a,#098e9f,#e8b546".split(",").reverse()
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