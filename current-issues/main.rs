mod scan;

use std::io::BufReader;

fn main() {
    let mut args = std::env::args().skip(1);

    let first_arg = args.next();
    let endpoint = first_arg.as_deref().unwrap_or("**/*.rs");

    match endpoint {
        "info" | "--help" => {
            let run_id = option_env!("GITHUB_RUN_ID");
            let date = option_env!("GIT_LAST_COMMIT").unwrap_or_default();
            let after = run_id
                .map(|commit| format!(" (commit {commit} {date})"))
                .unwrap_or_default();

            eprintln!("current-issues{after}");
            eprintln!("find temporary pieces of code in Rust projects");
            eprintln!("other languages in the future");
        }
        arg => {
            let mut pattern = arg;
            let mut json = false;

            // WIP
            for arg in args {
                match arg.as_str() {
                    "--json" => {
                        json = true;
                    }
                    arg => {
                        eprintln!("unknown argument {arg:?}");
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

            let paths = glob::glob(pattern)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|path| path.is_file());

            for path in paths {
                let content = std::fs::File::open(&path).unwrap();
                let mut buffer_reader = BufReader::new(content);
                let ranges = scan::scan(&mut buffer_reader);
                if !ranges.is_empty() {
                    if json {
                        for range in ranges {
                            println!(
                                "{}",
                                json_builder_macro::json! { start: range.start as u64, end: range.end as u64, message: "found temporary item" }
                            );
                        }
                    } else {
                        use codespan_reporting::diagnostic::{Diagnostic, Label};
                        use codespan_reporting::files::SimpleFile;
                        use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
                        use codespan_reporting::term::{self, Config};

                        let file = SimpleFile::new(
                            path.display().to_string(),
                            // TODO double read
                            std::fs::read_to_string(path).unwrap(),
                        );
                        let writer = StandardStream::stderr(ColorChoice::Auto);
                        let config = Config::default();

                        for range in ranges {
                            let diagnostic = Diagnostic::error()
                                .with_message("unexpected item")
                                .with_labels(vec![
                                    Label::primary((), range).with_message("item here"),
                                ]);
                            term::emit_to_write_style(
                                &mut writer.lock(),
                                &config,
                                &file,
                                &diagnostic,
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }
    }
}
