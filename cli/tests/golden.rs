//! Compares the CLI with answers the Python scripts recorded in `tests/fixtures/expected`, so the
//! comparison outlives the scripts.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};

use common::{FakeClaude, copy_dir, fixtures, mudora, succeed, text};
use tempfile::TempDir;

const FINDINGS: &str = "\n1. **Intro**: unclear.\n\n2. **Table**: wrong total.\n\n";
const LEAN_LAUNCH: &[&str] = &[
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

fn expected(name: &str) -> String {
    fs::read_to_string(fixtures().join("expected").join(name)).unwrap()
}

fn page(name: &str) -> String {
    fixtures()
        .join("pages")
        .join(name)
        .to_str()
        .unwrap()
        .to_owned()
}

fn page_text(pages: &[String]) -> String {
    succeed(mudora().arg("page-text").args(pages).output().unwrap())
}

struct Review {
    dir: TempDir,
    skill: PathBuf,
    fake: FakeClaude,
}

impl Review {
    fn new(findings: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let skill = dir.path().join("skill");
        copy_dir(&fixtures().join("skill"), &skill);
        let fake = FakeClaude::new(dir.path().join("fake"), findings);
        Self {
            skill: skill.canonicalize().unwrap(),
            dir,
            fake,
        }
    }

    fn out(&self) -> PathBuf {
        self.dir.path().join("out.md")
    }

    fn run(&self, args: &[&str]) -> Output {
        mudora()
            .arg("review")
            .args(args)
            .arg("--out")
            .arg(self.out())
            .arg("--skill-dir")
            .arg(&self.skill)
            .env("MUDORA_CLAUDE", self.fake.program())
            .env("FAKE_DIR", &self.fake.dir)
            .output()
            .unwrap()
    }

    fn log(&self) -> String {
        let today = chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        fs::read_to_string(self.skill.join("review-log.md"))
            .unwrap()
            .replace(&today, "DATE")
    }

    fn assert_recorded(&self, name: &str) {
        assert_eq!(
            self.fake.prompt(),
            expected(&format!("{name}.prompt")),
            "prompt"
        );
        let findings = fs::read_to_string(self.out()).unwrap();
        assert_eq!(findings, expected(&format!("{name}.findings")), "findings");
        assert_eq!(self.log(), expected(&format!("{name}.log")), "log");
    }
}

#[test]
fn page_text_is_the_recorded_text() {
    assert_eq!(page_text(&[page("edges.html")]), expected("edges.txt"));
}

#[test]
fn text_diff_is_the_recorded_diff() {
    assert_eq!(
        page_text(&[page("old.html"), page("new.html")]),
        expected("old-new.diff")
    );
}

#[test]
fn full_review_is_the_recorded_review() {
    let review = Review::new(FINDINGS);
    let stdout = succeed(review.run(&[&page("new.html"), "--subject", "the sync work"]));
    review.assert_recorded("full");
    let log = review.skill.join("review-log.md");
    let line = format!(
        "2 findings in {}, logged in {}\n",
        review.out().display(),
        log.display()
    );
    assert_eq!(stdout, line);
}

#[test]
fn diff_review_is_the_recorded_review() {
    let review = Review::new(FINDINGS);
    succeed(review.run(&[
        &page("new.html"),
        "--subject",
        "the sync work",
        "--baseline",
        &page("old.html"),
        "--request",
        "Use the new number",
        "--link",
        "https://claude.ai/artifact/abc",
    ]));
    review.assert_recorded("diff");
}

#[test]
fn type_without_a_checklist_is_the_recorded_review() {
    let review = Review::new(FINDINGS);
    succeed(review.run(&[
        &page("edges.html"),
        "--subject",
        "edges",
        "--type",
        "plan",
        "--readers",
        "QA",
        "--kind",
        "note",
    ]));
    review.assert_recorded("no-checklist");
}

#[test]
fn type_from_the_page_is_the_recorded_review() {
    let review = Review::new(FINDINGS);
    succeed(review.run(&[&page("edges.html"), "--subject", "edges", "--type", ""]));
    review.assert_recorded("meta-type");
}

#[test]
fn page_with_no_type_is_the_recorded_review() {
    let review = Review::new(FINDINGS);
    succeed(review.run(&[&page("plain.html"), "--subject", "plain"]));
    review.assert_recorded("no-type");
}

#[test]
fn reviewer_starts_lean_with_no_tools() {
    let review = Review::new(FINDINGS);
    succeed(review.run(&[&page("plain.html"), "--subject", "plain"]));
    assert_eq!(review.fake.args(), LEAN_LAUNCH);
}

#[test]
fn skill_folder_defaults_to_the_installed_one() {
    let home = TempDir::new().unwrap();
    let skill = home.path().join(".claude/skills/mudora");
    copy_dir(&fixtures().join("skill"), &skill);
    let fake = FakeClaude::new(home.path().join("fake"), FINDINGS);
    let out = home.path().join("out.md");
    let output = mudora()
        .args(["review", &page("plain.html"), "--subject", "plain", "--out"])
        .arg(&out)
        .env("HOME", home.path())
        .env("MUDORA_CLAUDE", fake.program())
        .env("FAKE_DIR", &fake.dir)
        .output()
        .unwrap();
    succeed(output);
    assert_eq!(fake.prompt(), expected("no-type.prompt"));
    assert!(
        fs::read_to_string(skill.join("review-log.md"))
            .unwrap()
            .contains("Page: plain.html")
    );
}

#[test]
fn unchanged_page_never_starts_the_reviewer() {
    let review = Review::new(FINDINGS);
    let page = page("new.html");
    let stdout = succeed(review.run(&[&page, "--subject", "s", "--baseline", &page]));
    assert_eq!(
        stdout,
        "No text changes since the last review; nothing to review.\n"
    );
    assert!(!review.fake.was_called());
    assert!(!review.out().exists());
    assert_eq!(review.log(), "# Review log\n");
}

#[test]
fn failing_reviewer_leaves_no_findings_or_log_entry() {
    let review = Review::new(FINDINGS);
    review.fake.fail_with(7);
    let output = review.run(&[&page("plain.html"), "--subject", "s"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!review.out().exists());
    assert_eq!(review.log(), "# Review log\n");
}

#[test]
fn unknown_type_names_the_missing_file() {
    let review = Review::new(FINDINGS);
    let output = review.run(&[&page("plain.html"), "--subject", "s", "--type", "nope"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with("unknown page type 'nope': no "),
        "{stderr}"
    );
    assert!(!review.fake.was_called());
}

#[test]
fn missing_skill_folder_is_a_clear_error() {
    let review = Review::new(FINDINGS);
    let output = mudora()
        .args(["review", &page("plain.html"), "--subject", "s", "--out"])
        .arg(review.out())
        .args(["--skill-dir", "/no/such/skill"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).starts_with("no skill folder at /no/such/skill"));
}

#[test]
fn large_page_reads_in_about_linear_time() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("large.html");
    fs::write(&path, "<p class=\"a\">some &amp; text</p>\n".repeat(50_000)).unwrap();
    let started = Instant::now();
    let path = path.to_str().unwrap().to_owned();
    assert_eq!(page_text(&[path]).lines().count(), 50_000);
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "{:?}",
        started.elapsed()
    );
}

fn session(home: &Path, link: &str) -> String {
    succeed(
        mudora()
            .args(["find-session", link])
            .env("HOME", home)
            .output()
            .unwrap(),
    )
}

#[test]
fn session_lookup_is_the_recorded_answer() {
    let home = TempDir::new().unwrap();
    copy_dir(
        &fixtures().join("projects"),
        &home.path().join(".claude/projects"),
    );
    for (name, link) in [
        ("older-and-newer", "https://claude.ai/artifact/aaa111"),
        ("uploaded", "bbb222"),
        ("mcp-version", "ccc333-uuid"),
        ("mcp-files", "ddd444"),
        ("mcp-edit", "eee555"),
    ] {
        let recorded = expected(&format!("session-{name}.json"));
        assert_eq!(session(home.path(), link), recorded, "{link}");
    }
}
