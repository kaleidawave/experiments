use std::env;
use std::io::{self, BufRead};

fn main() {
    let is_uppercase = env::args().any(|flag| matches!(flag.as_str(), "--uppercase"));
    let stdin = io::stdin();
    let mut buf = Vec::new();

    println!("start");

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };

        if line == "close" {
            if !buf.is_empty() {
                eprintln!("no end to message {buf:?}");
            }
            break;
        }

        if line == "end" {
            let output = String::from_utf8_lossy(&buf);
            if is_uppercase {
                println!("{output}", output = output.to_uppercase());
            } else {
                println!("{output}");
            }

            println!("end");
            buf.clear();
            continue;
        }

        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
    }

    // println!("Finished!");
}
