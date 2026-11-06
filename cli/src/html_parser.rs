//! Python's `html.parser.HTMLParser`, as it behaves when a whole page is fed and `close` is never
//! called: anything unfinished at the end of the page is dropped. The regexes are Python's, with
//! `\G` for `match(text, pos)` and `\s` widened to what Python counts as space.
//!
//! The regexes that may not match are optional as a whole, so they always match at `\G` and leave
//! group 1 empty. A real failure would make the engine retry at every later position of the page.

use std::sync::LazyLock;

use fancy_regex::{Captures, Regex as FancyRegex, RegexBuilder};
use regex::Regex;

use crate::pytext::{is_space, strip};
use crate::unescape::unescape;

pub type Attr = (String, Option<String>);

pub trait Handler {
    fn start_tag(&mut self, tag: &str, attrs: &[Attr]);
    fn end_tag(&mut self, tag: &str);
    fn data(&mut self, data: &str);
}

fn fancy(pattern: &str) -> FancyRegex {
    let pattern = pattern
        .replace("_S_", r"[\s\x1c-\x1f]")
        .replace("_s_", r"\s\x1c-\x1f");
    RegexBuilder::new(&pattern)
        .backtrack_limit(usize::MAX)
        .build()
        .unwrap()
}

fn plain(pattern: &str) -> Regex {
    Regex::new(&pattern.replace("_S_", r"[\s\x1c-\x1f]")).unwrap()
}

static TAG_NAME: LazyLock<FancyRegex> =
    LazyLock::new(|| fancy(r"\G(?:([a-zA-Z][^\t\n\r\x0c />\x00]*)(?:_S_|/(?!>))*)?"));

static ATTRIBUTE: LazyLock<FancyRegex> = LazyLock::new(|| {
    fancy(concat!(
        r#"\G(?:((?<=['"_s_/])[^_s_/>][^_s_/=>]*)"#,
        r#"(_S_*=+_S_*('[^']*'|"[^"]*"|(?!['"])[^>_s_]*))?"#,
        r"(?:_S_|/(?!>))*)?",
    ))
});

static START_TAG_END: LazyLock<FancyRegex> = LazyLock::new(|| {
    fancy(concat!(
        r"\G<[a-zA-Z][^\t\n\r\x0c />\x00]*",
        r"(?:[_s_/]*",
        r#"(?:(?<=['"_s_/])[^_s_/>][^_s_/=>]*"#,
        r#"(?:_S_*=+_S_*(?:'[^']*'|"[^"]*"|(?!['"])[^>_s_]*)_S_*)?"#,
        r"(?:_S_|/(?!>))*",
        r")*",
        r")?",
        r"_S_*",
    ))
});

static END_TAG: LazyLock<Regex> =
    LazyLock::new(|| plain(r"\A</_S_*([a-zA-Z][-.a-zA-Z0-9:_]*)_S_*>"));
static COMMENT_CLOSE: LazyLock<Regex> = LazyLock::new(|| plain(r"--_S_*>"));
static MARKED_SECTION_CLOSE: LazyLock<Regex> = LazyLock::new(|| plain(r"]_S_*]_S_*>"));
static MS_MARKED_SECTION_CLOSE: LazyLock<Regex> = LazyLock::new(|| plain(r"]_S_*>"));
static DECLARATION_NAME: LazyLock<Regex> =
    LazyLock::new(|| plain(r"\A[a-zA-Z][-_.a-zA-Z0-9]*_S_*"));
static SCRIPT_CLOSE: LazyLock<Regex> = LazyLock::new(|| plain(r"(?i)</_S_*script_S_*>"));
static STYLE_CLOSE: LazyLock<Regex> = LazyLock::new(|| plain(r"(?i)</_S_*style_S_*>"));

#[derive(Clone, Copy, PartialEq)]
enum RawText {
    Script,
    Style,
}

impl RawText {
    fn of(tag: &str) -> Option<Self> {
        match tag {
            "script" => Some(Self::Script),
            "style" => Some(Self::Style),
            _ => None,
        }
    }

    fn close(self) -> &'static Regex {
        match self {
            Self::Script => &SCRIPT_CLOSE,
            Self::Style => &STYLE_CLOSE,
        }
    }
}

struct Parser<'a, H> {
    raw: &'a str,
    handler: &'a mut H,
    raw_text: Option<RawText>,
}

type Step = Result<Option<usize>, String>;

fn captures_at<'t>(regex: &FancyRegex, text: &'t str, at: usize) -> Result<Captures<'t>, String> {
    regex
        .captures_from_pos(text, at)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "tag pattern did not match".to_string())
}

/// With no `<` left, text ending in what may be half a character reference is held back for more
/// input that never comes.
fn ends_in_open_reference(raw: &str, from: usize) -> bool {
    let window = raw.char_indices().rev().nth(33).map_or(0, |(at, _)| at);
    let start = window.max(from);
    raw[start..]
        .rfind('&')
        .is_some_and(|at| !raw[start + at..].chars().any(|c| is_space(c) || c == ';'))
}

