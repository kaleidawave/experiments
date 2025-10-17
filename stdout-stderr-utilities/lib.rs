#![warn(clippy::pedantic)]

use std::io::{self, BufRead, BufReader};
use std::process::{self, Command, ExitStatus, Stdio};
use std::{sync, thread, time};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Channel {
    Stdout,
    Stderr,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessNotification {
    Message(Channel, String),
    Completed,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ProcessStatus {
    Finished,
    Continuing,
}

pub struct Process {
    child: process::Child,
    stdout_handle: thread::JoinHandle<()>,
    stderr_handle: thread::JoinHandle<()>,
    receiver: sync::mpsc::Receiver<ProcessNotification>,
}

impl Process {
    pub fn spawn(mut command: Command) -> io::Result<Self> {
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = BufReader::new(child.stdout.take().expect("Failed to capture stdout"));
        let stderr = BufReader::new(child.stderr.take().expect("Failed to capture stderr"));

        let (sender, receiver) = sync::mpsc::sync_channel::<ProcessNotification>(0);

        // Thread to read `stdout`
        let sender_stdout = sender.clone();
        let stdout_handle = thread::spawn(move || {
            for line in stdout.lines().map_while(Result::ok) {
                // TODO `expect` here
                sender_stdout
                    .send(ProcessNotification::Message(Channel::Stdout, line))
                    .expect("Failed to send stdout");
            }

            // TODO `expect` here
            sender_stdout
                .send(ProcessNotification::Completed)
                .expect("Failed to send stdout");
        });

        // Thread to read `stderr`
        let sender_stderr = sender; // .clone();
        let stderr_handle = thread::spawn(move || {
            for line in stderr.lines().map_while(Result::ok) {
                // TODO `expect` here
                sender_stderr
                    .send(ProcessNotification::Message(Channel::Stderr, line))
                    .expect("Failed to send stderr");
            }
        });

        Ok(Self {
            child,
            stdout_handle,
            stderr_handle,
            receiver,
        })
    }

    pub fn read_timeout(
        &self,
        timeout: time::Duration,
        end_message: &str,
    ) -> (Vec<(Channel, String)>, io::Result<ProcessStatus>) {
        let mut messages = Vec::new();
        loop {
            let out = self.receiver.recv_timeout(timeout);
            match out {
                Ok(item) => match item {
                    ProcessNotification::Message(channel, message) => {
                        if message == end_message {
                            break;
                        }
                        messages.push((channel, message));
                    }
                    ProcessNotification::Completed => {
                        return (messages, Ok(ProcessStatus::Finished));
                    }
                },
                Err(_timeout) => {
                    let result = Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "timed out reading notification",
                    ));
                    return (messages, result);
                }
            }
        }

        (messages, Ok(ProcessStatus::Continuing))
    }

    pub fn end(mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait()?;
        self.stdout_handle.join().unwrap();
        self.stderr_handle.join().unwrap();
        Ok(status)
    }
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
        let result = Process::spawn(command).unwrap();
        let (result, out) = result.read_timeout(time::Duration::MAX, "");
        assert_eq!(
            &result,
            &[
                (Channel::Stdout, "Hiya".into()),
                (Channel::Stderr, "Hello".into()),
                (Channel::Stdout, "Hola".into()),
                (Channel::Stderr, "Hi".into())
            ]
        );
        assert_eq!(out.unwrap(), ProcessStatus::Finished);
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
        let result = Process::spawn(command).unwrap();

        let (result, out) = result.read_timeout(time::Duration::MAX, "test");
        assert_eq!(
            &result,
            &[
                (Channel::Stdout, "Hiya".into()),
                (Channel::Stderr, "Hello".into()),
            ]
        );
        assert_eq!(out.unwrap(), ProcessStatus::Continuing);

        let (result, out) = result.read_timeout(time::Duration::MAX, "");
        assert_eq!(
            &result,
            &[
                (Channel::Stdout, "test".into()),
                (Channel::Stdout, "end".into()),
            ]
        );
        assert_eq!(out.unwrap(), ProcessStatus::Finished);
    }

    #[test]
    fn timeout() {
        let mut command = Command::new("node");
        let program = r#"
            console.log('Hiya');
            setTimeout(() => { }, 2000);
        "#;
        let _ = command.args(&["--eval", program]);
        let result = Process::spawn(command).unwrap();
        let (result, out) = result.read_timeout(time::Duration::MAX, "");
        assert_eq!(&result, &[(Channel::Stdout, "Hiya".into())]);
        assert_eq!(out.unwrap(), ProcessStatus::Finished);

        let mut command = Command::new("node");
        let program = r#"
            console.log('Hiya');
            setTimeout(() => { }, 2000);
        "#;
        let _ = command.args(&["--eval", program]);
        let result = Process::spawn(command).unwrap();
        let (result, out) = result.read_timeout(time::Duration::from_millis(1000), "");
        assert_eq!(&result, &[(Channel::Stdout, "Hiya".into())]);
        assert_eq!(out.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }
}
