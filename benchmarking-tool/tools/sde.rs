use crate::{OutputFormat, utilities};

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use utilities::{MAX_WIDTH, WHITESPACE};

const TEMP_FILE: &str = "sde-out.txt";

pub fn run_sde(input: super::BenchmarkInput) {
    let file_path: &str = input.keep.as_deref().unwrap_or(TEMP_FILE);

    {
        let mut command = Command::new("sde");
        command.args([
            "-omix",
            file_path,
            "-mix_filter_no_shared_libs",
            "-top_blocks",
            // TODO hmm
            &(2 * input.limit).to_string(),
            "--",
        ]);
        command.arg(input.program);
        command.args(input.arguments);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let _ = child.wait().unwrap();
    }

    let file = std::fs::File::open(file_path).unwrap();

    {
        let out = BufReader::new(file);

        let mut rows = sde_output_parser::parse(out, input.skip_internals);

        if let Some(ref sort) = input.sort {
            match sort.field.as_str() {
                "name" => {
                    rows.sort_unstable_by(|lhs, rhs| sort.direction.compare(&lhs.0, &rhs.0));
                }
                "total" => {
                    rows.sort_unstable_by(|lhs, rhs| {
                        sort.direction.compare(&lhs.1.total, &rhs.1.total)
                    });
                }
                field => {
                    eprintln!("unknown field {field:?}");
                }
            }
        }

        let skip = if let Some(utilities::Sorting {
            direction: utilities::Direction::Descending,
            ..
        }) = input.sort
        {
            rows.len().saturating_sub(input.limit)
        } else {
            0
        };

        let rows = &rows[skip..];
        let rows = &rows[..std::cmp::min(rows.len(), input.limit)];

        // TODO abstract ?
        match input.format {
            OutputFormat::Plain => {
                let max_name_width = {
                    let mut max_name_width = 0;
                    for (name, _) in rows {
                        max_name_width = std::cmp::max(max_name_width, name.len());
                    }
                    std::cmp::min(max_name_width, MAX_WIDTH)
                };

                for (section, count) in rows {
                    use std::borrow::Cow;
                    use utilities::count_with_seperator;

                    let section: Cow<'_, str> = if section.len() > MAX_WIDTH {
                        Cow::Owned(format!("{prefix}...", prefix = &section[..MAX_WIDTH - 3]))
                    } else {
                        Cow::Borrowed(section)
                    };
                    let fill = &WHITESPACE[..max_name_width - section.len()];

                    // TODO wip
                    print!("{section}{fill}");
                    print!(" t:  {}", count_with_seperator(count.total as usize));
                    print!(" mr: {}", count_with_seperator(count.mem_read as usize));
                    print!(" mw: {}", count_with_seperator(count.mem_write as usize));
                    print!(" call: {}", count_with_seperator(count.call as usize));
                    println!();
                }
            }
            OutputFormat::JSON => {
                let mut buf = String::from("[");
                for (section, count) in rows {
                    if buf.len() > 1 {
                        buf.push(',');
                    }
                    buf.push_str(&json_builder_macro::json! {
                        name: section.as_str(),
                        total: count.total,
                        mem_read: count.mem_read,
                        mem_write: count.mem_write,
                        stack_read: count.stack_read,
                        stack_write: count.stack_write,
                        call: count.call,
                    });
                }
                buf.push(']');
                println!("{buf}");
            }
            format => {
                todo!("output format '{format:?}'");
            }
        }
    }

    if input.keep.is_none() {
        std::fs::remove_file(file_path).unwrap();
    }
}
