# Components shared across page types

Rules for pieces that show up on more than one type of page. A page type's own file (`types/<type>.md`) can narrow these for its type; when they differ, the type file wins for that type only.

When the user's feedback is about one of these pieces wherever it appears (every option card, every chart), the rule goes here. When it's about the piece on one type of page only, it goes in that type's file.

## Cards that compare options

The pattern: each change is a card with a saving bar, labelled rows and a pill.

- **Pills on sibling cards use one scale.** If most cards show risk, every card shows risk. Don't mix in a status ("Approved", "Not recommended"); a status answers a different question and breaks the comparison.
- **Each labelled row holds only what its label says, in one short line.** "Cost: PR runs no longer upload their builds" is right. A cost row that also carries progress ("one green run so far, a tool version fixes…") is a status update under the wrong label; if it needs extra words to make sense, the reader still won't know what the cost is. Progress and caveats go elsewhere or nowhere.
- **A cost is what you give up, not what must happen first.** Work needed before an option can ship (tests to fix, a cleanup step) goes in the risks section as a fix-before-shipping item. If an option has no real cost left, drop its Cost row.
- **Anything you're not doing says so, where it sits.** A reader skimming the cards can't tell an alternative from part of the plan. Every option card that isn't being done carries a visible verdict line under its title ("Not doing" or "Later, not planned yet") with one line why, on a muted card, under a divider that says it ("Not doing now"), never a neutral "Also considered". A not-done option never appears as a numbered step in the recommendation or the work list: if it may come later, show it once as a muted "Later" note after the list. Questions and findings that only matter for it are labelled as being for it. Options tested and dropped for good go in one closed section ("Tried and passed on") with a line each on why.
- **Show measured changes as marks, not repeated sentences** (rule 13). If every card writes "X to Y minutes", show the before and after on the card's bar instead.
- **Every card in a series shows the same kind of number.** If the other cards show what their own change measures, this one does too: the upload card shows the upload job (1.7 min to none), not the whole run (56.7 to 55.3 min), which reads as the job taking 56 min. A total goes in the hover tip if it helps.

## Status cells in grids

A grid cell with a status (Covered, Partly, Not covered, Works) shows a dot, the word, and at most a short qualifier on its own line under the word, lined up with the word, in small grey text. The qualifier never sits beside the word, where a narrow column makes it wrap into a ragged stack. Build the cell as a two-column grid (dot, then text) with the qualifier in the text column.

## Charts

- **Split dense data with a segment control** (for example iOS / Android) instead of showing everything at once.
- **Replays run slowly enough to follow:** step by step, one event at a time, with a speed control. The scrub timeline moves at a constant pace and keeps playing while scrubbing. Autoplay when an item is selected.
- **Choose the chart type from [charts.md](charts.md).** Check the "don't use it for" column before using one.
- **Let the reader explore a comparison.** When a chart compares runs across several conditions (load on the agents, which changes are on, which platform), make it interactive: controls to switch conditions or toggle each change on and off, and the chart updates. Use only measured values; a condition that wasn't measured isn't a control.
- **Make each chart look different enough to tell apart,** and pick the form that fits the data. Instance counts are shown as blocks, not a line chart. Axes start at zero, and a baseline must mean what it looks like (a line at 0 can't stand for 1).
- **Show whole numbers only,** in live counters and in "+N" deltas. A disappearing decimal makes the number jump. This is easy to miss, so check the rendered output, not just the code.
- **Add a playhead,** a vertical line across all charts, so several charts can be followed together.
- **Add "expected" lines only when they make sense** and can be drawn clearly.

## Interactive pages (review tools, answerable cards)

- **Highlight only what's active.** For example, focus the tab that was just tapped, and grey out the rest.
- **Warn about unsaved or unshared work on the page itself.** Pages run inside a claude.ai frame, where the browser's leave-page prompt never shows. Keep a reminder in view while work is pending (a bar pinned to the bottom with the action in it), and give the button that clears it the attention colour. Once that reminder is up, drop any toast that says the same thing; two messages about one state stack and cover each other. Fade the reminder to about 40% until hover, so it stays quiet while people work; keep it solid on touch screens, which have no hover.
- **Let a reviewer hide other people's input.** A page where several people review the same thing gets a switch, next to the view controls and reachable with the comments closed, that hides everyone else's comments, replies and unread marks, so a reviewer can form a view first. Who has viewed or approved stays visible: it's progress, not an opinion. Keep the reviewer's own items, remember the choice for that viewer, and say quietly how many items are hidden.
- **A control never moves when it's used.** Text that appears or changes next to a checkbox, button or switch (a count, a status) goes on the side away from its anchor, or in a reserved space, so the control stays under the pointer.
- **Group people by what they did.** When several people share a status, show it once: their avatars overlapped in a row, then the names, then the status ("Ana and Ben viewed"), not "Ana · viewed  Ben · viewed".
