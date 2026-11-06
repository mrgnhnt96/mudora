# Research and proposal pages

**Marker:** `<meta name="mudora-type" content="research">` (proposal variant: `content="proposal"`)
**Starter:** `starters/research.html` (its header shows the proposal pill and eyebrow: for a proposal, set the marker to `proposal`; for plain research, drop the pill and use the area as the eyebrow)
**Building blocks in the starter:** `.tldr`, `.rgrid` results grid (or `.devs` device cards), `.tiles` with `.ic.cant/.work/.know/.todo`, `.opts .opt.rec` option cards, `.flows` flow rows, `.work-group .wi` with `.own.team/.auto/.app` and `.size`, `.qs .q` question cards, `.closed details.more`, `ol.refs`.

A research page answers "can we do X, and how?" It shows what works today, what's missing, the ways to close the gap, and the work that follows. A proposal is the same page written for the team to decide on: the recommendation comes first.

## Shape

1. **Header:** title, eyebrow, lede (rule 29).
2. **TL;DR** (rule 19): at most five finding rows.
3. **How it works today, per device.** A grid (device × build or step × device) or one card per device. Try every cell before marking it "not tried" (rule 22). Rows are named by kind (rule 23).
4. **What's missing:** short tiles grouped by device or area (rule 14), each saying plainly what hasn't been tried and why the obvious route doesn't work (rules 25, 35).
5. **"Three ways to…"** (or however many there are): option cards per [components.md](../components.md#cards-that-compare-options), with one marked Recommended. Options that were tried and rejected don't go here.
6. **Follow-up work:** one row per piece of work, with an owner tag (Team / Automation tool / App) and a size of S, M or L. No days or weeks.
7. **Closed sections, in this order:** Tried and passed on, Every measurement, Setup (rule 7: device drawings), Code references (rule 6).

## Proposal variant

- A status pill in the header: "For team review".
- **No follow-up work list.** A proposal says what to do and why; the step-by-step work belongs in the plan once the team agrees. Leave out the "Work it would need" section.
- The TL;DR is a definition list: **Problem**, **Finding**, **Proposal**, **Limit**, one line each.
- **What it unlocks is a short list, not an audit.** Name the cases or features the work unlocks as chips with at most a one-line lead-in. Leave out where the list came from ("a QA review found 16 of 36…") and what the source doesn't say ("the notes give only categories, not case IDs").
- **Recommend building past the limit, not accepting it.** When a guard already exists (a switch, a setting, a build flag), propose putting the stronger capability behind that same guard rather than living with the limit. When the team owns the code, its lower layers are in reach too: a small native or platform-level piece counts as an option. "Accept the limit" is only the recommendation when nothing can be built past it.
- The recommended option comes first, above the alternatives. The others sit below it under "Not doing now", each with its "Not doing" or "Later" verdict (components.md).
- The eyebrow says "Proposal", not "Implementation plan". An implementation plan is a [plan](plan.md) page.

## Checklist

- Is every cell in the device grid either a measured result or a stated reason it can't be tried?
- Does each option card hold only one short line per labelled row, with pills on one scale?
- Is exactly one option marked Recommended, does every other option card show "Not doing" or "Later" with a reason, and is none of them a numbered step in the recommendation or work list?
- (Research, not proposal) Does every follow-up row have an owner tag and an S/M/L size, with no days or weeks?
- Is there no open-questions section, unless the user asked for one (rule 36)?
- (Proposal) Does the TL;DR give Problem, Finding, Proposal and Limit, and does the recommendation come first?
