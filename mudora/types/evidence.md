# Fix evidence pages (before and after)

**Marker:** `<meta name="mudora-type" content="evidence">`
**Starter:** `starters/evidence.html`
**Building blocks in the starter:** `.tldr dl`, `.media-pair .media` with `.cap-pill.before/.after` (files in `media/`), `.note` for a missing before video, `.closed details.more`.

A fix evidence page shows that a change to how the product looks or behaves works, with pictures.

## Shape

1. **Header:** title, eyebrow, lede.
2. **TL;DR** as a definition list: **Problem**, **Fix**, **What users notice**, **How we know**, one line each.
3. **Before and after:** videos or screenshots side by side, labelled "Before" and "After" with the device kind. Media are supporting files published with the page (`files`), not inline data URIs.
4. When there's no before video (the bug can't be reproduced on demand, or the old build won't run), say so in one line next to the after video, with why.
5. **Closed sections:** The change (plain bullets, then a closed file list), Setup.

## Checklist

- Does each before and after pair show the same screen, device and step?
- Does "What users notice" describe what a user sees, not the code change?
- Is a missing before video explained in one line?
- Are media files published alongside the page rather than inlined?
