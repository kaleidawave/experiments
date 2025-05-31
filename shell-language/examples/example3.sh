let out = run bun run ./private/script.js
let out = rafter $out "Something"
let out = repeat $out 3
echo Recieved $out
