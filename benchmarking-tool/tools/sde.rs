use std::io::BufReader;
use std::process::{Command, Stdio};

pub const TEMP_FILE: &str = "sde-out.txt";

pub fn run_sde(input: super::BenchmarkInput) {
    let file_path: &str = input.keep.as_deref().unwrap_or(TEMP_FILE);

    {
        let mut command = Command::new("sde");
        command.args([
            "-omix",
            file_path,
            "-mix_filter_no_shared_libs",
            "-top_blocks",
            // TODO hmm
            &(2 * input.limit).to_string(),
            "--",
        ]);
        command.arg(input.program);
        command.args(input.arguments);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let _ = child.wait().unwrap();
    }

    let file = std::fs::File::open(file_path).unwrap();

    let out = BufReader::new(file);

    let rows = sde_output_parser::parse(out, input.skip_internals);

    let rows: Vec<_> = rows
        .into_iter()
        .map(|(name, item)| crate::Entry {
            name,
            total: item.total,
            entries: vec![
                ("mem_read".to_owned(), item.mem_read),
                ("mem_write".to_owned(), item.mem_write),
                ("stack_read".to_owned(), item.stack_read),
                ("stack_write".to_owned(), item.stack_write),
                ("call".to_owned(), item.call),
            ],
        })
        .collect();

    let total_count: usize = rows.iter().fold(0, |acc, row| acc + row.total as usize);

    crate::print_results(rows, total_count, input.format, input.sort, input.limit);

    if input.keep.is_none() {
        std::fs::remove_file(file_path).unwrap();
    }
}
