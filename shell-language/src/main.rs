pub mod check_and_explain;
pub mod evaluate;
pub mod parsing;
pub mod utilities;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs};

    let mut args = env::args().skip(1);
    let Some(first) = args.next() else {
        let commit = option_env!("GIT_LAST_COMMIT");
        let date = option_env!("GITHUB_RUN_DATE").unwrap_or_default();
        let after = commit
            .map(|commit| format!(" (commit {commit} {date})"))
            .unwrap_or_default();
        println!("the Ben shell (WIP){after}");
        return Ok(());
    };

    if let "--interactive" | "-i" = first.as_str() {
        let mut state = crate::interactive::InteractiveState::new();
        loop {
            let input = {
                use std::io;
                print!("> ");
                std::io::Write::flush(&mut io::stdout()).unwrap();
                let mut input = String::new();
                let std_in = &mut io::stdin();

                // multiline_term_input only works on windows for now
                #[cfg(target_family = "windows")]
                let _n = multiline_term_input::read_string(std_in, &mut input);

                #[cfg(target_family = "unix")]
                let _n = std_in.read_line(&mut input).unwrap();

                input
            };

            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if let "exit" | "quit" = input {
                break;
            }

            state.parse_and_evaluate_command(input.to_owned());
        }
    } else {
        let (path, source) = if let "--evaluate" | "-e" = first.as_str() {
            ("...".to_owned(), args.next().unwrap_or_default())
        } else {
            let content = fs::read_to_string(&first)?;
            (first, content)
        };

        let rest: Vec<_> = args.collect();

        let debug_program: bool = rest.iter().any(|flag| flag == "--debug-program");
        let check: bool = rest.iter().any(|flag| flag == "--check");
        let explain: bool = rest.iter().any(|flag| flag == "--explain");

        if check {
            let file = codespan_reporting::files::SimpleFile::new(path, source);
            let state = check_and_explain::State {
                file,
                config: codespan_reporting::term::Config::default(),
                explain,
            };
            let mut writer = codespan_reporting::term::termcolor::StandardStream::stdout(
                codespan_reporting::term::termcolor::ColorChoice::default(),
            );
            let program = parsing::parsing::parse_program(state.file.source());
            check_and_explain::check_program(&program, &state, &mut writer);
            // TODO exit code
        } else {
            let program = parsing::parsing::parse_program(&source);

            if debug_program {
                eprintln!("{program:#?}");
            } else {
                evaluate::evaluate_program(&program);
            }
        }
    }

    Ok(())
}

mod interactive {
    use super::evaluate::{Context, evaluate_statement};
    use super::parsing::parsing::{Lines, parse_statement};

    #[derive(Default)]
    pub struct InteractiveState {
        context: Context<'static>,
    }

    impl InteractiveState {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn parse_and_evaluate_command(&mut self, command: String) {
            // TODO can we append it somewhere, that doesn't move. Pinned?
            let command = String::leak(command);
            let statement =
                parse_statement(command, &mut Lines::new("", &[]), 0).expect("no statement");
            evaluate_statement(&statement, &mut self.context);
        }
    }
}
