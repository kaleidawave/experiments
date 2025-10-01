const editor = document.querySelector("#editor");

const pickerOptions = {
	types: [
		{
			description: "text documents",
			accept: {
				"image/*": [".txt", ".md"],
			},
		},
	],
	excludeAcceptAllOption: true,
	multiple: false,
};

let fileName = "";

function markDirty() {
	document.querySelector("#dirty").removeAttribute("hidden")
}

function markClean() {
	document.querySelector("#dirty").setAttribute("hidden", "")
}

async function writeBanner(text) {
	const banner = document.querySelector("#banner");
	banner.removeAttribute("hidden");
	banner.innerText = text;
	const keyframes = [{ opacity: 1, offset: 0.7 }, { opacity: 0 }];
	const animation = banner.animate(keyframes, 2000);
	await new Promise((res, _rej) => animation.onfinish = res);
	banner.setAttribute("hidden", "");
}

async function saveFile() {
	const file = await showSaveFilePicker({ ...pickerOptions, suggestedName: fileName });
	document.querySelector("#currently-editing").innerText = (fileName = file.name);
	const writableStream = await file.createWritable();
	await writableStream.write(editor.value);
	await writableStream.close();
	markClean();
	writeBanner(`saved ${fileName}`)
}

async function openFile() {
	const [result] = await showOpenFilePicker(pickerOptions);
	const file = await result.getFile();
	document.querySelector("#currently-editing").innerText = (fileName = file.name);
	const content = await file.text();
	editor.value = content;
	markClean();
	editor.addEventListener("input", (_ev) => markDirty(), { once: true });
	writeBanner(`loaded ${fileName}`)
}

editor.addEventListener("input", (_ev) => markDirty(), { once: true });

document.body.addEventListener("click", async (ev) => {
	const { id } = ev.target;
	try {
		if (id === "load") {
			await openFile()
		} else if (id === "save") {
			await saveFile()
		} else if (id === "clear") {
			editor.value = "";
			markClean();
			writeBanner(`cleared`)
		} else if (id === "copy") {
			await navigator.clipboard.writeText(editor.value);
			writeBanner("copied")
		} else if (id === "paste") {
			const content = await navigator.clipboard.readText();
			editor.value = content;
			markDirty();
			writeBanner("pasted");
		}
	} catch (error) {
		writeBanner("error (check console)")
		console.error(error);
	}
});

let editorFontSize = 12;
const lineHeightRatio = 1.5;

document.addEventListener("keydown", async (ev) => {
	if (ev.target === editor) {
		const pm = ev.key === "=" || ev.key === "-";
		if (pm && ev.ctrlKey) {
			editorFontSize += (ev.key === "=" ? 2 : -2);
			editor.style.setProperty('--font-size', `${editorFontSize}px`);
			editor.style.setProperty('--line-height', `${editorFontSize * lineHeightRatio}px`);
			ev.preventDefault();
		}
	}

	if (ev.key === "o" && ev.ctrlKey) {
		ev.preventDefault();
		document.querySelector("#load").click();
	} else if (ev.key === "s" && ev.ctrlKey) {
		ev.preventDefault();
		document.querySelector("#save").click();
	}
})