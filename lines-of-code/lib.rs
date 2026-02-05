use derive_more::{Add, AddAssign};
use std::io::{BufRead, BufReader};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Package,
    Module,
    Function,
    Struct,
    Impl,
    /// from `macro_rules` blocks
    Macro,
}

/// TODO break up?
#[derive(Debug)]
pub struct RustSection {
    /// Can be `mod *name*`, `fn *name*` or `macro_rules *name*`
    pub kind_and_name: (Kind, String),
    pub statistics: RustStatistics,
    pub nested: Vec<RustSection>,
}

#[derive(Debug, Default, Clone, Copy, Add, AddAssign)]
pub struct RustStatistics {
    pub lines: u64,
    /// lines dedicated to tests.
    /// these can be within the `tests` directory or surrounded by `#[cfg(test)]`
    pub test_lines: u64,
    /// lines dedicated to examples (based on file paths)
    pub example_lines: u64,
    /// number of `enum` definitions
    enums: u64,
    /// number of `struct` definitions
    structs: u64,
    /// number of `type` definitions
    type_aliases: u64,
    /// number of `fn` definitions
    fns: u64,
    /// number of `macro_rules` definitions
    macro_rules: u64,
    /// number of `impl` definitions
    impls: u64,
    /// number of `let` definitions
    variables: u64,
    /// number of lines which are comments
    comments: u64,
    /// number of lines which are '}' etc
    delimeters: u64,
    /// blank lines
    whitespace: u64,
    pub modules: u64,
}

pub fn measure_package<R: std::io::Read>(on: BufReader<R>) -> RustSection {
    measure_block(&mut on.lines(), (Kind::default(), String::new()), None)
}

/// # Panics
///
/// Can panic if source is not UTF8
pub fn measure_block<R: std::io::Read>(
    on: &mut std::io::Lines<BufReader<R>>,
    kind_and_name: (Kind, String),
    break_on: Option<&str>,
) -> RustSection {
    let mut is_test = false;
    let mut test_indent: Option<String> = None;
    let mut statistics = RustStatistics::default();
    let mut nested: Vec<RustSection> = Vec::new();
    let mut is_multiline_comment = false;

    while let Some(line) = on.next() {
        let original_line = line.unwrap();
        if let Some(ref indent) = test_indent {
            if let Some(rest) = original_line.strip_prefix(indent)
                && rest == "}"
            {
                statistics.delimeters += 1;
                test_indent = None;
            } else {
                statistics.test_lines += 1;
                {
                    let line = original_line.trim_start();
                    let without_semicolon = line.strip_suffix(';').unwrap_or(line);
                    let without_comma = without_semicolon
                        .strip_suffix(';')
                        .unwrap_or(without_semicolon);
                    if !without_comma.is_empty()
                        && without_comma.bytes().all(|b: u8| b"([{}])".contains(&b))
                    {
                        statistics.delimeters += 1;
                    }
                }
            }
            continue;
        }

        if is_multiline_comment {
            if let Some((lhs, _rhs)) = original_line.rsplit_once("*/") {
                is_multiline_comment = false;
                statistics.comments += 1; // one of these counts as one commen
                if lhs.trim().is_empty() {
                    continue;
                }
            }
            continue;
        }

        // waiting for item
        if std::mem::take(&mut is_test) {
            if original_line.trim_end().ends_with('{') {
                let indent_count = original_line.len() - original_line.trim_start().len();
                test_indent = Some(original_line[..indent_count].to_owned());
            }
            continue;
        }

        if let Some(end) = original_line.strip_suffix('}')
            && let Some(expected_end) = break_on
            && end == expected_end
        {
            statistics.lines += 1;
            statistics.delimeters += 1;
            break;
        }

        let line = original_line.trim_start();
        if line.is_empty() {
            statistics.whitespace += 1;
        } else {
            if line.starts_with("//") {
                statistics.comments += 1;
                continue;
            }

            if line.starts_with("#[cfg(test)]") {
                is_test = true;
                continue;
            }

            // Some of these could be in a multiline string literal... but we move

            // If the entire line is this we could mark it as comment
            if let Some((lhs, rhs)) = line.rsplit_once("/*")
                && !rhs.contains("*/")
            {
                is_multiline_comment = true;
                if lhs.trim().is_empty() {
                    continue;
                }
            }

            statistics.lines += 1;

            {
                let without_semicolon = line.strip_suffix(';').unwrap_or(line);
                let without_comma = without_semicolon
                    .strip_suffix(';')
                    .unwrap_or(without_semicolon);
                if without_comma.bytes().all(|b: u8| b"([{}])".contains(&b)) {
                    statistics.delimeters += 1;
                    continue;
                }
            }

            if let Some(_rest) = line.strip_prefix("macro_rules! ") {
                // let indent_count = original_line.len() - original_line.trim_start().len();
                // let macro_name = rest
                //     .split_once(|c: char| !(c.is_alphanumeric() || matches!(c, '_')))
                //     .map_or(rest, |(before, _)| before);
                // dbg!(macro_name, indent_count);
                statistics.macro_rules += 1;
            } else if line.starts_with("impl ") || line.starts_with("impl<") {
                statistics.impls += 1;
            } else {
                let after_pub = if let Some(rest) = line.strip_prefix("pub") {
                    let rest = rest.trim_start();
                    if let Some(after) = rest.strip_prefix('(') {
                        // TODO `unwrap_or` should be unreachable?
                        after
                            .split_once(')')
                            .map_or(after, |(_, after)| after)
                            .trim_start()
                    } else {
                        rest
                    }
                } else {
                    line
                };

                if after_pub.starts_with("enum ") {
                    statistics.enums += 1;
                } else if after_pub.starts_with("mod ") {
                    statistics.modules += 1;
                } else if after_pub.starts_with("let ") {
                    statistics.variables += 1;
                } else if after_pub.starts_with("type ") {
                    statistics.type_aliases += 1;
                } else if after_pub.starts_with("struct ") {
                    statistics.structs += 1;
                } else if let Some(rest) = after_pub.strip_prefix("fn ") {
                    let func_name = rest
                        .split_once(|c: char| !(c.is_alphanumeric() || matches!(c, '_')))
                        .map_or(rest, |(before, _)| before);
                    statistics.fns += 1;

                    // dbg!(func_name);

                    if true {
                        let prefix = &original_line[..(original_line.len() - line.len())];
                        // dbg!(prefix);
                        let mut inner =
                            measure_block(on, (Kind::Function, func_name.to_owned()), Some(prefix));

                        statistics += inner.statistics;
                        if inner.statistics.lines > 10 {
                            // To include this line
                            inner.statistics.lines += 1;
                            nested.push(inner);
                        }
                    }
                }
            }
        }
    }

    RustSection {
        kind_and_name,
        statistics,
        nested,
    }
}

