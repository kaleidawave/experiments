use rust_type_sizes::{Field, FieldOrPadding, Item, Kind, Skip, item_from_input};
use std::path::PathBuf;
use std::process::{Command, Stdio};

static SKIPPED_PREFIX: &[&str] = &[
    "std::",
    "core::",
    "alloc::",
    // "syn::",
    "proc_macro::",
    "proc_macro2::",
    "syn_helpers::",
    "{closure",
    "<std::",
    "hashbrown::",
    "self_rust_tokenize::",
    "de::",
    "__private::",
    "quote::",
    "internals::",
    "either_n",
    // "Win32"
];

static SKIPPED_CONTAINS: &[&str] = &["syn"];

// TODO more
#[derive(Debug, Clone, Copy)]
struct Skipper {
    pub size_threshold: usize,
}

impl Skip for Skipper {
    fn should_skip(&self, name: &str, size: usize) -> bool {
        SKIPPED_PREFIX.iter().any(|prefix| name.starts_with(prefix))
            || SKIPPED_CONTAINS.iter().any(|slice| name.contains(slice))
            || size < self.size_threshold
    }
}

fn main() {
    let mut file: Option<PathBuf> = None;
    let mut example: Option<String> = None;
    let mut to_tables: bool = false;
    let mut log: Option<PathBuf> = None;

    let mut skipper = Skipper { size_threshold: 0 };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from-file" => {
                let path = args.next().expect("expected path to file");
                file = Some(PathBuf::from(path));
            }
            "--example" => {
                let example_ = args.next().expect("expected example name");
                example = Some(example_);
            }
            "--threshold" => {
                let size = args.next().expect("expected threshold size");
                skipper.size_threshold = size.parse().unwrap();
            }
            "--to-tables" => {
                to_tables = true;
            }
            "--help" => {
                println!("rust-type-sizes");
                println!("TODO explanation etc");
            }
            "--log-file" => {
                let path = args.next().expect("expected path to file");
                log = Some(PathBuf::from(path));
            }
            arg => {
                panic!("unknown {arg:?}");
            }
        }
    }

    let out: String = if let Some(file) = file {
        std::fs::read_to_string(file).unwrap()
    } else {
        // we only get type size output if the build is *fresh*. To do this we *dirty* the project by changing
        // the last update time of a file
        // TODO test
        {
            // TODO find lib.rs
            use filetime::{FileTime, set_file_mtime};
            let res = set_file_mtime("src/lib.rs", FileTime::now());
            if let Err(_res) = res {
                let _res = set_file_mtime("lib.rs", FileTime::now());
                eprintln!("could not update filetime");
            }
        }

        let mut cargo_args = vec!["build".to_owned()];
        if let Some(example) = example {
            cargo_args.push("--example".into());
            cargo_args.push(example);
        }

        eprintln!("Running 'cargo' with {cargo_args:?}");

        let output = Command::new("cargo")
            .args(cargo_args)
            // For nightly feature
            .env("RUSTC_BOOTSTRAP", "1")
            // Important
            .env("RUSTFLAGS", "-Zprint-type-sizes")
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute process");

        let out = String::from_utf8(output.stdout).expect("invalid utf8");

        if let Some(log) = log {
            let _ = std::fs::write(log, &out);
        }

        if out.is_empty() {
            panic!(
                "'-Zprint-type-sizes' does not output on cached builds. Make a change to 'lib.rs' to force a rebuild so that information can be printed"
            );
        }
        out
    };

    let mut items = Vec::new();

    let delimeter = "print-type-size type";
    let mut last = 0;

    for (idx, _matched) in out.match_indices(delimeter).skip(1) {
        let out = &out[last..idx];
        if let Some(item) = item_from_input(out, &skipper) {
            items.push(item);
        }
        last = idx;
    }

    let out = &out[last..];

    if let Some(item) = item_from_input(out, &skipper) {
        items.push(item);
    }

    if to_tables {
        print_to_tables(&items);
    } else {
        for item in items {
            debug_item(&item);
            // println!("{item:?}");
        }
    }
}

fn debug_item(item: &Item) {
    let kind = if matches!(item.kind, Kind::EnumItem { .. }) {
        "enum"
    } else {
        "struct"
    }; // Or other
    println!(
        "{name} = {size} ({kind})",
        name = &item.total.name,
        size = item.total.size
    );
    match &item.kind {
        Kind::EnumItem {
            discriminant: _,
            variants,
        } => {
            // let big = variants.len() > 2;
            for variant in variants {
                // if big {
                print!("\t");
                // }
                print!(
                    "{name} = {size}",
                    name = &variant.total.name,
                    size = variant.total.size
                );
                print!("(");
                for field in &variant.fields {
                    match field {
                        FieldOrPadding::Field(field) => {
                            print!(".{name} = {size},", name = &field.name, size = field.size);
                        }
                        FieldOrPadding::Padding(_) => {}
                    }
                }
                print!("),");
                // if big {
                //     println!()
                // }
            }
        }
        Kind::StructItem { fields } => {
            for field in fields {
                match field {
                    FieldOrPadding::Field(Field { name, size, .. }) => {
                        print!(" .{name} = {size},");
                    }
                    FieldOrPadding::Padding(size) => {
                        print!(" (padding {size}),");
                    }
                }
            }
        }
    }
    println!();
}

fn delimeter(item: &str) -> &str {
    if item.contains(',') { "\"" } else { "" }
}

fn print_to_tables(items: &[Item]) {
    println!("--- items ---");
    for item in items {
        let kind = if matches!(item.kind, Kind::EnumItem { .. }) {
            "enum"
        } else {
            "struct"
        };
        // Or other
        let d = delimeter(&item.total.name);
        println!(
            "{kind},{d}{name}{d},{size},",
            name = &item.total.name,
            size = item.total.size
        );
    }
    println!("--- variants ---");
    for item in items {
        let d = delimeter(&item.total.name);
        if let Kind::EnumItem { ref variants, .. } = item.kind {
            for variant in variants {
                println!(
                    "{d}{name}{d},{variant_name},{size},",
                    name = &item.total.name,
                    variant_name = &variant.total.name,
                    size = variant.total.size
                );
            }
        }
    }
    println!("--- fields ---");
    for item in items {
        let d = delimeter(&item.total.name);
        match &item.kind {
            Kind::EnumItem {
                discriminant: _,
                variants,
            } => {
                for variant in variants {
                    for field in &variant.fields {
                        match field {
                            FieldOrPadding::Field(Field { name, size, .. }) => {
                                let e = if name.starts_with(|chr: char| chr.is_ascii_digit()) {
                                    "\""
                                } else {
                                    ""
                                };
                                println!(
                                    "{d}{name}{d},{variant_name},{e}{name}{e},{size},",
                                    name = &item.total.name,
                                    variant_name = &variant.total.name
                                );
                            }
                            FieldOrPadding::Padding(_) => {}
                        }
                    }
                }
            }
            Kind::StructItem { fields } => {
                for field in fields {
                    match field {
                        FieldOrPadding::Field(Field { name, size, .. }) => {
                            println!(
                                "{d}{total_name}{d},null,{name},{size},",
                                total_name = &item.total.name,
                            );
                        }
                        FieldOrPadding::Padding(_) => {}
                    }
                }
            }
        }
    }
}
