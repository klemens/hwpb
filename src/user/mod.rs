use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use std::thread::sleep;

// Ensure the child process exits (prevent zombies)
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.0.kill().ok();
        self.0.wait().ok();
    }
}

/// Authenticate user by trying to login using su
#[cfg(unix)]
pub fn authenticate(username: &str, password: &str) -> Result<bool, ()> {
    // Check if the user has supplied the correct password
    let mut su = ChildGuard(Command::new("/bin/su")
        .args(&["--command", "false"]) // for restricted shells that executes commands
        .args(&["--shell", "/bin/true"])
        .arg("--") // further arguments are not parsed as su options
        .arg(username)
        .env_clear()
        .current_dir("/")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ())?
    );

    {
        let stdin = su.0.stdin.as_mut().ok_or(())?;
        stdin.write_all(password.as_bytes()).map_err(|_| ())?;
        stdin.write_all("\n".as_bytes()).map_err(|_| ())?;
    }

    // Wait at most 1 sec for the correct password
    let now = Instant::now();
    let max_duration = Duration::from_millis(1000);
    loop {
        // Check for the successful exit code every 50 ms
        sleep(Duration::from_millis(50));
        match su.0.try_wait() {
            Ok(Some(status)) => return Ok(status.success()),
            Ok(None) => {} // Wait a little longer
            Err(_) => return Err(()),
        }

        if now.elapsed() > max_duration {
            return Err(());
        }
    }
}
