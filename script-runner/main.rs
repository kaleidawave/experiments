use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let mut args = std::env::args().skip(1);

    let to_run = args.next();
    let to_run = to_run.as_deref().unwrap_or("info");

    if let "info" | "--help" = to_run {
        eprintln!("Script runner (development):");
        eprintln!("run commands in 'Cargo.toml' (and soon package.json)");
        eprintln!("also: --watch *path* and --loop");
        return;
    }

    let mut watch: Option<String> = None;
    let mut r#loop: bool = false;
    let mut surpress_exit_code: bool = false;
    while let Some(arg) = args.next() {
        if let "--watch" = arg.as_str() {
            watch = Some(args.next().expect("no watch path"));
        } else if let "--loop" = arg.as_str() {
            r#loop = true;
        } else if let "--supress-exit-code" | "--sec" = arg.as_str() {
            surpress_exit_code = true;
        }
    }

    let mut commands: Vec<String> = Vec::new();

    let cargo_toml = Path::new("./Cargo.toml");
    let selected = if cargo_toml.is_file() {
        let cargo_toml = std::fs::read_to_string(cargo_toml).unwrap();
        parse_cargo_toml(&cargo_toml, to_run, &mut commands)
    } else {
        let package_json = Path::new("./package.json");
        if package_json.is_file() {
            let package_json = std::fs::read_to_string(package_json).unwrap();
            parse_package_json(&package_json, to_run, &mut commands)
        } else {
            panic!("no commands file...")
        }
    };

    let Some(value) = selected else {
        eprintln!("Could not found {to_run:?}. Commands are {commands:?}");
        return;
    };

    let parts = utilities::ArgumentIter::new(&value);
    let arguments: Vec<_> = parts.map(|part| part.into_owned()).collect();

    let command = UserCommand::new(arguments);

    if let Some(path_to_watch) = watch {
        use notify_debouncer_full::{new_debouncer, notify::*};
        use std::time::Duration;

        let spawned = command.spawn().expect("could not start command");
        let mut running = spawned;

        eprintln!("Watching {path_to_watch:?}");

        let (tx, rx) = std::sync::mpsc::channel();

        // Select recommended watcher for debouncer.
        // Using a callback here, could also be a channel.
        let debounce_time = Duration::from_secs(1);
        let mut debouncer = new_debouncer(debounce_time, None, tx).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        debouncer
            .watch(path_to_watch, RecursiveMode::Recursive)
            .unwrap();

        for result in rx {
            match result {
                Ok(_events) => {
                    // events.iter().for_each(|event| println!("{event:?}"));
                    let _ = running.kill();
                    let spawned = command.spawn().expect("could not start command");
                    running = spawned;
                }
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            }
        }
    } else if r#loop {
        use std::io::BufRead;

        let spawned = command.spawn().expect("could not start command");
        let mut running = spawned;
        loop {
            let _ = running.wait();
            let mut buffer = String::new();
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();
            handle.read_line(&mut buffer).unwrap();
            if let "exit" = buffer.trim() {
                break;
            }
            let spawned = command.spawn().expect("could not start command");
            running = spawned;
        }
    } else {
        let mut spawned = command.spawn().expect("could not start command");
        let result = spawned.wait().unwrap();
        if !(result.success() || surpress_exit_code) {
            eprintln!("Command exited with {result}");
        }
    }
}

fn parse_cargo_toml(source: &str, to_run: &str, commands: &mut Vec<String>) -> Option<String> {
    use simple_toml_parser::{RootTOMLValue, TOMLKey, parse_toml};

    let mut selected = None;
    parse_toml(source, |keys, value| {
        let path = &[
            TOMLKey::Slice("package"),
            TOMLKey::Slice("metadata"),
            TOMLKey::Slice("commands"),
        ];
        if let Some(rest) = keys.strip_prefix(path) {
            if let &[TOMLKey::Slice(key)] = rest {
                if utilities::specifier_matches(key, to_run) {
                    if let RootTOMLValue::String(value) = value {
                        selected = Some(value.raw().trim().to_owned());
                        // TODO break
                    } else {
                        panic!("expected string, found {value:?}");
                    }
                } else {
                    commands.push(key.to_owned());
                }
            } else {
                eprintln!("complex chain {rest:?}");
            }
        }
    })
    .unwrap();

    selected
}

