# Reviewing a page before it's published

Every page gets read by a fresh reviewer before the user or the team sees it. The writer has been inside the work for hours and can't see what a newcomer trips on: a sentence that only makes sense from the experiment, a section that doesn't move the story forward, a claim that doesn't follow from the numbers next to it. The reviewer reads the page cold and only reports; the writer decides and fixes.

The reviewer runs in a **headless Claude** (`claude -p`, started by `review_page.py`), not as a subagent, so the page text, the rules and the reviewer's reading stay out of the writing session. Only the short findings file comes back. Don't start the review with the Agent tool, and don't paste the page text into the session to "help" it.

## When to review, and how much

| Situation | Review |
|---|---|
| First version of a page | **Full review** of the whole page |
| The user asked for changes | **Diff review**: only what changed since the last reviewed version, plus anything the change broke elsewhere |
| A full rewrite, or the diff touches most of the page | Full review |
| The text diff is empty (layout, colour, spacing only) | No review (the script says so and stops); check the rendered layout as usual |
| The change is only the user's exact wording, pasted in | No review |

## "Review this doc: <link>"

The review only reports. The fixes are made by **the session that wrote the page** (the one that last published it), because it has the context: the code, the data, what the user asked for. Nobody asks the user for permission at any point; a request to review is a request to fix.

1. Read the page with `Artifact` `action: "read"` (the full link, share key and all). Copy the saved file it names into the scratchpad as `page.html`.
2. Run a **full review** (no baseline) with `--link <the link>`. Fill in `--readers` and `--subject` from the page itself; the type comes from the page's `mudora-type` marker, or pass `--type` (see pages.md) for a page made before markers. Don't edit or republish the page yourself.
3. Find the writer: `python3 ~/.claude/skills/mudora/find_doc_session.py "<link>"`. It prints the `conversation_id`, `cwd` and `source` file of the session that last published the page.
4. Hand the findings to that session:
   - If that session is still running where you can reach it, send it the message.
   - Otherwise resume it from its `cwd` with `claude --resume <conversation_id>` and the message as the prompt.
   - If no session published it (made elsewhere), start a new session in the right project with the message, and say the source is the live page.
5. Tell the user in two lines: how many findings, and which session is fixing them.

The message to the writer session:

```
A fresh review of <link> found N things to fix. Findings: <path to findings file>.
Load the mudora skill. Decide each finding (fix it, or drop it with a reason), apply the fixes to <source> and republish to the same link without asking the user. Then run a diff review of your changes (review.md, step 3), fill in the Decisions line for this review in ~/.claude/skills/mudora/review-log.md, save the published file as <name>.reviewed.html, and tell the user in a few lines what changed.
```

## Keeping the last reviewed version

Diff review needs a baseline: the page as it was the last time it passed review.

- After a review passes and the page is published, copy the published file next to it as `<name>.reviewed.html` (overwrite the old one). This is the baseline for the next round.
- If there's no `.reviewed.html` (a new session, or the source lives elsewhere), read the live page with `Artifact` `action: "read"` and use the saved file it names as the baseline. The live page was reviewed before it went out, so it's a fair baseline.

## Steps

1. **Run the review.** Findings go to a file in the scratchpad; the command prints only the count.
   - Full review:
     `python3 ~/.claude/skills/mudora/review_page.py page.html --readers "<who reads it>" --type <type from SKILL.md's type table, if the page has no marker> --subject "<one plain line>" --link "<the page's link, once it has one>" --out <scratchpad>/review.md`
   - Diff review: add `--baseline page.reviewed.html --request "<the user's request, verbatim>"`.
   - If the page builds text from script data (chart labels, tips, cards rendered by JS), the extractor can't see it. Write those strings (only the changed ones, for a diff review) to a file and pass `--extra <file>`.
   - Give `--subject` as a plain description of the page, not your opinion of it or what you think is wrong. The reviewer reads the page, not your view of it.
   - Run it with a long Bash timeout (up to 15 minutes); a review usually takes under a minute.
2. **Read the findings file and decide each finding.** A finding is a lead, not an order. Fix it, or drop it with a reason. Drop anything that would undo what the user asked for, or break a rule in SKILL.md. A finding that a claim looks unsupported means checking it against the code or data (rule 8), not rewording it.
3. **Re-check the fixes, not the page.** If the fixes changed more than a few sentences, run one more diff review with the pre-fix file as `--baseline`. Stop after that second round; anything left goes to the user in chat.
4. **Publish, then save the baseline** (`.reviewed.html`).
5. **Log the decisions.** In `review-log.md`, replace the entry's "Decisions" line with one line per finding (`1 kept`, `3 dropped: <why>`).
6. **Tell the user in one or two lines** what the review caught and changed ("Review: cut a TL;DR row that repeated the risks section, rewrote a tip that only made sense from the experiment"). Mention any finding you dropped only if the user would want to know (for example, it questioned something the user asked for). Don't paste the findings list.

## Learning from reviews

`review-log.md` in this folder is the global record of every review: its findings and what the writer did with them. Before writing or revising a page, skim its "Patterns" list. When a kind of finding shows up on two or more pages, add it to "Patterns" and turn it into a rule, placed as SKILL.md's "Where the user's feedback goes" says: in `types/<type>.md`'s Checklist when all the pages were one type, in SKILL.md's "Rules for every page" when they were different types (with a corrections.md entry naming the file). The reviewer is given those rules, so it starts checking for it too. A problem the user has flagged before goes straight into the rules.

## Files

- `review_page.py` extracts the page text and the diff, fills in `reviewer-prompt.md` with them, the "Rules for every page" section of SKILL.md and the Checklist of the page's type file, runs the headless reviewer with no write, shell or web tools, and writes its reply to `--out`.
- `review-log.md` is the global log of findings and decisions; `review_page.py` appends to it.
- `find_doc_session.py <link>` finds the session that last published a page, from the Claude transcripts.
- `page_text.py` is the text extractor (`page_text.py page.html` prints what a reader sees; `page_text.py old.html new.html` prints the text diff).
- `reviewer-prompt.md` is what the reviewer is asked. Change the questions there, not in the script.
