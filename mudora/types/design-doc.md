# Design docs

**Marker:** `<meta name="mudora-type" content="design-doc">`
**Starter:** `starters/design-doc-template.html` (one platform) or `starters/design-doc-tabs-template.html` (platform tabs)

There are two starters in `starters/`:
- `starters/design-doc-template.html` is a single-platform doc.
- `starters/design-doc-tabs-template.html` is a doc with platform tabs.

Reuse their layout: colour tokens, platform badges (Android, iOS, Both), severity pills, stack diagrams, collapsible sections, test checklists and the tab bar.

1. **Header:** title, one-sentence lede, and status, owner, date and platforms. Don't add a list of finding codes.
2. **TL;DR, always.** It's the executive summary, four lines at the top: Problem, Fix, What users notice, and How we know it worked.
3. **Problem:** a short paragraph with the key numbers in plain words, then a real-app example. Measurements go in a collapsed section.
4. **Solution:** the idea in a few short bullets or a paragraph, a diagram if it helps, and one line on what stays the same for the user. How it works, event tables and code are collapsed.
5. **What changes:** a handful of plain bullets. The file-by-file table is collapsed.
6. **Risks:** the top 4–5, one line each, with a severity pill and the mitigation. Only risks specific to this work that could stop it landing or working; leave out caveats every UI test lives with, such as device language, screen size or layout.
7. **Testing:** a short pass bar, then collapsed checklists (unit, tool checks, manual QA, performance targets). Keep checkbox IDs stable across edits so saved ticks survive. This is for design docs only. Plan pages have no checkboxes or saved state at all (see [plan.md](plan.md)).

**Split by platform when the platforms differ.** Put the TL;DR above the tabs, then use three tabs: **Shared**, **Android** and **iOS**. Each tab has its own problem, solution, changes, risks and testing. The Shared tab holds what both platforms use. Keep a single page, with no tabs, when the work is all shared code.

**Include an automated leak test** whenever the work touches subscriptions, listeners or anything that could leak. The test counts subscribers on the shared services (the event system, the connection, settings) and repeats a real navigation or reconnect path. It fails if the count goes up. Say which path each platform runs, that it fails on today's code, and what the test needs to run (for example a signed-in test account). The mechanism goes in a collapsed section.

**Test behaviour that users can see.** When the change touches navigation, include a check that transitions and pop animations look the same as on the main branch.

**Leave out:** rollout, effort, open questions, PR breakdowns and complexity scores. A PR plan scored with the team's complexity rubric belongs in a **sprint proposal**, which is a different kind of doc from these design docs. Give estimates in chat, unless the user asks for them on the page; questions follow rule 36.

**Effort and impact section (when asked).** When someone asks for effort and user impact, add an "Effort and impact" section. In a tabbed doc it goes inside each tab (Shared, Android, iOS), just before Risks, with only that tab's items. Never put it above the tabs. In a single-page doc it goes right after the TL;DR. Effort is described by scope (how many screens, spots, test checks), never by days or weeks. "What improves" lists results after the change, not the current state. Keep it plain and brief, with only measured or counted data. No mention of test builds.

**Scope:** cover every affected platform. When the work is designed for one platform, the matching work for the others goes in the same doc.

## Checklist

- Does the TL;DR give Problem, Fix, What users notice and How we know it worked?
- Is every code snippet, event table and file list collapsed, out of the explanation?
- Are there 4–5 risks at most, each specific to this work, with a pill and a mitigation?
- If the platforms differ, are there Shared, Android and iOS tabs, with the TL;DR above them?
- If the work touches subscriptions or listeners, is there an automated leak test?
- Are rollout, effort (unless asked), open questions and PR breakdowns left out?
