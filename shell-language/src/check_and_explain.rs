use crate::parsing::ast::{Command, Program, Statement};

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::{
    self, Config,
    termcolor::{StandardStream},
};

pub struct State {
    pub file: SimpleFile<String, String>,
    pub config: Config,
    pub explain: bool,
}

pub fn check_program(program: &Program, state: &State, stream: &mut StandardStream) {
    let mut names = Vec::new();
    for statement in &program.0 {
        check_statement(statement, state, stream, &mut names);
    }
}

pub type NamesInContext = Vec<String>;

fn check_statement(
    statement: &Statement,
    state: &State,
    stream: &mut StandardStream,
    names_context: &mut NamesInContext,
) {
    match statement {
        Statement::Declaration { name, value } => {
            check_command(value, state, stream, names_context);
            names_context.push(name.to_string());
        }
        Statement::Assignment { name, value } => {
            let contains = names_context.iter().any(|name2| name2 == name);
            if !contains {
                let start = (name.as_ptr() as usize) - (state.file.source().as_ptr() as usize);
                let label = Label::primary((), start..(start + name.len()))
                    .with_message(format!("Variable '{name}' does not exist"));
                let diagnostic = Diagnostic::error().with_labels(vec![label]);
                term::emit(&mut stream.lock(), &state.config, &state.file, &diagnostic).unwrap();
            }
			check_command(value, state, stream, names_context);
        }
        Statement::Command(command) => {
            check_command(command, state, stream, names_context);
        }
        Statement::For {
            iterator,
            statements,
        } => {
            check_command(iterator, state, stream, names_context);
            // TODO introduce names
            for statement in statements {
                check_statement(statement, state, stream, names_context);
            }
        }
        Statement::If {
            condition,
            statements,
        } => {
            check_command(condition, state, stream, names_context);
            for statement in statements {
                check_statement(statement, state, stream, names_context);
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn check_command<'a>(
    command: &Command,
    state: &State,
    stream: &mut StandardStream,
    names_context: &NamesInContext,
) {
	// TODO check arguments
    match command.name {
        "echo" | "echo_stdout" => {}
        "echo_stderr" => {}
        "run" | "with" => {}
        "env" => {}
        // Filesystem manipulation
        "mv" | "move" => {}
        "cp" | "copy" => {}
        "rm" | "remove" => {}
        "ee" | "ensure_executable" => {}
        "files" => {}
        "write" => {}
        "read" => {}
        "append" => {}
        "repeat" => {}
        "replace" => {}
        "debug" => {}
        "split" => {}
        "concatenate" => {}
        "concatenate_separator" => {}
        "before" | "after" | "rbefore" | "rafter" => {}
        "last_line" | "first_line" => {}
        "size" => {}
        "lines" => {}
        "trim" => {}
        "format_number" => {}
        #[cfg(feature = "regular-expressions")]
        "regexp" => {}
        #[cfg(feature = "date")]
        "date" => {}
        "if_equal" => {}
        "if_contains" => {}
        // TODO WIP. "known programs"
        "cargo" | "git" | "gh" | "hyperfine" | "jq" | "yq" | "node" | "deno" | "bun"
        | "sqlite3" | "python" | "npm" | "bat" => {}
        "literal" | "constant" => {}
        name => {
            let start = (name.as_ptr() as usize) - (state.file.source().as_ptr() as usize);
            let label = Label::primary((), start..(start + name.len()))
                .with_message(format!("Unknown command '{name}'"));
            let diagnostic = Diagnostic::error().with_labels(vec![label]);
            term::emit(&mut stream.lock(), &state.config, &state.file, &diagnostic).unwrap();
        }
    };

    for argument in &command.arguments {
        let mut last_was_escape_backslash = false;
        for (idx, matched) in argument.0.match_indices(['$', '\\', '\'', '"']) {
            let skip = last_was_escape_backslash && matched == "\\";
            last_was_escape_backslash = false;
            if skip {
                continue;
            }

            if let "$" = matched {
                let rest = &argument.0[(idx + 1)..];
                let reference = rest
                    .split_once(|chr: char| !(chr.is_alphanumeric() || matches!(chr, '_')))
                    .map_or(rest, |(rest, _)| rest);
                if !names_context.iter().any(|name2| name2 == reference) {
                    let start = (reference.as_ptr() as usize) - (state.file.source().as_ptr() as usize) - 1;
                    let label = Label::primary((), start..(start + reference.len() + 1))
                        .with_message(format!("Variable '{reference}' does not exist"));
                    let diagnostic = Diagnostic::error().with_labels(vec![label]);
                    term::emit(&mut stream.lock(), &state.config, &state.file, &diagnostic).unwrap();
                }
            }
        }
    }

    if let Some(ref _then) = command.then {
        todo!()
    }
}
