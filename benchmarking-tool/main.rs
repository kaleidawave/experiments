mod utilities;

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use utilities::ArgumentIter;

#[derive(Debug)]
struct Benchmark {
    running: Vec<String>,
    elapsed: Duration,
}

impl Benchmark {
    pub fn name_and_arguments(&self) -> (&str, &[String]) {
        (&self.running[0], &self.running[1..])
    }

    pub fn duration_nanos(&self) -> u128 {
        self.elapsed.as_nanos()
    }

    pub fn elapsed(&self) -> &Duration {
        &self.elapsed
    }
}

fn main() {
    let commands = {
        let mut commands: Vec<Vec<String>> = Vec::new();
        let command = std::env::args().nth(1).unwrap();
        let mut current = Vec::new();
        for item in ArgumentIter::new(&command) {
            if let "," | "\n" = &*item {
                commands.push(std::mem::take(&mut current));
            } else {
                current.push(item.into_owned());
            }
        }
        commands.push(std::mem::take(&mut current));
        commands
    };

    // TODO data
    let mut results = Vec::with_capacity(commands.len());

    // TODO clear afterwards?
    println!("running {count} commands", count = commands.len());

    for command in commands {
        let running = command;
        let mut arguments = running.iter();
        let name = arguments.next().expect("expected command name");
        let mut command = Command::new(&name);
        for argument in arguments {
            command.arg(argument);
        }
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());

        let now = Instant::now();
        let mut result = command.spawn().expect("could not spawn command");
        let out = result.wait().expect("command not started?");
        let elapsed = now.elapsed();

        // TODO test exit code here

        let result = Benchmark { running, elapsed };
        results.push(result);
    }

    println!("Benchmarks:");

    results.sort_unstable_by_key(|result| u128::MAX - result.duration_nanos());
    for result in &results {
        let (name, arguments) = result.name_and_arguments();
        let mut arguments = utilities::List::new(arguments);
        arguments.with_prefix("with");
        println!("  {name}{arguments} took");
        println!("     {elapsed:?}", elapsed = result.elapsed());
    }

    if let [fastest, others @ ..] = results.as_slice()
        && !others.is_empty()
    {
        {
            let (name, arguments) = fastest.name_and_arguments();
            let mut arguments = utilities::List::new(arguments);
            arguments.with_prefix("with");
            println!("{name}{arguments} ran",);
        }
        for result in others {
            let difference = fastest.duration_nanos() as f64 / result.duration_nanos() as f64;
            let (name, arguments) = result.name_and_arguments();
            let mut arguments = utilities::List::new(arguments);
            arguments.with_prefix("with");
            println!(" {difference}x faster than {name}{arguments}");
        }
    }
}