fn attribute_value(rest: Option<&str>, value: Option<&str>) -> Option<String> {
    rest.filter(|rest| !rest.is_empty())?;
    let value = value.unwrap_or("");
    let inner = ['\'', '"']
        .iter()
        .find_map(|&quote| value.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(value);
    Some(unescape(inner).into_owned())
}

impl<H: Handler> Parser<'_, H> {
    fn after_gt(&self, from: usize) -> Option<usize> {
        self.raw[from..].find('>').map(|at| from + at + 1)
    }

    fn run(&mut self) -> Result<(), String> {
        let raw = self.raw;
        let n = raw.len();
        let mut i = 0;
        while i < n {
            let j = if let Some(kind) = self.raw_text {
                match kind.close().find_at(raw, i) {
                    Some(found) => found.start(),
                    None => break,
                }
            } else {
                match raw[i..].find('<') {
                    Some(at) => i + at,
                    None if ends_in_open_reference(raw, i) => break,
                    None => n,
                }
            };
            if i < j {
                if self.raw_text.is_some() {
                    self.handler.data(&raw[i..j]);
                } else {
                    self.handler.data(&unescape(&raw[i..j]));
                }
            }
            i = j;
            if i == n {
                break;
            }
            let rest = &raw[i..];
            let next = if rest.as_bytes().get(1).is_some_and(u8::is_ascii_alphabetic) {
                self.parse_start_tag(i)?
            } else if rest.starts_with("</") {
                self.parse_end_tag(i)?
            } else if rest.starts_with("<!--") {
                COMMENT_CLOSE.find_at(raw, i + 4).map(|found| found.end())
            } else if rest.starts_with("<?") {
                self.after_gt(i + 2)
            } else if rest.starts_with("<!") {
                self.parse_declaration(i)?
            } else if i + 1 < n {
                self.handler.data("<");
                Some(i + 1)
            } else {
                break;
            };
            match next {
                Some(end) => i = end,
                None => break,
            }
        }
        Ok(())
    }

    fn parse_declaration(&self, i: usize) -> Step {
        let rest = &self.raw.as_bytes()[i..];
        if rest.starts_with(b"<![") {
            return self.parse_marked_section(i);
        }
        if rest.len() >= 9 && rest[..9].eq_ignore_ascii_case(b"<!doctype") {
            return Ok(self.after_gt(i + 9));
        }
        Ok(self.after_gt(i + 2))
    }

    fn parse_marked_section(&self, i: usize) -> Step {
        let raw = self.raw;
        let start = i + 3;
        if start == raw.len() {
            return Ok(None);
        }
        let Some(found) = DECLARATION_NAME.find(&raw[start..]) else {
            return Err("expected a name in a marked section of the page".into());
        };
        if start + found.end() == raw.len() {
            return Ok(None);
        }
        let name = strip(found.as_str()).to_lowercase();
        let close: &Regex = match name.as_str() {
            "temp" | "cdata" | "ignore" | "include" | "rcdata" => &MARKED_SECTION_CLOSE,
            "if" | "else" | "endif" => &MS_MARKED_SECTION_CLOSE,
            _ => return Err(format!("unknown status keyword {name:?} in marked section")),
        };
        Ok(close.find_at(raw, start).map(|end| end.end()))
    }

    fn whole_start_tag_end(&self, i: usize) -> Step {
        let raw = self.raw;
        let captures = captures_at(&START_TAG_END, raw, i)?;
        let j = captures.get(0).map_or(i, |found| found.end());
        Ok(match raw[j..].chars().next() {
            Some('>') => Some(j + 1),
            Some('/') => raw[j..].starts_with("/>").then_some(j + 2),
            None => None,
            Some(c) if c.is_ascii_alphabetic() || c == '=' => None,
            Some(_) => Some(j),
        })
    }

    fn parse_start_tag(&mut self, i: usize) -> Step {
        let raw = self.raw;
        let Some(end) = self.whole_start_tag_end(i)? else {
            return Ok(None);
        };
        let name = captures_at(&TAG_NAME, raw, i + 1)?;
        let tag = name.get(1).map_or("", |m| m.as_str()).to_lowercase();
        let mut k = name.get(0).map_or(i + 1, |m| m.end());
        let mut attrs = Vec::new();
        while k < end {
            let attribute = captures_at(&ATTRIBUTE, raw, k)?;
            let Some(attr_name) = attribute.get(1) else {
                break;
            };
            let value = attribute_value(
                attribute.get(2).map(|m| m.as_str()),
                attribute.get(3).map(|m| m.as_str()),
            );
            attrs.push((attr_name.as_str().to_lowercase(), value));
            k = attribute.get(0).map_or(k, |m| m.end());
        }

        let tail = if k < end { strip(&raw[k..end]) } else { "" };
        if tail != ">" && tail != "/>" {
            self.handler.data(&raw[i..end]);
            return Ok(Some(end));
        }
        self.handler.start_tag(&tag, &attrs);
        if tail == "/>" {
            self.handler.end_tag(&tag);
        } else if let Some(kind) = RawText::of(&tag) {
            self.raw_text = Some(kind);
        }
        Ok(Some(end))
    }

    fn parse_end_tag(&mut self, i: usize) -> Step {
        let raw = self.raw;
        let Some(gt_end) = self.after_gt(i + 1) else {
            return Ok(None);
        };
        let Some(found) = END_TAG.captures(&raw[i..]) else {
            return self.parse_loose_end_tag(i, gt_end);
        };
        let elem = found[1].to_lowercase();
        if self.raw_text.is_some() && self.raw_text != RawText::of(&elem) {
            self.handler.data(&raw[i..gt_end]);
            return Ok(Some(gt_end));
        }
        self.handler.end_tag(&elem);
        self.raw_text = None;
        Ok(Some(gt_end))
    }

    fn parse_loose_end_tag(&mut self, i: usize, gt_end: usize) -> Step {
        let raw = self.raw;
        if self.raw_text.is_some() {
            self.handler.data(&raw[i..gt_end]);
            return Ok(Some(gt_end));
        }
        let name = captures_at(&TAG_NAME, raw, i + 2)?;
        let Some(tag) = name.get(1) else {
            if raw[i..].starts_with("</>") {
                return Ok(Some(i + 3));
            }
            return Ok(self.after_gt(i + 2));
        };
        let after_name = name.get(0).map_or(i + 2, |m| m.end());
        self.handler.end_tag(&tag.as_str().to_lowercase());
        Ok(self.after_gt(after_name))
    }
}

