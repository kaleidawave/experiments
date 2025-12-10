use std::io::{BufRead, BufReader, Read};

pub type Range = std::ops::Range<usize>;

pub fn scan<R: Read>(on: &mut BufReader<R>) -> Vec<Range> {
    let mut count = 0;
    let mut ranges = Vec::new();
    let mut is_multiline_comment = false;

    for line in on.lines() {
        let line = line.unwrap();
        let line_len = line.len();

        if is_multiline_comment && let Some((lhs, _rhs)) = line.rsplit_once("*/") {
            is_multiline_comment = false;
            if lhs.trim().is_empty() {
                count += line_len + 1;
                continue;
            }
        }

        let line = line.trim_start();
        let whitespace = line_len - line.len();

        if !line.is_empty() {
            // Could be in string literal... but we move
            if let Some((lhs, rhs)) = line.rsplit_once("/*")
                && !rhs.contains("*/")
            {
                is_multiline_comment = true;
                if lhs.trim().is_empty() {
                    count += line_len + 1;
                    continue;
                }
            }

            // ---

            let comment = line.strip_prefix("//").or_else(|| line.strip_prefix("///"));

            // TODO options for what is including: after.starts_with("FUTURE") || after.starts_with("WIP")
            // TODO what about markdown > and - modifiers etc
            // TODO more optimal version of this
            let offset =
                if let Some(offset) = comment.is_some().then(|| line.find("dbg!")).flatten() {
                    Some(offset)
                } else if let Some(offset) = line.find("dbg!") {
                    Some(offset)
                } else if let Some(offset) = line.find("todo!") {
                    Some(offset)
                } else if let Some(offset) = line.find("unimplemented!") {
                    Some(offset)
                } else if let Some(offset) = line.find("println!(\"TODO") {
                    Some(offset)
                } else if let Some(offset) = line.find("eprintln!(\"TODO") {
                    Some(offset)
                } else {
                    None
                };

            if let Some(offset) = offset {
                ranges.push((count + offset + whitespace)..(count + line_len));
            }
        }

        // TODO assumes that the source is just LF;
        count += line_len + 1;
    }

    ranges
}
