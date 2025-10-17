use std::{io, process::Command, time};
use stdout_stderr_utilities::{Process, Channel, ProcessStatus};

fn main() {
	let now = time::Instant::now();

    let mut command = Command::new("node");
    let program = r#"
            console.log('Hiya');
            setTimeout(() => { }, 5000);
        "#;
    let _ = command.args(&["--eval", program]);
    let result = Process::spawn(command).unwrap();
    let (result, out) = result.read_timeout(time::Duration::MAX, "");
    assert_eq!(result, vec![(Channel::Stdout, "Hiya".into())]);
    assert_eq!(out.unwrap(), ProcessStatus::Finished);

	dbg!(now.elapsed());

    let mut command = Command::new("node");
    let program = r#"
            console.log('Hiya');
            setTimeout(() => { }, 5000);
        "#;
    let _ = command.args(&["--eval", program]);
    let result = Process::spawn(command).unwrap();
    let (result, out) = result.read_timeout(time::Duration::from_millis(300), "");
    assert_eq!(result, vec![(Channel::Stdout, "Hiya".into())]);
    assert_eq!(out.unwrap_err().kind(), io::ErrorKind::TimedOut);

	dbg!(now.elapsed());
}