pub fn parse(raw: &str, handler: &mut impl Handler) -> Result<(), String> {
    Parser {
        raw,
        handler,
        raw_text: None,
    }
    .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Events(Vec<String>);

    impl Handler for Events {
        fn start_tag(&mut self, tag: &str, attrs: &[Attr]) {
            self.0.push(format!("<{tag} {attrs:?}"));
        }
        fn end_tag(&mut self, tag: &str) {
            self.0.push(format!(">{tag}"));
        }
        fn data(&mut self, data: &str) {
            self.0.push(format!("={data}"));
        }
    }

    fn events(page: &str) -> Vec<String> {
        let mut events = Events::default();
        parse(page, &mut events).unwrap();
        events.0
    }

    #[test]
    fn reads_tags_attributes_and_text() {
        assert_eq!(
            events(r#"<A Href='x&amp;y' hidden b=2>hi &amp; bye</A>"#),
            [
                r#"<a [("href", Some("x&y")), ("hidden", None), ("b", Some("2"))]"#,
                "=hi & bye",
                ">a",
            ]
        );
    }

    #[test]
    fn self_closed_tags_end_at_once_and_script_stays_normal() {
        assert_eq!(
            events("<script/><b>x"),
            ["<script []", ">script", "<b []", "=x"]
        );
    }

    #[test]
    fn script_text_is_raw_up_to_its_own_close_tag() {
        assert_eq!(
            events("<script>a<b>&amp;</i></SCRIPT >c"),
            ["<script []", "=a<b>&amp;</i>", ">script", "=c"]
        );
    }

    #[test]
    fn unclosed_script_drops_the_rest_of_the_page() {
        assert_eq!(events("<script>a"), ["<script []"]);
    }

    #[test]
    fn unfinished_tag_at_the_end_is_dropped() {
        assert_eq!(events("a<b c=\"d"), ["=a"]);
        assert_eq!(events("a<"), ["=a"]);
        assert_eq!(events("a<!-- b"), ["=a"]);
    }

    #[test]
    fn stray_less_than_is_text() {
        assert_eq!(events("a < b"), ["=a ", "=<", "= b"]);
    }

    #[test]
    fn half_a_reference_at_the_end_of_the_page_is_dropped() {
        assert_eq!(events("<p>tail &am"), ["<p []"]);
        assert_eq!(events("<p>tail &am;"), ["<p []", "=tail &am;"]);
        assert_eq!(events("<p>tail &am x"), ["<p []", "=tail &am x"]);
    }

    #[test]
    fn comments_declarations_and_instructions_produce_nothing() {
        assert_eq!(
            events("<!doctype html><!-- a --><?pi?><!bogus><![if x]>y<![endif]>"),
            ["=y"]
        );
    }

    #[test]
    fn loose_end_tags_are_still_end_tags() {
        assert_eq!(events("</a b='>'>c"), [">a", "='>c"]);
        assert_eq!(events("</>x</ 1>y"), ["=x", "=y"]);
    }

    #[test]
    fn unknown_marked_section_fails_like_python() {
        let mut events = Events::default();
        assert!(parse("<![nope x]]>", &mut events).is_err());
        assert!(parse("<![ x", &mut events).is_err());
    }
}