fn parse_package_json(source: &str, to_run: &str, commands: &mut Vec<String>) -> Option<String> {
    use simple_json_parser::{JSONKey, RootJSONValue, parse as parse_json};

    let mut selected = None;
    parse_json(source, |keys, value| {
        let path = &[JSONKey::Slice("scripts")];
        if let Some(rest) = keys.strip_prefix(path) {
            if let &[JSONKey::Slice(key)] = rest {
                if utilities::specifier_matches(key, to_run) {
                    if let RootJSONValue::String(value) = value {
                        selected = Some(value.to_owned());
                        // TODO break
                    } else {
                        panic!("expected string, found {value:?}");
                    }
                } else {
                    commands.push(key.to_owned());
                }
            }
        }
    })
    .unwrap();

    selected
}

struct UserCommand {
    to_run: Vec<(String, Vec<String>)>,
}

impl UserCommand {
    pub fn new(arguments: Vec<String>) -> Self {
        let split = arguments.split(|arg| arg == "&&");
        let mut to_run = Vec::new();
        for group in split {
            let name = group.first().expect("no command name").clone();
            let arguments = group[1..].to_vec();
            to_run.push((name, arguments));
        }
        Self { to_run }
    }

    pub fn spawn(&self) -> std::io::Result<std::process::Child> {
        let last = self.to_run.len() - 1;
        for (name, arguments) in &self.to_run[..last] {
            let mut command = Self::buld_command(name, &arguments);
            let mut child = command.spawn()?;
            let out = child.wait()?;
            if !out.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("process '{name}' did not exit successfully"),
                ));
            }
        }
        let (name, arguments) = &self.to_run[last];
        Self::buld_command(name, arguments).spawn()
    }

    fn buld_command(name: &str, arguments: &[String]) -> Command {
        let mut command = Command::new(name);
        command.args(arguments);
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command
    }
}

mod utilities {
    use std::borrow::Cow;

    pub struct ArgumentIter<'a> {
        on: &'a str,
        last: usize,
    }

    impl<'a> ArgumentIter<'a> {
        pub fn new(on: &'a str) -> Self {
            // Trim?
            Self { on, last: 0 }
        }
    }

    impl<'a> Iterator for ArgumentIter<'a> {
        type Item = Cow<'a, str>;

        fn next(&mut self) -> Option<Self::Item> {
            let start = self.last;
            if let Some((idx, matched)) =
                self.on[self.last..].match_indices(&[' ', '\'', '"']).next()
            {
                match matched {
                    " " => {
                        let end = self.last + idx;
                        self.last += idx + matched.len();
                        Some(Cow::Borrowed(self.on[start..end].trim()))
                    }
                    "\"" | "\'" => {
                        let rest = &self.on[self.last..][1..];
                        let (idx2, _) = rest
                            .match_indices(matched)
                            .filter(|(idx, _)| !rest[..*idx].ends_with('\\'))
                            .next()
                            .expect("no end to quoted item");

                        self.last += idx + idx2 + 2;
                        if let Some(rest) = self.on.get(self.last..) {
                            self.last += rest.len() - rest.trim_start().len();
                        }
                        let content = &rest[..idx2];
                        if content.contains('\\') {
                            Some(Cow::Owned(content.replace('\\', "")))
                        } else {
                            Some(Cow::Borrowed(content))
                        }
                    }
                    item => unreachable!("{item}"),
                }
            } else if start < self.on.len() {
                self.last = self.on.len();
                Some(Cow::Borrowed(&self.on[start..]))
            } else {
                None
            }
        }
    }

    pub fn specifier_matches(key: &str, selected: &str) -> bool {
        if key == selected {
            true
        } else {
            let mut selected = selected.chars();
            if let Some(s_first) = selected.next()
                && let Some(rest) = key.strip_prefix(s_first)
            {
                for (idx, _) in rest.match_indices(['_', '-']) {
                    if let Some(after) = rest[idx..][1..].chars().next()
                        && selected.next().is_none_or(|sc| sc != after)
                    {
                        return false;
                    }
                }
                selected.next().is_none()
            } else {
                false
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::specifier_matches;

        #[test]
        fn specifier_matches_test() {
            assert!(specifier_matches("tests", "tests"));
            assert!(specifier_matches("tests", "t"));
            assert!(specifier_matches("test-parser", "tp"));
            assert!(specifier_matches("test_parser", "tp"));

            assert!(!specifier_matches("test_parser", "t"));
            assert!(!specifier_matches("test_parser", "tpa"));
        }
    }
}
