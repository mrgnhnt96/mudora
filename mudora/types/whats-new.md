# What's-new and QA handoff pages

**Marker:** `<meta name="mudora-type" content="whats-new">`
**Starter:** none yet; reuse `starters/base.css`

A page about what changed: release notes, a what's-new, a handoff to QA.

## Rules

- **Only what changed** (rule 32): the changes and how to get them. Limits, open items and coming-soon work stay off the page.
- **One card per change:** what it lets the reader do, in one line, then how to use it as a short step path (rule 15).
- **Don't write to "you"** (rule 34); name the reader's thing ("problems from earlier test runs").
- **Handoffs link the download, never a sent file.** The install section links the product's install docs and the exact versioned build the page was written against (pick the one for the commit the page covers). Never write "unzip the file you were sent" or point at a build made on a dev machine: a later build can replace it under the same name.

## Checklist

- Is every item a change that exists today, with no limits or coming-soon work?
- Does each change say what it lets the reader do before how?
- On a handoff, does the install section link the install docs and the versioned build, with no "file you were sent"?
