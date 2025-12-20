use crate::Entry;

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn run_qbdi(
    request: crate::CommandRequest,
    options: crate::ToolOptions,
) -> Result<crate::ToolOutput, ()> {
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
                return Err(());
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
                return Err(());
            }
            command.arg(library.display().to_string());
        }

        command.arg(request.program);
        command.args(request.arguments);
        command
    } else {
        let mut command = Command::new(request.program);

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
                return Err(());
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
                return Err(());
            }
            command.env("LD_BIND_NOW", "1");
            command.env("LD_PRELOAD", library.display().to_string());
        }

        command.args(request.arguments);
        command
    };

    command.stdout(Stdio::piped());

    let mut child = command.spawn().unwrap();

    let content = BufReader::new(child.stdout.take().unwrap());

    #[derive(Default)]
    struct Item {
        total: u32,
        instruction_kind: Vec<(String, u32)>,
    }

    // TODO this seems highly inefficient
    let mut items: HashMap<String, Item> = HashMap::new();

    let mut total = 0;
    for line in content.lines() {
        let line = line.unwrap();

        #[cfg(target_os = "linux")]
        eprintln!("TEMP linux: {line}");

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

            total += count;

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

            if options.skip_internals {
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

    let symbols: Vec<_> = items
        .into_iter()
        .map(|(name, item)| Entry {
            symbol_name: name,
            total: item.total,
            entries: item.instruction_kind,
        })
        .collect();

    Ok(crate::ToolOutput::SymbolInstructionCounts { total, symbols })
}
