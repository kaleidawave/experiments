use benchmarking_tool::{CommandRequest, Entry, ToolOptions, ToolOutput, tools, utilities};
use std::collections::HashMap;
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
            let request = CommandRequest {
                program: input.program,
                arguments: input.arguments,
            };
            let options = ToolOptions {
                keep: input.keep,
                skip_internals: input.skip_internals,
            };
            let result = tools::qbdi::run_qbdi(request, options).unwrap();
            match result {
                ToolOutput::SymbolInstructionCounts { symbols, total } => print_results(
                    symbols,
                    total as usize,
                    input.format,
                    input.sort,
                    input.limit,
                    input.breakdown,
                ),
                _ => todo!(),
            }
        }
        "time" => {
            todo!()
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        "sde" => {
            let request = CommandRequest {
                program: input.program,
                arguments: input.arguments,
            };
            let options = ToolOptions {
                keep: input.keep,
                skip_internals: input.skip_internals,
            };
            let result = tools::sde::run_sde(request, options).unwrap();
            match result {
                ToolOutput::SymbolInstructionCounts { symbols, total } => print_results(
                    symbols,
                    total as usize,
                    input.format,
                    input.sort,
                    input.limit,
                    input.breakdown,
                ),
                _ => todo!(),
            }
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
    /// include all inst
    pub breakdown: bool,
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
            breakdown: false,
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
                "--breakdown" => {
                    this.breakdown = true;
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

pub fn print_results(
    mut rows: Vec<Entry>,
    total_count: usize,
    output_format: OutputFormat,
    sorting: Option<utilities::Sorting>,
    limit: usize,
    breakdown: bool,
) {
    use std::borrow::Cow;
    use utilities::count_with_seperator;

    const MAX_WIDTH: usize = 100;
    const WHITESPACE: &str = if let Ok(result) = str::from_utf8(&[b' '; MAX_WIDTH]) {
        result
    } else {
        ""
    };

    if let Some(ref sort) = sorting {
        match sort.field.as_str() {
            "name" => {
                rows.sort_unstable_by(|lhs, rhs| {
                    sort.direction.compare(&lhs.symbol_name, &rhs.symbol_name)
                });
            }
            "total" => {
                rows.sort_unstable_by(|lhs, rhs| sort.direction.compare(&lhs.total, &rhs.total));
            }
            field => {
                eprintln!("unknown field {field:?}");
            }
        }
    }

    let skip = if let Some(utilities::Sorting {
        direction: utilities::Direction::Descending,
        ..
    }) = sorting
    {
        rows.len().saturating_sub(limit)
    } else {
        0
    };

    let rows = &rows[skip..];
    let rows = &rows[..std::cmp::min(rows.len(), limit)];

    match output_format {
        OutputFormat::Plain => {
            let max_name_width = {
                let mut max_name_width = 0;
                for row in rows {
                    max_name_width = std::cmp::max(max_name_width, row.symbol_name.len());
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
                total = count_with_seperator(total_count as usize)
            );
            for row in rows {
                let symbol_name: Cow<'_, str> = if row.symbol_name.len() > MAX_WIDTH {
                    Cow::Owned(format!(
                        "{prefix}...",
                        prefix = &row.symbol_name[..MAX_WIDTH - 3]
                    ))
                } else {
                    Cow::Borrowed(&row.symbol_name)
                };
                let fill = &WHITESPACE[..max_name_width - symbol_name.len()];

                // TODO wip
                print!("{symbol_name}{fill}");
                print!(
                    " total:  {count}",
                    count = count_with_seperator(row.total as usize)
                );
                if breakdown {
                    for (name, count) in &row.entries {
                        print!(
                            " {name}:  {count}",
                            count = count_with_seperator(*count as usize)
                        );
                    }
                }
                println!();
            }
        }
        OutputFormat::JSON => {
            let mut buf = String::from("[");
            for row in rows {
                if buf.len() > 1 {
                    buf.push(',');
                }
                if breakdown {
                    buf.push_str(&json_builder_macro::json! {
                        symbol_name: row.symbol_name.as_str(),
                        total: row.total,
                        kinds: row.entries
                    });
                } else {
                    buf.push_str(&json_builder_macro::json! {
                        symbol_name: row.symbol_name.as_str(),
                        total: row.total
                    });
                }
            }
            buf.push(']');
            println!("{buf}");
        }
        format => {
            todo!("output format '{format:?}'");
        }
    }
}
