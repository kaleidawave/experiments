#![warn(clippy::pedantic)]

use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{self, Child, Command, ExitStatus, Stdio};
use std::sync;
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Stdout,
    Stderr,
}

pub fn spawn_command(mut command: Command, grouped: bool) -> io::Result<CommandOut> {
    if grouped {
        let (reader, writer) = io::pipe()?;

        let child = command.stdout(writer.try_clone()?).stderr(writer).spawn()?;

        Ok(CommandOut::Grouped(child, BufReader::new(reader)))
    } else {
        let (reader, writer) = io::pipe()?;

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(writer.try_clone()?)
            .spawn()?;

        let stdout = BufReader::new(child.stdout.take().expect("Failed to capture stdout"));
        // let stderr = BufReader::new(child.stderr.take().expect("Failed to capture stderr"));
        let stderr = BufReader::new(reader);

        Ok(CommandOut::Separated {
            child,
            stdout,
            stderr,
            stderr_writer: writer,
        })
    }
}

pub enum CommandOut {
    Grouped(Child, BufReader<io::PipeReader>),
    Separated {
        child: Child,
        stdout: BufReader<process::ChildStdout>,
        stderr: BufReader<io::PipeReader>,
        // Used to 'blip' the writer so that it can exit
        stderr_writer: io::PipeWriter,
    },
}

impl CommandOut {
    pub fn get_child(&mut self) -> &mut Child {
        match self {
            CommandOut::Grouped(child, _) | CommandOut::Separated { child, .. } => child,
        }
    }

    pub fn read_until(&mut self, cb: impl Fn(&str) -> bool) -> io::Result<(String, String)> {
        match self {
            CommandOut::Grouped(_child, reader) => {
                let mut buf = String::new();
                for line in reader.lines().map_while(Result::ok) {
                    if cb(&line) {
                        break;
                    }
                    buf.push_str(&line);
                    buf.push('\n');
                }
                Ok((buf, String::default()))
            }
            CommandOut::Separated {
                child: _,
                stdout,
                stderr,
                stderr_writer,
            } => {
                let (mut stdout_buf, mut stderr_buf) = thread::scope(|s| {
                    let thread = s.spawn(|| {
                        let mut stderr_buf = String::new();
                        for line in stderr.lines().map_while(Result::ok) {
                            if line == "please finish" {
                                break;
                            }
                            stderr_buf.push_str(&line);
                            stderr_buf.push('\n');
                        }
                        stderr_buf
                    });

                    let mut stdout_buf = String::new();
                    for line in stdout.lines().map_while(Result::ok) {
                        if cb(&line) {
                            break;
                        }
                        stdout_buf.push_str(&line);
                        stdout_buf.push('\n');
                    }

                    writeln!(stderr_writer, "please finish").unwrap();

                    let stderr_buf = thread.join().unwrap();

                    (stdout_buf, stderr_buf)
                });

                stdout_buf.truncate(stdout_buf.trim_end().len());
                stderr_buf.truncate(stderr_buf.trim_end().len());
                Ok((stdout_buf, stderr_buf))
            }
        }
    }

    pub fn read_to_end(self) -> io::Result<(String, String, ExitStatus)> {
        match self {
            CommandOut::Grouped(mut child, mut reader) => {
                let mut buf = String::new();
                reader.read_to_string(&mut buf)?;
                let status = child.wait()?;
                Ok((buf, String::default(), status))
            }
            CommandOut::Separated {
                mut child,
                mut stdout,
                mut stderr,
                stderr_writer,
            } => {
                let mut stdout_buf = String::new();
                let mut stderr_buf = String::new();

                stdout.read_to_string(&mut stdout_buf)?;
                drop(stderr_writer);
                stderr.read_to_string(&mut stderr_buf)?;

                stdout_buf.truncate(stdout_buf.trim_end().len());
                stderr_buf.truncate(stderr_buf.trim_end().len());

                let status = child.wait()?;

                Ok((stdout_buf, stderr_buf, status))
            }
        }
    }

    pub fn read_independent_to_end(self) -> io::Result<(Vec<(Channel, String)>, ExitStatus)> {
        if let Self::Separated {
            mut child,
            stdout,
            stderr,
            stderr_writer,
        } = self
        {
            let (tx, rx) = sync::mpsc::channel();

            drop(stderr_writer);

            // Thread to read `stdout`
            let tx_stdout = tx.clone();
            thread::spawn(move || {
                for line in stdout.lines().map_while(Result::ok) {
                    // TODO `expect` here
                    tx_stdout
                        .send((Channel::Stdout, line))
                        .expect("Failed to send stdout");
                }
            });

            // Thread to read `stderr`
            thread::spawn(move || {
                for line in stderr.lines().map_while(Result::ok) {
                    // TODO `expect` here
                    tx.send((Channel::Stderr, line))
                        .expect("Failed to send stderr");
                }
            });

            let out: Vec<_> = rx.into_iter().collect();
            let status = child.wait()?;

            Ok((out, status))
        } else {
            panic!("cannot read merged independently");
        }
    }
}

