fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, env};

    let mut args = env::args().skip(1);
    let Some(first) = args.next() else {
        println!("the Ben shell (WIP)");
        return Ok(());
    };

    let source: String = if let "--evaluate" | "-e" = first.as_str() {
        args.next().unwrap_or_default()
    } else {
        fs::read_to_string(first)?
    };

    let program = parsing::parse_program(&source);
    eprintln!("{program:?}");
    evaluate::evaluate_program(program);

    Ok(())
}

mod ast {
    #[derive(Debug)]
    pub struct Program<'a>(pub Vec<Statement<'a>>);

    #[derive(Debug)]
    pub enum Statement<'a> {
        Declaration { name: &'a str, value: Command<'a> },
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
    use super::ast::*;

    pub fn parse_program<'a>(on: &'a str) -> Program<'a> {
        let mut stmts: Vec<Statement> = Vec::new();

        for line in on.lines() {
            // comment or empty
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some(rest) = line.strip_prefix("let ") {
                let (name, rest) = rest.split_once(" = ").expect("let declaration needs ' = '");
                stmts.push(Statement::Declaration {
                    name,
                    value: parse_command(rest),
                });
            } else {
                stmts.push(Statement::Command(parse_command(line)));
            }
        }
        Program(stmts)
    }

    fn parse_command<'a>(on: &'a str) -> Command<'a> {
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
            arguments.push(Argument(rest));
        }
        Command { name, arguments }
    }
}

mod evaluate {
    use super::ast::*;

    use std::{fs, env};
    use std::borrow::Cow;
    use std::collections::HashMap;

    type Context<'a> = HashMap<&'a str, String>;

    pub fn evaluate_program(program: Program<'_>) {
        let mut ctx: Context<'_> = HashMap::new();
        for statement in program.0 {
            match statement {
                Statement::Declaration { name, value } => {
                    ctx.insert(name, evaluate_command(value, &ctx));
                }
                Statement::Command(command) => {
                    let _ = evaluate_command(command, &ctx);
                }
            }
        }
    }

    /// interpolate variables
    fn evaluate_argument<'a>(argument: &Argument<'a>, ctx: &'a Context<'a>) -> Cow<'a, str> {
        let mut result = Cow::Borrowed("");
        let mut start = 0;
        for (index, _matched) in argument.0.match_indices('$') {
            result += &argument.0[start..index];
            let left = &argument.0[(index + 1)..];
            let reference = left.split_once(' ').map_or(left, |(left, _)| left);
            if let Some(argument) = ctx.get(&reference) {
                result += Cow::Borrowed(argument.as_str());
            } else {
                eprintln!("Could not find reference {reference}")
            }
            start = index + 1 + reference.len();
        }
        result += &argument.0[start..];
        result
    }

    pub fn evaluate_command(command: Command<'_>, ctx: &Context) -> String {
        match command.name {
            "literal" => {
                // skip any others
                let first_argument = command.arguments.iter().next().unwrap();
                evaluate_argument(first_argument, ctx).into_owned()
            }
            "echo" => {
                for (idx, argument) in command.arguments.iter().enumerate() {
                    if idx > 0 {
                        print!(" ");
                    }
                    print!("{}", evaluate_argument(argument, ctx));
                }
                println!();
                String::new()
            }
            "run" => {
                use std::io::{Read, pipe};
                use std::process::Command;
                let (mut reader, writer) = pipe().expect("could not create pipe");

                let mut arguments = command.arguments.iter();
                let first_argument = arguments.next().unwrap();
                let command: &str = &evaluate_argument(first_argument, ctx);
                let args = arguments
                    .map(|arg| evaluate_argument(arg, ctx).into_owned())
                    .collect::<Vec<String>>();

                let _result = Command::new(command)
                    .args(args)
                    .stdout(writer.try_clone().expect("could not clone writer pipre"))
                    .stderr(writer)
                    .output()
                    .expect("Failed to execute command");

                let mut output = String::new();
                reader.read_to_string(&mut output).expect("invalid UTF8");
                output
            }
            // Environment variables
            "env" => {
                // eprintln!("{:?}", env::vars());
                let mut arguments = command.arguments.iter();
                let name: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                if let Some(value) = env::vars().find_map(|(n, v)| (n == name).then_some(v.into())) {
                    value
                } else {
                    eprintln!("Could not find environment variable {name}");
                    Default::default()
                }
            }
            // File system
            "write" => {
                let mut arguments = command.arguments.iter();
                let path: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                let output: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                fs::write(path, output).unwrap();
                String::new()
            }
            "read" => {
                let mut arguments = command.arguments.iter();
                let path: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                if let Ok(content) = fs::read_to_string(path) {
                    content
                } else {
                    eprintln!("Could not read {path}");
                    Default::default()
                }
            }
            // String commands
            "repeat" => {
                let mut arguments = command.arguments.iter();
                let item: &str = &evaluate_argument(arguments.next().unwrap(), ctx);
                let repeat: usize = evaluate_argument(arguments.next().unwrap(), ctx).parse().expect("invalid repeater");
                item.repeat(repeat)
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
                if let Some((before, after)) = item {
                    if str_slice_cmd.ends_with("before") {
                        before.to_owned()
                    } else {
                        after.to_owned()
                    }
                } else {
                    String::new()
                }
            }
            line_cmd @ ("last_line" | "first_line") => {
                let mut arguments = command.arguments.iter();
                let first_argument = arguments.next().unwrap();
                let item: &str = &evaluate_argument(first_argument, ctx);

                let mut lines = item.lines();
                if line_cmd.starts_with("first") {
                    lines.next().unwrap_or_default().to_owned()
                } else {
                    lines.next_back().unwrap_or_default().to_owned()
                }
            }
            name => {
                eprintln!("unknown command '{name}'");
                String::new()
            }
        }
    }
}
