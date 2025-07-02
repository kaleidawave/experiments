use simple_toml_parser::{matches, parse as parse_toml, RootTOMLValue};
use std::process::{Command, Stdio};

fn main() {
    let cargo_toml = std::fs::read_to_string("./Cargo.toml").unwrap();
    let arg = std::env::args().nth(1).unwrap_or_default();

    parse_toml(&cargo_toml, |keys, value| {
        let matched = matches(&["package", "metadata", "commands", &arg], keys);
        if matched {
            if let RootTOMLValue::String(run) = value {
                let mut args = run.split(' ');
                let name = args.next().expect("no command name");
                let mut command = Command::new(name);
                command.args(args.collect::<Vec<_>>());
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
                let _ = command.spawn();
            }
        }
    })
    .unwrap();
}
