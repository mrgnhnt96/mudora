//! Runs the Python scripts and the CLI on the same input and compares what they produce. Delete
//! this file along with the scripts; `golden.rs` keeps the recorded answers.

mod common;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use common::{FakeClaude, copy_dir, crate_dir, fixtures, mudora, succeed, text};
use serde_json::Value;
use tempfile::TempDir;

const FINDINGS: &str =
    "\n1. **Intro**: unclear.\n\n2. **Table**: wrong total.\nnot 3. a finding\n\n";
const FUZZ_PAGES: usize = 1000;
const DIFF_PAGE_PAIRS: usize = 200;

const HARNESS: &str = r#"
import json, sys
sys.path.insert(0, sys.argv[1])
from page_text import page_lines, text_diff
answers = []
for line in sys.stdin:
    paths = line.rstrip("\n").split("\t")
    try:
        answers.append(page_lines(paths[0]) if len(paths) == 1 else list(text_diff(*paths)))
    except Exception:
        answers.append(None)
print(json.dumps(answers))
"#;

const SOUP: &[&str] = &[
    "<p>",
    "</p>",
    "<div>",
    "</div>",
    "<li>",
    "<ul>",
    "</ul>",
    "<h2>",
    "</h2>",
    "<br>",
    "<br/>",
    "<summary>",
    "<b>",
    "</b>",
    "<span title='a tip'>",
    "<a aria-label=\"x &amp; y\">",
    "</a>",
    "<i data-tip=' t '>",
    "<i data-tooltip>",
    "<x title=a title=b>",
    "<svg>",
    "</svg>",
    "<style>",
    "</style>",
    "<script>",
    "</script>",
    "</SCRIPT >",
    "<template>",
    "</template>",
    "<noscript>",
    "</noscript>",
    "<script/>",
    "<svg/>",
    "<!-- c -->",
    "<!--",
    "-->",
    "-- >",
    "<!doctype html>",
    "<!DOCTYPE",
    "<?pi?>",
    "<?",
    "<!x>",
    "<![CDATA[y]]>",
    "<![if a]>",
    "<![endif]>",
    "<![",
    "<!",
    "<![nope]>",
    "</>",
    "</",
    "</ x>",
    "</1>",
    "<",
    ">",
    "/>",
    "/",
    "=",
    "'",
    "\"",
    "&",
    ";",
    "&amp",
    "&amp;",
    "&lt;",
    "&#65;",
    "&#x41;",
    "&#128;",
    "&#0;",
    "&notit;",
    "&bogus;",
    "&#xD800;",
    "&#",
    "text",
    "two words",
    " ",
    "  ",
    "\n",
    "\r\n",
    "\r",
    "\t",
    "\u{a0}",
    "\u{3000}",
    "\u{85}",
    "\x1c",
    "\x1f",
    "\x0c",
    "\0",
    "é",
    "😀",
    "<a b",
    "<a b=",
    "<a b='",
    "<a b=\"c",
    "<a\tb=c/>",
    "<a/b/c>",
    "<a b=c d>",
    "<A HREF=X>",
    "<a =b>",
    "<a 'b'>",
    "<a,b>",
    "<p/",
    "<p\n",
    "<é>",
    "<a é=1>",
];

const LINES: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
];

struct Random(u64);

impl Random {
    fn below(&mut self, limit: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % limit as u64) as usize
    }
}

fn scripts() -> PathBuf {
    crate_dir().join("../mudora")
}

fn python() -> Command {
    let mut command = Command::new("python3");
    command.env("PYTHONUTF8", "1");
    command
}

fn run_script(dir: &Path, script: &str, args: &[&str]) -> Output {
    python()
        .arg(dir.join(script))
        .args(args)
        .output()
        .expect("these tests need python3 on the PATH")
}

fn pages() -> Vec<PathBuf> {
    let mut pages = Vec::new();
    for dir in [scripts().join("starters"), fixtures().join("pages")] {
        let found = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path());
        pages.extend(found.filter(|path| path.extension().is_some_and(|ext| ext == "html")));
    }
    pages.sort();
    assert!(pages.len() >= 5);
    pages
}

