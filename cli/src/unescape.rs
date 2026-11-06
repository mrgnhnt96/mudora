//! Python's `html.unescape`.

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::entities::{HTML5, INVALID_CHARREFS, INVALID_CODEPOINTS};

const REPLACEMENT: &str = "\u{fffd}";

static CHARREF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"&(#[0-9]+;?|#[xX][0-9a-fA-F]+;?|[^\t\n\x0c <&#;]{1,32};?)").unwrap()
});

fn named(name: &str) -> Option<&'static str> {
    HTML5
        .binary_search_by_key(&name, |(key, _)| key)
        .ok()
        .map(|at| HTML5[at].1)
}

fn numeric(digits: &str, radix: u32) -> String {
    let digits = digits.trim_end_matches(';');
    let Ok(code) = u32::from_str_radix(digits, radix) else {
        return REPLACEMENT.into();
    };
    if let Ok(at) = INVALID_CHARREFS.binary_search_by_key(&code, |(key, _)| *key) {
        return INVALID_CHARREFS[at].1.into();
    }
    let Some(character) = char::from_u32(code) else {
        return REPLACEMENT.into();
    };
    if INVALID_CODEPOINTS.binary_search(&code).is_ok() {
        return String::new();
    }
    character.into()
}

fn longest_named_prefix(reference: &str) -> Option<String> {
    let boundaries: Vec<usize> = reference.char_indices().map(|(at, _)| at).collect();
    boundaries
        .iter()
        .skip(2)
        .rev()
        .find_map(|&at| Some(format!("{}{}", named(&reference[..at])?, &reference[at..])))
}

fn replace(reference: &str) -> String {
    if let Some(number) = reference.strip_prefix('#') {
        return match number.strip_prefix(['x', 'X']) {
            Some(hex) => numeric(hex, 16),
            None => numeric(number, 10),
        };
    }
    if let Some(text) = named(reference) {
        return text.into();
    }
    longest_named_prefix(reference).unwrap_or_else(|| format!("&{reference}"))
}

pub fn unescape(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }
    CHARREF.replace_all(text, |captures: &Captures| replace(&captures[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_references_need_no_semicolon_when_html_allows_it() {
        assert_eq!(unescape("&amp; &amp &copy2 &notin;"), "& & ©2 ∉");
    }

    #[test]
    fn unknown_names_stay_as_written() {
        assert_eq!(unescape("&nosuch; & a&b"), "&nosuch; & a&b");
    }

    #[test]
    fn longest_prefix_wins() {
        assert_eq!(unescape("&notit;"), "¬it;");
    }

    #[test]
    fn numeric_references_follow_the_html5_table() {
        assert_eq!(unescape("&#65;&#x41;&#X41"), "AAA");
        assert_eq!(unescape("&#128;"), "€");
        assert_eq!(
            unescape("&#0;&#xD800;&#x110000;"),
            "\u{fffd}\u{fffd}\u{fffd}"
        );
        assert_eq!(unescape("&#99999999999999999999;"), "\u{fffd}");
        assert_eq!(unescape("a&#1;b&#xFFFE;c"), "abc");
    }
}
