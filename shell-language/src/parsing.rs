pub mod ast {
    #[derive(Debug)]
    pub struct Program<'a>(pub Vec<Statement<'a>>);

    #[derive(Debug)]
    pub enum Statement<'a> {
        Declaration {
            name: &'a str,
            value: Command<'a>,
        },
        Assignment {
            name: &'a str,
            value: Command<'a>,
        },
        For {
            iterator: Command<'a>,
            statements: Vec<Statement<'a>>,
        },
        If {
            condition: Command<'a>,
            statements: Vec<Statement<'a>>,
        },
        Command(Command<'a>),
    }

    #[derive(Debug)]
    pub struct Command<'a> {
        pub name: &'a str,
        pub arguments: Vec<Argument<'a>>,
        pub then: Option<Box<Self>>,
    }

    /// Holds strings and literals
    #[derive(Debug)]
    pub struct Argument<'a>(pub &'a str);
}

pub mod parsing {
    use super::ast::{Argument, Command, Program, Statement};

    pub type Lines<'a> = std::iter::Peekable<std::str::Lines<'a>>;

    #[must_use]
    pub fn parse_program(on: &str) -> Program<'_> {
        let mut stmts: Vec<Statement> = Vec::new();

        let mut lines = on.lines().peekable();
        while let Some(line) = lines.next() {
            if let Some(stmt) = parse_statement(line, &mut lines, 0) {
                stmts.push(stmt);
            }
        }
        Program(stmts)
    }

    #[must_use]
    pub fn strip_ident(mut line: &str, upto: usize) -> &str {
        for _ in 0..upto {
            line = line
                .strip_prefix("\t")
                .or_else(|| line.strip_prefix("  "))
                .unwrap_or(line);
        }
        line
    }

    pub fn parse_statement<'a>(
        line: &'a str,
        lines: &mut Lines<'a>,
        depth: usize,
    ) -> Option<Statement<'a>> {
        let line = strip_ident(line, depth);
        // comment or empty
        if line.starts_with('#') || line.trim().is_empty() {
            None
        } else if let Some(rest) = line.trim_start().strip_prefix("let ") {
            let (name, rest) = rest.split_once(" = ").expect("let declaration needs ' = '");
            let value = parse_command(rest);
            Some(Statement::Declaration { name, value })
        } else if let Some(rest) = line.trim_start().strip_prefix("set ") {
            let (name, rest) = rest.split_once(" = ").expect("set declaration needs ' = '");
            let value = parse_command(rest);
            Some(Statement::Assignment { name, value })
        } else if let Some(inner) = line
            .trim_start()
            .strip_prefix("for ")
            .and_then(|line| line.strip_suffix(" each"))
        {
            let iterator = parse_command(inner);
            let mut statements = Vec::new();
            loop {
                let line = lines.next().expect("expected for loop body");
                if let Some(stmt) = parse_statement(line, lines, depth + 1) {
                    statements.push(stmt);
                }
                let r#continue = lines
                    .peek()
                    .map(|line| strip_ident(line, depth))
                    .is_some_and(|line: &str| {
                        line.is_empty() || line.starts_with('\t') || line.starts_with("  ")
                    });
                if !r#continue {
                    break;
                }
            }
            Some(Statement::For {
                iterator,
                statements,
            })
        } else if let Some(inner) = line.trim_start().strip_prefix("if ") {
            let condition = parse_command(inner);
            let mut statements = Vec::new();
            loop {
                let line = lines.next().expect("expected if body");
                if let Some(stmt) = parse_statement(line, lines, depth + 1) {
                    statements.push(stmt);
                }
                let r#continue = lines
                    .peek()
                    .map(|line| strip_ident(line, depth))
                    .is_some_and(|line: &str| {
                        line.is_empty() || line.starts_with('\t') || line.starts_with("  ")
                    });
                if !r#continue {
                    break;
                }
            }
            Some(Statement::If {
                condition,
                statements,
            })
        } else {
            Some(Statement::Command(parse_command(line)))
        }
    }

    fn parse_command(on: &str) -> Command<'_> {
        let mut name = "";
        let mut arguments = Vec::new();
        let mut in_string: Option<char> = None;
        let mut last = 0;
        let mut escaped = false;
        for (idx, chr) in on.char_indices() {
            if let Some(matcher) = in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if chr == matcher {
                    // arguments.push(Argument(&on[last..=idx]));
                    // last = idx + 1;
                    in_string = None;
                }
                escaped = chr == '\\';
            } else if let ('"' | '\'' | '`', "") = (chr, on[last..idx].trim()) {
                in_string = Some(chr);
                // last = idx;
            } else if let ' ' = chr {
                let part = on[last..idx].trim();
                if !part.is_empty() {
                    if part == "then" {
                        let next = parse_command(&on[idx..]);
                        return Command {
                            name,
                            arguments,
                            then: Some(Box::new(next)),
                        };
                    }

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
        Command {
            name,
            arguments,
            then: None,
        }
    }
}
