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
            let (value, exit_code) = evaluate_command(
                value.name,
                Arguments::new(&value.arguments, ctx),
                value.then.as_deref(),
            );
            ctx.insert(name, value);
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }
        }
        Statement::Assignment { name, value } => {
            let (value, exit_code) = evaluate_command(
                value.name,
                Arguments::new(&value.arguments, ctx),
                value.then.as_deref(),
            );
            match ctx.entry(name) {
                Entry::Occupied(mut existing) => {
                    existing.insert(value);
                }
                Entry::Vacant(_) => {
                    panic!("set requires variable {name:?} to be defined")
                }
            }
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }
        }
        Statement::Command(command) => {
            let (_, exit_code) = evaluate_command(
                command.name,
                Arguments::new(&command.arguments, ctx),
                command.then.as_deref(),
            );
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }
        }
        Statement::For {
            iterator,
            statements,
        } => {
            let (result, exit_code) = evaluate_command(
                iterator.name,
                Arguments::new(&iterator.arguments, ctx),
                iterator.then.as_deref(),
            );
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }

            ctx.insert("break", String::new());
            for part in result.trim_end().split('\n') {
                if ctx.get("break").expect("no 'break' variable") == "break" {
                    break;
                }

                let part = part.strip_suffix('\r').unwrap_or(part);
                // Set iterator variable name
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
        Statement::If {
            condition,
            statements,
        } => {
            let (result, exit_code) = evaluate_command(
                condition.name,
                Arguments::new(&condition.arguments, ctx),
                condition.then.as_deref(),
            );
            if let Some(exit_code) = exit_code {
                ctx.insert("exit_code", exit_code.to_string());
            }

            if !result.is_empty() {
                for statement in statements {
                    evaluate_statement(statement, ctx);
                }
            }
        }
    }
}

struct CommandContext<'a> {
    // Because cannot mutate argument half way through
    last: Option<String>,
    ctx: &'a Context<'a>,
}

/// interpolate variables
fn evaluate_argument<'a>(argument: &Argument<'a>, ctx: &'a CommandContext<'a>) -> Cow<'a, str> {
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
                result += Cow::Owned(format!("{ctx:?}", ctx=ctx.ctx));
            } else if let ("piped" | "last", Some(argument)) = (reference, &ctx.last) {
                result += Cow::Borrowed(argument.as_str());
            } else if let Some(argument) = ctx.ctx.get(&reference) {
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
                || on[..idx].ends_with(['=', '\\'])
                || on[idx..][1..].is_empty();
            start = if skip { idx + 1 } else { idx };
        } else {
            unreachable!("matched '{matched}'");
        }
    }
    result += &on[start..];
    result
}

pub struct Arguments<'a> {
    context: CommandContext<'a>,
    arguments: &'a [crate::parsing::ast::Argument<'a>],
    idx: usize,
}

impl<'a> Arguments<'a> {
    pub fn peek_first(&self) -> &'a str {
        self.arguments.first().map(|arg| arg.0).unwrap_or_default()
    }

    pub fn context(&self) -> &'a Context {
        self.context.ctx
    }

    pub fn new(arguments: &'a [crate::parsing::ast::Argument<'a>], ctx: &'a Context<'a>) -> Self {
        Self {
            context: CommandContext { last: None, ctx },
            arguments,
            idx: 0,
        }
    }

    pub fn new_with_last(
        arguments: &'a [crate::parsing::ast::Argument<'a>],
        ctx: &'a Context<'a>,
        last: String,
    ) -> Self {
        Self {
            context: CommandContext {
                last: Some(last),
                ctx,
            },
            arguments,
            idx: 0,
        }
    }
}

