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

import { generateHistogram, generatePieChart, generateScatterplot } from "./charts.mjs";

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
	slot.innerHTML = "";
	switch (document.querySelector("#inner").getAttribute("data-view")) {
		case "pie-chart": {
			const index = form.elements["key"].value;
			const selection = Object.entries(data).map(([name, object]) => ({
				name, value: retrieveKey(object, index)
			}));
			const chart = generatePieChart(selection, index);
			slot.append(chart);
			break;
		}
		case "scatterplot": {
			const xIndex = form.elements["x-key"].value;
			const yIndex = form.elements["y-key"].value;
			const selection = Object.entries(data).map(([name, object]) => ({
				name,
				x: retrieveKey(object, xIndex),
				y: retrieveKey(object, yIndex)
			}));
			const chart = generateScatterplot(selection, xIndex, yIndex);
			slot.append(chart);
			break;
		}
		case "histogram": {
			const index = form.elements["key"].value;
			const selection = Object.entries(data).map(([name, object]) => ({
				name, value: retrieveKey(object, index)
			}));
			const chart = generateHistogram(selection, index);
			slot.append(chart);
			break
		}
	}
}
