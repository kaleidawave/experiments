use spectra_lib::{Command, RunConfiguration, extract_tests, run_tests};

static SPECIFICATION: &str = include_str!("../examples/specification.md");

#[test]
fn pass() {
    let tests = extract_tests(SPECIFICATION, false);

    let mut runner = Command::new("bun run examples/example_program.js {content}");
    assert_eq!(
        run_tests(&tests, &mut runner, &RunConfiguration::default()).failures,
        Vec::new()
    );

    let mut runner = Command::new("bun run examples/example_program.js {content} --uppercase");
    assert_eq!(
        run_tests(&tests, &mut runner, &RunConfiguration::default())
            .failures
            .len(),
        4
    );
}

#[test]
fn pass_stdout_stderr() {
    let tests = extract_tests(SPECIFICATION, false);

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --stdin-stdout-communication",
    );
    assert_eq!(
        run_tests(&tests, &mut runner, &RunConfiguration::default()).failures,
        Vec::new()
    );

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --uppercase --stdin-stdout-communication",
    );
    assert_eq!(
        run_tests(&tests, &mut runner, &RunConfiguration::default())
            .failures
            .len(),
        4
    );
}