fn path_text(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn page_text_matches_on_every_page() {
    for page in pages() {
        let page = path_text(&page);
        let expected = succeed(run_script(&scripts(), "page_text.py", &[page]));
        let actual = succeed(mudora().args(["page-text", page]).output().unwrap());
        assert_eq!(actual, expected, "{page}");
    }
}

#[test]
fn text_diff_matches_between_every_pair_of_pages() {
    let pages = pages();
    for old in &pages {
        for new in &pages {
            let pair = [path_text(old), path_text(new)];
            let expected = succeed(run_script(&scripts(), "page_text.py", &pair));
            let output = mudora().arg("page-text").args(pair).output().unwrap();
            assert_eq!(succeed(output), expected, "{pair:?}");
        }
    }
}

fn answers_from_python(requests: &str) -> Vec<Value> {
    let mut child = python()
        .args(["-c", HARNESS, path_text(&scripts())])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("these tests need python3 on the PATH");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(requests.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    match serde_json::from_slice(&output.stdout).unwrap() {
        Value::Array(answers) => answers,
        other => panic!("{other}"),
    }
}

fn joined(answer: &Value) -> String {
    let lines = answer.as_array().unwrap().iter();
    let lines: Vec<&str> = lines.map(|line| line.as_str().unwrap()).collect();
    format!("{}\n", lines.join("\n"))
}

fn assert_same_answers(requests: &[Vec<PathBuf>]) {
    let lines: Vec<String> = requests
        .iter()
        .map(|paths| {
            paths
                .iter()
                .map(|path| path_text(path))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect();
    let answers = answers_from_python(&format!("{}\n", lines.join("\n")));
    assert_eq!(answers.len(), requests.len());
    for (paths, answer) in requests.iter().zip(answers) {
        let output = mudora().arg("page-text").args(paths).output().unwrap();
        let page = fs::read(&paths[0]).unwrap();
        if answer.is_null() {
            assert!(
                !output.status.success(),
                "Python failed on {:?}",
                text(&page)
            );
        } else {
            let stderr = text(&output.stderr);
            assert!(output.status.success(), "{stderr} on {:?}", text(&page));
            assert_eq!(
                text(&output.stdout),
                joined(&answer),
                "on {:?}",
                text(&page)
            );
        }
    }
}

/// Later Pythons changed how `html.parser` treats broken markup, and the port follows 3.9, the
/// Python these scripts run on. Well-formed pages read the same on all of them.
fn python_parses_like_the_port() -> bool {
    let check = "import sys; sys.exit(sys.version_info[:2] != (3, 9))";
    let status = python().args(["-c", check]).status();
    status
        .expect("these tests need python3 on the PATH")
        .success()
}

#[test]
fn tag_soup_reads_the_same() {
    if !python_parses_like_the_port() {
        eprintln!("skipped: tag soup is only compared against Python 3.9");
        return;
    }
    let dir = TempDir::new().unwrap();
    let mut random = Random(0x9e37_79b9_7f4a_7c15);
    let mut requests = Vec::new();
    for at in 0..FUZZ_PAGES {
        let pieces = 1 + random.below(30);
        let page: String = (0..pieces)
            .map(|_| SOUP[random.below(SOUP.len())])
            .collect();
        let path = dir.path().join(format!("{at}.html"));
        fs::write(&path, page).unwrap();
        requests.push(vec![path]);
    }
    assert_same_answers(&requests);
}

fn lines_page(random: &mut Random, lines: usize) -> String {
    (0..lines)
        .map(|_| format!("<p>{}</p>\n", LINES[random.below(LINES.len())]))
        .collect()
}

fn edited(random: &mut Random, page: &str) -> String {
    let mut lines: Vec<String> = page.lines().map(String::from).collect();
    for _ in 0..1 + random.below(12) {
        let at = random.below(lines.len());
        match random.below(3) {
            0 => drop(lines.remove(at)),
            1 => lines.insert(at, format!("<p>new {}</p>", random.below(9))),
            _ => lines[at] = format!("<p>changed {}</p>", random.below(9)),
        }
        if lines.is_empty() {
            lines.push("<p>only</p>".into());
        }
    }
    lines.join("\n")
}

#[test]
fn random_edits_diff_the_same_including_past_the_popular_line_cutoff() {
    let dir = TempDir::new().unwrap();
    let mut random = Random(0x2545_f491_4f6c_dd1d);
    let mut requests = Vec::new();
    for at in 0..DIFF_PAGE_PAIRS {
        let line_count = if at % 4 == 0 {
            200 + random.below(300)
        } else {
            1 + random.below(60)
        };
        let old = lines_page(&mut random, line_count);
        let new = edited(&mut random, &old);
        let (old_path, new_path) = (
            dir.path().join(format!("{at}-old.html")),
            dir.path().join(format!("{at}-new.html")),
        );
        fs::write(&old_path, old).unwrap();
        fs::write(&new_path, new).unwrap();
        requests.push(vec![old_path, new_path]);
    }
    assert_same_answers(&requests);
}

/// The Python script logs next to itself, so each side reviews from its own copy of the skill.
struct Sides {
    dir: TempDir,
    python_skill: PathBuf,
    rust_skill: PathBuf,
    python_fake: FakeClaude,
    rust_fake: FakeClaude,
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    out_file: Option<String>,
    log: String,
    prompt: Option<String>,
}

impl Sides {
    fn new(findings: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let python_skill = dir.path().join("python-skill");
        let rust_skill = dir.path().join("rust-skill");
        copy_dir(&scripts(), &python_skill);
        copy_dir(&scripts(), &rust_skill);
        let python_fake = FakeClaude::new(dir.path().join("python-fake"), findings);
        let rust_fake = FakeClaude::new(dir.path().join("rust-fake"), findings);
        Self {
            python_skill: python_skill.canonicalize().unwrap(),
            rust_skill: rust_skill.canonicalize().unwrap(),
            dir,
            python_fake,
            rust_fake,
        }
    }

    fn collect(&self, output: Output, skill: &Path, fake: &FakeClaude, out: &Path) -> Run {
        let neutral = |text: String| {
            let out = path_text(out);
            text.replace(path_text(skill), "SKILL").replace(out, "OUT")
        };
        Run {
            code: output.status.code(),
            stdout: neutral(text(&output.stdout)),
            stderr: neutral(text(&output.stderr)),
            out_file: fs::read_to_string(out).ok(),
            log: fs::read_to_string(skill.join("review-log.md")).unwrap(),
            prompt: fake.was_called().then(|| fake.prompt()),
        }
    }

    fn run_python(&self, args: &[&str]) -> Run {
        let out = self.dir.path().join("python-findings.md");
        let path = format!(
            "{}:{}",
            path_text(&self.python_fake.dir),
            std::env::var("PATH").unwrap()
        );
        let output = python()
            .arg(self.python_skill.join("review_page.py"))
            .args(args)
            .args(["--out", path_text(&out)])
            .env("PATH", path)
            .env("FAKE_DIR", &self.python_fake.dir)
            .output()
            .unwrap();
        self.collect(output, &self.python_skill, &self.python_fake, &out)
    }

    fn run_rust(&self, args: &[&str]) -> Run {
        let out = self.dir.path().join("rust-findings.md");
        let output = mudora()
            .arg("review")
            .args(args)
            .args([
                "--out",
                path_text(&out),
                "--skill-dir",
                path_text(&self.rust_skill),
            ])
            .env("MUDORA_CLAUDE", self.rust_fake.program())
            .env("FAKE_DIR", &self.rust_fake.dir)
            .output()
            .unwrap();
        self.collect(output, &self.rust_skill, &self.rust_fake, &out)
    }

    fn assert_same(&self, args: &[&str]) -> Run {
        let (expected, actual) = (self.run_python(args), self.run_rust(args));
        assert_eq!(actual.prompt, expected.prompt, "prompt for {args:?}");
        assert_eq!(actual.out_file, expected.out_file, "findings for {args:?}");
        assert_eq!(actual.log, expected.log, "log for {args:?}");
        assert_eq!(actual.stdout, expected.stdout, "stdout for {args:?}");
        assert_eq!(actual.code, expected.code, "exit code for {args:?}");
        actual
    }
}

fn starter(name: &str) -> String {
    path_text(&scripts().join("starters").join(name)).to_owned()
}

#[test]
fn full_review_matches_on_every_page() {
    for page in pages() {
        let sides = Sides::new(FINDINGS);
        let run = sides.assert_same(&[path_text(&page), "--subject", "the sync work"]);
        assert!(
            run.stdout
                .starts_with("2 findings in OUT, logged in SKILL/review-log.md")
        );
    }
}

#[test]
fn every_type_fills_the_same_checklist() {
    let page = starter("research.html");
    for kind in [
        "research",
        "proposal",
        "results",
        "check",
        "data-report",
        "evidence",
        "design-doc",
        "plan",
        "presentation",
        "deck",
        "explainer",
        "whats-new",
        "guide",
        "review-tool",
    ] {
        let sides = Sides::new(FINDINGS);
        let run = sides.assert_same(&[&page, "--subject", "s", "--type", kind]);
        assert!(run.prompt.unwrap().contains("<type_checklist>"), "{kind}");
    }
}

#[test]
fn every_option_reaches_the_prompt_and_log_the_same() {
    let sides = Sides::new(FINDINGS);
    let extra = sides.dir.path().join("extra.txt");
    fs::write(&extra, "Shown by script\n{not a field}\n").unwrap();
    let (page, baseline) = (starter("results.html"), starter("evidence.html"));
    sides.assert_same(&[
        &page,
        "--subject",
        "{braces} · é",
        "--readers",
        "QA",
        "--kind",
        "note",
        "--extra",
        path_text(&extra),
        "--baseline",
        &baseline,
        "--request",
        "Shorten \"the intro\"",
        "--link",
        "https://claude.ai/artifact/abc",
    ]);
}

#[test]
fn diff_review_without_a_request_says_so_the_same_way() {
    let sides = Sides::new(FINDINGS);
    let (page, baseline) = (starter("results.html"), starter("research.html"));
    let run = sides.assert_same(&[&page, "--subject", "s", "--baseline", &baseline]);
    assert!(run.prompt.unwrap().contains("(not given)"));
}

#[test]
fn no_findings_counts_zero() {
    let sides = Sides::new("No findings. Looks fine.\n1. stray\n");
    let run = sides.assert_same(&[&starter("results.html"), "--subject", "s"]);
    assert!(run.stdout.starts_with("0 findings"));
}

#[test]
fn unchanged_page_skips_the_reviewer_the_same_way() {
    let sides = Sides::new(FINDINGS);
    let page = starter("results.html");
    let run = sides.assert_same(&[&page, "--subject", "s", "--baseline", &page]);
    assert_eq!(
        run.stdout,
        "No text changes since the last review; nothing to review.\n"
    );
    assert!(run.prompt.is_none() && run.out_file.is_none());
}

#[test]
fn unknown_type_fails_the_same_way() {
    let sides = Sides::new(FINDINGS);
    let args = [starter("results.html")];
    let args = [args[0].as_str(), "--subject", "s", "--type", "nope"];
    let (expected, actual) = (sides.run_python(&args), sides.run_rust(&args));
    assert_eq!(actual.stderr, expected.stderr);
    assert_eq!(actual.code, expected.code);
    assert!(actual.prompt.is_none() && actual.out_file.is_none());
}

#[test]
fn failing_reviewer_fails_the_same_way() {
    let sides = Sides::new(FINDINGS);
    sides.python_fake.fail_with(7);
    sides.rust_fake.fail_with(7);
    let args = [starter("results.html")];
    let args = [args[0].as_str(), "--subject", "s"];
    let (expected, actual) = (sides.run_python(&args), sides.run_rust(&args));
    assert_eq!(actual.stderr, expected.stderr);
    assert_eq!(actual.code, expected.code);
    assert_eq!(actual.log, expected.log);
    assert_eq!(actual.out_file, None);
}

fn session_lookup(home: &Path, link: &str) -> [(Option<i32>, String, String); 2] {
    let python = python()
        .arg(scripts().join("find_doc_session.py"))
        .arg(link)
        .env("HOME", home)
        .output()
        .unwrap();
    let rust = mudora()
        .args(["find-session", link])
        .env("HOME", home)
        .output()
        .unwrap();
    [python, rust].map(|output| {
        let (out, err) = (text(&output.stdout), text(&output.stderr));
        (output.status.code(), out, err)
    })
}

#[test]
fn session_lookup_matches_on_every_recorded_publish() {
    let home = TempDir::new().unwrap();
    copy_dir(
        &fixtures().join("projects"),
        &home.path().join(".claude/projects"),
    );
    for link in [
        "https://claude.ai/artifact/aaa111",
        "https://claude.ai/artifact/bbb222?x=1",
        "ccc333-uuid",
        "ddd444",
        "eee555",
        "https://claude.ai/artifact/errored",
        "https://claude.ai/artifact/assetonly",
        "https://claude.ai/artifact/nothing",
    ] {
        let [expected, actual] = session_lookup(home.path(), link);
        assert_eq!(actual, expected, "{link}");
    }
}

#[test]
fn session_lookup_without_any_transcripts_fails_the_same_way() {
    let home = TempDir::new().unwrap();
    let [expected, actual] = session_lookup(home.path(), "abc");
    assert_eq!(actual, expected);
}
