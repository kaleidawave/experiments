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

##### Glob matching

##### RegExp matching

##### RegExp extraction

## Control flow

> Stuff here is WIP

### Loops

Based on new lines

### Conditionals

#### `if_equal`

```sh
let condition = literal test
```

```
...
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