const TIMEOUT_MESSAGE: &[u8] = b"=== TIMEOUT ===";
pub const END_MESSAGE: &[u8] = b"=== end ===";

pub fn run_timeout<O>(
    timeout: Duration,
    command: impl FnOnce(std::io::PipeWriter) -> O,
) -> Result<(O, Vec<u8>), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Condvar, Mutex};

    let (ping_rx, ping_tx) = io::pipe()?;

    let mut ping_tx2 = ping_tx.try_clone().expect("could not clone writer");

    let main_pair = Arc::new((Mutex::new(false), Condvar::new()));
    let thread_pair = Arc::clone(&main_pair);

    let _monitor = thread::spawn(move || {
        let (lock, cvar) = &*thread_pair;
        let completed = lock.lock().unwrap();
        let result = cvar.wait_timeout(completed, timeout);
        if let Ok((_finished, result)) = result {
            if result.timed_out() {
                io::Write::write_all(&mut ping_tx2, TIMEOUT_MESSAGE).unwrap();
            }
        }
    });

    // My command
    let out = command(ping_tx);

    let (lock, cvar) = &*main_pair;

    let mut reader = BufReader::new(ping_rx);
    let mut buf = Vec::new();
    while let Ok(bytes) = reader.fill_buf() {
        if bytes.ends_with(TIMEOUT_MESSAGE) {
            return Err(String::from("Timed out").into());
        }
        if bytes.ends_with(END_MESSAGE) {
            {
                *lock.lock().unwrap() = true;
                // We notify the condvar that the value has changed.
                cvar.notify_one();
            }
            break;
        }
        let to_continue = bytes.len();
        buf.extend_from_slice(bytes);
        if to_continue == 0 {
            break;
        }
        reader.consume(to_continue);
    }

    Ok((out, buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split() {
        let mut command = Command::new("node");
        let program = r#"
            console.log('Hiya');
            console.error('Hello');
            console.log('Hola');
            console.error('Hi');
        "#;
        let _ = command.args(&["--eval", program]);
        let result = spawn_command(command, false).unwrap();
        let (result, _exit_code) = result.read_independent_to_end().unwrap();
        assert_eq!(
            result,
            vec![
                (Channel::Stdout, "Hiya".into()),
                (Channel::Stderr, "Hello".into()),
                (Channel::Stdout, "Hola".into()),
                (Channel::Stderr, "Hi".into())
            ]
        );
    }

    #[test]
    fn until() {
        let mut command = Command::new("node");
        let program = r#"
            console.log('Hiya');
            console.error('Hello');
            console.log('test');
            console.log('end');
        "#;
        let _ = command.args(&["--eval", program]);
        let mut result = spawn_command(command, false).unwrap();
        let (stdout, stderr) = result.read_until(|line| matches!(line, "end")).unwrap();
        assert_eq!((stdout.as_str(), stderr.as_str()), ("Hiya\ntest", "Hello"));
    }

    #[test]
    fn to_end() {
        let mut command = Command::new("node");
        let program = r#"
            console.log('Hiya');
            console.error('Hello');
            console.log('test');
            console.log('end');
        "#;
        let _ = command.args(&["--eval", program]);
        let result = spawn_command(command, false).unwrap();
        let (stdout, stderr, _) = result.read_to_end().unwrap();
        assert_eq!(
            (stdout.as_str(), stderr.as_str()),
            ("Hiya\ntest\nend", "Hello")
        );
    }

    #[test]
    fn timeout() {
        let timeout = Duration::from_millis(1000);

        type ThreadResult<T> = thread::JoinHandle<io::Result<T>>;

        let out = run_timeout::<ThreadResult<_>>(timeout, |mut writer| {
            thread::spawn(move || {
                writer.write_all(&[1, 2, 3])?;
                thread::sleep(Duration::from_millis(100));
                writer.write_all(&[4, 5, 6])?;
                thread::sleep(Duration::from_millis(100));
                writer.write_all(END_MESSAGE)?;
                Ok(())
            })
        });

        assert_eq!(out.expect("timed out").1, &[1, 2, 3, 4, 5, 6]);

        let out = run_timeout::<ThreadResult<_>>(timeout, |mut writer| {
            thread::spawn(move || {
                writer.write_all(&[1, 2, 3])?;
                thread::sleep(Duration::from_millis(1000));
                writer.write_all(&[4, 5, 6])?;
                thread::sleep(Duration::from_millis(3000));
                writer.write_all(END_MESSAGE)?;
                Ok(())
            })
        });

        assert!(out.is_err());
    }
}
