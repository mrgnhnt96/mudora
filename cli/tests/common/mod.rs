use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FAKE_CLAUDE: &str = r#"#!/bin/sh
printf '%s\n' "$@" > "$FAKE_DIR/args"
cat > "$FAKE_DIR/prompt"
cat "$FAKE_DIR/findings"
exit "$(cat "$FAKE_DIR/exit-code" 2>/dev/null || echo 0)"
"#;

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixtures() -> PathBuf {
    crate_dir().join("tests/fixtures")
}

pub fn mudora() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mudora"));
    command.env_remove("MUDORA_CLAUDE");
    command
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn succeed(output: Output) -> String {
    assert!(output.status.success(), "{}", text(&output.stderr));
    text(&output.stdout)
}

pub fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A folder holding a program called `claude` that records its arguments and prompt there and
/// answers with whatever `findings` says.
pub struct FakeClaude {
    pub dir: PathBuf,
}

impl FakeClaude {
    pub fn new(dir: PathBuf, findings: &str) -> Self {
        fs::create_dir_all(&dir).unwrap();
        let program = dir.join("claude");
        fs::write(&program, FAKE_CLAUDE).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(dir.join("findings"), findings).unwrap();
        Self { dir }
    }

    pub fn program(&self) -> PathBuf {
        self.dir.join("claude")
    }

    pub fn fail_with(&self, code: u8) {
        fs::write(self.dir.join("exit-code"), code.to_string()).unwrap();
    }

    pub fn was_called(&self) -> bool {
        self.dir.join("prompt").exists()
    }

    pub fn prompt(&self) -> String {
        fs::read_to_string(self.dir.join("prompt")).unwrap()
    }

    #[allow(dead_code)]
    pub fn args(&self) -> Vec<String> {
        let args = fs::read_to_string(self.dir.join("args")).unwrap();
        args.lines().map(String::from).collect()
    }
}
