use std::{env, fs, path::Path};

pub fn get_environment_variable(name: &str) -> Option<String> {
    env::vars().find_map(|(n, v)| (n == name).then_some(v))
}

/// `remove_after = true => move`, `remove_after = false => copy`
/// TODO if `remove_after`, in some cases can [rename](https://doc.rust-lang.org/std/fs/fn.rename.html) sometimes here
pub fn move_copy_file(
    from: &Path,
    to: &Path,
    remove_after: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if from.is_dir() {
        todo!("copy/move directory");
    } else if from.is_file() {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = fs::read(from)?;
        fs::write(to, content)?;

        let metadata = fs::metadata(from)?;
        let permissions = metadata.permissions();
        fs::set_permissions(to, permissions)?;

        if remove_after {
            fs::remove_file(from)?;
        }

        Ok(())
    } else {
        Err("Unknown path item to move".into())
    }
}

/// Reverse <https://howtospell.co.uk/y-to-ies-or-s-plural-rule>
pub fn depluralise(on: &str) -> Option<&str> {
    on.strip_suffix("ies").or_else(|| on.strip_suffix("s"))
}

pub fn run_command(
    command: String,
    args: Vec<String>,
    env: Option<Vec<(String, String)>>,
    capture_stdout: bool,
    capture_stderr: bool,
) -> (String, std::process::ExitStatus) {
    use std::io::{Read, pipe};
    use std::process::{Command, Stdio};

    let env = env.unwrap_or_default();

    let (stdout, stderr, reader): (Stdio, Stdio, Option<_>) = match (capture_stdout, capture_stderr)
    {
        (true, true) => {
            // Using pipe we collect both stdout and stderr in order
            let (reader, writer) = pipe().expect("could not create pipe");
            (
                writer
                    .try_clone()
                    .expect("could not clone writer pipe")
                    .into(),
                writer.into(),
                Some(reader),
            )
        }
        (true, false) => {
            // Shouldn't need pipe here
            let (reader, writer) = pipe().expect("could not create pipe");
            (writer.into(), Stdio::inherit(), Some(reader))
        }
        (false, true) => {
            // Shouldn't need pipe here
            let (reader, writer) = pipe().expect("could not create pipe");
            (Stdio::null(), writer.into(), Some(reader))
        }
        (false, false) => (Stdio::inherit(), Stdio::inherit(), None),
    };

    let mut child = Command::new(command)
        .args(args)
        .stdout(stdout)
        .stderr(stderr)
        .envs(env)
        .spawn()
        .expect("Failed to spawn command");

    if let Some(mut reader) = reader {
        let mut output = String::new();
        reader.read_to_string(&mut output).expect("invalid UTF8");
        let result = child.wait().expect("command not finished");
        // Remove whitespace from end
        output.truncate(output.trim_end().len());
        (output, result)
    } else {
        let result = child.wait().expect("command not finished");
        (String::default(), result)
    }
}

pub fn separate_numbers(whole_part: &str) -> String {
    let n: Vec<char> = whole_part.chars().collect();
    let mut s = String::new();
    for part in n.rchunks(3).rev() {
        if !s.is_empty() {
            s.push(' ');
        }
        let chunk: String = part.iter().copied().collect();
        s.push_str(&chunk);
    }
    s
}

pub fn separate_numbers_fract(fract_part: &str) -> String {
    let n: Vec<char> = fract_part.chars().collect();
    let mut s = String::new();
    for part in n.chunks(3) {
        if !s.is_empty() {
            s.push(' ');
        }
        let chunk: String = part.iter().copied().collect();
        s.push_str(&chunk);
    }
    s
}
