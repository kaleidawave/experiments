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
            todo!()
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
            eprintln!("{library_name:?} not adjacent to {root:?}. {library} does not exist", library=library.display());
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
            eprintln!("{library_name:?} not adjacent to {root:?}. {library} does not exist", library=library.display());
            return;
        }
        command.env("LD_PRELOAD", &library.display().to_string());
    }

    // TODO could extract things here
    for arg in args {
        let _ = command.arg(arg);
    }

    command.stdout(Stdio::piped());

    let child = command.spawn().unwrap();

    let mut content = BufReader::new(child.stdout.unwrap());

    let mut items: HashMap<String, Vec<(String, usize)>> = HashMap::new();
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
            items
                .entry(func.to_owned())
                .or_default()
                .push((kind.to_owned(), count));
        } else {
            println!("{line}");
        }
    }

    for (func, mut items) in items {
        let func = format!("{func:#}", func = rustc_demangle::demangle(&func));
        let func: &str = if let Some(rest) = func.strip_prefix('<') {
            let (_, rhs) = rest.split_once(" as ").unwrap();
            let (lhs, _) = rhs.split_once('>').unwrap();
            lhs
        } else {
            &func
        };
        let bad_prefixes = &["std::", "core::", "alloc::", "_", "*", "OUTLINED_FUNCTION_"];
        let skip = bad_prefixes.iter().any(|prefix| func.starts_with(prefix));
        if skip {
            continue;
        }
        print!("  {func}");
        items.sort_unstable_by_key(|(_, value)| usize::MAX - value);
        for (kind, count) in items {
            print!(" [{kind}: {count}]");
        }
        println!();
    }
}
