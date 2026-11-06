use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use clap::Args;
use regex::Regex;

use crate::page_text::{page_lines, text_diff};
use crate::pytext::{format_template, read_text, read_text_lossy, strip};
use crate::reviewer;

const FULL_SCOPE: &str = "This is the first version. Review the whole page.";

const DIFF_SCOPE: &str = r#"This is a revision. The user asked for: "{request}".
The changes since the last review are in <diff> (a unified diff of the visible text).
Review only the changed lines, and the places the change affects: a TL;DR row or intro that
now disagrees with a changed section, a removed section something still refers to, a term
the change introduces without explaining, a section that no longer flows into the next.
Use <page_text> only for that context. Don't report on unchanged parts of the page. Don't
question what the user asked for; check that it was done well and fits the rest."#;

const NEXT_SECTION: &str = "\n## ";

const TYPE_LABELS: &[(&str, &str)] = &[
    ("research", "research page"),
    ("proposal", "proposal"),
    ("results", "measured results report"),
    ("check", "short check"),
    ("data-report", "data report"),
    ("evidence", "before-and-after fix evidence page"),
    ("design-doc", "design doc"),
    ("plan", "plan"),
    ("presentation", "presentation"),
    ("deck", "slide deck"),
    ("explainer", "explainer"),
    ("whats-new", "what's-new page"),
    ("guide", "how-to guide"),
    ("review-tool", "review tool"),
];

const TYPE_FILES: &[(&str, &str)] = &[
    ("proposal", "research"),
    ("check", "results"),
    ("data-report", "results"),
];

static TYPE_META: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r#"<meta[\s\x1c-\x1f]+name=["']mudora-type["'][\s\x1c-\x1f]+"#,
        r#"content=["']([\w-]+)["']"#,
    ))
    .unwrap()
});
static FINDING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\d+\.").unwrap());

#[derive(Args)]
#[command(infer_long_args = true, args_override_self = true)]
pub struct ReviewArgs {
    page: String,
    /// Last reviewed version; reviews only the diff
    #[arg(long)]
    baseline: Option<String>,
    /// The user's change request, verbatim
    #[arg(long, allow_hyphen_values = true)]
    request: Option<String>,
    #[arg(long, default_value = "the team", allow_hyphen_values = true)]
    readers: String,
    /// Page type (types/<type>.md); read from the page's mudora-type meta tag if left out
    #[arg(long = "type")]
    page_type: Option<String>,
    /// How to describe the page to the reviewer; defaults to the type's name
    #[arg(long, allow_hyphen_values = true)]
    kind: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    subject: String,
    /// File with text the page's script renders
    #[arg(long)]
    extra: Option<String>,
    /// Where to write the findings
    #[arg(long)]
    out: String,
    /// The page's claude.ai link, for the review log
    #[arg(long)]
    link: Option<String>,
    /// The installed skill folder: templates and the review log
    #[arg(long)]
    skill_dir: Option<PathBuf>,
}

fn given(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|value| !value.is_empty())
}

