use std::io::{IsTerminal, Read};

fn main() {
    let mut args = std::env::args().skip(1);

    let mut stdin = std::io::stdin();
    let input = if stdin.is_terminal() {
        args.next().expect("expected")
    } else {
        let mut buf = String::new();
        let _ = stdin.read_to_string(&mut buf).expect("invalid string");
        buf
    };

    let mut prefix_: String;

    let mut break_at = 40;
    let mut prefix = "";
    let mut splitter = ',';

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--break-at" => {
                break_at = args.next().expect("width").parse().expect("not a number");
            }
            "--prefix" => {
                prefix_ = args.next().expect("prefix");
                prefix = &prefix_;
            }
            "--splitter" => {
                splitter = args.next().expect("splitter").chars().next().unwrap();
            }
            arg => {
                eprintln!("unknown arg {arg:?}");
            }
        }
    }

    let out = to_buckets(&input, splitter, break_at, prefix);
    println!("{out}");
}

fn to_buckets(on: &str, splitter: char, break_at: usize, prefix: &str) -> String {
    let mut parts: Vec<&str> = on.split(splitter).collect();
    parts.sort_by_key(|item| item.len());

    let total = on.len(); // sort of
    let bucket_count = total / std::cmp::max(break_at, 1);
    let mut buckets: Vec<Vec<&str>> = vec![Vec::new(); bucket_count];

    for (idx, part) in parts.drain(..).enumerate() {
        buckets[idx % bucket_count].push(part.trim());
    }

    let mut buf = String::new();
    for mut bucket in buckets {
        buf.push_str(prefix);
        if !buf.is_empty() {
            buf.push_str("\n");
        }
        bucket.sort_by_key(|part| (part.chars().next().unwrap() as u32 * 12345) % 12);
        for (idx, part) in bucket.into_iter().enumerate() {
            if idx > 0 {
                buf.push_str(", ");
            }
            buf.push_str(part);
        }
    }

    buf
}
