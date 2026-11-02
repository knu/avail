use anyhow::{Context, Result, bail};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub fn run(program: &str, args: &[&str], seconds: u64) -> Result<Output> {
    // Files avoid filling a pipe while waiting for a potentially large search result.
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?)
        .spawn()
        .with_context(|| format!("failed to run {program}"))?;
    let status = match child.wait_timeout(Duration::from_secs(seconds))? {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            bail!("{program} search timed out after {seconds}s");
        }
    };
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut output = Output {
        status,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    stdout.read_to_end(&mut output.stdout)?;
    stderr.read_to_end(&mut output.stderr)?;
    Ok(output)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn timeout_stops_a_command() {
        let result = run("/bin/sh", &["-c", "exec sleep 30"], 0);
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }
}
