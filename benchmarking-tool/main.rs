#![allow(unused)]

mod utilities;

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};
use utilities::ArgumentIter;

#[derive(Debug)]
struct RunData {
    pub duration: Duration,
    #[cfg(unix)]
    pub instructions: usize,
    #[cfg(unix)]
    pub memory_usage: usize,
}

#[derive(Debug)]
struct Benchmark {
    command: String,
    arguments: Vec<String>,
    pub(crate) total_elapsed: Duration,
    runs: Vec<RunData>,
}

impl Benchmark {
    pub fn name_and_arguments(&self) -> (&str, &[String]) {
        (&self.command, &self.arguments)
    }

    pub fn duration_nanos(&self) -> u128 {
        self.total_elapsed.as_nanos()
    }

    pub fn elapsed(&self) -> &Duration {
        &self.total_elapsed
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let kind = args.next();
    let kind = kind.as_deref().unwrap_or("help");

    match kind {
        "--info" | "help" => {
            println!("benchmarking-tool");
            println!("run 'qbdi', 'sde' or 'time'");
        }
        "qbdi" => {
            run_qbdi(args);
        }
        "sde" => {
            run_sde(args);
        }
        "time" => {
            todo!()
        }
        arg => {
            println!("unknown {arg}");
        }
    }
}

fn run_qbdi(mut args: impl Iterator<Item = String>) {
    let mut command = Command::new(args.next().unwrap());

    #[cfg(target_os = "macos")]
    {
        let library_name = "libqbdi_tracer.dylib";
        let root = std::env::current_exe().unwrap();
        let library = root.parent().unwrap().join(library_name);
        if !library.is_file() {
            eprintln!(
                "{library_name:?} not adjacent to {root:?}. {library} does not exist",
                library = library.display()
            );
            return;
        }
        command.env("DYLD_BIND_AT_LAUNCH", "1");
        command.env("DYLD_INSERT_LIBRARIES", &library.display().to_string());
    }

    #[cfg(target_os = "linux")]
    {
        let library_name = "libqbdi_tracer.so";
        let root = std::env::current_exe().unwrap();
        let library = root.parent().unwrap().join(library_name);
        if !library.is_file() {
            eprintln!(
                "{library_name:?} not adjacent to {root:?}. {library} does not exist",
                library = library.display()
            );
            return;
        }
        command.env("LD_PRELOAD", &library.display().to_string());
    }

    // Work in progress
    let mut filter = true;
    let mut only: Option<String> = None;
    while let Some(arg) = args.next() {
        if let "--qbdi-all" = arg.as_str() {
            filter = false;
        } else if let "--qbdi-only" = arg.as_str() {
            only = args.next();
        } else {
            let _ = command.arg(arg);
        }
    }

    command.stdout(Stdio::piped());

    let child = command.spawn().unwrap();

    let mut content = BufReader::new(child.stdout.unwrap());

    #[derive(Default)]
    struct Item {
        total: usize,
        instruction_kind: Vec<(String, usize)>,
    }

    // TODO this seems highly inefficient
    let mut items: HashMap<String, Item> = HashMap::new();

    let mut total_count = 0;
    for line in content.lines() {
        let line = line.unwrap();
        if let Some(rest) = line.strip_prefix("bm::") {
            let Some((func, rest)) = rest.split_once('/') else {
                // TODO not sure why some items do not finish?
                continue;
            };

            let (kind, count) = rest.split_once('/').unwrap();
            let Ok(count) = count.parse() else {
                // TODO ...?
                continue;
            };

            total_count += count;

            let func = format!("{func:#}", func = rustc_demangle::demangle(&func));
            // let func: Sting = if let Some(rest) = func.strip_prefix('<') {
            //     let (_, rhs) = rest.split_once(" as ").unwrap();
            //     let (lhs, _) = rhs.split_once('>').unwrap();
            //     formatlhs.to_owned()
            // } else {
            //     func
            // };

            if let Some(ref only) = only {
                if &func != only {
                    continue;
                }
            } else if filter {
                let bad_prefixes = &["std::", "core::", "alloc::", "_", "*", "OUTLINED_FUNCTION_"];
                let skip = bad_prefixes.iter().any(|prefix| func.starts_with(prefix));
                if skip {
                    continue;
                }
            }

            let item = items.entry(func).or_default();

            item.total += count;
            item.instruction_kind.push((kind.to_owned(), count));
        } else {
            println!("{line}");
        }
    }

    let mut items: Vec<(String, Item)> = Vec::from_iter(items);
    items.sort_unstable_by_key(|(_, value)| usize::MAX - value.total);

    for (func, mut item) in items {
        print!("{func} - {total} instructions", total = item.total);
        item.instruction_kind
            .sort_unstable_by_key(|(_, value)| usize::MAX - value);
        for (kind, count) in &item.instruction_kind {
            print!(" ({kind}={count})");
        }
        println!();
    }
}

fn run_sde(mut args: impl Iterator<Item = String>) {
    const TEMP_FILE: &str = "sde-out.txt";

    let mut command_string = None;
    let mut blocks = 20;
    let mut keep = None;

    let mut format = None;
    let mut sort = None;
    let mut sort_direction = utilities::Direction::Ascending;
    let mut limit = 25;
    let mut skip_rust_internals = true;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "program" => {
                // command_args.push(arg.value.unwrap());
                command_string = args.next();
            }
            "format" => {
                format = args.next();
            }
            "sort" => {
                sort = args.next();
            }
            "sort-desc" => {
                sort_direction = utilities::Direction::Descending;
            }
            "blocks" => {
                blocks = args.next().unwrap().parse().expect("invalid top blocks");
            }
            "limit" => {
                limit = args.next().unwrap().parse().expect("invalid limit");
            }
            "keep" => {
                keep = args.next();
            }
            "all" => {
                skip_rust_internals = false;
            }
            arg => {
                eprintln!("unknown {arg:?}");
            }
        }
    }

    let file_path: &str = keep.as_deref().unwrap_or(TEMP_FILE);

    {
        let mut command = Command::new("sde");
        command.args([
            "-omix",
            file_path,
            "-mix_filter_no_shared_libs",
            "-top_blocks",
            &blocks.to_string(),
            "--",
        ]);
        {
            // TODO use argument parser instead of split
            let arguments = command_string.expect("no command!");
            let arguments = arguments.split(' ').collect::<Vec<_>>();
            command.args(arguments);
        }
        // command.args(command_args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let _ = child.wait().unwrap();
    }

    let file = std::fs::File::open(file_path).unwrap();

    let sort = sort.as_deref().map(|field| utilities::Sorting {
        field,
        direction: sort_direction,
    });

    let format = format.as_deref().unwrap_or("plain");

    {
        fn parse_and_print(
            out: impl BufRead,
            skip_rust_internals: bool,
            sort: Option<utilities::Sorting>,
            limit: usize,
            format: &str,
        ) {
            const MAX_WIDTH: usize = 100;
            const WHITESPACE: &str = if let Ok(result) = str::from_utf8(&[b' '; MAX_WIDTH]) {
                result
            } else {
                ""
            };

            let mut rows = sde_output_parser::parse(out, skip_rust_internals);

            if let Some(sort) = sort {
                match sort.field {
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
            }) = sort
            {
                rows.len().saturating_sub(limit)
            } else {
                0
            };

            if let "plain" = format {
                let max_name_width = {
                    let mut max_name_width = 0;
                    for (name, _) in rows.iter().skip(skip).take(limit) {
                        max_name_width = std::cmp::max(max_name_width, name.len());
                    }
                    std::cmp::min(max_name_width, MAX_WIDTH)
                };

                for (section, count) in rows.iter().skip(skip).take(limit) {
                    use std::borrow::Cow;
                    use utilities::to_denary;

                    let section: Cow<'_, str> = if section.len() > MAX_WIDTH {
                        Cow::Owned(format!("{prefix}...", prefix = &section[..MAX_WIDTH - 3]))
                    } else {
                        Cow::Borrowed(section)
                    };
                    let fill = &WHITESPACE[..max_name_width - section.len()];

                    // TODO wip
                    print!("{section}:{fill}");
                    print!(" t:  {}", to_denary(count.total as usize, "\u{00A0}"));
                    print!(" mr: {}", to_denary(count.mem_read as usize, "\u{00A0}"));
                    print!(" mw: {}", to_denary(count.mem_write as usize, "\u{00A0}"));
                    print!(" call: {}", to_denary(count.call as usize, "\u{00A0}"));
                    println!();
                }
            } else if let "json" = format {
                let mut buf = String::from("[");
                for (section, count) in rows.iter().skip(skip).take(limit) {
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
            } else {
                eprintln!("unknown format {format:?}");
            }
        }

        parse_and_print(
            BufReader::new(file),
            skip_rust_internals,
            sort,
            limit,
            format,
        );
    }

    if keep.is_none() {
        std::fs::remove_file(file_path).unwrap();
    }
}
