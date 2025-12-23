use std::io::BufReader;
use std::process::{Command, Stdio};

pub const TEMP_FILE: &str = "sde-out.txt";

pub fn run_sde(
    request: crate::CommandRequest,
    options: crate::ToolOptions,
) -> Result<crate::ToolOutput, ()> {
    let file_path: &str = options.keep.as_deref().unwrap_or(TEMP_FILE);

    // TODO hmm
    let blocks = 50;

    {
        let sde_path = std::env::var("SDE_PATH").map(|dir| format!("{dir}/sde"));
        let sde_path = sde_path.as_deref().unwrap_or("sde");

        let mut command = Command::new(sde_path);
        command.args([
            "-omix",
            file_path,
            "-mix_filter_no_shared_libs",
            "-top_blocks",
            &blocks.to_string(),
            "--",
        ]);
        command.arg(request.program);
        command.args(request.arguments);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn().unwrap();
        let _ = child.wait().unwrap();
    }

    let file = std::fs::File::open(file_path).unwrap();

    let out = BufReader::new(file);

    let rows = sde_output_parser::parse(out, options.skip_internals);

    let symbols: Vec<_> = rows
        .into_iter()
        .map(|(symbol_name, item)| crate::Entry {
            symbol_name,
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

    let total = symbols.iter().fold(0, |acc, row| acc + row.total);

    if options.keep.is_none() {
        std::fs::remove_file(file_path).unwrap();
    }

    Ok(crate::ToolOutput::SymbolInstructionCounts { total, symbols })
}
