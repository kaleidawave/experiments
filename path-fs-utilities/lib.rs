use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

pub fn visit_dir_recursive(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir_recursive(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}
