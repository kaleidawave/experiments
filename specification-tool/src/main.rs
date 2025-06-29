use spectra_lib::{
    Command, Commands, RunConfiguration, extract_tests, run_tests_under_path,
    utilities::{filter, visit_specification_files},
};
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("info" | "--help") | None => {
            let run_id = option_env!("GITHUB_RUN_ID");
            let date = option_env!("GIT_LAST_COMMIT").unwrap_or_default();
            let after = run_id
                .map(|commit| format!(" (commit {commit} {date})"))
                .unwrap_or_default();

            eprintln!("spectra (WIP){after} (powered by 'simple-markdown-parser')");
            ExitCode::SUCCESS
        }
        Some("check") => {
            // TODO timeout
            let path = args.next().expect("expected path");
            let command = args.next().expect("expected command");

            let mut run_configuration = RunConfiguration::default();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    // skip and including options
                    "--only" | "--skip" | "--only-cs" | "--skip-cs" => {
                        let matcher = args.next().expect("expected matcher");
                        let filter = filter::StringMatch {
                            case_sensitive: arg.ends_with("-cs"),
                            positive: arg.starts_with("--only"),
                            matcher: matcher.split(',').map(ToOwned::to_owned).collect(),
                        };
                        run_configuration.filter = Some(Box::new(filter));
                    }
                    // run configuration
                    "--interactive" => run_configuration.interactive = true,
                    "--dry-run" => run_configuration.dry_run = true,
                    "--lists-as-expected" => run_configuration.lists_to_code_block = true,
                    // // command configuration
                    // "--ignore-exit-code" => command_configuration.ignore_exit_code = true,
                    // "--stdin-stdout-communication" => command_configuration.stdin_stdout_communication = true,
                    flag => {
                        eprintln!("unknown flag {flag:?}");
                        return ExitCode::FAILURE;
                    }
                }
            }

            let command = Command::new(&command);
            let result = run_tests_under_path(Path::new(&path), command, &run_configuration);
            if result.is_err() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Some("compare") => {
            let path = args.next().expect("expected path");
            let command_pattern = args.next().expect("expected command");

            let mut run_configuration = spectra_lib::RunConfiguration {
                dry_run: true,
                ..Default::default()
            };
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    // skip and including options
                    "--only" | "--skip" | "--only-cs" | "--skip-cs" => {
                        let matcher = args.next().expect("expected matcher");
                        let filter = filter::StringMatch {
                            case_sensitive: arg.ends_with("-cs"),
                            positive: arg.starts_with("--only"),
                            matcher: matcher.split(',').map(ToOwned::to_owned).collect(),
                        };
                        run_configuration.filter = Some(Box::new(filter));
                    }
                    // run configuration
                    "--interactive" => run_configuration.interactive = true,
                    "--lists-as-expected" => run_configuration.lists_to_code_block = true,
                    // // command configuration
                    // "--ignore-exit-code" => command_configuration.ignore_exit_code = true,
                    // "--stdin-stdout-communication" => command_configuration.stdin_stdout_communication = true,
                    flag => {
                        eprintln!("unknown flag {flag:?}");
                        return ExitCode::FAILURE;
                    }
                }
            }

            let command = Commands::new(&command_pattern);
            let result = run_tests_under_path(Path::new(&path), command, &run_configuration);
            if result.is_err() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Some("list-tests") => {
            let path = args.next().expect("expected path");
            let path = Path::new(&path);
            let _prefix = path.parent().map(|path| path.display().to_string());

            let mut count = 0;
            let mut files = 0;
            let () = visit_specification_files(path, &mut |path| {
                let content = std::fs::read_to_string(path).unwrap();
                let tests = extract_tests(&content, true);
                println!("--- {path} ---", path = path.display());
                for test in tests {
                    println!("{name}", name = test.name);
                    count += 1;
                }
                files += 1;
            })
            .expect("could not walk files");
            eprintln!("found {count} tests across {files} files");
            ExitCode::SUCCESS
        }
        Some("concatenate-cases") => {
            let input_path = args.next().expect("expected input_path");
            let input_path = Path::new(&input_path);

            let output_path = args.next().expect("expected output_path");
            let output_path = Path::new(&output_path);

            let mut file =
                std::fs::File::create(output_path).expect("could not create output file");
            writeln!(&mut file, "---").unwrap();

            let () = visit_specification_files(input_path, &mut |path| {
                let content = std::fs::read_to_string(path).unwrap();
                let tests = extract_tests(&content, true);
                for test in tests {
                    writeln!(&mut file, "{case}", case = test.case).unwrap();
                    writeln!(&mut file, "---").unwrap();
                }
            })
            .expect("could not walk files");
            ExitCode::SUCCESS
        }
        Some("add-test") => {
            todo!("add-test command")
        }
        Some(command) => {
            eprintln!("unknown command {command:?}. expected 'check'");
            ExitCode::FAILURE
        }
    }
}
