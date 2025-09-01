use lines_of_code::{RustSection, measure};

fn main() {
    let arg = std::env::args().nth(1);

    if let Some("--interactive") = arg.as_deref() {
        run_interactive()
    } else {
        let mut code = RustSection::default();

        let pattern = arg.as_deref().unwrap_or("**/*.rs");

        let files = glob::glob(pattern)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|path| path.is_file());

        for path in files {
            code.modules += 1;
            let content = std::fs::read_to_string(&path).unwrap();
            let file = measure(&content);

            let path_str = path.as_os_str().to_str();
            if path_str.is_some_and(|path| path.contains("examples")) {
                code.example_lines += file.lines;
            } else if path_str.is_some_and(|path| path.contains("tests")) {
                code.test_lines += file.lines;
            } else {
                code += file;
            }
        }

        println!("{json}", json = code.to_json());
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
            let output = String::from_utf8_lossy(&buf);

            measure(&output).debug();

            println!("end");
            buf.clear();
            continue;
        }

        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
    }
}
