use spectra::{RunConfiguration, extract_tests, run_tests, runners::program::Command};

static SPECIFICATION_UPPERCASE: &str = include_str!("../examples/specification.uppercase.md");
static SPECIFICATION_LIST: &str = include_str!("../examples/specification.lists.md");
static SPECIFICATION_OPTIONS: &str = include_str!("../examples/specification.options.md");

#[test]
fn pass() {
    let tests = extract_tests(SPECIFICATION_UPPERCASE, false);

    let mut runner = Command::new("bun run examples/example_program.js {content} --uppercase");
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert!(results.failures.is_empty());

    let mut runner = Command::new("bun run examples/example_program.js {content}");
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert_eq!(results.failures.len(), 3);
}

#[test]
fn pass_stdout_stderr() {
    let tests = extract_tests(SPECIFICATION_UPPERCASE, false);

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --uppercase --stdin-stdout-communication",
    );
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert!(results.failures.is_empty());

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --stdin-stdout-communication",
    );
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert_eq!(results.failures.len(), 3);
}

#[test]
fn pass_lists() {
    let tests = extract_tests(SPECIFICATION_LIST, true);

    let mut runner =
        Command::new("bun run examples/example_program.js {content} --uppercase --use-lists");
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    if !results.failures.is_empty() {
        for (test, out) in &results.failures {
            println!("{test}\n{out}");
        }
        panic!("not empty")
    }

    let mut runner = Command::new("bun run examples/example_program.js --use-lists");
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert_eq!(results.failures.len(), 3);
}

#[test]
fn program_crash() {
    let tests = extract_tests(SPECIFICATION_UPPERCASE, false);

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --uppercase --stdin-stdout-communication --intentional-crash",
    );
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert_eq!(results.failures.len(), 1);
    assert_eq!(
        results.failures.get(0).map(|(lhs, _rhs)| lhs.as_str()),
        Some("Test 2")
    );
}

#[test]
fn program_timeout() {
    let tests = extract_tests(SPECIFICATION_UPPERCASE, false);

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --uppercase --stdin-stdout-communication --intentional-timeout --timeout 1000",
    );
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    // test 2 does not run in under 1000 ms
    assert_eq!(&results.failures, &[("Test 2".into(), "".into())]);

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --uppercase --stdin-stdout-communication --intentional-timeout --timeout 5000",
    );

    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert!(results.failures.is_empty());
}

#[test]
fn program_options() {
    let tests = extract_tests(SPECIFICATION_OPTIONS, false);

    let mut runner = Command::new(
        "bun run examples/example_stdin_stdout_program.js --uppercase --stdin-stdout-communication",
    );
    let results = run_tests(&tests, &mut runner, &RunConfiguration::default());
    assert!(
        &results.failures.is_empty(),
        "found failures {failures:#?}",
        failures = &results.failures
    );
}
