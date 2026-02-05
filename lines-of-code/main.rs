use lines_of_code::{Kind, RustSection, measure_package};
use std::io::{BufRead, BufReader};

fn main() {
    let mut arguments = std::env::args().skip(1);

    let first_argument = arguments.next();
    let endpoint = first_argument.as_deref().unwrap_or("**/*.rs");

    match endpoint {
        // For testing
        "--interactive" => run_interactive(),
        "info" | "--help" => {
            let run_id = option_env!("GITHUB_RUN_ID");
            let date = option_env!("GIT_LAST_COMMIT").unwrap_or_default();
            let after = run_id
                .map(|commit| format!(" (commit {commit} {date})"))
                .unwrap_or_default();

            eprintln!("lines-of-code{after}");
            eprintln!("count lines of code and other items in Rust projects");
        }
        argument => {
            let mut pattern = argument;

            let mut count = RustSection {
                kind_and_name: (Kind::Package, String::new()),
                statistics: lines_of_code::RustStatistics::default(),
                nested: Vec::new(),
            };

            // Logs files that are walked
            let mut trace = false;
            // let mut count_function_sizes = false;

            for argument in arguments {
                match argument.as_str() {
                    "--trace" => {
                        trace = true;
                    }
                    // "--count-function-sizes" => {
                    //     count_function_sizes = true;
                    // }
                    argument => {
                        eprintln!("unknown argument {argument:?}");
                    }
                }
            }

            let p;
            if let Ok(entry) = std::fs::metadata(pattern)
                && entry.is_dir()
            {
                p = format!("{pattern}/**");
                pattern = &p;
            }

            {
                let path = pattern.split_once('*').unwrap_or((pattern, "")).0;
                let path = path.rsplit_once(['\\', '/']).unwrap_or((path, "")).0;
                let path = if path.ends_with("src") {
                    &path[..path.len() - 4]
                } else {
                    path
                };

                {
                    use std::fs::File;
                    use std::path::Path;

                    let cargo_toml_path = Path::new(path).join("Cargo.toml");
                    if let Ok(cargo_toml) = File::open(cargo_toml_path) {
                        for line in BufReader::new(cargo_toml).lines().map_while(Result::ok) {
                            if let Some(item) = line.strip_prefix("name") {
                                let (_, item) = item.split_once('"').unwrap();
                                let (item, _) = item.rsplit_once('"').unwrap();
                                item.clone_into(&mut count.kind_and_name.1);
                                break;
                            }
                        }
                    } else {
                        eprintln!("No Cargo.toml?");
                    }
                }
            }

            let files = glob::glob(pattern)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|path| path.is_file());

            for path in files {
                use std::fs::File;

                if trace {
                    eprintln!("Reading {path}", path = path.display());
                }
                count.statistics.modules += 1;

                let content = File::open(&path).unwrap();
                let mut inner_count = measure_package(BufReader::new(content));

                let path_str = path.as_os_str().to_str();
                if path_str.is_some_and(|path| path.contains("examples")) {
                    count.statistics.example_lines += inner_count.statistics.lines;
                } else if path_str.is_some_and(|path| path.contains("tests")) {
                    count.statistics.test_lines += inner_count.statistics.lines;
                } else {
                    count.statistics += inner_count.statistics;
                }

                count.nested.append(&mut inner_count.nested);
            }

            let json = json_builder_macro::ToJSON::as_json_string(&count);
            println!("{json}");
        }
    }
}

fn run_interactive() {
    use std::io::{BufRead, stdin};
    let stdin = stdin();
    let mut buf = Vec::new();

    println!("start");

    for line in stdin.lock().lines().map_while(Result::ok) {
        if line == "close" {
            if !buf.is_empty() {
                eprintln!("no end to message {buf:?}");
            }
            break;
        }

        if line == "end" {
            measure_package(BufReader::new(buf.as_slice()))
                .statistics
                .debug();

            println!("end");
            buf.clear();
            continue;
        }

        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
    }
}
