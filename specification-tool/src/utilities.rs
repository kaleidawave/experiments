use std::io;
use std::path::Path;

#[must_use]
pub fn is_equal_ignore_new_line_sequence(lhs: &str, rhs: &str) -> bool {
    // We should not care about trailing new lines here...
    let mut lhs = lhs.lines();
    let mut rhs = rhs.lines();
    loop {
        match (lhs.next(), rhs.next()) {
            (Some(lhs), Some(rhs)) => {
                if lhs != rhs {
                    return false;
                }
            }
            (Some(_), _) | (_, Some(_)) => {
                return false;
            }
            (None, None) => {
                return true;
            }
        }
    }
}

pub mod filter {
    pub trait Filter {
        fn should_skip(&self, s: &str) -> bool;
    }

    #[derive(Debug, Clone)]
    pub struct StringMatch {
        pub matcher: Vec<String>,
        pub positive: bool,
        pub case_sensitive: bool,
    }

    impl StringMatch {
        fn matching_case(lhs: &str, rhs: &str, case_sensitive: bool) -> bool {
            if case_sensitive {
                lhs == rhs
            } else {
                lhs.eq_ignore_ascii_case(rhs)
            }
        }

        /// Split item and test each substring with rhs
        fn matching_item(item: &str, matcher: &str, case_sensitive: bool) -> bool {
            item.split(&[' ', '-'])
                .any(|item| Self::matching_case(item, matcher, case_sensitive))
        }
    }

    impl Filter for StringMatch {
        fn should_skip(&self, name: &str) -> bool {
            let result = self
                .matcher
                .iter()
                .any(|matcher| Self::matching_item(name, matcher, self.case_sensitive));

            if self.positive { !result } else { result }
        }
    }
}

pub mod commands {
    use std::io::{self, BufRead, BufReader, Read, Write};
    use std::process::{self, Child, Command, ExitStatus, Stdio};
    use std::sync;
    use std::thread;

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
                                // dbg!(&line);
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

                        // if let Ok(Some(_)) = child.try_wait() {
                        //     drop(stderr_writer);
                        // }

                        // dbg!();
                        let stderr_buf = thread.join().unwrap();
                        // dbg!();

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
                stderr_writer: _,
            } = self
            {
                let (tx, rx) = sync::mpsc::channel();

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
}

pub fn visit_specification_files(path: &Path, cb: &mut dyn FnMut(&Path)) -> io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_specification_files(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    } else if path.is_file() {
        let skip = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_none_or(|ext| !matches!(ext, "md" | "mdspec"));
        if !skip {
            cb(path);
        }
    } else if path.is_symlink() {
        let path = std::fs::read_link(path)?;
        visit_specification_files(&path, cb)?;
    }
    Ok(())
}

pub fn run_in_alternative_display<T: Sized>(cb: impl FnOnce() -> T) -> T {
    // Clear, ClearType
    use crossterm::{
        cursor::MoveToRow,
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    };

    execute!(std::io::stdout(), EnterAlternateScreen, MoveToRow(0)).unwrap();
    let result = cb();
    // Clear(ClearType::All)
    execute!(std::io::stdout(), LeaveAlternateScreen).unwrap();

    result
}
