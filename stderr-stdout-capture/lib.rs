use std::io::{self, Read, Write};

pub fn capture_output<F>(f: F) -> io::Result<(String, String)>
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    #[cfg(unix)]
    {
        unix_capture(f)
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        compile_error!("This platform is not supported");
    }
}

#[cfg(unix)]
fn unix_capture<F>(f: F) -> io::Result<(String, String)>
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let (stdout_reader, stdout_writer) = create_pipe()?;
    let (stderr_reader, stderr_writer) = create_pipe()?;

    let original_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
    let original_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };
    
    if original_stdout == -1 || original_stderr == -1 {
        return Err(io::Error::last_os_error());
    }

    unsafe {
        libc::dup2(stdout_writer, libc::STDOUT_FILENO);
        libc::dup2(stderr_writer, libc::STDERR_FILENO);
        libc::close(stdout_writer);
        libc::close(stderr_writer);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    let _ = io::stdout().flush();
    let _ = io::stderr().flush();

    unsafe {
        libc::dup2(original_stdout, libc::STDOUT_FILENO);
        libc::dup2(original_stderr, libc::STDERR_FILENO);
        libc::close(original_stdout);
        libc::close(original_stderr);
    }

    let stdout_str = read_from_fd(stdout_reader)?;
    let stderr_str = read_from_fd(stderr_reader)?;

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }

    Ok((stdout_str, stderr_str))
}

#[cfg(unix)]
fn create_pipe() -> io::Result<(i32, i32)> {
    let mut fds = [0; 2];
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok((fds[0], fds[1]))
    }
}

#[cfg(unix)]
fn read_from_fd(fd: i32) -> io::Result<String> {
    use std::os::unix::io::FromRawFd;
    use std::fs::File;
    
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_writes() {
        let (stdout, stderr) = capture_output(|| {
            print!("Part 1 ");
            eprint!("Error 1 ");
            print!("Part 2");
            eprint!("Error 2");
        }).unwrap();
        
        assert_eq!(stdout, "Part 1 Part 2");
        assert_eq!(stderr, "Error 1 Error 2");
    }
}