## Specification

This defines what the shell language supports

### Commands

#### echo

```sh
echo hello
```

```
hello
```

##### echo stderr

```sh
echo_stdout hello from stdout
echo_stderr hello from stderr
```

```
hello from stdout
*hello from stderr*
```

#### Filesystem

##### Read

```sh
let content = read README.md
echo $content
```

```
A bash like language that is simpler
```

##### Write

```sh
let content = literal Hello
write private/out $content

let read = read private/out
echo $read
```

```
Hello
```

### Run

### Variables

> Variables are declared using `let` and can be referenced by prefixing items with `$`

#### `#ctx`

#### Environment variable inheritance

#### Name overwriting

```sh
let x = literal 5
let x = literal 6
echo $x
```

```
6
```

### Strings

#### Repeat

```sh
let input = literal "t"
let repeated = repeat $input 5
echo $repeated
```

```
ttttt
```

#### Replace

> Straightforward substring replace

```sh
let input = literal "Hello Ben"
let new = replace $input "Hello" "Hiya"
echo $new
```

```
Hiya Ben
```

#### Lines

```sh
let input = literal "\n\n\n\n"
let lines = lines $input
echo $lines
```

```
4
```

#### Before and after

> Prefixing with `r` means the search is the first from the right (as opposed from the left)

```sh
let input = literal "something - here - and here"
let b4 = before $input "-"
let aft = after $input "-"
echo "'$b4'"
echo "'$aft'"

let raft = rafter $input "-"
echo "'$raft'"
```

```
'something '
' here - and here'
' and here'
```

#### Matching and extraction

##### RegExp matching

```sh
let source = constant "Invalid match"
let result = regexp $source "Hello (.+?)"
echo $result
```

```
Ben
```

##### RegExp extraction

```sh
let source = constant "Hello Ben. Does this regular expression work?"
let name = regexp $source "Hello (?<name>.+?)\\b" extract name
echo $name
```

```
Ben
```

## Control flow

> Stuff here is WIP

### Loops

Based on new lines

### Conditionals

```sh
let x = "hi"
if literal $x then
	echo "found hi"

set x = ""
if literal $x then
	echo "found hello"
```

```
found hi
```

#### `if_equal`

```sh
let condition = literal test
let result = if_equal $condition "test" "is test" "not test"
echo $result

let result = if_equal $condition "asd" "is asd" "not asd"
echo $result
```

```
is test
not asd
```

#### conditional commands

```sh
what
```

```
???
```

### Pipe

```sh
literal "four" then repeat 5 then size then echo 
```

```
20
```

### Pipe reference using argument

```sh
literal "four" then echo "Command returned $last"
```

```
Command returned four
```

### Iteration

```sh
let lines = literal "hello\nworld"
for literal $lines each
	echo "line: $iter"
```

```
line: hello
line: world
```

### Iteration with break

```sh
let lines = literal "hello\nworld\nanother\nline\nhere"
for literal $lines each
	echo "line: $iter"
	set break = if_equal $iter "another" break ""
```

```
line: hello
line: world
line: another
```

## Syntax

### Backslash line continuations

```sh
echo "item 1" \
	"item 2"
```

```
item 1 item 2
```

### `then` line continuations

```sh
literal "four" then
	echo "Command returned $last"
```

```
Command returned four
```
