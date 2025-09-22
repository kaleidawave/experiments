use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitCode, Stdio};

const TEMP_FILE: &str = "sde-out.txt";

use lahl::{
    CLI, Endpoint, NamedParameter, PositionalParameter, argument_result_or_out,
    command_result_or_out,
};

static RUN_NAMED_PARAMETERS: &[NamedParameter] = &[
    // TODO required! and last? something with --
    // NamedParameter::multiple("program", "commands to run"),
    NamedParameter::value("program", "commands to run"),
    NamedParameter::value("blocks", "number of blocks"),
    NamedParameter::value("keep", "keep output with name"),
    // parse options
    NamedParameter::value("format", "JSON"), // or Markdown
    NamedParameter::value("sort", "alphanumeric or total"),
    NamedParameter::boolean("sort-desc", "sort in descending order (requires sort flag)"),
    NamedParameter::value("limit", "maximum entries to short"),
    // TODO optional filter?
    NamedParameter::boolean(
        "all",
        "shows all regions including those starting with `std`, `core` and `alloc`",
    ),
];

static PARSE_NAMED_PARAMETERS: &[NamedParameter] = &[
    NamedParameter::value("format", "JSON"), // or Markdown
    NamedParameter::value("sort", "alphanumeric or total"),
    NamedParameter::boolean("sort-desc", "sort in descending order (requires sort flag)"),
    NamedParameter::value("limit", "maximum entries to short"),
    // TODO optional filter?
    NamedParameter::boolean(
        "all",
        "shows all regions including those starting with `std`, `core` and `alloc`",
    ),
];

static ENDPOINTS: &[Endpoint] = &[
    Endpoint::new(
        "run",
        "run a program and print output",
        &[],
        RUN_NAMED_PARAMETERS,
    ),
    Endpoint::new(
        "parse",
        "parse output and print in tidy format",
        &[PositionalParameter::multiple("file", "files to parse")],
        PARSE_NAMED_PARAMETERS,
    ),
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => err,
    }
}

fn run() -> Result<(), ExitCode> {
    let cli = CLI::new(
        ENDPOINTS,
        "SDE parser. Parses output format for better readability",
        None,
    );
    let (binary_name, result) = cli.run();

    let (selected, arguments) = command_result_or_out(result, &binary_name)?;
    match selected.name {
        "run" => {
            // TODO combinations etc here
            // let mut command_args = Vec::new();
            let mut command_string = None;
            let mut blocks = 20;
            let mut keep = None;

            let mut format = None;
            let mut sort = None;
            let mut sort_direction = utilities::Direction::Ascending;
            let mut limit = 25;
            let mut skip_rust_internals = true;

            for argument in arguments {
                let argument = argument_result_or_out(argument)?;
                match argument.name {
                    "program" => {
                        // command_args.push(argument.value.unwrap());
                        command_string = argument.value;
                    }
                    "format" => {
                        format = argument.value;
                    }
                    "sort" => {
                        sort = argument.value;
                    }
                    "sort-desc" => {
                        sort_direction = utilities::Direction::Descending;
                    }
                    "blocks" => {
                        blocks = argument.value.unwrap().parse().expect("invalid top blocks");
                    }
                    "limit" => {
                        limit = argument.value.unwrap().parse().expect("invalid limit");
                    }
                    "keep" => {
                        keep = argument.value;
                    }
                    "all" => {
                        skip_rust_internals = false;
                    }
                    argument => unreachable!("{argument:?}"),
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

            let file = fs::File::open(file_path).unwrap();

            let sort = sort.as_deref().map(|field| utilities::Sorting {
                field,
                direction: sort_direction,
            });

            let format = format.as_deref().unwrap_or("plain");

            parse_and_print(
                BufReader::new(file),
                skip_rust_internals,
                sort,
                limit,
                format,
            );

            if keep.is_none() {
                fs::remove_file(file_path).unwrap();
            }

            Ok(())
        }
        "parse" => {
            let mut format = None;
            let mut sort = None;
            let mut sort_direction = utilities::Direction::Ascending;
            let mut limit = 25;
            let mut skip_rust_internals = true;

            let mut files = Vec::new();

            for argument in arguments {
                let argument = argument_result_or_out(argument)?;
                match argument.name {
                    "file" => {
                        files.push(argument.value.unwrap());
                    }
                    "format" => {
                        format = argument.value;
                    }
                    "sort" => {
                        sort = argument.value;
                    }
                    "sort-desc" => {
                        sort_direction = utilities::Direction::Descending;
                    }
                    "limit" => {
                        limit = argument.value.unwrap().parse().expect("invalid limit");
                    }
                    "all" => {
                        skip_rust_internals = false;
                    }
                    argument => unreachable!("{argument:?}"),
                }
            }

            let sort = sort.as_deref().map(|field| utilities::Sorting {
                field,
                direction: sort_direction,
            });

            let format = format.as_deref().unwrap_or("plain");

            for path in files {
                let Ok(file) = fs::File::open(path) else {
                    return Err(ExitCode::FAILURE);
                };
                parse_and_print(
                    BufReader::new(file),
                    skip_rust_internals,
                    sort,
                    limit,
                    format,
                );
            }

            Ok(())
        }
        command => unreachable!("{command:?}"),
    }
}

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

mod utilities {
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Sorting<'a> {
        pub field: &'a str,
        pub direction: Direction,
    }

    #[derive(Clone, Copy, Debug)]
    pub(crate) enum Direction {
        Ascending,
        Descending,
    }

    impl Direction {
        pub fn compare<T: std::cmp::Ord>(self, a: &T, b: &T) -> std::cmp::Ordering {
            let order = a.cmp(b);
            if let Self::Ascending = self {
                order.reverse()
            } else {
                order
            }
        }
    }

    pub(crate) fn to_denary(value: usize, seperator: &str) -> String {
        if value == 0 {
            return "0".to_owned();
        }
        let mut buf = String::new();
        for i in (0..=value.ilog10()).rev() {
            let j = (value / 10i32.pow(i) as usize) % 10;
            buf.push(b"0123456789"[j] as char);
            if i > 0 && i % 3 == 0 {
                buf.push_str(seperator);
            }
        }
        buf
    }
}
