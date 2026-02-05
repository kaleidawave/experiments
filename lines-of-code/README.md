### Lines of code

Counts lines of codes as well as certain declarations. Visits paths and prints result to JSON

> Currently this is designed for Rust, but could support more languages in the future

TODO

- examples
- check tests
- count parameters and destructured variables?
- count properties and variants?
- lines that are just `}`
- largest indent?

Lines

- all
- non-whitespace
- non-whitespace and `}` etc
- non-whitespace, comments and `}` etc

others: derive macros, doc-comments

## Tests

### Basic

> empty lines not counted,

```rust
let x = 2;

let y = 3;
```

```
lines: 2
variables: 2
whitespace: 1
```

### Functions, enums, type aliases, impls and modules

```rust
fn x() {}

struct Struct;

type MyStruct = Struct;

impl Foo for MyStruct {
	fn foo() {}
}

mod other {
	enum MyOption<T> {
		Item(T),
		None
	}
	
	impl<T> Foo for MyOptiom<T> {
		fn foo() {}
	}
}
```

```
lines: 15
modules: 1
enums: 1
structs: 1
type_aliases: 1
fns: 3
impls: 2
delimeters: 4
whitespace: 5
```

### Above public

```rust
fn func1() {}
pub fn func2() {}
pub(crate) fn func2() {}
```

```
lines: 3
fns: 3
```

### Comments

```rust
// hi
let x = 2;
thing();
```

```
lines: 2
variables: 1
comments: 1
```

### Comments (multiline)

```rust
/* hi
let y = 2;
*/
let x = 2;
thing();
```

```
lines: 2
variables: 1
comments: 1
```

### Tests

```rust
fn x() -> u64 {
	4
}

#[cfg(test)]
mod test {
	#[test]
	fn test_x() {
		assert_eq!(x(), 4);
	}
}
```

```
lines: 3
test_lines: 4
fns: 1
delimeters: 3
whitespace: 1
```
