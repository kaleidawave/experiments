#![warn(clippy::pedantic)]

use std::io::{self, BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Stdout,
    Stderr,
}

pub fn run_command(mut command: Command) -> io::Result<(Vec<(Channel, String)>, ExitStatus)> {
    // Spawn the child process
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Get handles to stdout and stderr
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Create buffered readers for stdout and stderr
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Channels to communicate between threads
    let (tx, rx) = std::sync::mpsc::channel();

    // Thread to read stdout
    let tx_stdout = tx.clone();
    thread::spawn(move || {
        for line in stdout_reader.lines().map_while(Result::ok) {
            tx_stdout
                .send((false, line))
                .expect("Failed to send stdout");
        }
    });

    // Thread to read stderr
    thread::spawn(move || {
        for line in stderr_reader.lines().map_while(Result::ok) {
            tx.send((true, line)).expect("Failed to send stderr");
        }
    });

    let mut out = Vec::new();

    // Receive and print lines in the order they arrive
    for (is_stderr, line) in rx {
        if is_stderr {
            out.push((Channel::Stderr, line));
        } else {
            out.push((Channel::Stdout, line));
        }
    }

    // Wait for the child process to exit
    let status = child.wait()?;

    Ok((out, status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut command = Command::new("node");
        let _ = command.args(&["--eval", r#"console.log('Hiya');console.error('Hello');console.log('Hola');console.error('Hi');"#]);
        let result = run_command(command).unwrap();
        assert_eq!(
            result.0,
            vec![
                (Channel::Stdout, "Hiya".into()),
                (Channel::Stderr, "Hello".into()),
                (Channel::Stdout, "Hola".into()),
                (Channel::Stderr, "Hi".into())
            ]
        );
    }
}
