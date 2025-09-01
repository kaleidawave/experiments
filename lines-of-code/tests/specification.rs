#[test]
fn main() -> std::process::ExitCode {
    let output = std::process::Command::new("spectra")
        .arg("check")
        .arg("README.md")
        .arg("./target/debug/lines-of-code --rpc --interactive")
        .status()
        .unwrap();

    if output.code().is_none_or(|item| item == 0) {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
