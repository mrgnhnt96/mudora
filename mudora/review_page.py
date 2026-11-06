#!/usr/bin/env python3
import argparse
import re
import subprocess
import sys
from datetime import date
from pathlib import Path

from page_text import page_lines, text_diff

SKILL_DIR = Path(__file__).resolve().parent
REVIEW_LOG = SKILL_DIR / "review-log.md"

FULL_SCOPE = "This is the first version. Review the whole page."

DIFF_SCOPE = """This is a revision. The user asked for: "{request}".
The changes since the last review are in <diff> (a unified diff of the visible text).
Review only the changed lines, and the places the change affects: a TL;DR row or intro that
now disagrees with a changed section, a removed section something still refers to, a term
the change introduces without explaining, a section that no longer flows into the next.
Use <page_text> only for that context. Don't report on unchanged parts of the page. Don't
question what the user asked for; check that it was done well and fits the rest."""


def page_rules():
    skill = (SKILL_DIR / "SKILL.md").read_text(encoding="utf-8")
    match = re.search(r"## Rules for every page\n(.*?)\n## ", skill, re.S)
    return match.group(1).strip() if match else ""


TYPE_LABELS = {
    "research": "research page", "proposal": "proposal", "results": "measured results report",
    "check": "short check", "data-report": "data report", "evidence": "before-and-after fix evidence page",
    "design-doc": "design doc", "plan": "plan", "presentation": "presentation", "deck": "slide deck",
    "explainer": "explainer", "whats-new": "what's-new page", "guide": "how-to guide",
    "review-tool": "review tool",
}
TYPE_FILES = {"proposal": "research", "check": "results", "data-report": "results"}


def page_type(page_path, given):
    if given:
        return given
    html = Path(page_path).read_text(encoding="utf-8", errors="replace")
    match = re.search(r'<meta\s+name=["\']mudora-type["\']\s+content=["\']([\w-]+)["\']', html)
    return match.group(1) if match else None


def type_checklist(kind):
    if not kind:
        return ""
    path = SKILL_DIR / "types" / f"{TYPE_FILES.get(kind, kind)}.md"
    if not path.exists():
        sys.exit(f"unknown page type {kind!r}: no {path}")
    match = re.search(r"## Checklist\n(.*?)(?=\n## |\Z)", path.read_text(encoding="utf-8"), re.S)
    if not match:
        return ""
    return (f"\n<type_checklist>\nThis page is a {TYPE_LABELS.get(kind, kind)}. Questions specific to this type of page; "
            f"report a failed one like any other finding:\n{match.group(1).strip()}\n</type_checklist>\n")


def build_prompt(args):
    page_text = "\n".join(page_lines(args.page))
    if args.extra:
        page_text += "\n\nText the page's script shows:\n" + Path(args.extra).read_text(encoding="utf-8")
    if args.baseline:
        diff = "\n".join(text_diff(args.baseline, args.page))
        scope = DIFF_SCOPE.format(request=args.request or "(not given)")
        diff_block = f"\n<diff>\n{diff}\n</diff>\n"
    else:
        diff, scope, diff_block = None, FULL_SCOPE, ""
    kind = page_type(args.page, args.type)
    kind_label = args.kind or TYPE_LABELS.get(kind, "report")
    template = (SKILL_DIR / "reviewer-prompt.md").read_text(encoding="utf-8")
    prompt = template.format(
        readers=args.readers, kind=kind_label, subject=args.subject, scope=scope,
        page_text=page_text, diff_block=diff_block, rules=page_rules(),
        checklist=type_checklist(kind),
    )
    return prompt, diff


def run_headless_reviewer(prompt):
    result = subprocess.run(
        ["claude", "-p", "--no-session-persistence", "--output-format", "text",
         "--disallowedTools", "Bash", "Edit", "Write", "NotebookEdit", "WebFetch", "WebSearch", "Agent"],
        input=prompt, capture_output=True, text=True, timeout=900,
    )
    if result.returncode != 0:
        sys.exit(f"reviewer failed: {result.stderr.strip()[:500]}")
    return result.stdout.strip()


def log_review(args, findings):
    mode = "diff" if args.baseline else "full"
    page = args.link or Path(args.page).name
    entry = (
        f"\n## {date.today().isoformat()} · {mode} review · {args.subject}\n\n"
        f"Page: {page}\n\n{findings}\n\n"
        "Decisions: (the writer adds kept/dropped per finding here)\n"
    )
    with REVIEW_LOG.open("a", encoding="utf-8") as log:
        log.write(entry)


def main():
    parser = argparse.ArgumentParser(description="Review an artifact page in a headless Claude.")
    parser.add_argument("page")
    parser.add_argument("--baseline", help="last reviewed version; reviews only the diff")
    parser.add_argument("--request", help="the user's change request, verbatim")
    parser.add_argument("--readers", default="the team")
    parser.add_argument("--type", help="page type (types/<type>.md); read from the page's mudora-type meta tag if left out")
    parser.add_argument("--kind", help="how to describe the page to the reviewer; defaults to the type's name")
    parser.add_argument("--subject", required=True)
    parser.add_argument("--extra", help="file with text the page's script renders")
    parser.add_argument("--out", required=True, help="where to write the findings")
    parser.add_argument("--link", help="the page's claude.ai link, for the review log")
    args = parser.parse_args()

    prompt, diff = build_prompt(args)
    if diff is not None and not diff.strip():
        print("No text changes since the last review; nothing to review.")
        return
    findings = run_headless_reviewer(prompt)
    Path(args.out).write_text(findings + "\n", encoding="utf-8")
    log_review(args, findings)
    count = 0 if findings.startswith("No findings") else len(re.findall(r"^\d+\.", findings, re.M))
    print(f"{count} findings in {args.out}, logged in {REVIEW_LOG}")


if __name__ == "__main__":
    main()
