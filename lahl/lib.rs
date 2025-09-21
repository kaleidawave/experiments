#![doc = include_str!("README.md")]

pub use definitions::{
    CLI, Endpoint, NamedParameter, NamedParameterKind, PositionalParameter, PositionalParameterKind,
};
pub use evaluation::{
    Argument, ArgumentError, CommandError, CommandResult, SelectedCommand, UnknownCommand,
    argument_result_or_out, command_result_or_out,
};

mod definitions {
    /// Main CLI program
    #[derive(Debug, Clone, Copy)]
    pub struct CLI {
        pub(crate) endpoints: &'static [Endpoint],
        pub(crate) description: &'static str,
        pub(crate) default: Option<&'static str>,
    }

    impl CLI {
        pub const fn new(
            endpoints: &'static [Endpoint],
            description: &'static str,
            default: Option<&'static str>,
        ) -> Self {
            Self {
                endpoints,
                description,
                default,
            }
        }

        pub const fn set_description(mut self, description: &'static str) -> Self {
            self.description = description;
            self
        }

        pub const fn new_just_endpoints(endpoints: &'static [Endpoint]) -> Self {
            Self {
                endpoints,
                description: "",
                default: None,
            }
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
                write!(out, "{description}", description = self.description)?;
                if let Some(default) = self.default {
                    write!(out, " (default command: {default})")?;
                }
                writeln!(out)?;
            }
            writeln!(out)?;
            for endpoint in self.endpoints {
                endpoint.write_help(out)?;
                writeln!(out)?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Endpoint {
        pub(crate) name: &'static str,
        pub(crate) group: Option<&'static str>,
        pub(crate) description: &'static str,
        pub(crate) positional_parameters: &'static [PositionalParameter],
        pub(crate) named_parameters: &'static [NamedParameter],
    }

    impl Endpoint {
        pub const fn new(
            name: &'static str,
            description: &'static str,
            positional_parameters: &'static [PositionalParameter],
            named_parameters: &'static [NamedParameter],
        ) -> Self {
            Self {
                name,
                group: None,
                description,
                positional_parameters,
                named_parameters,
            }
        }

        pub const fn new_group(
            group: &'static str,
            name: &'static str,
            description: &'static str,
            positional_parameters: &'static [PositionalParameter],
            named_parameters: &'static [NamedParameter],
        ) -> Self {
            Self {
                name,
                group: Some(group),
                description,
                positional_parameters,
                named_parameters,
            }
        }

        pub fn write_help(&self, out: &mut impl std::io::Write) -> std::io::Result<()> {
            let Endpoint {
                name,
                description,
                group,
                positional_parameters,
                named_parameters,
            } = self;
            if let Some(group) = group {
                writeln!(out, "{group} {name}: {description}")?;
            } else {
                writeln!(out, "{name}: {description}")?;
            }
            for parameter in positional_parameters.iter() {
                let PositionalParameter {
                    name,
                    description,
                    kind,
                } = parameter;
                let kind = match kind {
                    PositionalParameterKind::Single => "",
                    PositionalParameterKind::Multiple => " (multiple)",
                };
                writeln!(out, "  *{name}*: {description}{kind}")?;
            }
            for parameter in named_parameters.iter() {
                let NamedParameter {
                    name,
                    description,
                    kind,
                } = parameter;
                let kind = match kind {
                    NamedParameterKind::Boolean => "(boolean)",
                    NamedParameterKind::OptionalValue => "(optional value)",
                    NamedParameterKind::Value => "(value)",
                    NamedParameterKind::Multiple => "(multiple)",
                };
                writeln!(out, "  --{name}: {description} {kind}")?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct PositionalParameter {
        pub(crate) name: &'static str,
        // TODO auto short?
        // pub short: &'static str,
        pub(crate) description: &'static str,
        pub(crate) kind: PositionalParameterKind,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum PositionalParameterKind {
        Single,
        // TODO SingleElseStdin,
        Multiple,
    }

    impl PositionalParameter {
        pub const fn new(
            name: &'static str,
            description: &'static str,
            kind: PositionalParameterKind,
        ) -> Self {
            Self {
                name,
                description,
                kind,
            }
        }

        pub const fn single(name: &'static str, description: &'static str) -> Self {
            Self::new(name, description, PositionalParameterKind::Single)
        }

        pub const fn multiple(name: &'static str, description: &'static str) -> Self {
            Self::new(name, description, PositionalParameterKind::Multiple)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct NamedParameter {
        pub(crate) name: &'static str,
        // TODO auto short?
        // pub short: &'static str,
        pub(crate) description: &'static str,
        pub(crate) kind: NamedParameterKind,
    }

    // either boolean, optional value, value, spread. Also want value from STDIN
    #[derive(Debug, Clone, Copy)]
    pub enum NamedParameterKind {
        /// `--a`
        Boolean,
        /// `--a *value*` or `--a`. Breaks if argument starts with `--`
        OptionalValue,
        /// `--a *value*`
        Value,
        /// `--a *value1* *value2* *value3*`. Breaks once argument starts with `--`
        Multiple,
    }

    impl NamedParameter {
        pub const fn new(
            name: &'static str,
            description: &'static str,
            kind: NamedParameterKind,
        ) -> Self {
            Self {
                name,
                description,
                kind,
            }
        }

        pub const fn boolean(name: &'static str, description: &'static str) -> Self {
            Self::new(name, description, NamedParameterKind::Boolean)
        }

        pub const fn optional(name: &'static str, description: &'static str) -> Self {
            Self::new(name, description, NamedParameterKind::OptionalValue)
        }

        pub const fn value(name: &'static str, description: &'static str) -> Self {
            Self::new(name, description, NamedParameterKind::Value)
        }

        pub const fn multiple(name: &'static str, description: &'static str) -> Self {
            Self::new(name, description, NamedParameterKind::Multiple)
        }
    }
}

pub mod evaluation {
    use super::{CLI, Endpoint, NamedParameterKind, PositionalParameterKind};

    #[derive(Debug)]
    pub struct SelectedCommand {
        pub group: Option<&'static str>,
        pub name: &'static str,
    }

    pub struct ArgumentIterator<T: Iterator<Item = String>> {
        endpoint: Endpoint,
        arguments: std::iter::Peekable<T>,
        positional_idx: usize,
        current_multiple: Option<&'static str>,
    }

    impl<T: Iterator<Item = String>> ArgumentIterator<T> {
        pub fn new(endpoint: Endpoint, arguments: std::iter::Peekable<T>) -> Self {
            Self {
                endpoint,
                arguments,
                positional_idx: 0,
                current_multiple: None,
            }
        }
    }

    impl<T: Iterator<Item = String>> Iterator for ArgumentIterator<T> {
        type Item = Result<Argument, ArgumentError>;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(argument) = self.arguments.next() {
                if let "--help" = argument.as_str() {
                    return Some(Err(ArgumentError::EndpointHelp(self.endpoint)));
                }

                let initial = self.endpoint.positional_parameters.get(self.positional_idx);

                if let Some(parameter) = initial {
                    let name = parameter.name;
                    match parameter.kind {
                        PositionalParameterKind::Single => {
                            self.positional_idx += 1;
                            let argument = Argument {
                                name,
                                value: Some(argument),
                            };
                            return Some(Ok(argument));
                        }
                        // PositionalParameterKind::SingleElseStdin => {
                        //     if argument.starts_with("--") {
                        //         // TODO collect
                        //         continue;
                        //     }
                        //     self.positional_idx += 1;
                        // },
                        PositionalParameterKind::Multiple => {
                            if !argument.starts_with("--") {
                                let argument = Argument {
                                    name,
                                    value: Some(argument),
                                };
                                return Some(Ok(argument));
                            }
                        }
                    }
                }

                if let Some(name) = argument.strip_prefix("--") {
                    let _ = self.current_multiple.take();
                    let parameter = self
                        .endpoint
                        .named_parameters
                        .iter()
                        .find(|parameter| parameter.name == name);

                    let Some(parameter) = parameter else {
                        let err = ArgumentError::UnknownArgument(name.to_owned());
                        return Some(Err(err));
                    };

                    let name = parameter.name;
                    match parameter.kind {
                        NamedParameterKind::Boolean => {
                            let value = None;
                            let argument = Argument { name, value };
                            Some(Ok(argument))
                        }
                        NamedParameterKind::OptionalValue => {
                            let value = self.arguments.next_if(|value| !value.starts_with("--"));
                            let argument = Argument { name, value };
                            Some(Ok(argument))
                        }
                        NamedParameterKind::Value => {
                            let Some(value) = self.arguments.next() else {
                                let err = ArgumentError::ExpectedValue(name);
                                return Some(Err(err));
                            };
                            let value = Some(value);
                            let argument = Argument { name, value };
                            Some(Ok(argument))
                        }
                        NamedParameterKind::Multiple => {
                            // TODO could concatenate?
                            let value = self.arguments.next_if(|value| !value.starts_with("--"));
                            self.current_multiple = Some(name);
                            let argument = Argument { name, value };
                            Some(Ok(argument))
                        }
                    }
                } else if let Some(name) = self.current_multiple {
                    let value = Some(argument);
                    let argument = Argument { name, value };
                    Some(Ok(argument))
                } else {
                    let err = ArgumentError::ExpectedDashDash(argument);
                    Some(Err(err))
                }
            } else {
                let positional_parameter = self
                    .endpoint
                    .positional_parameters
                    .get(self.positional_idx)?;

                self.positional_idx += 1;

                let name = positional_parameter.name;
                match positional_parameter.kind {
                    PositionalParameterKind::Single => {
                        Some(Err(ArgumentError::ExpectedValue(name)))
                    }
                    // PositionalParameterKind::SingleElseStdin => {
                    //     // TODO read stdin
                    // },
                    // fine, maybe expect at least one though?
                    PositionalParameterKind::Multiple => None,
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum ArgumentError {
        EndpointHelp(Endpoint),
        UnknownArgument(String),
        ExpectedDashDash(String),
        ExpectedValue(&'static str),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Argument {
        pub name: &'static str,
        pub value: Option<String>,
    }

    #[derive(Debug)]
    pub struct UnknownCommand(pub String);

    pub type CommandResult<T> = Result<(SelectedCommand, ArgumentIterator<T>), CommandError>;

    #[derive(Debug)]
    pub enum CommandError {
        MissingCommand,
        CLIHelp(CLI),
        UnknownCommand(UnknownCommand),
    }

    impl CLI {
        pub fn run(&self) -> (String, CommandResult<std::env::Args>) {
            let mut args = std::env::args();
            let name = args.next().unwrap();
            (name, self.run_args(args))
        }

        pub fn run_args<T>(&self, arguments: T) -> CommandResult<T::IntoIter>
        where
            T: IntoIterator<Item = String>,
        {
            let mut arguments = arguments.into_iter().peekable();

            if let Some("--help") = arguments.peek().map(String::as_str) {
                return Err(CommandError::CLIHelp(*self));
            }

            let first = arguments.next_if(|arg| !arg.starts_with("--"));

            // Can be "" if unspecified
            let first: &str = first.as_deref().or(self.default).unwrap_or_default();

            let endpoint = self
                .endpoints
                .iter()
                .find(|endpoint| endpoint.name == first);

            if let Some(endpoint) = endpoint {
                let command = SelectedCommand {
                    name: endpoint.name,
                    group: endpoint.group,
                };
                let arguments = ArgumentIterator::new(*endpoint, arguments);
                Ok((command, arguments))
            } else if first.is_empty() {
                Err(CommandError::MissingCommand)
            } else {
                let second = arguments.next();
                let endpoint = second.and_then(|second| {
                    self.endpoints.iter().find(|endpoint| {
                        endpoint.group.is_some_and(|group| group == first)
                            && endpoint.name == second
                    })
                });
                if let Some(endpoint) = endpoint {
                    let arguments = ArgumentIterator::new(*endpoint, arguments);
                    let command = SelectedCommand {
                        name: endpoint.name,
                        group: endpoint.group,
                    };
                    Ok((command, arguments))
                } else {
                    Err(CommandError::UnknownCommand(UnknownCommand(
                        first.to_owned(),
                    )))
                }
            }
        }
    }

    pub fn command_result_or_out<T: Iterator<Item = String>>(
        result: CommandResult<T>,
        binary_name: &str,
    ) -> Result<(SelectedCommand, ArgumentIterator<T>), std::process::ExitCode> {
        match result {
            Ok(out) => Ok(out),
            // FUTURE could be lahl logic?
            Err(CommandError::MissingCommand) => {
                eprintln!("expected a command");
                Err(std::process::ExitCode::FAILURE)
            }
            Err(CommandError::UnknownCommand(unknown)) => {
                eprintln!("unknown command {unknown:?}");
                Err(std::process::ExitCode::FAILURE)
            }
            Err(CommandError::CLIHelp(cli)) => {
                cli.write_help(binary_name, &mut std::io::stdout()).unwrap();
                Err(std::process::ExitCode::SUCCESS)
            }
        }
    }

    pub fn argument_result_or_out(
        result: Result<Argument, ArgumentError>,
    ) -> Result<Argument, std::process::ExitCode> {
        match result {
            Ok(argument) => Ok(argument),
            Err(ArgumentError::EndpointHelp(endpoint)) => {
                endpoint.write_help(&mut std::io::stdout()).unwrap();
                Err(std::process::ExitCode::SUCCESS)
            }
            Err(err) => {
                eprintln!("error: {err:?}");
                Err(std::process::ExitCode::FAILURE)
            }
        }
    }
}

/// TODO assert lowercase, '-' instead of '_' etc, no quotes etc
#[cfg(test)]
pub mod lint {
    impl super::CLI {
        pub fn lint(&self) -> Result<(), CLIDefinitionError> {
            if let Some(default) = self.default
                && !self
                    .endpoints
                    .iter()
                    .any(|endpoint| endpoint.group.is_none() && endpoint.name == default)
            {
                return Err(CLIDefinitionError::InvalidDefault(default));
            }

            let mut endpoint_names = std::collections::HashSet::<&str>::new();

            for endpoint in self.endpoints {
                if endpoint.name.chars().any(char::is_whitespace) {
                    return Err(CLIDefinitionError::InvalidEndpointName(endpoint.name));
                }

                if !endpoint_names.insert(endpoint.name) {
                    return Err(CLIDefinitionError::DuplicateEndpointName(endpoint.name));
                }

                let mut multiple = false;
                for parameter in endpoint.positional_parameters {
                    if multiple {
                        return Err(CLIDefinitionError::PositionalParameterAfterMultiple(
                            parameter.name,
                        ));
                    }

                    if let super::PositionalParameterKind::Multiple = parameter.kind {
                        multiple = true;
                    }
                }

                let mut parameter_names = std::collections::HashSet::<&str>::new();

                for parameter in endpoint.named_parameters {
                    if parameter.name.is_empty() || parameter.name.chars().any(char::is_whitespace)
                    {
                        return Err(CLIDefinitionError::InvalidNamedParameterName(
                            parameter.name,
                        ));
                    }

                    if !parameter_names.insert(parameter.name) {
                        return Err(CLIDefinitionError::DuplicateParameterName(parameter.name));
                    }
                }
            }
            Ok(())
        }
    }

    /// Errors that can occur
    #[cfg(test)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum CLIDefinitionError {
        InvalidDefault(&'static str),
        InvalidEndpointName(&'static str),
        InvalidNamedParameterName(&'static str),
        PositionalParameterAfterMultiple(&'static str),
        DuplicateEndpointName(&'static str),
        DuplicateParameterName(&'static str),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Argument, ArgumentError, CLI, CommandError, Endpoint, NamedParameter, PositionalParameter,
        lint::CLIDefinitionError,
    };

    static CHECK_NAMED_PARAMETERS: &[NamedParameter] = &[
        NamedParameter::boolean("number-intrinsics", "test for number intrinsics"),
        NamedParameter::boolean("release", "release mode"),
    ];

    static FILE_PARAMETERS: &[PositionalParameter] =
        &[PositionalParameter::single("file", "input file")];

    static ENDPOINTS: &[Endpoint] = &[
        Endpoint::new("check", "type checks code", &[], CHECK_NAMED_PARAMETERS),
        Endpoint::new_group("experimental", "parse", "parse code", FILE_PARAMETERS, &[]),
        Endpoint::new_group(
            "experimental",
            "format",
            "format code",
            FILE_PARAMETERS,
            &[],
        ),
    ];

    static MORE_PARAMETERS: &[PositionalParameter] =
        &[PositionalParameter::multiple("file", "input files")];

    static MORE_ENDPOINTS: &[Endpoint] = &[Endpoint::new(
        "reverse",
        "flips files",
        MORE_PARAMETERS,
        &[],
    )];

    #[test]
    fn get_named_arguments() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["check".into(), "--release".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let (selected, arguments) = commands.run_args(arguments).unwrap();
        assert_eq!(selected.name, "check");
        assert_eq!(selected.group, None);

        let arguments = arguments
            .collect::<Result<Vec<_>, ArgumentError>>()
            .unwrap();
        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].name, "release");
        assert_eq!(arguments[0].value, None);
    }

    #[test]
    fn get_positional_arguments() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec![
            "reverse".into(),
            "file1.ts".into(),
            "file2.ts".into(),
            "file3.ts".into(),
        ];
        let commands = CLI::new_just_endpoints(MORE_ENDPOINTS);
        let (selected, arguments) = commands.run_args(arguments).unwrap();
        assert_eq!(selected.name, "reverse");
        assert_eq!(selected.group, None);
        let arguments = arguments
            .collect::<Result<Vec<_>, ArgumentError>>()
            .unwrap();
        assert_eq!(
            arguments,
            vec![
                Argument {
                    name: "file",
                    value: Some("file1.ts".into())
                },
                Argument {
                    name: "file",
                    value: Some("file2.ts".into())
                },
                Argument {
                    name: "file",
                    value: Some("file3.ts".into())
                },
            ]
        );
    }

    #[test]
    fn get_positional_and_named_arguments() {
        static POSITIONAL: &[PositionalParameter] = &[PositionalParameter::multiple("x", "")];
        static NAMED: &[NamedParameter] = &[NamedParameter::boolean("d", "")];
        static ENDPOINTS: &[Endpoint] = &[Endpoint::new("a", "", POSITIONAL, NAMED)];

        let arguments: Vec<String> = vec!["a".into(), "b".into(), "c".into(), "--d".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let (selected, arguments) = commands.run_args(arguments).unwrap();
        assert_eq!(selected.name, "a");
        assert_eq!(selected.group, None);
        let arguments = arguments
            .collect::<Result<Vec<_>, ArgumentError>>()
            .unwrap();
        assert_eq!(
            arguments,
            vec![
                Argument {
                    name: "x",
                    value: Some("b".into())
                },
                Argument {
                    name: "x",
                    value: Some("c".into())
                },
                Argument {
                    name: "d",
                    value: None
                },
            ]
        );
    }

    #[test]
    fn named_multiple() {
        static NAMED: &[NamedParameter] = &[NamedParameter::multiple("x", "")];
        static ENDPOINTS: &[Endpoint] = &[Endpoint::new("a", "", &[], NAMED)];

        let arguments: Vec<String> =
            vec!["a".into(), "--x".into(), "1".into(), "2".into(), "3".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let (selected, arguments) = commands.run_args(arguments).unwrap();
        assert_eq!(selected.name, "a");
        assert_eq!(selected.group, None);
        let arguments = arguments
            .collect::<Result<Vec<_>, ArgumentError>>()
            .unwrap();
        assert_eq!(
            arguments,
            vec![
                Argument {
                    name: "x",
                    value: Some("1".into())
                },
                Argument {
                    name: "x",
                    value: Some("2".into())
                },
                Argument {
                    name: "x",
                    value: Some("3".into())
                },
            ]
        );
    }

    #[test]
    fn default_endpoint() {
        let commands = CLI::new(ENDPOINTS, "", Some("check"));
        let (selected, arguments) = commands.run_args(Vec::new()).unwrap();
        assert_eq!(selected.name, "check");
        assert_eq!(selected.group, None);
        assert_eq!(arguments.count(), 0);
    }

    // help/description commands

    #[test]
    fn write_help() {
        let commands = CLI::new(ENDPOINTS, "a type checker for code", None);
        let mut description = Vec::new();
        let _ = commands.write_help("type-checker", &mut description);
        let description = String::from_utf8(description).unwrap();
        assert_eq!(
            description,
            "type-checker: a type checker for code

check: type checks code
  --number-intrinsics: test for number intrinsics (boolean)
  --release: release mode (boolean)

experimental parse: parse code
  *file*: input file

experimental format: format code
  *file*: input file

",
            "{description} did not match expected output"
        );
    }

    #[test]
    fn write_help_for_command() {
        let arguments: Vec<String> = vec!["check".into(), "--help".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let (_, mut arguments) = commands.run_args(arguments).unwrap();

        let Err(ArgumentError::EndpointHelp(endpoint)) = arguments.next().unwrap() else {
            panic!("expected description run error");
        };
        let mut description = Vec::new();
        let _ = endpoint.write_help(&mut description);
        let description = String::from_utf8(description).unwrap();
        assert_eq!(
            description,
            "check: type checks code
  --number-intrinsics: test for number intrinsics (boolean)
  --release: release mode (boolean)
",
            "{description} did not match expected output"
        );
    }

    // errors

    #[test]
    fn expected_a_command() {
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let result = commands.run_args(Vec::new());
        assert!(
            matches!(result, Err(CommandError::MissingCommand)),
            "Expected ExpectedCommand"
        );
    }

    #[test]
    fn invalid_first_command() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["quack".into(), "--release".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let result = commands.run_args(arguments);
        let is_expected = matches!(
            result,
            Err(CommandError::UnknownCommand(unknown)) if unknown.0 == "quack"
        );
        assert!(is_expected, "Expected not first command");
    }

    #[test]
    fn expected_positional_value() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["experimental".into(), "parse".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let (_, arguments) = commands.run_args(arguments).unwrap();
        let result: Result<Vec<Argument>, _> = arguments.collect();
        let is_expected = matches!(
            result,
            Err(ArgumentError::ExpectedValue(ref name)) if *name == "file"
        );
        assert!(is_expected, "Expected ExpectedValue, got {result:?}");
    }

    #[test]
    fn invalid_flag() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["check".into(), "release".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let (_, arguments) = commands.run_args(arguments).unwrap();
        let result: Result<Vec<Argument>, _> = arguments.collect();
        let is_expected = matches!(
            result,
            Err(ArgumentError::ExpectedDashDash(ref unknown)) if unknown == "release"
        );
        assert!(is_expected, "Expected ExpectedDashDash, got {result:?}");
    }

    // linting

    #[test]
    fn ok() {
        assert_eq!(CLI::new_just_endpoints(ENDPOINTS).lint(), Ok(()));
    }

    #[test]
    fn bad_endpoint_name() {
        static ENDPOINTS1: &[Endpoint] = &[Endpoint::new("", "", &[], &[])];
        assert_eq!(CLI::new_just_endpoints(ENDPOINTS1).lint(), Ok(()));

        static ENDPOINTS2: &[Endpoint] = &[Endpoint::new("two words", "", &[], &[])];
        assert_eq!(
            CLI::new_just_endpoints(ENDPOINTS2).lint(),
            Err(CLIDefinitionError::InvalidEndpointName("two words"))
        );

        static ENDPOINTS3: &[Endpoint] = &[Endpoint::new("valid-named", "", &[], &[])];
        assert_eq!(CLI::new_just_endpoints(ENDPOINTS3).lint(), Ok(()));
    }

    #[test]
    fn bad_parameter_name() {
        static PARAMETERS1: &[NamedParameter] = &[NamedParameter::boolean("", "")];
        static ENDPOINTS1: &[Endpoint] = &[Endpoint::new("a", "", &[], PARAMETERS1)];
        assert_eq!(
            CLI::new_just_endpoints(ENDPOINTS1).lint(),
            Err(CLIDefinitionError::InvalidNamedParameterName(""))
        );

        static PARAMETERS2: &[NamedParameter] = &[NamedParameter::boolean("two words", "")];
        static ENDPOINTS2: &[Endpoint] = &[Endpoint::new("a", "", &[], PARAMETERS2)];
        assert_eq!(
            CLI::new_just_endpoints(ENDPOINTS2).lint(),
            Err(CLIDefinitionError::InvalidNamedParameterName("two words"))
        );

        static PARAMETERS3: &[NamedParameter] = &[NamedParameter::boolean("valid", "")];
        static ENDPOINTS3: &[Endpoint] = &[Endpoint::new("a", "", &[], PARAMETERS3)];
        assert_eq!(CLI::new_just_endpoints(ENDPOINTS3).lint(), Ok(()));
    }

    #[test]
    fn positional_after_multiple() {
        static POSITIONAL: &[PositionalParameter] = &[
            PositionalParameter::multiple("a", ""),
            PositionalParameter::single("b", ""),
        ];
        static ENDPOINTS: &[Endpoint] = &[Endpoint::new("entry", "", POSITIONAL, &[])];
        assert_eq!(
            CLI::new_just_endpoints(ENDPOINTS).lint(),
            Err(CLIDefinitionError::PositionalParameterAfterMultiple("b"))
        );
    }

    #[test]
    fn invalid_default() {
        static ENDPOINTS: &[Endpoint] = &[Endpoint::new("entry", "", &[], &[])];
        assert_eq!(
            CLI::new(ENDPOINTS, "", Some("Entry")).lint(),
            Err(CLIDefinitionError::InvalidDefault("Entry"))
        );
        assert_eq!(CLI::new(ENDPOINTS, "", None).lint(), Ok(()));
    }

    #[test]
    fn duplicate_endpoints_and_parameter_names() {
        static ENDPOINTS1: &[Endpoint] = &[
            Endpoint::new("entry", "", &[], &[]),
            Endpoint::new("entry", "", &[], &[]),
        ];
        assert_eq!(
            CLI::new_just_endpoints(ENDPOINTS1).lint(),
            Err(CLIDefinitionError::DuplicateEndpointName("entry"))
        );

        static ENDPOINTS2: &[Endpoint] = &[Endpoint::new(
            "entry",
            "",
            &[],
            &[
                NamedParameter::boolean("yep", ""),
                NamedParameter::boolean("yep", ""),
            ],
        )];
        assert_eq!(
            CLI::new_just_endpoints(ENDPOINTS2).lint(),
            Err(CLIDefinitionError::DuplicateParameterName("yep"))
        );

        static ENDPOINT3: &[Endpoint] = &[Endpoint::new(
            "entry",
            "",
            &[],
            &[
                NamedParameter::boolean("yep1", ""),
                NamedParameter::boolean("yep2", ""),
            ],
        )];
        assert_eq!(CLI::new_just_endpoints(ENDPOINT3).lint(), Ok(()));
    }
}
