use crate::{OutputFormat, utilities};

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use utilities::{MAX_WIDTH, WHITESPACE};

pub fn run_qbdi(input: super::BenchmarkInput) {
    let mut command = if cfg!(target_os = "windows") {
        let root = std::env::current_exe().unwrap();
        let mut command = {
            let preloader_name = "QBDIWinPreloader.exe";
            let preloader = root.parent().unwrap().join(preloader_name);
            if !preloader.is_file() {
                eprintln!(
                    "{preloader_name:?} not adjacent to {root:?}. {preloader} does not exist",
                    preloader = preloader.display()
                );
                return;
            }
            Command::new(preloader.display().to_string())
        };
        {
            let library_name = "libqbdi_tracer.dll";
            let library = root.parent().unwrap().join(library_name);
            if !library.is_file() {
                eprintln!(
                    "{library_name:?} not adjacent to {root:?}. {library} does not exist",
                    library = library.display()
                );
                return;
            }
            command.arg(library.display().to_string());
        }

        command.arg(input.program);
        command.args(input.arguments);
        command
    } else {
        let mut command = Command::new(input.program);
        command.args(input.arguments);
        command
    };

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
        command.env("DYLD_INSERT_LIBRARIES", library.display().to_string());
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
        command.env("LD_PRELOAD", library.display().to_string());
    }

    command.stdout(Stdio::piped());

    let mut child = command.spawn().unwrap();

    let mut content = BufReader::new(child.stdout.take().unwrap());

    #[derive(Default)]
    struct Item {
        total: u32,
        instruction_kind: Vec<(String, u32)>,
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

            let func = format!("{func:#}", func = rustc_demangle::demangle(func));
            // let func: Sting = if let Some(rest) = func.strip_prefix('<') {
            //     let (_, rhs) = rest.split_once(" as ").unwrap();
            //     let (lhs, _) = rhs.split_once('>').unwrap();
            //     formatlhs.to_owned()
            // } else {
            //     func
            // };

            // if let Some(ref only) = only {
            //     if &func != only {
            //         continue;
            //     }
            // } else

            if input.skip_internals {
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

    child.wait().unwrap();

    let mut rows: Vec<(String, Item)> = Vec::from_iter(items);
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

            // for (func, mut item) in rows {
            //     print!("{func} - {total} instructions", total = item.total);
            //     item.instruction_kind
            //         .sort_unstable_by_key(|(_, value)| u32::MAX - value);
            //     for (kind, count) in &item.instruction_kind {
            //         print!(" ({kind}={count})");
            //     }
            //     println!();
            // }

            println!(
                "Run {total} instructions",
                total = utilities::count_with_seperator(total_count as usize)
            );
            for (section, count) in rows {
                let section: Cow<'_, str> = if section.len() > MAX_WIDTH {
                    Cow::Owned(format!("{prefix}...", prefix = &section[..MAX_WIDTH - 3]))
                } else {
                    Cow::Borrowed(section)
                };
                let fill = &WHITESPACE[..max_name_width - section.len()];

                // TODO wip
                print!("{section}{fill}");
                print!(
                    " total:  {total}",
                    total = utilities::count_with_seperator(count.total as usize)
                );
                // TODO more here
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
