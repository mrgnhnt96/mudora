use std::env;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::pytext::{strip, universal_newlines};

const TIMEOUT: Duration = Duration::from_secs(900);
const EXIT_POLL: Duration = Duration::from_millis(20);
const STDERR_LIMIT: usize = 500;

const LEAN_ARGS: &[&str] = &[
    "-p",
    "--setting-sources",
    "",
    "--strict-mcp-config",
    "--disable-slash-commands",
    "--no-session-persistence",
    "--output-format",
    "text",
    "--tools",
    "",
];

fn program() -> String {
    env::var("MUDORA_CLAUDE")
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "claude".into())
}

fn read_in_background(mut stream: impl Read + Send + 'static) -> Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stream.read_to_end(&mut bytes);
        let _ = sender.send(bytes);
    });
    receiver
}

fn timed_out(child: &mut Child, timeout: Duration) -> String {
    let _ = child.kill();
    let _ = child.wait();
    format!("reviewer timed out after {} seconds", timeout.as_secs())
}

pub fn run(prompt: &str) -> Result<String, String> {
    run_program(&program(), prompt, TIMEOUT)
}

fn run_program(name: &str, prompt: &str, timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(name)
        .args(LEAN_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start {name}: {e}"))?;

    let deadline = Instant::now() + timeout;
    let mut stdin = child.stdin.take().ok_or("reviewer has no stdin")?;
    let input = prompt.to_owned();
    thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let stdout = read_in_background(child.stdout.take().ok_or("reviewer has no stdout")?);
    let stderr = read_in_background(child.stderr.take().ok_or("reviewer has no stderr")?);

    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if Instant::now() >= deadline {
            return Err(timed_out(&mut child, timeout));
        }
        thread::sleep(EXIT_POLL);
    };
    let remaining = || deadline.saturating_duration_since(Instant::now());
    let collect = |receiver: &Receiver<Vec<u8>>| {
        receiver
            .recv_timeout(remaining())
            .map_err(|_| format!("reviewer timed out after {} seconds", timeout.as_secs()))
    };
    let stdout = collect(&stdout)?;
    let stderr = collect(&stderr)?;

    if !status.success() {
        let text = universal_newlines(&String::from_utf8_lossy(&stderr));
        let start: String = strip(&text).chars().take(STDERR_LIMIT).collect();
        return Err(format!("reviewer failed: {start}"));
    }
    let text = String::from_utf8(stdout).map_err(|e| format!("reviewer output: {e}"))?;
    Ok(strip(&universal_newlines(&text)).to_owned())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use super::*;

    const LONG_ENOUGH: Duration = Duration::from_secs(30);

    fn script(dir: &TempDir, body: &str) -> PathBuf {
        let path = dir.path().join("fake-claude");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn run_script(path: &Path, prompt: &str, timeout: Duration) -> Result<String, String> {
        run_program(path.to_str().unwrap(), prompt, timeout)
    }

    #[test]
    fn nine_hundred_seconds_is_the_limit() {
        assert_eq!(TIMEOUT.as_secs(), 900);
    }

    #[test]
    fn output_is_stripped_and_newlines_are_normalised() {
        let dir = TempDir::new().unwrap();
        let fake = script(&dir, r"cat >/dev/null; printf '\n 1. a\r\n2. b \n\n'");
        assert_eq!(run_script(&fake, "x", LONG_ENOUGH).unwrap(), "1. a\n2. b");
    }

    #[test]
    fn prompt_larger_than_a_pipe_buffer_reaches_the_reviewer() {
        let dir = TempDir::new().unwrap();
        let fake = script(&dir, "wc -c | tr -d ' '");
        let prompt = "x".repeat(1 << 20);
        assert_eq!(run_script(&fake, &prompt, LONG_ENOUGH).unwrap(), "1048576");
    }

    #[test]
    fn failure_reports_the_first_500_characters_of_stderr() {
        let dir = TempDir::new().unwrap();
        let body = "cat >/dev/null; yes e | head -n 600 | tr -d '\\n' >&2; exit 3";
        let error = run_script(&script(&dir, body), "x", LONG_ENOUGH).unwrap_err();
        assert_eq!(error, format!("reviewer failed: {}", "e".repeat(500)));
    }

    #[test]
    fn a_reviewer_that_never_finishes_is_killed_at_the_limit() {
        let dir = TempDir::new().unwrap();
        let fake = script(&dir, "sleep 5");
        let started = Instant::now();
        let error = run_script(&fake, "x", Duration::from_millis(200)).unwrap_err();
        assert!(error.contains("timed out"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn missing_program_is_a_clear_error() {
        let error = run_program("/no/such/claude", "x", LONG_ENOUGH).unwrap_err();
        assert!(
            error.starts_with("could not start /no/such/claude"),
            "{error}"
        );
    }
}
