use super::parsing::ast::{Argument, Command, Program, Statement};

use std::borrow::Cow;
use std::collections::{HashMap, hash_map::Entry};
use std::fs;

pub type Context<'a> = HashMap<&'a str, String>;

pub fn evaluate_program(program: &Program<'_>) {
    let mut ctx: Context<'_> = HashMap::new();
    for statement in &program.0 {
        evaluate_statement(statement, &mut ctx);
    }
}

pub fn evaluate_statement<'a>(statement: &Statement<'a>, ctx: &mut Context<'a>) {
    match statement {
        Statement::Declaration { name, value } => {
            let (value, exit_code) = evaluate_command(value, ctx);
            ctx.insert(name, value);
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }
        }
        Statement::Assignment { name, value } => {
            let (value, exit_code) = evaluate_command(value, ctx);
			match ctx.entry(name) {
				Entry::Occupied(mut existing) => {
					existing.insert(value);
				}
				Entry::Vacant(_) => {
					panic!("set requires variable {name:?} to be defined")
				}
			};
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }
        }
        Statement::Command(command) => {
            let (_, exit_code) = evaluate_command(command, ctx);
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }
        }
        Statement::For {
            iterator,
            statements,
        } => {
            let (result, exit_code) = evaluate_command(iterator, ctx);
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }

			ctx.insert("break", "".to_owned());
            for part in result.trim_end().split('\n') {
				if ctx.get("break").expect("no 'break' variable") == "break" {
					break;
				}

                let part = part.strip_suffix('\r').unwrap_or(part);
                match iterator.name {
                    "git" => match iterator.arguments.first().map(|arg| arg.0) {
                        Some("tag") => {
                            ctx.insert("tag", part.to_owned());
                        }
                        Some("log") => {
                            ctx.insert("ref", part.to_owned());
                        }
                        _ => {}
                    },
                    "files" => {
                        ctx.insert("file", part.to_owned());
                    }
                    "constant" => {
                        let first_argument = iterator.arguments[0].0;
                        if let Some(name) = first_argument
                            .strip_prefix('$')
                            .and_then(|rest| crate::utilities::depluralise(rest))
                        {
                            ctx.insert(name, part.to_owned());
                        }
                    }
                    _ => {}
                }
                ctx.insert("iter", part.to_owned());
                for statement in statements {
                    evaluate_statement(statement, ctx);
                }
            }
        }
    }
}

/// interpolate variables
fn evaluate_argument<'a>(argument: &Argument<'a>, ctx: &'a Context<'a>) -> Cow<'a, str> {
    let mut result = Cow::Borrowed("");
    let mut start = 0;
    let on = &argument.0;
    let mut last_was_escape_backslash = false;
    for (idx, matched) in on.match_indices(['$', '\\', '\'', '"']) {
        let skip = last_was_escape_backslash && matched == "\\";
        last_was_escape_backslash = false;
        if skip {
            continue;
        }

        result += &on[start..idx];
        if let "$" = matched {
            let rest = &on[(idx + 1)..];
            let reference = rest
                .split_once(|chr: char| !(chr.is_alphanumeric() || matches!(chr, '_')))
                .map_or(rest, |(rest, _)| rest);
            if let "ctx" = reference {
                result += Cow::Owned(format!("{ctx:?}"));
            } else if let Some(argument) = ctx.get(&reference) {
                result += Cow::Borrowed(argument.as_str());
            } else if let Some(env) = crate::utilities::get_environment_variable(reference) {
                result += Cow::Owned(env);
            } else {
                eprintln!("shell-language: Could not find reference {reference}");
            }
            start = idx + 1 + reference.len();
        } else if let "\\" = matched {
            match on[(idx + 1)..].chars().next() {
                Some('n') => {
                    result += Cow::Borrowed("\n");
                }
                Some('t') => {
                    result += Cow::Borrowed("\t");
                }
                Some('r') => {
                    result += Cow::Borrowed("\r");
                }
                Some('\\') => {
                    last_was_escape_backslash = true;
                    result += Cow::Borrowed("\\");
                }
                Some('"') => {
                    result += Cow::Borrowed("\"");
                }
                Some('\'') => {
                    result += Cow::Borrowed("'");
                }
                character => {
                    eprintln!("unknown escape {character:?}");
                }
            }
            start = idx + 2;
        } else if let "\"" | "'" = matched {
            let skip = on[..idx].is_empty()
                || on[..idx].ends_with(&['=', '\\'])
                || on[idx..][1..].is_empty();
            start = if skip { idx + 1 } else { idx };
        } else {
            unreachable!("matched '{matched}'");
        }
    }
    result += &on[start..];
    result
}

