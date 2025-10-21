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

    let mut break_after = 40;
    let mut prefix = "";
    let mut splitter = ',';
    let mut unstable = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--break-after" => {
                break_after = args.next().expect("width").parse().expect("not a number");
            }
            "--prefix" => {
                prefix_ = args.next().expect("prefix");
                prefix = &prefix_;
            }
            // re-organises
            "--unstable" => {
                unstable = true;
            }
            "--splitter" => {
                splitter = args.next().expect("splitter").chars().next().unwrap();
            }
            arg => {
                eprintln!("unknown arg {arg:?}");
            }
        }
    }

    let out = if unstable {
        to_buckets(&input, splitter, break_after, prefix)
    } else {
        let mut current_width = 0;
        let mut buf = prefix.to_string();
        for part in input.split(splitter) {
            current_width += part.len();
            buf.push_str(part.trim());
            buf.push(splitter);
            // TODO
            if current_width > break_after {
                buf.push('\n');
                buf.push_str(prefix);
                current_width = 0;
            } else if splitter != ' ' {
                buf.push(' ');
            }
        }
        buf.truncate(buf.trim_end().len());
        buf
    };

    println!("{out}");
}

fn to_buckets(on: &str, splitter: char, break_after: usize, prefix: &str) -> String {
    let mut parts: Vec<&str> = on.split(splitter).collect();
    parts.sort_by_key(|item| item.len());

    let total = on.len(); // sort of
    let bucket_count = total / std::cmp::max(break_after, 1);
    let mut buckets: Vec<Vec<&str>> = vec![Vec::new(); bucket_count];

    for (idx, part) in parts.drain(..).enumerate() {
        buckets[idx % bucket_count].push(part.trim());
    }

    let mut buf = String::new();
    for mut bucket in buckets {
        if !buf.is_empty() {
            buf.push_str("\n");
        }
        buf.push_str(prefix);
        bucket.sort_by_key(|part| (part.chars().next().unwrap() as u32 * 12345) % 12);
        for (idx, part) in bucket.into_iter().enumerate() {
            if idx > 0 {
                buf.push(splitter);
                // TODO
                if splitter != ' ' {
                    buf.push(' ');
                }
            }
            buf.push_str(part);
        }
    }

    buf
}
