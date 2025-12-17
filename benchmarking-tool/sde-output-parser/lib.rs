use std::io::BufRead;

#[derive(Default, Debug)]
pub struct Count {
    pub total: u32,
    pub mem_read: u32,
    pub mem_write: u32,
    pub stack_read: u32,
    pub stack_write: u32,
    pub call: u32,
}

#[allow(clippy::collapsible_else_if)]
pub fn parse(on: impl BufRead, skip_rust_internals: bool) -> Vec<(String, Count)> {
    let mut section: String = String::default();
    let mut count = Count::default();

    let mut parts: Vec<(String, Count)> = Vec::new();

    let mut start = false;

    for line in on.lines() {
        let line = line.unwrap();
        if "#GLOBAL_FUNCTION TOTALS " == line {
            start = true;
        }

        if !start {
            continue;
        }

        let name = if line == "# $global-dynamic-counts" {
            Some("global")
        } else if let Some(rest) = line.strip_prefix("# $dynamic-counts-for-function: ") {
            let (name, _) = rest.split_once(' ').unwrap();
            Some(name)
        } else {
            None
        };

        if let Some(new_name) = name {
            let skip = if skip_rust_internals {
                section.contains("alloc") || section.contains("std") || section.contains("core")
            } else {
                false
            };
            let skip = skip || section.is_empty();

            let count = std::mem::take(&mut count);
            if !skip {
                parts.push((section, count));
            }
            // TODO more efficient?
            let name = new_name.replace("$LT$", "<").replace("$GT$", ">");
            section = name;
        } else {
            // values
            if let Some(rest) = line.strip_prefix("*total") {
                count.total = rest.trim_start().parse().unwrap();
            } else if let Some(rest) = line.strip_prefix("*stack-read ") {
                count.stack_read = rest.trim_start().parse().unwrap();
            } else if let Some(rest) = line.strip_prefix("*stack-write ") {
                count.stack_write = rest.trim_start().parse().unwrap();
            } else if let Some(rest) = line.strip_prefix("*mem-read ") {
                count.mem_read = rest.trim_start().parse().unwrap();
            } else if let Some(rest) = line.strip_prefix("*mem-write ") {
                count.mem_write = rest.trim_start().parse().unwrap();
            } else if let Some(rest) = line.strip_prefix("*category-CALL") {
                count.call = rest.trim_start().parse().unwrap();
            }
        }
    }

    if !section.is_empty() {
        parts.push((section, std::mem::take(&mut count)));
    }

    parts
}
