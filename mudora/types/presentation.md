# Presentation pages

**Marker:** `<meta name="mudora-type" content="presentation">`
**Starter:** none yet; reuse `starters/base.css`

- It's an artifact page, not a slide show, and it's presented in 10 minutes or less. Keep the data, but lead with what matters. The presenter talks through the important parts.
- Put a **TL;DR** at the top: why the work was done, then only the most important findings.
- **Findings tables:** add a root-cause category column (for example Navigation, State Management). Drop "verified" checkmarks, because verification is assumed.
- **Use pictures, diagrams and interactive examples**, so technical and non-technical people can both follow.
- **Remove:** "try it yourself" sections, the measurement-scripts table, tool and build notes on each chart, and file paths to local docs.
- The presenter also gets **talking points** for each section: bulleted and brief, in chat or as a separate list.
- **Design findings answer two questions:** how does the product look and feel, and what makes it look and feel that way. Every design findings page has a look-and-feel section that names the visible trait and the cause behind it (theme, fonts, colour, shape, motion, touch feedback, images).
- **No "Decisions needed" cards on a findings presentation.** It reports what was found. Open questions follow rule 36.
- **Every item on a findings page helps a decision.** Leave out helpers, tooling and scaffolding the work used (for example a manual-run switch that saved agents during experiments). If an item is neither an option to ship nor a risk to one, it goes in chat, not on the page.

## Checklist

- Can the page be presented in 10 minutes, with what matters first?
- Does the TL;DR say why the work was done, then only the key findings?
- Is every item something that helps a decision (no helpers or tooling)?
- Are there no "Decisions needed" cards or try-it-yourself sections?
- (Design findings) Is there a look-and-feel section naming each trait and its cause?
