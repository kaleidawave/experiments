use lines_of_code::{RustSection, measure};
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

            let mut count = RustSection::default();
            let mut trace = false;

            // WIP
            for argument in arguments {
                if let "--trace" = argument.as_str() {
                    trace = true;
                } else {
                    eprintln!("unknown argument {argument:?}");
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

                if let Ok(cargo_toml) =
                    std::fs::File::open(std::path::Path::new(path).join("Cargo.toml"))
                {
                    for line in BufReader::new(cargo_toml).lines().map_while(Result::ok) {
                        if let Some(item) = line.strip_prefix("name") {
                            let (_, item) = item.split_once('"').unwrap();
                            let (item, _) = item.rsplit_once('"').unwrap();
                            count.package_name = item.to_owned();
                            break;
                        }
                    }
                }
            }

            let files = glob::glob(pattern)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|path| path.is_file());

            for path in files {
                if trace {
                    eprintln!("Reading {path:?}");
                }
                count.modules += 1;

                let content = std::fs::File::open(&path).unwrap();
                let inner_count = measure(BufReader::new(content));

                let path_str = path.as_os_str().to_str();
                if path_str.is_some_and(|path| path.contains("examples")) {
                    count.example_lines += inner_count.lines;
                } else if path_str.is_some_and(|path| path.contains("tests")) {
                    count.test_lines += inner_count.lines;
                } else {
                    count += inner_count;
                }
            }

            println!("{json}", json = count.to_json());
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
            measure(BufReader::new(buf.as_slice())).debug();

            println!("end");
            buf.clear();
            continue;
        }

        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
    }
}