fn lookup(table: &[(&str, &'static str)], key: &str) -> Option<&'static str> {
    table
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| *value)
}

fn skill_dir(given: Option<&Path>) -> Result<PathBuf, String> {
    let dir = match given {
        Some(dir) => dir.to_path_buf(),
        None => env::home_dir()
            .ok_or("no home folder")?
            .join(".claude/skills/mudora"),
    };
    dir.canonicalize()
        .map_err(|e| format!("no skill folder at {}: {e}", dir.display()))
}

fn section_after<'a>(text: &'a str, header: &str) -> Option<(&'a str, Option<usize>)> {
    let start = text.find(header)? + header.len();
    let body = &text[start..];
    Some((body, body.find(NEXT_SECTION)))
}

fn page_rules(skill: &Path) -> Result<String, String> {
    let text = read_text(&skill.join("SKILL.md"))?;
    Ok(match section_after(&text, "## Rules for every page\n") {
        Some((body, Some(end))) => strip(&body[..end]).to_owned(),
        _ => String::new(),
    })
}

fn page_type(page: &Path, given_type: Option<&str>) -> Result<Option<String>, String> {
    if let Some(kind) = given_type {
        return Ok(Some(kind.to_owned()));
    }
    let html = read_text_lossy(page)?;
    Ok(TYPE_META.captures(&html).map(|found| found[1].to_owned()))
}

fn type_checklist(skill: &Path, kind: Option<&str>) -> Result<String, String> {
    let Some(kind) = kind else {
        return Ok(String::new());
    };
    let file = lookup(TYPE_FILES, kind).unwrap_or(kind);
    let path = skill.join("types").join(format!("{file}.md"));
    if !path.exists() {
        return Err(format!("unknown page type '{kind}': no {}", path.display()));
    }
    let text = read_text(&path)?;
    let Some((body, end)) = section_after(&text, "## Checklist\n") else {
        return Ok(String::new());
    };
    let list = strip(&body[..end.unwrap_or(body.len())]);
    let label = lookup(TYPE_LABELS, kind).unwrap_or(kind);
    Ok(format!(
        "\n<type_checklist>\nThis page is a {label}. Questions specific to this type of page; \
         report a failed one like any other finding:\n{list}\n</type_checklist>\n"
    ))
}

fn build_prompt(args: &ReviewArgs, skill: &Path) -> Result<(String, Option<String>), String> {
    let page = Path::new(&args.page);
    let mut page_text = page_lines(page)?.join("\n");
    if let Some(extra) = given(&args.extra) {
        page_text.push_str("\n\nText the page's script shows:\n");
        page_text.push_str(&read_text(Path::new(extra))?);
    }
    let (diff, scope, diff_block) = match given(&args.baseline) {
        Some(baseline) => {
            let diff = text_diff(Path::new(baseline), page)?.join("\n");
            let request = given(&args.request).unwrap_or("(not given)");
            let block = format!("\n<diff>\n{diff}\n</diff>\n");
            (Some(diff), DIFF_SCOPE.replace("{request}", request), block)
        }
        None => (None, FULL_SCOPE.to_owned(), String::new()),
    };
    let kind = page_type(page, given(&args.page_type))?;
    let kind_label = given(&args.kind).unwrap_or_else(|| {
        kind.as_deref()
            .and_then(|kind| lookup(TYPE_LABELS, kind))
            .unwrap_or("report")
    });
    let template = read_text(&skill.join("reviewer-prompt.md"))?;
    let rules = page_rules(skill)?;
    let checklist = type_checklist(skill, kind.as_deref())?;
    let prompt = format_template(
        &template,
        &[
            ("readers", &args.readers),
            ("kind", kind_label),
            ("subject", &args.subject),
            ("scope", &scope),
            ("page_text", &page_text),
            ("diff_block", &diff_block),
            ("rules", &rules),
            ("checklist", &checklist),
        ],
    )?;
    Ok((prompt, diff))
}

fn log_review(args: &ReviewArgs, log: &Path, findings: &str) -> Result<(), String> {
    let mode = if given(&args.baseline).is_some() {
        "diff"
    } else {
        "full"
    };
    let name = Path::new(&args.page).file_name();
    let file_name = name.map_or(args.page.clone(), |name| {
        name.to_string_lossy().into_owned()
    });
    let page = given(&args.link).unwrap_or(&file_name);
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d");
    let entry = format!(
        "\n## {today} · {mode} review · {subject}\n\n\
         Page: {page}\n\n{findings}\n\n\
         Decisions: (the writer adds kept/dropped per finding here)\n",
        subject = args.subject,
    );
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .and_then(|mut file| file.write_all(entry.as_bytes()))
        .map_err(|e| format!("{}: {e}", log.display()))
}

fn finding_count(findings: &str) -> usize {
    if findings.starts_with("No findings") {
        return 0;
    }
    FINDING.find_iter(findings).count()
}

pub fn run(args: &ReviewArgs) -> Result<String, String> {
    let skill = skill_dir(args.skill_dir.as_deref())?;
    let (prompt, diff) = build_prompt(args, &skill)?;
    if diff.is_some_and(|diff| strip(&diff).is_empty()) {
        return Ok("No text changes since the last review; nothing to review.".into());
    }
    let findings = reviewer::run(&prompt)?;
    fs::write(&args.out, format!("{findings}\n")).map_err(|e| format!("{}: {e}", args.out))?;
    let log = skill.join("review-log.md");
    log_review(args, &log, &findings)?;
    let count = finding_count(&findings);
    Ok(format!(
        "{count} findings in {}, logged in {}",
        args.out,
        log.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_numbered_findings_at_line_starts() {
        assert_eq!(finding_count("1. a\ntext 2. b\n2. c\n10. d"), 3);
    }

    #[test]
    fn no_findings_counts_zero_even_with_numbers_after() {
        assert_eq!(finding_count("No findings.\n1. not one"), 0);
    }

    #[test]
    fn section_stops_at_the_next_heading_only() {
        let text = "## Checklist\n- a\n### b\n## Next\nc";
        let (body, end) = section_after(text, "## Checklist\n").unwrap();
        assert_eq!(&body[..end.unwrap()], "- a\n### b");
    }

    #[test]
    fn type_meta_reads_either_quote_style() {
        let html = "<meta  name='mudora-type'\n content=\"data-report\">";
        assert_eq!(&TYPE_META.captures(html).unwrap()[1], "data-report");
    }
}
