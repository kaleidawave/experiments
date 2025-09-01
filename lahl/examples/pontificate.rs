use lahl::{CLI, Endpoint, Parameter};

static CHECK_PARAMETERS: &[Parameter] = &[
    Parameter::boolean("number-intrinsics", "test for number intrinsics"),
    Parameter::boolean("release", "release mode"),
];

static ENDPOINTS: &[Endpoint] = &[
    Endpoint::new("check", "type checks code", CHECK_PARAMETERS),
    Endpoint::new_group("experimental", "parse", "parse code", &[]),
    Endpoint::new_group("experimental", "format", "format code", &[]),
];

fn main() {
    let commands = CLI::new(ENDPOINTS);

    let binary = std::env::args().next().unwrap();

    let _ = commands.write_help(&binary, &mut std::io::stdout());

    // let mut arguments = commands.run();

    // dbg!(arguments.next());

    // match parameters.command() {
    // 	"experiments" => {
    // 	  match parameters.subcommand() {
    // 	  }
    // 	}
    // }

    // commands.with_information("information");
    // commands.set_default("information");

    // static RUST_PARAMETERS: &[Parameter] = &[
    // 	Parameter::optional("working-directory", "set working directory"),
    // 	Parameter::boolean("release", "set working directory"),
    // ];

    // static ENDPOINTS: &[Endpoint] = &[
    // 	Endpoint::new("sprint", "runs the command. inherits commands from Rust", RUST_PARAMETERS),
    // 	Endpoint::new("test", "tests (no examples)", &[]),
    // ];
}
