```shell
cargo run -- compare ./examples/specification.md \
	"./target/debug/examples/example_stdin_stdout_program --uppercase --rpc,./target/debug/examples/example_stdin_stdout_program --rpc" \
	--only other
```

#### Features

- `--only` and `--skip`
- `compare` for running multiple binaries