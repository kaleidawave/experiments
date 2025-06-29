pub mod utilities;

use utilities::{
    commands, filter, is_equal_ignore_new_line_sequence, run_in_alternative_display,
    visit_specification_files,
};

use simple_markdown_parser::{CodeBlock, MarkdownElement, parse};
use std::io::{self, Write};
use std::process;

use colored::Colorize as Colourise;

/// TODO vec of vecs
#[derive(Debug, Default)]
pub struct Test {
    pub section: String,
    pub name: String,
    // options: (),
    pub case: String,
    pub expected: Option<String>,
}

pub trait Runner: Sized {
    /// Returns `Ok(*output*)`
    /// # Errors
    /// if test failed on runner, return a `Err` with some message about why it failed
    fn run(&mut self, test: &Test) -> Result<String, String>;

    /// Cleanup
    fn close(self) {}
}

#[derive(Default)]
pub struct RunConfiguration {
    pub interactive: bool,
    pub dry_run: bool,
    pub lists_to_code_block: bool,
    pub filter: Option<Box<dyn filter::Filter>>,
}

#[must_use]
pub fn extract_tests(content: &str, lists_to_code_block: bool) -> Vec<Test> {
    let mut tests: Vec<Test> = Vec::new();
    let mut current_test = Test::default();
    let mut section = String::new();

    let result = parse::<()>(content, |element| {
        if let MarkdownElement::Heading { level, content } = element {
            if level >= 3 {
                if !current_test.case.is_empty() {
                    tests.push(std::mem::take(&mut current_test));
                }
                current_test.name = content.no_decoration();
                section.clone_into(&mut current_test.section);
            } else {
                section = content.no_decoration();
            }
        } else if let MarkdownElement::Paragraph(_content) = element {
            // if content.0.ends_with("`top_level_separator = Some(\"\\n\")`") {
            //     current_test.options.top_level_separator = Some("\n");
            // }
        } else if let MarkdownElement::List(list) = element
            && lists_to_code_block
        {
            if !current_test.case.is_empty() && current_test.expected.is_none() {
                let _ = current_test.expected.insert(list.0.0.to_owned());
            }
        } else if let MarkdownElement::CodeBlock(CodeBlock { code, .. }) = element {
            if current_test.case.is_empty() {
                code.clone_into(&mut current_test.case);
            } else if current_test.expected.is_none() {
                let _ = current_test.expected.insert(code.to_owned());
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

#[derive(Debug, Default)]
pub struct TestResults {
    pub count: usize,
    pub skipped: usize,
    pub failures: Vec<(String, String)>,
}

impl TestResults {
    pub fn append(&mut self, mut new: TestResults) {
        self.count += new.count;
        self.skipped += new.skipped;
        self.failures.append(&mut new.failures);
    }
}

pub fn run_tests(
    tests: &[Test],
    runner: &mut impl Runner,
    configuration: &RunConfiguration,
) -> TestResults {
    let mut results = TestResults::default();

    for test in tests {
        results.count += 1;
        let name = &test.name;

        let skip_test = configuration
            .filter
            .as_ref()
            .is_some_and(|filter| filter.should_skip(&test.name));
        if skip_test {
            results.skipped += 1;
        }

        let result = runner.run(test);

        if configuration.dry_run {
            if !skip_test {
                if configuration.interactive {
                    let should_break = run_in_alternative_display(|| {
                        match result {
                            Ok(output) => eprintln!("Test {name}\nrecieved:\n{output}"),
                            Err(output) => eprintln!("Test {name}\nerrored: {output}"),
                        }

                        let mut input = String::new();
                        io::stdin()
                            .read_line(&mut input)
                            .expect("Failed to read line");

                        matches!(input.as_str().trim(), "exit" | "e" | "quit" | "q")
                    });
                    if should_break {
                        break;
                    }
                } else {
                    match result {
                        Ok(output) => eprintln!("Test {name}\nrecieved:\n{output}"),
                        Err(output) => eprintln!("Test {name}\nerrored: {output}"),
                    }
                }
            }
        } else if skip_test {
            println!("test {name} ... {result}", result = "skipped".blue());
        } else {
            let result = match result {
                Ok(output) => {
                    if let Some(ref expected) = test.expected {
                        if is_equal_ignore_new_line_sequence(&output, expected) {
                            Ok(())
                        } else {
                            Err(pretty_assertions::StrComparison::new(expected, &output)
                                .to_string())
                        }
                    } else {
                        Ok(())
                    }
                }
                Err(err) => Err(err),
            };

            match result {
                Ok(()) => {
                    println!("test {name} ... {result}", result = "passed".green());
                }
                Err(output) => {
                    println!("test {name} ... {result}", result = "failed".red());
                    results.failures.push((test.name.to_string(), output));
                }
            }
        }
    }

    results
}

pub fn run_tests_under_path(
    path: &std::path::Path,
    mut runner: impl Runner,
    configuration: &RunConfiguration,
) -> Result<(), usize> {
    let now = std::time::Instant::now();
    let mut results = TestResults::default();

    let () = visit_specification_files(path, &mut |path| {
        let content = std::fs::read_to_string(path).unwrap();
        let tests = extract_tests(&content, configuration.lists_to_code_block);
        let result = run_tests(&tests, &mut runner, configuration);
        results.append(result);
    })
    .expect("could not visit files");

    runner.close();

    let TestResults {
        count,
        failures,
        skipped,
    } = results;

    if configuration.dry_run {
        Ok(())
    } else {
        let elapsed = now.elapsed();

        if !failures.is_empty() {
            eprintln!("\nfailures:\n");

            if configuration.interactive {
                run_in_alternative_display(|| {
                    for (name, message) in &failures {
                        eprintln!("test {name} failed\n{message}\n");
                        let mut input = String::new();
                        io::stdin()
                            .read_line(&mut input)
                            .expect("Failed to read line");

                        if let "exit" | "e" | "quit" | "q" = input.as_str().trim() {
                            break;
                        }
                    }
                });
            } else {
                for (name, message) in &failures {
                    eprintln!("test {name} failed\n{message}\n");
                }
            }

            // TODO on single line?
            eprintln!("\nfailures:");
            for (name, _) in &failures {
                eprintln!("\t{name}");
            }
        }

        let result = if failures.is_empty() { "ok" } else { "err" };
        let passed = count - (failures.len() + skipped);
        let failed = failures.len();

        // FUTURE will we support these?
        let ignored = 0;
        let measured = 0;
        let filtered_out = skipped;

        eprintln!(
            "\ntest result: {result}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out; finished in {elapsed:?}"
        );

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.len())
        }
    }
}

/// Runs tests with runner and configuration, printing errors to stdout and stderr.
/// The output (should) mirror Rust's default test harness
///
/// # Errors
/// returns the number of failed tests
pub fn run_tests_under_content(
    content: &str,
    mut runner: impl Runner,
    configuration: &RunConfiguration,
) -> Result<(), usize> {
    let tests = extract_tests(content, configuration.lists_to_code_block);
    let count = tests.len();

    println!("\nrunning {count} tests");

    let now = std::time::Instant::now();

    let result = run_tests(&tests, &mut runner, configuration);

    let TestResults {
        count,
        failures,
        skipped,
    } = result;

    runner.close();

    if configuration.dry_run {
        Ok(())
    } else {
        let elapsed = now.elapsed();

        if !failures.is_empty() {
            eprintln!("\nfailures:\n");

            if configuration.interactive {
                run_in_alternative_display(|| {
                    for (name, message) in &failures {
                        eprintln!("test {name} failed\n{message}\n");
                        let mut input = String::new();
                        io::stdin()
                            .read_line(&mut input)
                            .expect("Failed to read line");

                        if let "exit" | "e" | "quit" | "q" = input.as_str().trim() {
                            break;
                        }
                    }
                });
            } else {
                for (name, message) in &failures {
                    eprintln!("test {name} failed\n{message}\n");
                }
            }

            // TODO on single line?
            eprintln!("\nfailures:");
            for (name, _) in &failures {
                eprintln!("\t{name}");
            }
        }

        let result = if failures.is_empty() { "ok" } else { "err" };
        let passed = count - (failures.len() + skipped);
        let failed = failures.len();

        // FUTURE will we support these?
        let ignored = 0;
        let measured = 0;
        let filtered_out = skipped;

        eprintln!(
            "\ntest result: {result}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out; finished in {elapsed:?}"
        );

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.len())
        }
    }
}

// #[derive(Debug, Default)]
// pub struct CommandConfiguration {
//     pub stdin_stdout_communication: bool,
//     pub ignore_exit_code: bool,
// }

/// TODO replace {file}
pub enum Command {
    SpawnCommand {
        name: String,
        arguments: Vec<String>,
        merge_stderr: bool,
        ignore_exit_code: bool,
    },
    Running {
        out: utilities::commands::CommandOut,
        stdin: process::ChildStdin,
    },
}

impl Command {
    /// # Panics
    /// panics if `data` is empty
    pub fn new(data: &str) -> Self {
        let mut iter = data.split(' ');
        let name = iter.next().expect("no command name");
        let mut arguments: Vec<String> = iter.map(ToOwned::to_owned).collect();

        let mut stdin_stdout_communication = false;
        let mut merge_stderr = false;
        let mut ignore_exit_code = false;

        if let Some(idx) = arguments
            .iter()
            .position(|arg| matches!(arg.as_str(), "--stdin-stdout-communication" | "--rpc"))
        {
            arguments.remove(idx);
            stdin_stdout_communication = true;
        }

        if let Some(idx) = arguments
            .iter()
            .position(|arg| matches!(arg.as_str(), "--merge-stderr"))
        {
            arguments.remove(idx);
            merge_stderr = true;
        }

        if let Some(idx) = arguments
            .iter()
            .position(|arg| matches!(arg.as_str(), "--ignore-exit-code"))
        {
            arguments.remove(idx);
            ignore_exit_code = true;
        }

        if stdin_stdout_communication {
            let mut command = process::Command::new(name);
            command.stdin(process::Stdio::piped());
            command.args(arguments);

            let mut out = commands::spawn_command(command, merge_stderr).unwrap();

            let child = out.get_child();
            let stdin = child.stdin.take().expect("Failed to open stdin");

            std::thread::sleep(std::time::Duration::from_millis(100));

            if let Ok(Some(status)) = child.try_wait() {
                panic!("exited with: {status}");
            }

            // std::thread::scope(|s| {
            //     // TODO abstract
            //     let started = std::sync::atomic::AtomicBool::new(false);

            //     s.spawn(|| {
            //         std::thread::sleep(std::time::Duration::from_secs(10));
            //         if !started.load(std::sync::atomic::Ordering::Relaxed) {
            //             eprintln!("program has not yielded 'start'");
            //         }
            //     });

            // Any prelude messages
            let (stdout, stderr) = out.read_until(|line| matches!(line, "start")).unwrap();
            // started.store(true, std::sync::atomic::Ordering::Relaxed);

            if !stdout.is_empty() || !stderr.is_empty() {
                eprintln!("{stdout}\n{stderr}");
            }

            Self::Running { out, stdin }
            // })
        } else {
            Self::SpawnCommand {
                name: name.to_owned(),
                arguments,
                ignore_exit_code,
                merge_stderr,
            }
        }
    }
}

impl Runner for Command {
    fn run(&mut self, test: &Test) -> Result<String, String> {
        match self {
            Self::SpawnCommand {
                name,
                arguments,
                ignore_exit_code,
                merge_stderr,
            } => {
                let arguments = arguments.iter().map(|argument| {
                    let argument = argument.as_str();
                    if let "{content}" = argument {
                        // TODO should this be part of the markdown parser
                        test.case.as_str().trim_end()
                    } else if let "{file}" = argument {
                        todo!("create file")
                    } else {
                        argument
                    }
                });
                let mut command = process::Command::new(name);
                command.args(arguments);

                let command = commands::spawn_command(command, *merge_stderr).unwrap();
                let (stdout_output, stderr_output, exit_code) = command.read_to_end().unwrap();

                if !*ignore_exit_code && !exit_code.success() {
                    Err(format!(
                        "Command failed with {exit_code:?}\n{stderr_output}"
                    ))
                } else {
                    if test.expected.is_none() && !stdout_output.is_empty() {
                        eprintln!(
                            "Possibly unexpected stdout output {stdout_output} from {name}",
                            name = test.name
                        );
                    }
                    Ok(stdout_output)
                }
            }
            Self::Running { out, stdin } => {
                for line in test.case.as_str().lines() {
                    // eprintln!("TEMP writing {line:?}");
                    writeln!(stdin, "{line}").expect("could not write");
                }

                writeln!(stdin, "end").expect("could not write");

                let (out, stderr) = out.read_until(|line| matches!(line, "end")).unwrap();

                // TODO WIP
                if out
                    .lines()
                    .next_back()
                    .is_some_and(|line| line.starts_with("error: "))
                {
                    Err(stderr)
                } else {
                    Ok(out)
                }
            }
        }
    }

    fn close(self) {
        if let Self::Running { mut stdin, out } = self {
            // Send the close signal
            writeln!(stdin, "close").unwrap();

            // TODO other fields here
            let (rest, _, _) = out.read_to_end().unwrap();
            for line in rest.lines() {
                println!("left over: {line}");
            }
        }
    }
}

pub struct Commands {
    commands: Vec<(String, Command)>,
}

impl Commands {
    #[must_use]
    pub fn new(data: &str) -> Self {
        let items = data.split(',');
        let commands = items
            .map(|item| (item.to_owned(), Command::new(item)))
            .collect();
        Self { commands }
    }
}

impl Runner for Commands {
    fn run(&mut self, test: &Test) -> Result<String, String> {
        let mut buf = String::new();
        for (name, command) in &mut self.commands {
            let out = command.run(test)?;
            buf.push_str(name);
            buf.push_str(":\n");
            buf.push_str(&out);
            buf.push('\n');
        }
        Ok(buf)
    }

    fn close(self) {
        self.commands
            .into_iter()
            .for_each(|(_, command)| command.close());
    }
}
