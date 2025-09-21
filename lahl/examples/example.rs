use lahl::{CLI, CommandError, Endpoint, NamedParameter, PositionalParameter, UnknownCommand};

static CHECK_NAMED_PARAMETERS: &[NamedParameter] = &[
    NamedParameter::boolean("number-intrinsics", "test for number intrinsics"),
    NamedParameter::boolean("release", "release mode"),
];

static INFORMATION_PARAMETERS: &[NamedParameter] =
    &[NamedParameter::boolean("version", "print version")];

static FILE_PARAMETERS: &[PositionalParameter] =
    &[PositionalParameter::single("file", "input file")];

static SINGLE_MULTIPLE_FILE_PARAMETERS: &[PositionalParameter] =
    &[PositionalParameter::multiple("file", "input files")];

static MULTIPLE_FILE_PARAMETERS: &[PositionalParameter] = &[
    PositionalParameter::single("specification", "the specification"),
    PositionalParameter::single("runner", "the runner"),
];

static ENDPOINTS: &[Endpoint] = &[
    Endpoint::new("info", "display information", &[], INFORMATION_PARAMETERS),
    Endpoint::new("check", "type checks code", &[], CHECK_NAMED_PARAMETERS),
    Endpoint::new_group("experimental", "parse", "parse code", FILE_PARAMETERS, &[]),
    Endpoint::new_group(
        "experimental",
        "format",
        "format code",
        FILE_PARAMETERS,
        &[],
    ),
    Endpoint::new_group(
        "experimental",
        "reverse",
        "reverse codes",
        SINGLE_MULTIPLE_FILE_PARAMETERS,
        &[],
    ),
    Endpoint::new_group(
        "experimental",
        "test",
        "test code",
        MULTIPLE_FILE_PARAMETERS,
        &[],
    ),
];

fn main() -> std::process::ExitCode {
    let cli = CLI::new(ENDPOINTS, "type checker", Some("info"));
    let (name, result) = cli.run();

    match result {
        Ok((selected, arguments)) => {
            let arguments: Vec<_> = arguments.collect();
            println!("{selected:?}");
            println!("{arguments:?}");
            std::process::ExitCode::SUCCESS
        }
        Err(CommandError::MissingCommand) => {
            eprintln!("Missing command. Run --help to see commands");
            std::process::ExitCode::FAILURE
        }
        Err(CommandError::CLIHelp(endpoint)) => {
            endpoint.write_help(&name, &mut std::io::stdout()).unwrap();
            std::process::ExitCode::SUCCESS
        }
        Err(CommandError::UnknownCommand(UnknownCommand(command))) => {
            eprintln!("Unknown command {command:?}. Run --help to see commands");
            std::process::ExitCode::FAILURE
        }
    }
}