impl json_builder_macro::ToJSON for RustSection {
    fn append_as_json_string(&self, buf: &mut String) {
        use std::fmt::Write;

        let Self {
            kind_and_name: (kind, name),
            statistics,
            nested,
        } = self;

        write!(buf, "{{").unwrap();
        write!(buf, "\"kind\":\"{kind:?}\",").unwrap();
        write!(buf, "\"name\":\"{name}\",").unwrap();
        write!(buf, "\"statistics\":").unwrap();
        statistics.append_as_json_string(buf);
        write!(buf, ",").unwrap();
        write!(buf, "\"nested\":").unwrap();
        nested.append_as_json_string(buf);
        write!(buf, "}}").unwrap();
    }
}

impl json_builder_macro::ToJSON for RustStatistics {
    fn append_as_json_string(&self, buf: &mut String) {
        use std::fmt::Write;

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
            delimeters,
            whitespace,
            macro_rules,
        } = self;

        write!(buf, "{{").unwrap();
        write!(buf, "\"enums\":{enums},").unwrap();
        write!(buf, "\"structs\":{structs},").unwrap();
        write!(buf, "\"fns\":{fns},").unwrap();
        write!(buf, "\"macro_rules\":{macro_rules},").unwrap();
        write!(buf, "\"impls\":{impls},").unwrap();
        write!(buf, "\"variables\":{variables},").unwrap();
        write!(buf, "\"comments\":{comments},").unwrap();
        write!(buf, "\"test_lines\":{test_lines},").unwrap();
        write!(buf, "\"lines\":{lines},").unwrap();
        write!(buf, "\"modules\":{modules},").unwrap();
        write!(buf, "\"example_lines\":{example_lines},").unwrap();
        write!(buf, "\"type_aliases\":{type_aliases},").unwrap();
        write!(buf, "\"delimeters\":{delimeters},").unwrap();
        write!(buf, "\"whitespace\":{whitespace}").unwrap();
        write!(buf, "}}").unwrap();
    }
}

impl RustStatistics {
    pub fn debug(&self) {
        let Self {
            enums,
            structs,
            fns,
            macro_rules,
            impls,
            variables,
            comments,
            test_lines,
            lines,
            modules,
            type_aliases,
            delimeters,
            whitespace,
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
        if *macro_rules > 0 {
            println!("macro_rules: {macro_rules}");
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
        if *delimeters > 0 {
            println!("delimeters: {delimeters}");
        }
        if *whitespace > 0 {
            println!("whitespace: {whitespace}");
        }
    }
}
