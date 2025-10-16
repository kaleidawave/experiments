mod utilities;

use std::process::{Command, Stdio, ExitStatus};
use std::time::{Duration, Instant};
use utilities::ArgumentIter;

#[derive(Debug)]
struct RunData {
    pub duration: Duration,
    #[cfg(unix)]
    pub instructions: usize,
    #[cfg(unix)]
    pub memory_usage: usize
}

#[derive(Debug)]
struct Benchmark {
    command: String,
    arguments: Vec<String>,
    pub(crate) total_elapsed: Duration,
    runs: Vec<RunData>
}

impl Benchmark {
    pub fn name_and_arguments(&self) -> (&str, &[String]) {
        (&self.command, &self.arguments)
    }

    pub fn duration_nanos(&self) -> u128 {
        self.total_elapsed.as_nanos()
    }

    pub fn elapsed(&self) -> &Duration {
        &self.total_elapsed
    }
}

#[cfg(unix)]
fn measure(mut command: Command) -> Result<(RunData, ExitStatus), ()> {
    use perf_event_open::config::{Cpu, Opts, Proc, SampleOn, Size};
    use perf_event_open::count::Counter;
    use perf_event_open::event::hw::Hardware;

    // Count retired instructions on current process, all CPUs.
    let event = Hardware::Instr;
    let target = (Proc::CURRENT, Cpu::ALL);

    let mut opts = Opts::default();
    opts.sample_on = SampleOn::Freq(1000); // 1000 samples per second.
    opts.sample_format.user_stack = Some(Size(8)); // Dump 8-bytes user stack in sample.

    let counter = Counter::new(event, target, opts).unwrap();
    let sampler = counter.sampler(10).unwrap(); // Allocate 2^10 pages to store samples.

    let instrs = counter.stat().unwrap().count;
    println!("{} instructions retired", instrs);

    for it in sampler.iter() {
        println!("{:-?}", it);
    }

    let now = Instant::now();
    counter.enable().unwrap(); // Start the counter.
    let exit_status = command.spawn().expect("could not spawn command").wait().expect("command not started?");
    let duration = now.elapsed();
    counter.disable().unwrap(); // Stop the counter.
    let instructions = counter.stat().unwrap().count;

    let data = RunData {
        duration,
        instructions,
        // TODO
        memory_usage: 0
    };

    Ok((data, exit_status))
}

#[cfg(not(unix))]
fn measure(mut command: Command) -> Result<(RunData, ExitStatus), ()> {
    let now = Instant::now();
    let exit_status = command
        .spawn()
        .expect("could not spawn command")
        .wait()
        .expect("command not started?");
    let duration = now.elapsed();

    let data = RunData {
        duration
    };
    Ok((data, exit_status))
}

fn main() {
    let mut to_run = {
        let mut to_run: Vec<Benchmark> = Vec::new();
        let command = std::env::args().nth(1).unwrap();
        let mut flat_arguments = ArgumentIter::new(&command);
        
        let mut current_command = flat_arguments.next().unwrap();
        let mut current_arguments = Vec::new();

        while let Some(item) = flat_arguments.next() {
            if let "," | "\n" = &*item {
                let next_command = flat_arguments.next().unwrap();
                let command = std::mem::replace(&mut current_command, next_command);
                let arguments = std::mem::take(&mut current_arguments);
                let benchmark = Benchmark {
                    command: command.into_owned(),
                    arguments,
                    total_elapsed: Duration::default(),
                    runs: Vec::default()
                };
                to_run.push(benchmark);
            } else {
                current_arguments.push(item.into_owned());
            }
        }
        let benchmark = Benchmark {
            command: current_command.into_owned(),
            arguments: current_arguments,
            total_elapsed: Duration::default(),
            runs: Vec::default()
        };
        to_run.push(benchmark);
        to_run
    };


    // TODO clear afterwards?
    println!("running {count} commands", count = to_run.len());

    let allow_non_zero_exit_codes = true;

    let mut running = true;

    while running {
        for to_run in to_run.iter_mut() {
            // Future: do we need to create the command each time.
            // can we run a command twice?
            let (name, arguments) = to_run.name_and_arguments();
            let mut command = Command::new(name);
            for argument in arguments {
                command.arg(argument);
            }
            command.stdout(Stdio::null());
            command.stderr(Stdio::null());

            let (data, exit_code) = measure(command).unwrap();

            to_run.total_elapsed += data.duration;

            // TODO test exit code here
            if !allow_non_zero_exit_codes && !exit_code.success() {
                panic!("command non-zero exit");
            }
        }

        // TODO after some count..
        running = false;
    }


    println!("Benchmarks:");

    to_run.sort_unstable_by_key(|result| u128::MAX - result.duration_nanos());

    for result in &to_run {
        let (name, arguments) = result.name_and_arguments();
        let mut arguments = utilities::List::new(arguments);
        arguments.with_prefix("with");
        println!("  {name}{arguments} took");
        println!("     {elapsed:?}", elapsed = result.elapsed());
    }

    if let [fastest, others @ ..] = to_run.as_slice()
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
