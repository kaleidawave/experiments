# lahl: lightweight-argument-handling-library

### Provides

- Defining the structure of a command-line program
- Help / usage message generation
- Non-macro based API

### Features

- *endpoint grouping*, e.g `*program* experimental build *args*` (two specifiers for name)
- *multiple* for named and positional arguments

### Key ideas

- CLI defined by `'static` slices
- An endpoint is a section of a CLI. Specified by 1-2 initial arguments
- An endpoint can have named and positional parameters
- At runtime a `Iterator<Item=String>` of is parsed to a program and it returns a structure containing parameter argument pairings
- Very little validation is done, it is upto the user to check required parameters are present and strings are parsed into their required forms

### Linting

During runtime the definition is not checked. Therefore it can have multiple problems

- Invalid endpoint or argument name. e.g. empty or contains whitespace
- Positional parameter after positional parameter accepting multiple (unreachable)

To catch these errors you can run `CLI::lint` as a test

### Future

- Run `lint` on `debug_assertions`?
- Lifetime on `CLI` (rather than `'static`)
- More checking in this library
- `struct` + derive-macro API with `FromString` logic
- Shorthand named parameter
- Aliases map (and replace `default`)
- Auto-completion generation for shells (Powershell, Bash, Fish etc)
- JavaScript library that mirrors Rust API
