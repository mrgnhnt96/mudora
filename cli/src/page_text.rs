use std::path::Path;

use crate::difflib::unified_diff;
use crate::html_parser::{self, Attr, Handler};
use crate::pytext::{collapse_space, read_text, strip};

const BLOCK: &[&str] = &[
    "p",
    "div",
    "section",
    "article",
    "header",
    "footer",
    "main",
    "aside",
    "nav",
    "li",
    "ul",
    "ol",
    "tr",
    "table",
    "details",
    "summary",
    "figure",
    "figcaption",
    "br",
    "hr",
    "blockquote",
    "pre",
    "dt",
    "dd",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
];
const SKIP: &[&str] = &["script", "style", "noscript", "template", "svg"];
const TIP_ATTRS: &[&str] = &["title", "aria-label", "data-tip", "data-tooltip"];

#[derive(Default)]
struct VisibleText {
    lines: Vec<String>,
    current: String,
    skip_depth: usize,
}

fn heading_level(tag: &str) -> Option<usize> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

impl VisibleText {
    fn end_line(&mut self) {
        let text = collapse_space(&self.current);
        if !text.is_empty() {
            self.lines.push(text);
        }
        self.current.clear();
    }
}

impl Handler for VisibleText {
    fn start_tag(&mut self, tag: &str, attrs: &[Attr]) {
        if SKIP.contains(&tag) {
            self.skip_depth += 1;
            return;
        }
        if self.skip_depth > 0 {
            return;
        }
        if BLOCK.contains(&tag) {
            self.end_line();
        }
        if let Some(level) = heading_level(tag) {
            self.current.push_str(&"#".repeat(level));
            self.current.push(' ');
        } else if tag == "summary" {
            self.current.push_str("▸ ");
        } else if tag == "li" {
            self.current.push_str("- ");
        }
        for name in TIP_ATTRS {
            let last_value = attrs.iter().rev().find(|(attr, _)| attr == name);
            if let Some((_, Some(value))) = last_value
                .filter(|(_, value)| value.as_ref().is_some_and(|value| !value.is_empty()))
            {
                self.end_line();
                self.lines.push(format!("[tip: {}]", strip(value)));
            }
        }
    }

    fn end_tag(&mut self, tag: &str) {
        if SKIP.contains(&tag) {
            self.skip_depth = self.skip_depth.saturating_sub(1);
        } else if self.skip_depth == 0 && BLOCK.contains(&tag) {
            self.end_line();
        }
    }

    fn data(&mut self, data: &str) {
        if self.skip_depth == 0 {
            self.current.push_str(data);
        }
    }
}

pub fn lines_of(html: &str) -> Result<Vec<String>, String> {
    let mut text = VisibleText::default();
    html_parser::parse(html, &mut text)?;
    text.end_line();
    Ok(text.lines)
}

pub fn page_lines(path: &Path) -> Result<Vec<String>, String> {
    lines_of(&read_text(path)?)
}

pub fn text_diff(old: &Path, new: &Path) -> Result<Vec<String>, String> {
    Ok(unified_diff(
        &page_lines(old)?,
        &page_lines(new)?,
        "last reviewed",
        "now",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(html: &str) -> Vec<String> {
        lines_of(html).unwrap()
    }

    #[test]
    fn marks_headings_lists_and_summaries() {
        assert_eq!(
            lines("<h2>Why</h2><ul><li>one<li>two</ul><details><summary>More</summary>x</details>"),
            ["## Why", "- one", "- two", "▸ More", "x"]
        );
    }

    #[test]
    fn hidden_elements_leave_no_text() {
        assert_eq!(
            lines("a<svg><title>b</title></svg><style>c</style><template><p>d</template>e"),
            ["ae"]
        );
    }

    #[test]
    fn inline_tags_stay_on_the_line_and_space_collapses() {
        assert_eq!(lines("<p>a  <b>b</b>\n c</p>"), ["a b c"]);
    }

    #[test]
    fn tooltips_get_their_own_line_and_the_last_duplicate_wins() {
        assert_eq!(
            lines(r#"<p>a<span title="x" title=" y " aria-label="">b</span></p>"#),
            ["a", "[tip: y]", "b"]
        );
    }
}
