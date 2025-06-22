use spectra_lib::{Command, Configuration, run_tests};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("info" | "--help") | None => {
            let run_id = option_env!("GITHUB_RUN_ID");
            let date = option_env!("GIT_LAST_COMMIT");
            let after = run_id
                .map(|commit| format!(" (commit {commit} {date:?})"))
                .unwrap_or_default();

            eprintln!("spectra (WIP){after} (powered by 'simple-markdown-parser')");
            ExitCode::SUCCESS
        }
        Some("check") => {
            // TODO dry run, command etc, pipe communication, timeout, exit code
            let path = args.next().expect("expected path");
            let command = args.next().expect("expected command");
            let mut command = Command::new(&command, Configuration::default());
            let result = run_tests(&std::path::PathBuf::from(path), &mut command);
            if result.is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some("add-test") => {
            todo!("add-test command")
        }
        Some("list-tests") => {
            todo!("list-tests and statistics")
        }
        Some(command) => {
            eprintln!("unknown command {command:?}. expected 'check'");
            ExitCode::FAILURE
        }
    }
}
