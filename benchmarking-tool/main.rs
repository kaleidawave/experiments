#![allow(unused)]

mod tools;
mod utilities;

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use utilities::{Direction, Sorting};

fn main() {
    let mut args = std::env::args().skip(1);
    let tool = args.next();
    let tool = tool.as_deref().unwrap_or("help");

    let input = BenchmarkInput::from_arguments(args);

    match tool {
        "--info" | "help" => {
            println!("benchmarking-tool");
            println!("run 'qbdi', 'sde', 'perf-events' or 'time'");
        }
        "qbdi" => {
            tools::qbdi::run_qbdi(input);
        }
        "time" => {
            todo!()
        }
        #[cfg(target_arch = "x86")]
        "sde" => {
            tools::qbdi::run_sde(input);
        }
        #[cfg(target_family = "unix")]
        "perf-events" => {
            todo!()
        }
        tool => {
            println!("unknown tool {tool:?}. run with 'qbdi', 'sde', 'perf-events' or 'time'");
        }
    }
}

#[derive(Debug, Default)]
pub enum OutputFormat {
    #[default]
    Plain,
    JSON,
    Markdown,
}

#[derive(Debug)]
pub struct BenchmarkInput {
    /// number of symbol entries to show
    pub limit: usize,
    pub sort: Option<Sorting>,
    /// plain, JSON, markdown, csv
    pub format: OutputFormat,
    // ...
    pub program: String,
    pub arguments: Vec<String>,
    // ...
    pub generic_arguments: HashMap<String, Vec<String>>,

    // TODO
    /// Save SDE file...
    pub keep: Option<String>,
    /// skip Rust internals
    pub skip_internals: bool,
}

impl BenchmarkInput {
    pub fn from_arguments(mut args: impl Iterator<Item = String>) -> Self {
        let mut this = Self {
            limit: 25,
            sort: None,
            format: OutputFormat::default(),
            // ...
            program: String::new(),
            arguments: Vec::new(),
            // ...
            generic_arguments: HashMap::new(),
            // ...
            keep: None,
            skip_internals: true,
        };

        let mut left_over: Option<String> = None;
        while let Some(arg) = left_over.take().or_else(|| args.next()) {
            match arg.as_str() {
                "--format" => {
                    let format = args.next().expect("no format given");
                    this.format = match format.as_str() {
                        "plain" => OutputFormat::Plain,
                        "json" => OutputFormat::JSON,
                        "markdown" => OutputFormat::Markdown,
                        format => {
                            eprintln!("Unknown output format '{format:?}'");
                            OutputFormat::Plain
                        }
                    };
                }
                "--sort" => {
                    let field = args.next().expect("expected field");
                    let next = args.next();
                    let direction = match next.as_deref() {
                        Some("asc" | "ascending") => Direction::Ascending,
                        Some("desc" | "descending") => Direction::Descending,
                        _ => {
                            left_over = next;
                            Direction::Ascending
                        }
                    };
                    this.sort = Some(Sorting { field, direction });
                }
                // "--blocks" => {
                //     blocks = args.next().unwrap().parse().expect("invalid top blocks");
                // }
                "--limit" => {
                    let limit = args.next().unwrap();
                    if "all" == limit {
                        this.limit = usize::MAX;
                    } else {
                        this.limit = limit.parse().expect("invalid limit");
                    }
                }
                "--keep" => {
                    this.keep = args.next();
                }
                "--all" => {
                    this.skip_internals = false;
                }
                "--arg" => {
                    let next = args.next().unwrap();
                    let (name, values) = next.split_once('=').unwrap();
                    this.generic_arguments.insert(
                        name.to_owned(),
                        values.split(',').map(str::to_owned).collect(),
                    );
                }
                // WIP
                _command => {
                    this.program = arg;
                    break;
                }
            }
        }

        this.arguments = args.collect();

        this
    }
}
