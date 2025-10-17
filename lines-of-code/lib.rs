use std::io::{BufRead, BufReader};

#[derive(Default, Debug)]
pub struct RustSection {
    pub package_name: String,
    pub lines: u64,
    /// lines dedicated to tests.
    /// these can be within the `tests` folder or surrounded by `#[cfg(test)]`
    pub test_lines: u64,
    /// lines dedicated to examples
    pub example_lines: u64,
    enums: u64,
    structs: u64,
    type_aliases: u64,
    fns: u64,
    impls: u64,
    variables: u64,
    comments: u64,
    pub modules: u64,
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
            package_name: _,
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

pub fn measure<R: std::io::Read>(on: BufReader<R>) -> RustSection {
    let mut is_test = false;
    let mut test_indent: Option<String> = None;
    let mut code = RustSection::default();
    let mut is_multiline_comment = false;

    for line in on.lines() {
        let line = line.unwrap();
        if let Some(ref indent) = test_indent {
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
                test_indent = Some(line[..indent_count].to_owned());
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
            package_name,
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

        let mut buf = String::new();
        {
            let mut builder = json_builder_macro::Builder::new(&mut buf);
            if !package_name.is_empty() {
                builder.add("name", package_name.as_str());
            }

            builder.add("modules", *modules);
            builder.add("lines", *lines);
            builder.add("variables", *variables);
            builder.add("type_aliases", *type_aliases);
            builder.add("comments", *comments);
            builder.add("enums", *enums);
            builder.add("structs", *structs);
            builder.add("functions", *fns);
            builder.add("implementations", *impls);
            builder.add("test_lines", *test_lines);
            builder.add("example_lines", *example_lines);
            builder.end();
        }
        buf
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
            package_name: _,
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
