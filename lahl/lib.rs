#[doc = include_str!("README.md")]

#[derive(Debug, Clone, Copy)]
pub struct CLI {
    endpoints: &'static [Endpoint],
    description: &'static str,
    default: Option<&'static str>,
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

    pub fn run(&self) -> CommandResult {
        self.run_args(std::env::args().skip(1))
    }

    pub fn run_args<T>(&self, arguments: T) -> CommandResult
    where
        T: IntoIterator<Item = String>,
    {
        // TODO as Iterator
        fn evaluate_endpoint<T>(
            endpoint: &Endpoint,
            mut arguments: std::iter::Peekable<T>,
        ) -> Result<ResolvedArguments, RunError>
        where
            T: Iterator<Item = String>,
        {
            let mut resolved = Vec::new();

            // TODO initial arguments
            let mut positional_idx = 0;

            while let Some(argument) = arguments.next() {
                let initial = endpoint.positional_parameters.get(positional_idx);

                if let Some(parameter) = initial {
                    let name = parameter.name;
                    match parameter.kind {
                        PositionalParameterKind::Single => {
                            resolved.push(Argument {
                                name,
                                value: Some(argument),
                            });
                            positional_idx += 1;
                            continue;
                        }
                        // PositionalParameterKind::SingleElseStdin => {
                        //     if argument.starts_with("--") {
                        //         // TODO collect
                        //         continue;
                        //     }
                        //     positional_idx += 1;
                        // },
                        PositionalParameterKind::Multiple => {
                            if !argument.starts_with("--") {
                                resolved.push(Argument {
                                    name,
                                    value: Some(argument),
                                });
                                continue;
                            }
                        }
                    }
                }

                if let Some(name) = argument.strip_prefix("--") {
                    if let "description" = name {
                        return Err(RunError::EndpointHelp(*endpoint));
                    }
                    let parameter = endpoint
                        .named_parameters
                        .iter()
                        .find(|parameter| parameter.name == name);
                    let Some(parameter) = parameter else {
                        let err = ArgumentError::UnknownArgument(name.to_owned());
                        return Err(RunError::InvalidArguments(err));
                    };
                    let name = parameter.name;
                    match parameter.kind {
                        NamedParameterKind::Boolean => {
                            resolved.push(Argument { name, value: None });
                        }
                        NamedParameterKind::OptionalValue => {
                            let value = arguments.next_if(|value| !value.starts_with("--"));
                            resolved.push(Argument { name, value });
                        }
                        NamedParameterKind::Value => {
                            let Some(value) = arguments.next() else {
                                let err = ArgumentError::ExpectedValue(name);
                                return Err(RunError::InvalidArguments(err));
                            };
                            let value = Some(value);
                            resolved.push(Argument { name, value });
                        }
                        NamedParameterKind::Multiple => {
                            // TODO could concatenate?
                            while let Some(value) =
                                arguments.next_if(|value| !value.starts_with("--"))
                            {
                                resolved.push(Argument {
                                    name,
                                    value: Some(value),
                                });
                            }
                        }
                    }
                } else {
                    let err = ArgumentError::ExpectedDashDash(argument);
                    return Err(RunError::InvalidArguments(err));
                }
            }

            // TODO could error here
            // {
            //     let initial = endpoint.positional_parameters.get(positional_idx);

            //     if let Some(parameter) = initial {
            //         let name = parameter.name;
            //         match parameter.kind {
            //             PositionalParameterKind::Single => {
            //                 return Err(RunError::InvalidArguments(ArgumentError::MissingInitialArgument(name)));
            //             },
            //             // PositionalParameterKind::SingleElseStdin => {
            //             //     // TODO read stdin
            //             // },
            //             // fine, maybe expect at least one though?
            //             PositionalParameterKind::Multiple => {}
            //         }
            //     }
            // }

            Ok(resolved)
        }

        let mut arguments = arguments.into_iter().peekable();

        if let Some("--help") = arguments.peek().map(String::as_str) {
            return Err(RunError::CLIHelp(*self));
        }

        let first = arguments.next_if(|arg| !arg.starts_with("--"));

        // Can be "" if unspecified
        let first: &str = first.as_deref().or(self.default).unwrap_or_default();

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
        } else if first.is_empty() {
            Err(RunError::MissingCommand)
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
                Err(RunError::UnknownCommand(UnknownCommand(first.to_owned())))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Endpoint {
    name: &'static str,
    group: Option<&'static str>,
    description: &'static str,
    positional_parameters: &'static [PositionalParameter],
    named_parameters: &'static [NamedParameter],
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
    pub name: &'static str,
    // TODO auto short?
    // pub short: &'static str,
    pub description: &'static str,
    pub kind: PositionalParameterKind,
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
    pub name: &'static str,
    // TODO auto short?
    // pub short: &'static str,
    pub description: &'static str,
    pub kind: NamedParameterKind,
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

pub type CommandResult = Result<(SelectedCommand, ResolvedArguments), RunError>;

#[derive(Debug)]
pub enum RunError {
    MissingCommand,
    UnknownCommand(UnknownCommand),
    CLIHelp(CLI),
    EndpointHelp(Endpoint),
    InvalidArguments(ArgumentError),
}

#[derive(Debug)]
pub enum ArgumentError {
    MissingInitialArgument(&'static str),
    UnknownArgument(String),
    ExpectedDashDash(String),
    ExpectedValue(&'static str),
}

/// TODO assert lowercase, '-' instead of '_' etc, no quotes etc
#[cfg(test)]
impl CLI {
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

                if let PositionalParameterKind::Multiple = parameter.kind {
                    multiple = true;
                }
            }

            let mut parameter_names = std::collections::HashSet::<&str>::new();

            for parameter in endpoint.named_parameters {
                if parameter.name.is_empty() || parameter.name.chars().any(char::is_whitespace) {
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

#[cfg(test)]
mod tests {
    use super::{
        Argument, ArgumentError, CLI, CLIDefinitionError, Endpoint, NamedParameter,
        PositionalParameter, RunError,
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
        assert_eq!(
            arguments,
            vec![Argument {
                name: "release",
                value: None
            }]
        );
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
        assert!(arguments.is_empty());
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
        let arguments: Vec<String> = vec!["check".into(), "--description".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let result = commands.run_args(arguments);
        let Err(RunError::EndpointHelp(endpoint)) = result else {
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
            matches!(result, Err(RunError::MissingCommand)),
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
            Err(RunError::UnknownCommand(unknown)) if unknown.0 == "quack"
        );
        assert!(is_expected, "Expected not first command");
    }

    #[test]
    fn invalid_flag() {
        // FUTURE allow Item=&str
        let arguments: Vec<String> = vec!["check".into(), "release".into()];
        let commands = CLI::new_just_endpoints(ENDPOINTS);
        let result = commands.run_args(arguments);
        let is_expected = matches!(
            result,
            Err(RunError::InvalidArguments(ArgumentError::ExpectedDashDash(ref unknown))) if unknown == "release"
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
