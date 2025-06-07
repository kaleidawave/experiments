let both = run node --eval "console.log('out');console.error('err');" --merge-stdout-and-stderr
echo "result: both='$both'\n"

let stdout = run node --eval "console.log('out');console.error('err');"
echo "result: stdout='$stdout'\n"

let stderr = run node --eval "console.log('out');console.error('err');" --only-capture-stderr
echo "result: stderr='$stderr'\n"

echo "result: echo run"
echo run node --eval "console.log('out');console.error('err');"