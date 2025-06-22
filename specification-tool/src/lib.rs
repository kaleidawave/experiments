use simple_markdown_parser::{CodeBlock, MarkdownElement, parse};
use std::path::Path;

pub trait Runner {
    /// # Errors
    /// if test failed on runner, return a `Err` with some message about why it failed
    fn run(&mut self, test: &Test) -> Result<(), String>;
}

#[derive(Debug, Default)]
pub struct Configuration {
    pub keep_alive_stdin_stdout: bool,
    pub ignore_exit_code: bool,
}

/// TODO vec of vecs
#[derive(Debug, Default)]
pub struct Test {
    name: String,
    // options: (),
    case: String,
    output: Option<String>,
}

fn get_tests(file: &Path) -> Vec<Test> {
    let mut tests: Vec<Test> = Vec::new();
    let mut current_test = Test::default();
    let content = std::fs::read_to_string(file).unwrap();
    let result = parse::<()>(&content, |element| {
        if let MarkdownElement::Heading { level, content } = element {
            if level >= 3 {
                if !current_test.case.is_empty() {
                    tests.push(std::mem::take(&mut current_test));
                }
                current_test.name = content.no_decoration();
            }
        } else if let MarkdownElement::Paragraph(_content) = element {
            // if content.0.ends_with("`top_level_separator = Some(\"\\n\")`") {
            //     current_test.options.top_level_separator = Some("\n");
            // }
        } else if let MarkdownElement::CodeBlock(CodeBlock { code, .. }) = element {
            if current_test.case.is_empty() {
                code.clone_into(&mut current_test.case);
            } else if current_test.output.is_none() {
                let _ = current_test.output.insert(code.to_owned());
            } else {
                // create a new test
                let next_name = format!("{} *", current_test.name);
                tests.push(std::mem::take(&mut current_test));
                current_test.name = next_name;
            }
        }
        Ok(())
    });

    assert!(result.is_ok(), "{result:?}");
    if !current_test.case.is_empty() {
        tests.push(current_test);
    }
    tests
}

/// # Errors
/// returns the number of failed tests
pub fn run_tests(file: &Path, runner: &mut impl Runner) -> Result<(), usize> {
    let tests = get_tests(file);
    let count = tests.len();

    println!("\nrunning {count} tests");

    let mut failures: Vec<(String, String)> = Vec::default();

    let now = std::time::Instant::now();

    for test_case in tests {
        // fn test<F>(_name: &str, cb: F) -> Result<(), ()>
        // where
        //     F: FnOnce() -> () + std::marker::Send + 'static,
        // {
        //     let res = std::thread::spawn(cb);
        //     match res.join() {
        //         Ok(_) => Ok(()),
        //         Err(_) => Err(()),
        //     }
        // }

        // eprintln!(
        //     "input {name}:\n{case:?}\nrecieved:\n{out}\n---\n",
        //     case = test_case.case
        // );
        // let expectation = test_case.output.trim_end();
        // assert_eq!(out.trim_end(), expectation, "expected {out}",)
        // });

        let result = runner.run(&test_case);

        let name = &test_case.name;
        match result {
            Ok(()) => {
                println!(
                    "test {name} ... \u{001b}\u{005b}\u{0033}\u{0032}\u{006d}\u{006f}\u{006b}\u{001b}\u{005b}\u{0033}\u{0039}\u{006d}"
                );
            }
            Err(output) => {
                println!(
                    "test {name} ... \u{001b}\u{005b}\u{0033}\u{0031}\u{006d}\u{0066}\u{0061}\u{0069}\u{006c}\u{0065}\u{0064}\u{001b}\u{005b}\u{0033}\u{0039}\u{006d}"
                );
                failures.push((test_case.name.to_string(), output));
            }
        }
    }

    let elapsed = now.elapsed();

    if !failures.is_empty() {
        eprintln!("\nfailures:\n");
        for (name, message) in &failures {
            eprintln!("test {name} failed");
            eprintln!("{message}\n");
        }
        // TODO on single line?
        eprintln!("\nfailures:");
        for (name, _) in &failures {
            eprintln!("\t{name}");
        }
    }

    {
        let result = if failures.is_empty() { "ok" } else { "err" };
        let passed = count - failures.len();
        let failed = failures.len();
        // FUTURE will we support this?
        let ignored = 0;
        let measured = 0;
        let filtered_out = 0;
        eprintln!(
            "\ntest result: {result}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out; finished in {elapsed:?}"
        );
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.len())
    }
}

/// TODO replace {file}
pub struct Command {
    name: String,
    arguments: Vec<String>,
    configuration: Configuration,
}

impl Command {
    /// # Panics
    /// panics if `data` is empty
    pub fn new(data: &str, configuration: Configuration) -> Self {
        let mut iter = data.split(' ');
        let name = iter.next().expect("no command name").to_owned();
        let arguments = iter.map(ToOwned::to_owned).collect();
        Command {
            name,
            arguments,
            configuration,
        }
    }
}

impl Runner for Command {
    fn run(&mut self, test: &Test) -> Result<(), String> {
        fn run_command<S: AsRef<std::ffi::OsStr>>(
            command: &str,
            args: impl Iterator<Item = S>,
            // env: Option<Vec<(String, String)>>,
            // capture_stdout: bool,
            // capture_stderr: bool,
        ) -> (String, std::process::ExitStatus) {
            use std::io::{Read, pipe};
            use std::process::{Command, Stdio};

            // let env = env.unwrap_or_default();

            // Shouldn't need pipe here
            let (mut reader, writer) = pipe().expect("could not create pipe");
            let (stdout, stderr): (Stdio, Stdio) = (writer.into(), Stdio::inherit());

            let mut child = Command::new(command)
                .args(args)
                .stdout(stdout)
                .stderr(stderr)
                // .envs(env)
                .spawn()
                .expect("Failed to spawn command");

            let mut output = String::new();
            reader.read_to_string(&mut output).expect("invalid UTF8");
            let result = child.wait().expect("command not finished");
            // Remove whitespace from end
            output.truncate(output.trim_end().len());
            (output, result)
        }

        let (output, exit_code) = run_command(
            &self.name,
            self.arguments.iter().map(|argument| {
                let argument = argument.as_str();
                if let "{content}" = argument {
                    // TODO should this be part of the markdown parser
                    test.case.as_str().trim_end()
                } else if let "{file}" = argument {
                    todo!("create file")
                } else {
                    argument
                }
            }),
        );

        // #[allow(clippy::nested)]
        if let Some(ref expected) = test.output {
            if !self.configuration.ignore_exit_code && !exit_code.success() {
                Err(format!("Command failed with {exit_code:?}"))
            } else if is_equal_ignore_new_line_sequence(&output, expected) {
                Ok(())
            } else {
                Err(pretty_assertions::StrComparison::new(expected, &output).to_string())
            }
        } else {
            if !output.is_empty() {
                eprintln!("Possibly unexpected stdout output {output}");
            }
            if exit_code.success() {
                Ok(())
            } else {
                Err(format!("Command failed with {exit_code:?}"))
            }
        }
    }
}

fn is_equal_ignore_new_line_sequence(lhs: &str, rhs: &str) -> bool {
    // We should not care about trailing new lines here...
    let mut lhs = lhs.lines();
    let mut rhs = rhs.lines();
    loop {
        match (lhs.next(), rhs.next()) {
            (Some(lhs), Some(rhs)) => {
                if lhs != rhs {
                    return false;
                }
            }
            (Some(_), _) | (_, Some(_)) => {
                return false;
            }
            (None, None) => {
                return true;
            }
        }
    }
}
