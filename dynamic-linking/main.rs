const OUT: &str = "private/example";
const NAME: &[u8] = b"get_result";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir("private")?;
    let _ = std::process::Command::new("rustc")
        .arg("example.rs")
        .arg("--crate-type")
        .arg("cdylib")
        .arg("-o")
        .arg(OUT)
        .output();
    let out = call_dynamic()?;
    dbg!(out);
    Ok(())
}

fn call_dynamic() -> Result<String, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new(OUT)?;
        let func: libloading::Symbol<unsafe extern "Rust" fn() -> String> = lib.get(NAME)?;
        Ok(func())
    }
}