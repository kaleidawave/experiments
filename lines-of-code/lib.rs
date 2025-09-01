#[derive(Default, Debug)]
pub struct RustSection {
    pub lines: usize,
    /// lines dedicated to tests.
    /// these can be within the `tests` folder or surrounded by `#[cfg(test)]`
    pub test_lines: usize,
    /// lines dedicated to examples
    pub example_lines: usize,
    enums: usize,
    structs: usize,
    type_aliases: usize,
    fns: usize,
    impls: usize,
    variables: usize,
    comments: usize,
    pub modules: usize,
}

impl std::ops::AddAssign for RustSection {
    fn add_assign(&mut self, other: Self) {
        let Self {
            enums,
            structs,
            fns,
            impls,
            variables,
            comments,
            test_lines,
            example_lines,
            lines,
            modules,
            type_aliases,
        } = other;
        self.enums += enums;
        self.structs += structs;
        self.fns += fns;
        self.impls += impls;
        self.variables += variables;
        self.comments += comments;
        self.test_lines += test_lines;
        self.example_lines += example_lines;
        self.lines += lines;
        self.modules += modules;
        self.type_aliases += type_aliases;
    }
}

pub fn measure(on: &str) -> RustSection {
    let mut is_test = false;
    let mut test_indent: Option<&str> = None;
    let mut code = RustSection::default();
    let mut is_multiline_comment = false;

    for line in on.lines() {
        if let Some(indent) = test_indent {
            if let Some(rest) = line.strip_prefix(indent)
                && rest == "}"
            {
                test_indent = None;
            } else {
                code.test_lines += 1;
            }
            continue;
        }

        if is_multiline_comment {
            if let Some((lhs, _rhs)) = line.rsplit_once("*/") {
                is_multiline_comment = false;
                code.comments += 1; // one of these counts as one commen
                if lhs.trim().is_empty() {
                    continue;
                }
            }
            continue;
        }

        // waiting for item
        if std::mem::take(&mut is_test) {
            if line.trim_end().ends_with('{') {
                let indent_count = line.len() - line.trim_start().len();
                test_indent = Some(&line[..indent_count]);
            }
            continue;
        }

        let line = line.trim_start();
        if !line.is_empty() {
            if line.starts_with("//") {
                code.comments += 1;
                continue;
            }

            if line.starts_with("#[cfg(test)]") {
                is_test = true;
                continue;
            }

            // Could be in string literal... but we move
            if let Some((lhs, rhs)) = line.rsplit_once("/*")
                && !rhs.contains("*/")
            {
                is_multiline_comment = true;
                if lhs.trim().is_empty() {
                    continue;
                }
            }

            code.lines += 1;

            let rest = if let Some(rest) = line.strip_prefix("pub") {
                let rest = rest.trim_start();
                if let Some(after) = rest.strip_prefix('(') {
                    // TODO `unwrap_or` should be unreachable?
                    after
                        .split_once(')')
                        .map(|(_, after)| after)
                        .unwrap_or(after)
                        .trim_start()
                } else {
                    rest
                }
            } else {
                line
            };

            if rest.starts_with("enum ") {
                code.enums += 1;
            } else if rest.starts_with("mod ") {
                code.modules += 1;
            } else if rest.starts_with("let ") {
                code.variables += 1;
            } else if rest.starts_with("type ") {
                code.type_aliases += 1;
            } else if rest.starts_with("struct ") {
                code.structs += 1;
            } else if rest.starts_with("fn ") {
                code.fns += 1;
            } else if rest.starts_with("impl ") || rest.starts_with("impl<") {
                code.impls += 1;
            }
        }
    }

    code
}

impl RustSection {
    pub fn to_json(&self) -> String {
        let Self {
            enums,
            structs,
            fns,
            impls,
            variables,
            comments,
            test_lines,
            lines,
            modules,
            example_lines,
            type_aliases,
        } = self;
        format!(
            r#"{{"modules":{modules},"lines":{lines},"variables":{variables},"type_aliases":{type_aliases},"example_lines":{example_lines},"comments":{comments},"enums":{enums},"structs":{structs},"fns":{fns},"impls":{impls},"test_lines":{test_lines}}}"#
        )
    }

    pub fn debug(&self) {
        let Self {
            enums,
            structs,
            fns,
            impls,
            variables,
            comments,
            test_lines,
            lines,
            modules,
            type_aliases,
            example_lines,
        } = self;
        if *lines > 0 {
            println!("lines: {lines}");
        }
        if *test_lines > 0 {
            println!("test_lines: {test_lines}");
        }
        if *example_lines > 0 {
            println!("example_lines: {example_lines}");
        }
        if *modules > 0 {
            println!("modules: {modules}");
        }
        if *enums > 0 {
            println!("enums: {enums}");
        }
        if *structs > 0 {
            println!("structs: {structs}");
        }
        if *type_aliases > 0 {
            println!("type_aliases: {type_aliases}");
        }
        if *fns > 0 {
            println!("fns: {fns}");
        }
        if *impls > 0 {
            println!("impls: {impls}");
        }
        if *variables > 0 {
            println!("variables: {variables}");
        }
        if *comments > 0 {
            println!("comments: {comments}");
        }
    }
}
