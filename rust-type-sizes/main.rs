use rust_type_sizes::item_from_input;
use std::env;
use std::process::{Command, Stdio};

static SKIPPED: &[&str] = &[
    "std::",
    "core::",
    "alloc::",
    "syn::",
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
];

fn skip(name: &str, _size: usize) -> bool {
    SKIPPED.iter().any(|prefix| name.starts_with(prefix))
}

fn main() {
    let mut cargo_args = vec!["build".to_owned()];
    cargo_args.extend(env::args().skip(1));

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
    if out.is_empty() {
        panic!(
            "'-Zprint-type-sizes' does not output on cached build. edit 'lib.rs' to break cache"
        );
    }

    let mut items = Vec::new();

    let delimeter = "print-type-size type";
    let mut last = 0;

    for (idx, _matched) in out.match_indices(delimeter).skip(1) {
        let out = &out[last..idx];
        if let Some(item) = item_from_input(out, skip) {
            items.push(item);
        }
        last = idx;
    }

    let out = &out[last..];

    if let Some(item) = item_from_input(out, skip) {
        items.push(item);
    }

    for item in items {
        println!("{item:?}");
    }
}
