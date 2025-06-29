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
    use super::utilities;

    pub type Lines<'a> = utilities::LinesWithContinuation<'a>;

    static LINE_CONTINUATIONS: &[&str] = &["then", "\\"];

    #[must_use]
    pub fn parse_program(on: &str) -> Program<'_> {
        let mut stmts: Vec<Statement> = Vec::new();

        let mut lines = Lines::new(on, LINE_CONTINUATIONS);
        while let Some(line) = lines.next() {
            if let Some(stmt) = parse_statement(line, &mut lines, 0) {
                stmts.push(stmt);
            }
        }
        Program(stmts)
    }

    pub fn parse_statement<'a>(
        line: &'a str,
        lines: &mut Lines<'a>,
        depth: usize,
    ) -> Option<Statement<'a>> {
        let line = utilities::strip_indent(line, depth);
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
            while let Some(line) = lines.next() {
                if let Some(stmt) = parse_statement(line, lines, depth + 1) {
                    statements.push(stmt);
                }
                let next = utilities::strip_indent(lines.rest(), depth);
                let r#continue =
                    next.is_empty() || next.starts_with('\t') || next.starts_with("  ");
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
            while let Some(line) = lines.next() {
                if let Some(stmt) = parse_statement(line, lines, depth + 1) {
                    statements.push(stmt);
                }
                let next = utilities::strip_indent(lines.rest(), depth);
                let r#continue =
                    next.is_empty() || next.starts_with('\t') || next.starts_with("  ");
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
        // TODO match indices?
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
            } else if let '"' | '\'' | '`' = chr
                && on.get(last..idx).is_none_or(|item| item.trim().is_empty())
            {
                in_string = Some(chr);
                // last = idx;
            } else if chr.is_whitespace() {
                let part = on.get(last..idx).unwrap_or_default().trim();
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
            } else if let '\\' = chr
                && let Some(n) = utilities::new_line_sequence_length(&on[(idx + 1)..])
            {
                last = idx + n + 1;
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

mod utilities {

    #[must_use]
    pub fn strip_indent(mut line: &str, upto: usize) -> &str {
        for _ in 0..upto {
            line = line
                .strip_prefix("\t")
                .or_else(|| line.strip_prefix("  "))
                .unwrap_or(line);
        }
        line
    }

    pub struct LinesWithContinuation<'a> {
        matches: &'static [&'static str],
        on: &'a str,
        last: usize,
    }

    impl<'a> LinesWithContinuation<'a> {
        pub fn new(on: &'a str, matches: &'static [&'static str]) -> Self {
            LinesWithContinuation {
                on,
                matches,
                last: 0,
            }
        }

        pub fn rest(&self) -> &'a str {
            &self.on[self.last..]
        }
    }

    impl<'a> Iterator for LinesWithContinuation<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<Self::Item> {
            let start = self.last;
            while let Some((idx, seq)) = find_new_line_sequence(&self.on[self.last..]) {
                self.last += idx + seq;
                let last = self.on[..self.last].trim_end();
                if self.matches.iter().any(|matcher| last.ends_with(matcher)) {
                    continue;
                } else {
                    return Some(&self.on[start..self.last].trim_end());
                }
            }
            if start < self.on.len() {
                self.last = self.on.len();
                Some(&self.on[start..].trim_end())
            } else {
                None
            }
        }
    }

    fn find_new_line_sequence(on: &str) -> Option<(usize, usize)> {
        for (idx, matched) in on.match_indices(['\r', '\n']) {
            // TODO does this check need to be done?
            if matched == "\r" && on[idx..].starts_with("\r\n") {
                return Some((idx, 2));
            } else {
                return Some((idx, 1));
            }
        }
        None
    }

    pub fn new_line_sequence_length(on: &str) -> Option<usize> {
        if on.starts_with("\r\n") {
            Some(2)
        } else if on.starts_with("\n") {
            Some(1)
        } else {
            None
        }
    }

    pub fn starts_with_new_line_sequence(on: &str) -> bool {
        on.starts_with("\r\n") || on.starts_with("\n")
    }
}
