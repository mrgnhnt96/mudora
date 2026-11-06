# Mudora

A Claude Code skill that turns work into pages people can actually read: reports, plans, design docs, research, decks, guides and QA handoffs, published as Claude artifacts.

The name comes from the Book of Mudora in The Legend of Zelda, which translates ancient text into words you can read. This is an unofficial fan reference, with no link to Nintendo.

## What it does

- **Picks a page type** from 14 kinds (research, proposal, results, design doc, plan and more), each with its own shape and checklist.
- **Applies one set of writing rules** to every page: plain words, the point first, code tucked into closed sections, charts instead of number tables, and nothing claimed that wasn't measured.
- **Starts from HTML templates** with a shared style, in light and dark.
- **Reviews before publishing.** A fresh headless Claude reads each page as a newcomer and reports what doesn't make sense.
- **Learns from feedback.** Each correction is fixed on the page and written into the narrowest rule that covers it, so it doesn't come up again.

## Install

Follow [INSTALL.md](INSTALL.md), or ask Claude Code to.

## Layout

- `mudora/SKILL.md`: entry point, the rules for every page, and where feedback goes
- `mudora/types/*.md`: one guide per page type
- `mudora/starters/`: page templates, `base.css`, and `sync_css.py` to copy the style into them
- `mudora/components.md`, `mudora/charts.md`: shared pieces and how to pick a chart
- `mudora/review.md`, `review_page.py`, `reviewer-prompt.md`, `page_text.py`, `find_doc_session.py`: the review before publishing
- `mudora/pages.md`, `corrections.md`, `review-log.md`: your history, empty to start

## License

[MIT](LICENSE)
