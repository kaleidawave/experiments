fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs};

    let mut args = env::args().skip(1);
    let Some(first) = args.next() else {
        let run_id = option_env!("GITHUB_RUN_ID");
        let date = option_env!("GITHUB_RUN_DATE");
        let after = run_id
            .map(|commit| format!(" (commit {commit} {date:?})"))
            .unwrap_or_default();
        println!("the Ben shell (WIP){after}");
        return Ok(());
    };

    let source: String = if let "--evaluate" | "-e" = first.as_str() {
        args.next().unwrap_or_default()
    } else {
        fs::read_to_string(first)?
    };

    let rest: Vec<_> = args.collect();

    let debug_program: bool = rest.iter().any(|flag| flag == "--debug-program");

    let program = parsing::parse_program(&source);

    if debug_program {
        eprintln!("{program:#?}");
    } else {
        evaluate::evaluate_program(&program);
    }

    Ok(())
}

mod ast {
    #[derive(Debug)]
    pub struct Program<'a>(pub Vec<Statement<'a>>);

    #[derive(Debug)]
    pub enum Statement<'a> {
        Declaration {
            name: &'a str,
            value: Command<'a>,
        },
        For {
            iterator: Command<'a>,
            statements: Vec<Statement<'a>>,
        },
        Command(Command<'a>),
    }

    #[derive(Debug)]
    pub struct Command<'a> {
        pub name: &'a str,
        pub arguments: Vec<Argument<'a>>,
    }

    /// Holds strings and literals
    #[derive(Debug)]
    pub struct Argument<'a>(pub &'a str);
}

mod parsing {
    use super::ast::{Argument, Command, Program, Statement};

    pub fn parse_program(on: &str) -> Program<'_> {
        let mut stmts: Vec<Statement> = Vec::new();

        let mut lines = on.lines();
        while let Some(line) = lines.next() {
            if let Some(stmt) = parse_statement(line, &mut lines) {
                stmts.push(stmt);
            }
        }
        Program(stmts)
    }

    fn parse_statement<'a>(
        line: &'a str,
        lines: &mut dyn Iterator<Item = &'a str>,
    ) -> Option<Statement<'a>> {
        // comment or empty
        if line.starts_with('#') || line.trim().is_empty() {
            None
        } else if let Some(rest) = line.strip_prefix("let ") {
            let (name, rest) = rest.split_once(" = ").expect("let declaration needs ' = '");
            Some(Statement::Declaration {
                name,
                value: parse_command(rest),
            })
        } else if let Some(inner) = line
            .strip_prefix("for ")
            .and_then(|line| line.strip_suffix(" each"))
        {
            let iterator = parse_command(inner);
            let mut rest = lines.map_while(|line| {
                if let line @ Some(_) = line.strip_prefix("\t") {
                    line
                } else if let line @ Some(_) = line.strip_prefix("  ") {
                    line
                } else if line.trim().is_empty() {
                    Some(line)
                } else {
                    None
                }
            });
            let mut statements = Vec::new();
            while let Some(line) = rest.next() {
                if let Some(stmt) = parse_statement(line, &mut rest) {
                    statements.push(stmt);
                }
            }
            Some(Statement::For {
                iterator,
                statements,
            })
        } else {
            Some(Statement::Command(parse_command(line)))
        }
    }

    fn parse_command(on: &str) -> Command<'_> {
        let mut name = "";
        let mut arguments = Vec::new();
        let mut in_string = false;
        let mut last = 0;
        for (idx, chr) in on.char_indices() {
            if in_string {
                if let '"' | '\'' = chr {
                    arguments.push(Argument(&on[last..idx]));
                    last = idx + 1;
                    in_string = false;
                }
            } else if let '"' | '\'' = chr {
                in_string = true;
                last = idx + 1;
            } else if let ' ' = chr {
                let part = &on[last..idx].trim();
                if !part.is_empty() {
                    if name.is_empty() {
                        name = part;
                    } else {
                        arguments.push(Argument(part));
                    }
                    last = idx + chr.len_utf8();
                }
            }
        }
        let rest = &on[last..].trim();
        if !rest.is_empty() {
            if name.is_empty() {
                name = rest;
            } else {
                arguments.push(Argument(rest));
            }
        }
        Command { name, arguments }
    }
}

mod evaluate {
    use super::ast::{Argument, Command, Program, Statement};

    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::fs;

    type Context<'a> = HashMap<&'a str, String>;

    pub fn evaluate_program(program: &Program<'_>) {
        let mut ctx: Context<'_> = HashMap::new();
        for statement in &program.0 {
            evaluate_statement(statement, &mut ctx);
        }
    }

