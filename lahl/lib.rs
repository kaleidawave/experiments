#[derive(Clone, Copy)]
pub struct CLI {
    endpoints: &'static [Endpoint],
    description: &'static str,
    default: Option<&'static str>,
}

impl CLI {
    pub const fn new(endpoints: &'static [Endpoint]) -> Self {
        Self {
            endpoints,
            description: "",
            default: None,
        }
    }

    pub fn set_default(&mut self, default: &'static str) -> &mut Self {
        self.default = Some(default);
        self
    }

    pub fn set_description(&mut self, description: &'static str) -> &mut Self {
        self.description = description;
        self
    }

    pub fn write_help(
        &self,
        binary_name: &str,
        out: &mut impl std::io::Write,
    ) -> std::io::Result<()> {
        if !binary_name.is_empty() {
            write!(out, "{binary_name}: ")?;
        }
        if !self.description.is_empty() {
            writeln!(out, "{description}", description = self.description)?;
        }
        writeln!(out)?;
        for endpoint in self.endpoints {
            let Endpoint {
                name,
                help,
                group,
                parameters,
            } = endpoint;
            if let Some(group) = group {
                writeln!(out, "{group} {name}: {help}")?;
            } else {
                writeln!(out, "{name}: {help}")?;
            }
            for parameter in parameters.iter() {
                let Parameter { name, help, .. } = parameter;
                writeln!(out, "  --{name}: {help}")?;
            }
            writeln!(out)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn lint(&self) -> Result<(), ()> {
        todo!("names all okay etc")
        // Ok(())
    }

    pub fn run(&self) -> CommandResult {
        self.run_args(std::env::args().skip(1))
    }

    pub fn run_args<T>(&self, arguments: T) -> CommandResult
    where
        T: IntoIterator<Item = String>,
    {
        fn evaluate_endpoint(
            endpoint: &Endpoint,
            arguments: impl Iterator<Item = String>,
        ) -> Result<ResolvedArguments, RunError> {
            let mut arguments = arguments.peekable();
            let mut resolved = Vec::new();

            // TODO initial arguments etc

            while let Some(argument) = arguments.next() {
                if let Some(name) = argument.strip_prefix("--") {
                    if let "help" = name {
                        let help_text = HelpText(endpoint.help);
                        return Err(RunError::Help(help_text));
                    }
                    let parameter = endpoint
                        .parameters
                        .iter()
                        .find(|parameter| parameter.name == name);
                    let Some(parameter) = parameter else {
                        let err = ArgumentError::UnknownArgument(name.to_owned());
                        return Err(RunError::InvalidArguments(err));
                    };
                    let name = parameter.name;
                    match parameter.kind {
                        ParameterType::Boolean => {
                            resolved.push(Argument { name, value: None });
                        }
                        ParameterType::OptionalValue => {
                            let value = arguments.next_if(|value| !value.starts_with("--"));
                            resolved.push(Argument { name, value });
                        }
                        ParameterType::Value => {
                            let Some(value) = arguments.next() else {
                                let err = ArgumentError::ExpectedValue(name);
                                return Err(RunError::InvalidArguments(err));
                            };
                            let value = Some(value);
                            resolved.push(Argument { name, value });
                        }
                        ParameterType::Multiple => {
                            const SPLITTER: char = '\t';
                            let mut value = String::new();
                            while let Some(next) =
                                arguments.next_if(|value| !value.starts_with("--"))
                            {
                                if !value.is_empty() {
                                    value.push(SPLITTER);
                                }
                                value.push_str(&next);
                            }
                            // FUTURE enforce empty?
                            let value = Some(value);
                            resolved.push(Argument { name, value });
                        }
                        ParameterType::First | ParameterType::FirstElseTakeSTDIN => unreachable!(),
                    }
                } else {
                    let err = ArgumentError::ExpectedDashDash(argument);
                    return Err(RunError::InvalidArguments(err));
                }
            }

            Ok(resolved)
        }

        let mut arguments = arguments.into_iter();

        let Some(first) = arguments.next() else {
            return Err(RunError::MissingCommand);
        };
        let endpoint = self
            .endpoints
            .iter()
            .find(|endpoint| endpoint.name == first);
        if let Some(endpoint) = endpoint {
            let arguments = evaluate_endpoint(endpoint, arguments)?;
            let command = SelectedCommand {
                name: endpoint.name,
                group: endpoint.group,
            };
            Ok((command, arguments))
        } else {
            let second = arguments.next();
            let endpoint = second.and_then(|second| {
                self.endpoints.iter().find(|endpoint| {
                    endpoint.group.is_some_and(|group| group == first) && endpoint.name == second
                })
            });
            if let Some(endpoint) = endpoint {
                let arguments = evaluate_endpoint(endpoint, arguments)?;
                let command = SelectedCommand {
                    name: endpoint.name,
                    group: endpoint.group,
                };
                Ok((command, arguments))
            } else {
                Err(RunError::UnknownCommand(UnknownCommand(first)))
            }
        }
    }
}

pub struct Endpoint {
    name: &'static str,
    group: Option<&'static str>,
    help: &'static str,
    parameters: &'static [Parameter],
}

impl Endpoint {
    pub const fn new(
        name: &'static str,
        help: &'static str,
        parameters: &'static [Parameter],
    ) -> Self {
        Self {
            name,
            group: None,
            help,
            parameters,
        }
    }

    pub const fn new_group(
        group: &'static str,
        name: &'static str,
        help: &'static str,
        parameters: &'static [Parameter],
    ) -> Self {
        Self {
            name,
            group: Some(group),
            help,
            parameters,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Parameter {
    pub name: &'static str,
    // TODO auto short?
    // pub short: &'static str,
    pub help: &'static str,
    pub kind: ParameterType,
}

// either boolean, optional value, value, spread. Also want value from STDIN
#[derive(Debug, Clone, Copy)]
pub enum ParameterType {
    Boolean,
    OptionalValue,
    Value,
    First,
    Multiple,
    FirstElseTakeSTDIN,
}

impl Parameter {
    pub const fn boolean(name: &'static str, help: &'static str) -> Self {
        Self {
            name,
            help,
            kind: ParameterType::Boolean,
        }
    }

    pub const fn optional(name: &'static str, help: &'static str) -> Self {
        Self {
            name,
            help,
            kind: ParameterType::OptionalValue,
        }
    }
}

#[derive(Debug)]
pub struct SelectedCommand {
    pub group: Option<&'static str>,
    pub name: &'static str,
}

pub type ResolvedArguments = Vec<Argument>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    name: &'static str,
    value: Option<String>,
}

#[derive(Debug)]
pub struct UnknownCommand(pub String);

#[derive(Debug)]
pub struct HelpText(pub &'static str);

pub type CommandResult = Result<(SelectedCommand, ResolvedArguments), RunError>;

#[derive(Debug)]
pub enum RunError {
    MissingCommand,
    UnknownCommand(UnknownCommand),
    Help(HelpText),
    InvalidArguments(ArgumentError),
}

#[derive(Debug)]
pub enum ArgumentError {
    MissingFirstArgument,
    UnknownArgument(String),
    ExpectedDashDash(String),
    ExpectedValue(&'static str),
}

#[cfg(test)]
mod tests {
    use super::{Argument, ArgumentError, CLI, Endpoint, Parameter, RunError};

    static CHECK_PARAMETERS: &[Parameter] = &[
        Parameter::boolean("number-intrinsics", "test for number intrinsics"),
        Parameter::boolean("release", "release mode"),
    ];

    static ENDPOINTS: &[Endpoint] = &[
        Endpoint::new("check", "type checks code", CHECK_PARAMETERS),
        Endpoint::new_group("experimental", "parse", "parse code", &[]),
        Endpoint::new_group("experimental", "format", "format code", &[]),
    ];

    #[test]
    fn write_help() {
        let mut commands = CLI::new(ENDPOINTS);
        commands.set_description("a type checker for code");
        let mut help = Vec::new();
        let _ = commands.write_help("type-checker", &mut help);
        let help = String::from_utf8(help).unwrap();
        assert_eq!(
            help,
            "type-checker: a type checker for code

check: type checks code
  --number-intrinsics: test for number intrinsics
  --release: release mode

experimental parse: parse code

experimental format: format code

"
        );
    }

    #[test]
    fn get_values() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["check".into(), "--release".into()];
        let commands = CLI::new(ENDPOINTS);
        let (selected, arguments) = commands.run_args(arguments).unwrap();
        assert_eq!(selected.name, "check");
        assert_eq!(selected.group, None);
        assert_eq!(
            arguments,
            vec![Argument {
                name: "release",
                value: None
            }]
        );
    }

    #[test]
    fn help_command() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["check".into(), "--help".into()];
        let commands = CLI::new(ENDPOINTS);
        let result = commands.run_args(arguments);
        let is_expected = matches!(
            result,
            Err(RunError::Help(help)) if help.0 == "type checks code"
        );
        assert!(is_expected, "Help command invalid");
    }

    #[test]
    fn expected_a_command() {
        let commands = CLI::new(ENDPOINTS);
        let result = commands.run_args(Vec::new());
        assert!(
            matches!(result, Err(RunError::MissingCommand)),
            "Expected ExpectedCommand"
        );
    }

    #[test]
    fn invalid_first_command() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["quack".into(), "--release".into()];
        let commands = CLI::new(ENDPOINTS);
        let result = commands.run_args(arguments);
        let is_expected = matches!(
            result,
            Err(RunError::UnknownCommand(unknown)) if unknown.0 == "quack"
        );
        assert!(is_expected, "Expected not first command");
    }

    #[test]
    fn invalid_flag() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["check".into(), "release".into()];
        let commands = CLI::new(ENDPOINTS);
        let result = commands.run_args(arguments);
        let is_expected = matches!(
            result,
            Err(RunError::InvalidArguments(ArgumentError::ExpectedDashDash(ref unknown))) if unknown == "release"
        );
        assert!(is_expected, "Expected ExpectedDashDash, got {result:?}");
    }
}
