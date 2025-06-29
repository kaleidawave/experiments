const uppercase = process.argv[2] === "--uppercase";

// console.log(uppercase, process.argv);

console.log("start");
let s = "";
for await (const line of console) {
	if (line == "close") break;

	if (line == "end") {
		if (uppercase) console.log(s.toUpperCase());
		else console.log(s);
		console.log("end");
		s = "";
		continue
	}

	s += line;
	s += "\n";
}

console.log("finished");
