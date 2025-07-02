use language_tool::{File, loader, scan, tools};
use std::path::Path;

fn main() {
    let mut parser = loader::get_rust_parser();

    eprintln!("initiated parser");

    let mut args = std::env::args().skip(1);

    let first = args.next();
    match first.as_deref() {
        None | Some("info" | "--help") => {
            eprintln!("The language-tool")
        }
        Some("transform") => {
            let tool = args.next().expect("tool");
            let path = args.next().expect("path");
            match tool.as_str() {
                "rust:change_visibility" => {
                    visit_files(Path::new(&path), &mut |path| {
                        eprintln!("--- {path} ---", path = path.display());
                        let source = std::fs::read_to_string(path).unwrap();

                        let mut tool = tools::rust_change_visibility::FindRustItems::default();

                        let file = File::new(path.to_owned(), source);

                        scan(&file, &mut parser.parser, &mut tool);

                        let mut new = String::new();
                        let mut cur = 0;
                        for (range, item) in tool.changes {
                            let (lhs, rhs) = (range.start, range.end);
                            new.push_str(&file.source()[cur..lhs]);
                            new.push_str(item);
                            cur = rhs;
                        }
                        new.push_str(&file.source()[cur..]);

                        let mut temp_path = file.path().to_owned();
                        let name = temp_path.file_stem().unwrap_or_default().display();
                        temp_path.set_file_name(format!("{name}2.rs"));
                        std::fs::write(temp_path, new).unwrap();
                    })
                    .expect("could not walk directory");
                }
                name => panic!("unknown tool {name}"),
            }
        }
        Some("query") => {
            let tool = args.next().expect("tool");
            let path = args.next().expect("path");
            match tool.as_str() {
                "rust:find_impls" => {
                    let filter = args.next();
                    let mut tool = tools::rust_find_impls::FindRustImplementations {
                        filter: filter.clone(),
                        ..Default::default()
                    };

                    visit_files(Path::new(&path), &mut |path| {
                        eprintln!("--- {path} ---", path = path.display());
                        let source = std::fs::read_to_string(path).unwrap();
                        let file = File::new(path.to_owned(), source);
                        scan(&file, &mut parser.parser, &mut tool);
                    })
                    .expect("Could not visit files");

                    for (_trait_name, item_name, _position) in tool.found {
                        println!("{item_name}");
                    }
                }
                name => panic!("unknown tool {name}"),
            }
        }
        Some(command) => {
            eprintln!("Unknown command {command:?}");
        }
    }
}

pub fn visit_files(
    path: &std::path::Path,
    cb: &mut dyn FnMut(&std::path::Path),
) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_files(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    } else if path.is_file() {
        let skip = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_none_or(|ext| !matches!(ext, "rs"));
        if !skip {
            cb(path)
        }
    } else if path.is_symlink() {
        let path = std::fs::read_link(path)?;
        visit_files(&path, cb)?;
    }
    Ok(())
}
