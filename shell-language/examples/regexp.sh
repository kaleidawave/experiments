let source = constant "Hello Ben. Does this regular expression work"
let name = regexp $source "Hello (?<name>.+?)\\b" extract name
echo $name