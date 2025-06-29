const args = process.argv.slice(2);
const value = args[0];
if (args[1] === "--uppercase") console.log(value.toUpperCase());
else console.log(value);