    fn evaluate_statement<'a>(statement: &Statement<'a>, ctx: &mut Context<'a>) {
        match statement {
            Statement::Declaration { name, value } => {
                let (value, exit_code) = evaluate_command(value, ctx);
                ctx.insert(name, value);
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

                for part in result.trim_end().split('\n') {
                    let part = part.strip_suffix('\r').unwrap_or(part);
                    match iterator.name {
                        "git" if iterator.arguments.first().is_some_and(|arg| arg.0 == "tag") => {
                            ctx.insert("tag", part.to_owned());
                        }
                        "files" => {
                            ctx.insert("file", part.to_owned());
                        }
                        "constant" => {
                            if let Some(name) =
                                crate::utilities::depluralise(iterator.arguments[0].0)
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
        for (index, matched) in on.match_indices(['$', '\\']) {
            let skip = last_was_escape_backslash && matched == "\\";
            last_was_escape_backslash = false;
            if skip {
                continue;
            }

            result += &on[start..index];
            if let "$" = matched {
                let rest = &on[(index + 1)..];
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
                    eprintln!("Could not find reference {reference}");
                }
                start = index + 1 + reference.len();
            } else if let "\\" = matched {
                match on[(index + 1)..].chars().next() {
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
                    Some('\"') => {}
                    character => {
                        eprintln!("unknown escape {character:?}");
                    }
                }
                start = index + 2;
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
                use std::io::{Read, pipe};
                use std::process::Command;

                // Using pipe we collect both stdout and stderr in order
                let (mut reader, writer) = pipe().expect("could not create pipe");

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
                let command: &str = &evaluate_argument(first_argument, ctx);
                let args = arguments
                    .map(|arg| evaluate_argument(arg, ctx).into_owned())
                    .filter(|arg| !arg.is_empty())
                    .collect::<Vec<String>>();

                let mut child = Command::new(command)
                    .args(args)
                    .stdout(writer.try_clone().expect("could not clone writer pipe"))
                    .stderr(writer)
                    .envs(env)
                    .spawn()
                    .expect("Failed to spawn command");

                let mut output = String::new();
                reader.read_to_string(&mut output).expect("invalid UTF8");

                let result = child.wait().expect("command not finished");

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
                let response =
                    crate::utilities::move_copy_file(Path::new(from), Path::new(to), true);
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

                let response =
                    crate::utilities::move_copy_file(Path::new(from), Path::new(to), false);
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
                            output
                                .push_str(&path.unwrap().display().to_string().replace('\\', "/"));
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
                let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                let from: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                let to: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                (item.replace(from, to), None)
            }
            "concatenate" => {
                use std::fmt::Write;
                let mut s = String::new();
                for argument in &command.arguments {
                    if !s.is_empty() {
                        writeln!(&mut s).unwrap();
                    }
                    let argument = evaluate_argument(argument, ctx);
                    if !argument.is_empty() {
                        write!(&mut s, "{argument}").unwrap();
                    }
                }
                (s, None)
            }
            "concatenate_separator" => {
                use std::fmt::Write;
                let mut s = String::new();
                let mut arguments = command.arguments.iter();
                let separator: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                for argument in arguments {
                    if !s.is_empty() {
                        write!(&mut s, "{separator}").unwrap();
                    }
                    let argument = evaluate_argument(argument, ctx);
                    if !argument.is_empty() {
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
                    String::new()
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
            // Control flow
            "if_equal" => {
                let mut arguments = command.arguments.iter();
                let first_argument = arguments.next().unwrap();
                let second_argument = arguments.next().unwrap();
                let third_argument = arguments.next().unwrap();
                let equal = evaluate_argument(first_argument, ctx)
                    == evaluate_argument(second_argument, ctx);
                let out = if equal {
                    evaluate_argument(third_argument, ctx)
                } else {
                    let fourth_argument = arguments.next();
                    if let Some(fourth_argument) = fourth_argument {
                        evaluate_argument(fourth_argument, ctx)
                    } else {
                        Cow::Borrowed("")
                    }
                };
                (out.into_owned(), None)
            }
            // TODO WIP. "known programs"
            command_name @ ("cargo" | "git" | "gh" | "hyperfine" | "jq" | "yq" | "node"
            | "deno" | "bun" | "sqlite3" | "python" | "npm" | "bat") => {
                use std::io::{Read, pipe};
                use std::process::Command;

                let (mut reader, writer) = pipe().expect("could not create pipe");

                let arguments = command.arguments.iter();
                let args = arguments
                    .map(|arg| evaluate_argument(arg, ctx).into_owned())
                    .filter(|arg| !arg.is_empty())
                    .collect::<Vec<String>>();

                let mut child = Command::new(command_name)
                    .args(args)
                    .stdout(writer.try_clone().expect("could not clone writer pipe"))
                    .stderr(writer)
                    .spawn()
                    .expect("Failed to spawn command");

                let mut output = String::new();
                reader.read_to_string(&mut output).expect("invalid UTF8");

                let result = child.wait().expect("command not finished");

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
}

mod utilities {
    use std::{env, fs, path::Path};

    pub fn get_environment_variable(name: &str) -> Option<String> {
        env::vars().find_map(|(n, v)| (n == name).then_some(v))
    }

    /// `remove_after = true => move`, `remove_after = false => copy`
    /// TODO if `remove_after`, in some cases can [rename](https://doc.rust-lang.org/std/fs/fn.rename.html) sometimes here
    pub fn move_copy_file(
        from: &Path,
        to: &Path,
        remove_after: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if from.is_dir() {
            todo!("copy/move directory");
        } else if from.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = fs::read(from)?;
            fs::write(to, content)?;

            let metadata = fs::metadata(from)?;
            let permissions = metadata.permissions();
            fs::set_permissions(to, permissions)?;

            if remove_after {
                fs::remove_file(from)?;
            }

            Ok(())
        } else {
            Err("Unknown path item to move".into())
        }
    }

    /// Reverse <https://howtospell.co.uk/y-to-ies-or-s-plural-rule>
    pub fn depluralise(on: &str) -> Option<&str> {
        on.strip_suffix("ies").or_else(|| on.strip_suffix("s"))
    }
}