impl<'a> Iterator for Arguments<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0
            && self.context.last.is_some()
            && !self.arguments.iter().any(|arg| arg.0.contains("$piped"))
        {
            let value = self.context.last.take().unwrap();
            Some(Cow::Owned(value))
        } else {
            let idx = self.idx;
            self.idx += 1;
            self.arguments
                .get(idx)
                .map(|arg| evaluate_argument(arg, &self.context))
        }
    }
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn evaluate_command<'a>(
    name: &'a str,
    mut arguments: Arguments<'a>,
    then: Option<&'a Command>,
) -> (String, Option<i32>) {
    let (out, exit_code): (String, Option<i32>) = match name {
        // Command line printing
        "echo" | "echo_stdout" => {
            if let "run" = arguments.peek_first() {
                // Skip "run"
                let _ = arguments.next();
                let command = arguments.next().expect("command name");
                let args = arguments
                    .by_ref()
                    .filter(|arg| !arg.is_empty())
                    .map(|arg| arg.into_owned())
                    .collect::<Vec<String>>();

                let (_output, result) =
                    crate::utilities::run_command(command.into_owned(), args, None, false, false);

                (String::new(), result.code())
            } else {
                let mut some = false;
                for (idx, argument) in arguments.by_ref().enumerate() {
                    if !argument.is_empty() {
                        some = true;
                        if idx > 0 {
                            print!(" ");
                        }
                        print!("{argument}");
                    }
                }
                if some {
                    println!();
                }
                (String::new(), None)
            }
        }
        "echo_stderr" => {
            let mut some = false;
            for (idx, argument) in arguments.by_ref().enumerate() {
                if !argument.is_empty() {
                    some = true;
                    if idx > 0 {
                        eprint!(" ");
                    }
                    eprint!("{argument}");
                }
            }
            if some {
                eprintln!();
            }
            (String::new(), None)
        }
        // Run command
        name @ ("run" | "with") => {
            let mut env: Vec<(String, String)> = Vec::new();
            if let "with" = name {
                while let Some(key) = arguments.next() {
                    // TODO this can come from computed...?
                    if let "run" = &*key {
                        break;
                    }
                    let value = arguments.next().expect("env value");
                    if !value.is_empty() {
                        env.push((key.into_owned(), value.into_owned()));
                    }
                }
            }

            let command = arguments.next().expect("command name");
            let mut args: Vec<String> = arguments.by_ref().map(|arg| arg.into_owned()).collect();

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
            let name: &str = &arguments.next().expect("no env name");
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

            let from: &str = &arguments.next().unwrap();
            let to: &str = &arguments.next().unwrap();
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

            let from: &str = &arguments.next().unwrap();
            let to: &str = &arguments.next().unwrap();

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

            let path: &str = &arguments.next().unwrap();
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
        // When downloading executables it loses information, this corrects
        "ee" | "ensure_executable" => {
            #[cfg(unix)]
            let result: std::io::Result<()> = {
                use std::fs::{metadata, set_permissions};
                use std::os::unix::fs::PermissionsExt;
                use std::path::Path;

                let path: &str = &arguments.next().unwrap();
                crate::utilities::visit_paths(Path::new(path), &|file_path| {
                    let metadata = metadata(&path).unwrap();
                    let mut permissions = metadata.permissions();
                    let mode = permissions.mode();
                    // Add user/group/other execute bits (0111)
                    permissions.set_mode(mode | 0o111);
                    let res = set_permissions(file_path, permissions);
                    if res.is_err() {
                        eprintln!("Error setting permission {res:?}");
                    } else {
                        eprintln!(
                            "Made {file_path} executable",
                            file_path = file_path.display()
                        );
                    }
                })
            };

            #[cfg(not(unix))]
            let result: std::io::Result<()> = Ok(());

            match result {
                Ok(()) => (String::default(), Some(0)),
                Err(err) => {
                    eprintln!("Ensuring executables {err:?}");
                    (String::default(), Some(1))
                }
            }
        }
        // Scan files
        "files" => {
            let pattern: &str = &arguments.next().expect("expected file pattern");
            match glob::glob(pattern) {
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
            let path: &str = &arguments.next().unwrap();
            let output: &str = &arguments.next().unwrap();
            if fs::write(path, output).is_ok() {
                (String::default(), Some(0))
            } else {
                eprintln!("Could not write to {path}");
                (String::default(), Some(1))
            }
        }
        "read" => {
            let path: &str = &arguments.next().unwrap();
            if let Ok(content) = fs::read_to_string(path) {
                (content, Some(0))
            } else {
                eprintln!("Could not read {path}");
                (String::default(), Some(1))
            }
        }
        "append" => {
            let path: &str = &arguments.next().unwrap();
            let to_append: &str = &arguments.next().unwrap();
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
            let item: &str = &arguments.next().unwrap();
            let repeat: usize = arguments.next().unwrap().parse().expect("invalid repeater");
            (item.repeat(repeat), None)
        }
        "replace" => {
            let item: &str = &arguments.next().expect("no item");
            let from: &str = &arguments.next().expect("no item to replace");
            let to: &str = &arguments.next().expect("no replacer");
            (item.replace(from, to), None)
        }
        "debug" => {
            let item: &str = &arguments.next().expect("no item");
            (format!("{item:?}"), None)
        }
        "split" => {
            use std::fmt::Write;

            let item: &str = &arguments.next().expect("no item");
            let splitter: &str = &arguments.next().expect("no splitter to replace");

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
            for argument in arguments.by_ref() {
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
            let separator: &str = &arguments.next().unwrap();
            for argument in arguments.by_ref() {
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
            let item: &str = &arguments.next().unwrap();
            let splitter: &str = &arguments.next().unwrap();

            let item = if str_slice_cmd.starts_with('r') {
                item.rsplit_once(splitter)
            } else {
                item.split_once(splitter)
            };
            let out = if let Some((before, after)) = item {
                if str_slice_cmd.ends_with("before") {
                    Cow::Borrowed(before)
                } else {
                    Cow::Borrowed(after)
                }
            } else {
                arguments.next().unwrap_or_default()
            };
            (out.into_owned(), None)
        }
        line_cmd @ ("last_line" | "first_line") => {
            let item: &str = &arguments.next().unwrap();

            let mut lines = item.lines();
            let out = if line_cmd.starts_with("first") {
                lines.next()
            } else {
                lines.next_back()
            };
            let out = out.unwrap_or_default().to_owned();
            (out, None)
        }
        "size" => {
            let item: &str = &arguments.next().unwrap();
            (item.len().to_string(), None)
        }
        "lines" => {
            let item: &str = &arguments.next().unwrap();
            (item.lines().count().to_string(), None)
        }
        "trim" => {
            let item: &str = &arguments.next().unwrap();
            (item.trim().to_owned(), None)
        }
        "format_number" => {
            let item: &str = &arguments.next().unwrap();
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

            let source: &str = &arguments.next().unwrap();
            let expression: &str = &arguments.next().unwrap();

            let re = Regex::new(expression).unwrap();
            let result = re.find(source);

            if let Some(r#match) = result {
                if let Some("extract") = arguments.next().as_deref() {
                    let name: &str = &arguments.next().unwrap();
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

            let format = arguments
                .next()
                .unwrap_or(Cow::Borrowed("%a %-d %b %Y %T %z"));
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
            let first_argument = arguments.next().unwrap();
            let second_argument = arguments.next().unwrap();
            let then = arguments.next().unwrap();
            let equal = first_argument == second_argument;
            let out = if equal {
                then
            } else {
                arguments.next().unwrap_or_default()
            };
            (out.into_owned(), None)
        }
        "if_contains" => {
            let item = arguments.next().unwrap();
            let substring = arguments.next().unwrap();
            let then = arguments.next();
            let contains = item.contains(&*substring);
            let out = if contains {
                then.unwrap()
            } else {
                arguments.next().unwrap_or_default()
            };
            (out.into_owned(), None)
        }
        // TODO WIP. "known programs"
        command_name @ ("cargo" | "git" | "gh" | "hyperfine" | "jq" | "yq" | "node" | "deno"
        | "bun" | "sqlite3" | "python" | "npm" | "bat") => {
            let args = arguments
                .by_ref()
                .map(|arg| arg.into_owned())
                .collect::<Vec<String>>();

            let (output, result) =
                crate::utilities::run_command(command_name.to_owned(), args, None, true, false);

            (output, result.code())
        }
        // For constants
        "literal" | "constant" => (arguments.next().unwrap().into_owned(), None),
        // For conditionally invoking commands
        "noop" => (String::default(), None),
        name => {
            eprintln!("unknown command '{name}'");
            (String::default(), Some(1))
        }
    };

    if let Some(ref then) = then
        && exit_code.is_none_or(|code| code != 0)
    {
        // let context =
        evaluate_command(
            then.name,
            Arguments::new_with_last(&then.arguments, arguments.context(), out),
            then.then.as_deref(),
        )
    } else {
        (out, exit_code)
    }
}
