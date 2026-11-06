use std::fs;
use std::path::Path;

use serde_json::Value;

/// `str.isspace()`: Rust's `char::is_whitespace` plus the four ASCII separators Python counts.
pub fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

pub fn strip(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// `re.sub(r"\s+", " ", text).strip()`.
pub fn collapse_space(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text.split(is_space).filter(|word| !word.is_empty()) {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

/// What Python's text mode does to line endings when reading.
pub fn universal_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn read_text(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(universal_newlines(&text))
}

pub fn read_text_lossy(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(universal_newlines(&String::from_utf8_lossy(&bytes)))
}

/// `str.format` for the `{name}`, `{{` and `}}` that reviewer-prompt.md uses.
pub fn format_template(template: &str, fields: &[(&str, &str)]) -> Result<String, String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(at) = rest.find(['{', '}']) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if tail.starts_with("{{") || tail.starts_with("}}") {
            out.push_str(&tail[..1]);
            rest = &tail[2..];
        } else if tail.starts_with('}') {
            return Err("reviewer-prompt.md: single '}' encountered in format string".into());
        } else {
            let close = tail
                .find('}')
                .ok_or("reviewer-prompt.md: expected '}' before end of string")?;
            let name = &tail[1..close];
            let value = fields
                .iter()
                .find(|(field, _)| *field == name)
                .ok_or_else(|| format!("reviewer-prompt.md: unknown field {{{name}}}"))?;
            out.push_str(value.1);
            rest = &tail[close + 1..];
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// Python truthiness of a parsed JSON value.
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64() != Some(0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}

fn push_json_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            ' '..='~' => out.push(c),
            _ => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
        }
    }
    out.push('"');
}

fn push_json(out: &mut String, value: &Value, indent: Option<usize>, depth: usize) {
    let separator = |out: &mut String, first: bool, depth: usize| {
        if !first {
            out.push_str(if indent.is_some() { "," } else { ", " });
        }
        if let Some(width) = indent {
            out.push('\n');
            out.push_str(&" ".repeat(width * depth));
        }
    };
    let close = |out: &mut String, bracket: char| {
        if let Some(width) = indent {
            out.push('\n');
            out.push_str(&" ".repeat(width * depth));
        }
        out.push(bracket);
    };
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => out.push_str(&number.to_string()),
        Value::String(text) => push_json_string(out, text),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Object(fields) if fields.is_empty() => out.push_str("{}"),
        Value::Array(items) => {
            out.push('[');
            for (at, item) in items.iter().enumerate() {
                separator(out, at == 0, depth + 1);
                push_json(out, item, indent, depth + 1);
            }
            close(out, ']');
        }
        Value::Object(fields) => {
            out.push('{');
            for (at, (key, item)) in fields.iter().enumerate() {
                separator(out, at == 0, depth + 1);
                push_json_string(out, key);
                out.push_str(": ");
                push_json(out, item, indent, depth + 1);
            }
            close(out, '}');
        }
    }
}

/// `json.dumps(value, indent=indent)`, whose exact text the publish lookup searches and prints.
pub fn json_dumps(value: &Value, indent: Option<usize>) -> String {
    let mut out = String::new();
    push_json(&mut out, value, indent, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strip_removes_what_python_calls_space() {
        assert_eq!(strip("\x1c\u{a0} a b\u{3000}\x1f"), "a b");
    }

    #[test]
    fn collapse_space_joins_runs_of_space() {
        assert_eq!(collapse_space("  a\n\t b\u{2003}c\x1d "), "a b c");
    }

    #[test]
    fn universal_newlines_turns_every_ending_into_a_newline() {
        assert_eq!(universal_newlines("a\r\nb\rc\n"), "a\nb\nc\n");
    }

    #[test]
    fn format_fills_fields_once_and_unescapes_braces() {
        let filled = format_template("{{x}} {a} {b}", &[("a", "{b}"), ("b", "2")]).unwrap();
        assert_eq!(filled, "{x} {b} 2");
    }

    #[test]
    fn format_refuses_unknown_fields_and_lone_braces() {
        assert!(format_template("{nope}", &[]).is_err());
        assert!(format_template("a } b", &[]).is_err());
        assert!(format_template("a { b", &[]).is_err());
    }

    #[test]
    fn dumps_matches_python_separators_and_ascii_escapes() {
        let value = json!([{"a": "é😀\u{7f}\"", "b": [], "c": {}}, null, true, 1]);
        assert_eq!(
            json_dumps(&value, None),
            r#"[{"a": "\u00e9\ud83d\ude00\u007f\"", "b": [], "c": {}}, null, true, 1]"#
        );
    }

    #[test]
    fn dumps_indents_like_python() {
        let value = json!({"a": null, "b": ["x"]});
        assert_eq!(
            json_dumps(&value, Some(2)),
            "{\n  \"a\": null,\n  \"b\": [\n    \"x\"\n  ]\n}"
        );
    }
}
