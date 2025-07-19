fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let directory = args
        .next()
        .expect("directory to scan for empty files and folders");
    check_items(std::path::Path::new(&directory))
}

fn check_items(path: &std::path::Path) -> std::io::Result<()> {
    if path.is_dir() {
        let mut entries = 0;
        for entry in std::fs::read_dir(path)? {
            entries += 1;
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                check_items(&path)?;
            } else {
                let item = std::fs::read(&path)?;
                if item.trim_ascii_start().len() == 0 {
                    eprintln!("{path} is empty", path = path.display());
                }
            }
        }
        if entries == 0 {
            eprintln!("{path} is empty", path = path.display());
        }
    }
    Ok(())
}
