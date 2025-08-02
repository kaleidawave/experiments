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