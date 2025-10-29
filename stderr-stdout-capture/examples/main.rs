use stderr_stdout_capture::capture_output;

fn main() {
    match capture_output(|| {
        println!("This goes to stdout");
        eprintln!("This goes to stderr");
        print!("More stdout without newline");
    }) {
        Ok((stdout, stderr)) => {
            println!("\n=== Captured Output ===");
            println!("STDOUT: {:?}", stdout);
            println!("STDERR: {:?}", stderr);
        }
        Err(e) => {
            eprintln!("Error capturing output: {}", e);
        }
    }
}

