use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;

type ProcessResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> ProcessResult {
    let mut args = env::args().skip(1);

    let first = args.next();
    let second = args.next();
    let third = args.next();

    if let Some("info" | "--help") | None = first.as_deref() {
        let name: &str = env!("CARGO_BIN_NAME");
        eprintln!("Usage: {name} <input.tar.gz|.tar|.zip> <output_dir>");
        if first.is_some() {
            return Ok(());
        } else {
            return Err("Expected two arguments".into());
        }
    }

    let first = first.unwrap();
    let second = second.unwrap();

    let input_path = Path::new(&first);
    let output_path = Path::new(&second);

    if input_path.exists() {
        let result = match utilities::file_suffix(input_path).and_then(|e| e.to_str()) {
            Some("zip") => extract_zip(input_path, output_path),
            Some("tar.gz") => extract_tar_gz(input_path, output_path),
            Some("tar") => extract_tar(input_path, output_path),
            out => {
                if let Some("--ignore-uncompressed") = third.as_deref() {
                    Ok(())
                } else {
                    let error = format!(
                        "Unsupported file type: {input_path} (suffix {out})",
                        input_path = input_path.display(),
                        out = out.unwrap_or_default()
                    );
                    return Err(error.into());
                }
            }
        };

        match result {
            Ok(()) => {
                let keep = matches!(third.as_deref(), Some("--keep"));
                if !keep {
                    std::fs::remove_file(input_path)?;
                }
                println!(
                    "Successfully extracted to {output_path}",
                    output_path = output_path.display()
                );
                Ok(())
            }
            Err(err) => {
                eprintln!("Error extracting archive: {err}");
                Err(err)
            }
        }
    } else {
        let error = format!(
            "Input file does not exist: {input_path}",
            input_path = input_path.display()
        );
        Err(error.into())
    }
}

fn extract_tar(path: &Path, output_dir: &Path) -> ProcessResult {
    let file = File::open(path)?;
    let mut archive = Archive::new(file);
    archive.unpack(output_dir)?;
    Ok(())
}

fn extract_tar_gz(path: &Path, output_dir: &Path) -> ProcessResult {
    let file = File::open(path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(output_dir)?;
    Ok(())
}

fn extract_zip(path: &Path, output_dir: &Path) -> ProcessResult {
    use std::{fs, io};

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = output_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // TODO is this needed?
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// modified source from https://doc.rust-lang.org/stable/src/std/path.rs.html#2721-2723
mod utilities {
    use std::ffi::OsStr;

    pub fn file_suffix(this: &std::path::Path) -> Option<&OsStr> {
        this.file_name()
            .map(split_file_at_dot)
            .and_then(|(_before, after)| after)
    }

    fn split_file_at_dot(file: &OsStr) -> (&OsStr, Option<&OsStr>) {
        if file.as_encoded_bytes() == b".." {
            return (file, None);
        }

        let slice = file.as_encoded_bytes();
        let mut ridx = None;
        for (idx, chr) in slice.iter().enumerate().rev() {
            if let b'.' = chr {
                ridx = Some(idx);
            } else if chr.is_ascii_whitespace() || matches!(chr, b'/' | b'\\' | b'-' | b'_') {
                break;
            }
        }
        if let Some(ridx) = ridx {
            let (left, right) = (&slice[..ridx], &slice[ridx + 1..]);

            // The unsafety here stems from converting between &OsStr and &[u8]
            // and back. This is safe to do because (1) we only look at ASCII
            // contents of the encoding and (2) new &OsStr values are produced
            // only from ASCII-bounded slices of existing &OsStr values.
            unsafe {
                (
                    OsStr::from_encoded_bytes_unchecked(left),
                    Some(OsStr::from_encoded_bytes_unchecked(right)),
                )
            }
        } else {
            (file, None)
        }
    }
}
