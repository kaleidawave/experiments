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

#### run

```sh
let output = run node --print "5341"
echo $output
```

```
5341
```

#### echo run

```sh
echo run node --print "5934"
```

```
5934
```

#### with

```sh
let my_value = literal 2812
let output = with my_value $my_value run node --print "process.env.my_value"
echo $output
```

```
2812
```

#### echo merge stdout and stderr

```sh
let output = run node --eval "console.log('ok');console.error('err');" --merge-stdout-and-stderr
echo "Recieved $output"
```

```
Recieved ok
err
```

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
echo 'Recieved $exit_code "$result""
```

```
Recieved 1 ""
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
let x = constant "hi"
if literal $x
	echo "found hi"

set x = constant ""
if literal $x
	echo "found hello"
```

```
found hi
```

#### `if_equal`

```sh
let condition = constant "test"
let result = if_equal $condition "test" "is test" "not test"
echo $result

let result = if_equal $condition "asd" "is asd" "not asd"
echo $result
```

```
is test
not asd
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

### Nested iteration

```sh
let char1s = concatenate a b c
for constant $char1s each
	let char2s = concatenate 1 2 3
	for constant $char2s each
		echo "Found $char1 $char2"
	echo "---"
```

> This is a test for parsing as well. Iteration can introduce variables based on dropping the final 's'

```
Found a 1
Found a 2
Found a 3
---
Found b 1
Found b 2
Found b 3
---
Found c 1
Found c 2
Found c 3
---
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