#[allow(clippy::too_many_lines)]
pub fn evaluate_command(command: &Command<'_>, ctx: &Context) -> (String, Option<i32>) {
    match command.name {
        // Command line printing
        "echo" | "echo_stdout" => {
            let mut some = false;
            if let Some("run") = command.arguments.first().map(|arg| arg.0) {
                let mut arguments = command.arguments[1..].iter();

                let first_argument = arguments.next().expect("command name");
                let command = evaluate_argument(first_argument, ctx);
                let args = arguments
                    .map(|arg| evaluate_argument(arg, ctx).into_owned())
                    .filter(|arg| !arg.is_empty())
                    .collect::<Vec<String>>();

                let (_output, result) =
                    crate::utilities::run_command(command.into_owned(), args, None, false, false);

                (String::new(), result.code())
            } else {
                for (idx, argument) in command.arguments.iter().enumerate() {
                    let result = evaluate_argument(argument, ctx);
                    if !result.is_empty() {
                        some = true;
                        if idx > 0 {
                            print!(" ");
                        }
                        print!("{result}");
                    }
                }
                if command.arguments.is_empty() || some {
                    println!();
                }
                (String::new(), None)
            }
        }
        "echo_stderr" => {
            for (idx, argument) in command.arguments.iter().enumerate() {
                if idx > 0 {
                    eprint!(" ");
                }
                eprint!("{}", evaluate_argument(argument, ctx));
            }
            eprintln!();
            (String::new(), None)
        }
        // Run command
        name @ ("run" | "with") => {
            let mut arguments = command.arguments.iter();
            let mut env: Vec<(String, String)> = Vec::new();
            if let "with" = name {
                while let Some(key) = arguments.next() {
                    if let "run" = key.0 {
                        break;
                    }
                    let key = evaluate_argument(key, ctx);
                    let value = arguments.next().expect("env value");
                    let value = evaluate_argument(value, ctx);
                    if !value.is_empty() {
                        env.push((key.into_owned(), value.into_owned()));
                    }
                }
            }

            let first_argument = arguments.next().expect("command name");
            let command = evaluate_argument(first_argument, ctx);
            let mut args: Vec<String> = arguments
                .map(|arg| evaluate_argument(arg, ctx).into_owned())
                .filter(|arg| !arg.is_empty())
                .collect();

            let (capture_stdout, capture_stderr) = if args
                .pop_if(|top| top == "--merge-stdout-and-stderr")
                .is_some()
            {
                (true, true)
            } else if args.pop_if(|top| top == "--only-capture-stderr").is_some() {
                (false, true)
            } else {
                (true, false)
            };

            let (output, result) = crate::utilities::run_command(
                command.into_owned(),
                args,
                Some(env),
                capture_stdout,
                capture_stderr,
            );

            (output, result.code())
        }
        // Environment variables
        "env" => {
            let mut arguments = command.arguments.iter();
            let name: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            if let Some(value) = crate::utilities::get_environment_variable(name) {
                (value, Some(0))
            } else {
                eprintln!("Could not find environment variable {name}");
                (String::default(), Some(1))
            }
        }
        // Filesystem manipulation
        "mv" | "move" => {
            use std::path::Path;

            let mut arguments = command.arguments.iter();
            let from: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let to: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let response = crate::utilities::move_copy_file(Path::new(from), Path::new(to), true);
            match response {
                Ok(()) => (String::default(), Some(0)),
                Err(err) => {
                    eprintln!("error moving file: {err:?}");
                    (String::default(), Some(0))
                }
            }
        }
        "cp" | "copy" => {
            use std::path::Path;

            let mut arguments = command.arguments.iter();
            let from: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let to: &str = &evaluate_argument(arguments.next().unwrap(), ctx);

            let response = crate::utilities::move_copy_file(Path::new(from), Path::new(to), false);
            match response {
                Ok(()) => (String::default(), Some(0)),
                Err(err) => {
                    eprintln!("error copying file: {err:?}");
                    (String::default(), Some(0))
                }
            }
        }
        "rm" | "remove" => {
            use std::path::Path;

            let mut arguments = command.arguments.iter();
            let path: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let path: &Path = Path::new(path);
            if path.is_dir() {
                fs::remove_dir(path).unwrap();
                (String::default(), Some(0))
            } else if path.is_file() {
                fs::remove_file(path).unwrap();
                (String::default(), Some(0))
            } else {
                eprintln!("unknown path item to remove");
                (String::default(), Some(1))
            }
        }
        // Scan files
        "files" => {
            let pattern = if let Some(arg) = command.arguments.first() {
                evaluate_argument(arg, ctx)
            } else {
                Cow::Borrowed("")
            };
            match glob::glob(&pattern) {
                Ok(paths) => {
                    let mut output = String::new();
                    for path in paths {
                        if !output.is_empty() {
                            output.push('\n');
                        }
                        output.push_str(&path.unwrap().display().to_string().replace('\\', "/"));
                    }
                    (output, Some(0))
                }
                Err(err) => {
                    eprintln!("Error reading files glob {err:?}");
                    (String::default(), Some(1))
                }
            }
        }
        // File reads and writes
        "write" => {
            let mut arguments = command.arguments.iter();
            let path: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let output: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            if fs::write(path, output).is_ok() {
                (String::default(), Some(0))
            } else {
                eprintln!("Could not write to {path}");
                (String::default(), Some(1))
            }
        }
        "read" => {
            let mut arguments = command.arguments.iter();
            let path: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            if let Ok(content) = fs::read_to_string(path) {
                (content, Some(0))
            } else {
                eprintln!("Could not read {path}");
                (String::default(), Some(1))
            }
        }
        "append" => {
            let mut arguments = command.arguments.iter();
            let path: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let to_append: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            if let Ok(mut content) = fs::read_to_string(path) {
                // I think this is okay
                content.push('\n');
                content.push_str(to_append);
                if fs::write(path, content).is_ok() {
                    (String::default(), Some(0))
                } else {
                    eprintln!("Could not write to {path}");
                    (String::default(), Some(1))
                }
            } else {
                eprintln!("Could not read {path}");
                (String::default(), Some(1))
            }
        }
        // String commands
        "repeat" => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let repeat: usize = evaluate_argument(arguments.next().unwrap(), ctx)
                .parse()
                .expect("invalid repeater");
            (item.repeat(repeat), None)
        }
        "replace" => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().expect("no item"), ctx);
            let from: &str = &evaluate_argument(arguments.next().expect("no item to replace"), ctx);
            let to: &str = &evaluate_argument(arguments.next().expect("no replacer"), ctx);
            (item.replace(from, to), None)
        }
        "split" => {
            use std::fmt::Write;

			let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().expect("no item"), ctx);
            let splitter: &str = &evaluate_argument(arguments.next().expect("no splitter to replace"), ctx);

			let mut s = String::default();
			for item in item.split(splitter) {
				if !s.is_empty() {
					writeln!(&mut s).unwrap();
				}
				write!(&mut s, "{item}").unwrap();
            }
            (s, None)
        }
        "concatenate" => {
            use std::fmt::Write;

            let mut s = String::new();
            for argument in &command.arguments {
                let argument = evaluate_argument(argument, ctx);
                if !argument.is_empty() {
                    if !s.is_empty() {
                        writeln!(&mut s).unwrap();
                    }
                    write!(&mut s, "{argument}").unwrap();
                }
            }
            (s, None)
        }
        "concatenate_separator" => {
            // could concatenate + replace...
            use std::fmt::Write;

            let mut s = String::new();
            let mut arguments = command.arguments.iter();
            let separator: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            for argument in arguments {
                let argument = evaluate_argument(argument, ctx);
                if !argument.is_empty() {
                    if !s.is_empty() {
                        write!(&mut s, "{separator}").unwrap();
                    }
                    write!(&mut s, "{argument}").unwrap();
                }
            }
            (s, None)
        }
        str_slice_cmd @ ("before" | "after" | "rbefore" | "rafter") => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let splitter: &str = &evaluate_argument(arguments.next().unwrap(), ctx);

            let item = if str_slice_cmd.starts_with('r') {
                item.rsplit_once(splitter)
            } else {
                item.split_once(splitter)
            };
            let out = if let Some((before, after)) = item {
                if str_slice_cmd.ends_with("before") {
                    before.to_owned()
                } else {
                    after.to_owned()
                }
            } else {
                arguments
                    .next()
                    .map(|default| evaluate_argument(default, ctx).into_owned())
                    .unwrap_or_default()
            };
            (out, None)
        }
        line_cmd @ ("last_line" | "first_line") => {
            let mut arguments = command.arguments.iter();
            let first_argument = arguments.next().unwrap();
            let item: &str = &evaluate_argument(first_argument, ctx);

            let mut lines = item.lines();
            let out = if line_cmd.starts_with("first") {
                lines.next()
            } else {
                lines.next_back()
            };
            (out.unwrap_or_default().to_owned(), None)
        }
        "size" => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            (item.len().to_string(), None)
        }
        "lines" => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            (item.lines().count().to_string(), None)
        }
        "trim" => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            (item.trim().to_owned(), None)
        }
        "format_number" => {
            let mut arguments = command.arguments.iter();
            let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            // implementation is a little borked
            let result = if item.contains('.') {
                let num: f64 = item.parse().expect("cannot format non float");
                let is_negative = num.is_sign_negative();
                let abs_num = num.abs();
                let (whole, fract) = (abs_num.trunc(), abs_num.fract());
                let sign = if is_negative { "-" } else { "" };
                let whole = crate::utilities::separate_numbers(&whole.to_string());
                let fract = if fract == 0.0 {
                    String::new()
                } else {
                    let fract = crate::utilities::separate_numbers_fract(&fract.to_string()[2..]);
                    format!(" . {fract}")
                };
                format!("{sign}{whole}{fract}")
            } else if item.contains('-') {
                let num: i64 = item.parse().expect("cannot format non integer");
                let is_negative = num.is_negative();
                let sign = if is_negative { "-" } else { "" };
                let whole = crate::utilities::separate_numbers(&num.abs().to_string());
                format!("{sign}{whole}")
            } else {
                let num: u64 = item.parse().expect("cannot format non natural");
                crate::utilities::separate_numbers(&num.to_string())
            };
            (result, None)
        }
        // regular expressions string conditionals
        #[cfg(feature = "regular-expressions")]
        "regexp" => {
            use regress::Regex;

            let mut arguments = command.arguments.iter();
            let source: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
            let expression: &str = &evaluate_argument(arguments.next().unwrap(), ctx);

            let re = Regex::new(expression).unwrap();
            let result = re.find(source);

            if let Some(r#match) = result {
                if let Some("extract") = arguments.next().map(|argument| argument.0) {
                    let name: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                    let range = r#match
                        .named_groups()
                        .find_map(|(key, range)| (key == name).then_some(range));
                    if let Some(range) = range.flatten() {
                        (source[range].to_owned(), Some(0))
                    } else {
                        eprintln!("No group {name}");
                        (String::default(), Some(1))
                    }
                } else {
                    (source[r#match.range].to_owned(), Some(0))
                }
            } else {
                eprintln!("No match");
                (String::default(), Some(1))
            }
        }
        // Utilities
        #[cfg(feature = "date")]
        "date" => {
            use jiff::{Timestamp, fmt::strtime};

            let mut arguments = command.arguments.iter();
            let format = if let Some(argument) = arguments.next() {
                evaluate_argument(argument, ctx)
            } else {
                Cow::Borrowed("%a %-d %b %Y %T %z")
            };
            let now = Timestamp::now();
            if let Ok(rendered) = strtime::format(&*format, now) {
                (rendered, None)
            } else {
                eprintln!("Invalid format {format}");
                (String::default(), Some(1))
            }
        }
        // Control flow
        "if_equal" => {
            let mut arguments = command.arguments.iter();
            let first_argument = arguments.next().unwrap();
            let second_argument = arguments.next().unwrap();
            let then = arguments.next().unwrap();
            let equal =
                evaluate_argument(first_argument, ctx) == evaluate_argument(second_argument, ctx);
            let out = if equal {
                evaluate_argument(then, ctx).into_owned()
            } else {
                arguments
                    .next()
                    .map(|r#else| evaluate_argument(r#else, ctx).into_owned())
                    .unwrap_or_default()
            };
            (out, None)
        }
        "if_contains" => {
            let mut arguments = command.arguments.iter();
            let item = arguments.next().unwrap();
            let substring = arguments.next().unwrap();
            let then = arguments.next().unwrap();
            let contains =
                evaluate_argument(item, ctx).contains(&*evaluate_argument(substring, ctx));
            let out = if contains {
                evaluate_argument(then, ctx).into_owned()
            } else {
                arguments
                    .next()
                    .map(|r#else| evaluate_argument(r#else, ctx).into_owned())
                    .unwrap_or_default()
            };
            (out, None)
        }
        // TODO WIP. "known programs"
        command_name @ ("cargo" | "git" | "gh" | "hyperfine" | "jq" | "yq" | "node" | "deno"
        | "bun" | "sqlite3" | "python" | "npm" | "bat") => {
            let args = command
                .arguments
                .iter()
                .map(|arg| evaluate_argument(arg, ctx).into_owned())
                .filter(|arg| !arg.is_empty())
                .collect::<Vec<String>>();

            let (output, result) =
                crate::utilities::run_command(command_name.to_owned(), args, None, true, false);

            (output, result.code())
        }
        // For constants
        "literal" | "constant" => {
            // skip any others
            let first_argument = command.arguments.first().unwrap();
            (evaluate_argument(first_argument, ctx).into_owned(), None)
        }
        // For conditionally invoking commands
        "noop" => (String::default(), None),
        name => {
            eprintln!("unknown command '{name}'");
            (String::default(), Some(1))
        }
    }
}
