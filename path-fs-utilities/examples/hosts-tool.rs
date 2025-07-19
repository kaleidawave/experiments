#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::fs::{self, File};
    use std::io::Write;

    let hosts_path = "C:\\Windows\\System32\\drivers\\etc\\hosts";

    let mode = std::env::args()
        .nth(1)
        .expect("expected 'pause' or 'unpause'");
    let content = fs::read_to_string(hosts_path)?;

    let mut file = File::create(hosts_path)?;

    let mut sites = false;
    for mut line in content.lines() {
        if !sites {
            sites = line.starts_with("# sites");
        }

        if sites {
            if mode == "pause" {
                if line.starts_with("127.0.0.1") {
                    write!(&mut file, "# ")?;
                }
            } else if mode == "unpause" {
                if line.starts_with("# 127.0.0.1") {
                    line = &line[2..];
                }
            }
        }
        writeln!(&mut file, "{line}")?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    panic!("cfg is only supported on windows")
}
