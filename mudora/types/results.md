# Measured results pages

**Marker:** `<meta name="mudora-type" content="results">` (variants: `content="check"`, `content="data-report"`)
**Starter:** `starters/results.html`
**Building blocks in the starter:** `.tldr`, `.stats .stat` tiles, `.chart-card` with `.ba` before/after bars or `.sb` stacked bars (built from the `BA` and `SB` arrays), `.tiles` for why it differs, `.closed details.more`.

A results page reports what a measured change did. The numbers lead; the explanation follows.

## Shape

1. **Header:** title, eyebrow, lede (rule 29).
2. **TL;DR** (rule 19), then the **stat tiles**. The TL;DR always comes first; the tiles hold the two to four headline numbers and repeat nothing in the TL;DR's wording.
3. **Two to five sections**, each headed by its finding (rule 26: "Cached runs finish 9 min sooner", not "Cache results"), each with one chart picked from [charts.md](../charts.md) (rule 13).
4. **Why it differs:** tiles naming the cause of each result that stands out (rule 18).
5. **Closed sections, in this order:** Command (what was run, so someone can repeat it), Every measurement, Setup.

## Short check variant (`check`)

For "does X still hold on the main branch?" or a one-question investigation. Header, TL;DR, one section with the answer and its evidence (a chart, a log line, or a pass/fail grid), then Command and Setup closed. No stat tiles unless there are two or more numbers worth showing.

## Data report variant (`data-report`)

For numbers pulled from a source the team doesn't control (crash dashboards, store data). Same shape as results, with two changes:
- "Setup" becomes **Setup and sources**: where each number came from and the date range.
- The open items become **Open questions, and the access that would answer them**: each question with the access or data that's missing.

## Checklist

- Does the TL;DR come before the stat tiles, and do the tiles avoid repeating it?
- Is each section heading a finding a skimmer understands on its own?
- Does each section have exactly one chart, of a type charts.md allows for that question, with no table repeating it?
- Is every number a measured result, worded so it can't be read as a forecast?
- Does a result that differs from the others have its cause named?
- Are Command, Every measurement and Setup closed at the bottom, in that order?
