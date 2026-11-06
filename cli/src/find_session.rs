use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Map, Value, json};

use crate::pytext::{json_dumps, truthy};

const NOT_FOUND: &str = "no session published this artifact";

static SHORT_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"artifact/([A-Za-z0-9]+)").unwrap());
static PUBLISHED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Published ([^\s\x1c-\x1f]+) at https://claude\.ai/[^\s\x1c-\x1f]+").unwrap()
});
static MCP_PUBLISH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^mcp__.*artifacts.*__(publish_version|publish_files|edit_version)\n?\z").unwrap()
});

struct Publish {
    time: Value,
    source: Value,
    cwd: Value,
}

impl Publish {
    fn sort_key(&self) -> &str {
        self.time.as_str().unwrap_or("")
    }
}

fn artifact_id(link: &str) -> &str {
    SHORT_ID
        .captures(link)
        .and_then(|found| found.get(1))
        .map_or(link, |id| id.as_str())
}

fn is_publish_tool(name: &str) -> bool {
    name == "Artifact" || MCP_PUBLISH.is_match(name)
}

fn is_page_publish(name: &str, call: &Value) -> bool {
    if name != "Artifact" {
        return true;
    }
    let is_publish = call.get("action").is_none_or(|action| action == "publish");
    is_publish && !call.get("asset").is_some_and(truthy)
}

fn source_of(call: &Value, result_text: &str) -> Value {
    if let Some(found) = PUBLISHED.captures(result_text) {
        return Value::String(found[1].to_owned());
    }
    if let Some(path) = call.get("file_path").filter(|path| truthy(path)) {
        return path.clone();
    }
    let files = call.get("files").and_then(Value::as_array);
    let page = files
        .into_iter()
        .flatten()
        .find(|file| file.get("as").is_some_and(|name| name == "index.html"));
    page.and_then(|file| file.get("path"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn or_empty_object(value: Option<&Value>) -> Value {
    value.cloned().unwrap_or(Value::Object(Map::new()))
}

#[derive(Default)]
struct Transcript {
    calls: HashMap<String, (String, Value)>,
    cwd: Option<Value>,
    found: Vec<Publish>,
}

impl Transcript {
    fn read_block(&mut self, entry: &Value, block: &Value, id: &str) {
        let kind = block.get("type").and_then(Value::as_str);
        if kind == Some("tool_use") {
            let name = block.get("name").and_then(Value::as_str);
            let call_id = block.get("id").and_then(Value::as_str);
            if let (Some(name), Some(call_id)) = (name.filter(|n| is_publish_tool(n)), call_id) {
                let input = or_empty_object(block.get("input"));
                self.calls
                    .insert(call_id.to_owned(), (name.to_owned(), input));
            }
        } else if kind == Some("tool_result") {
            let call_id = block.get("tool_use_id").and_then(Value::as_str);
            let Some((name, call)) = call_id.and_then(|call_id| self.calls.get(call_id)) else {
                return;
            };
            if !is_page_publish(name, call) || block.get("is_error").is_some_and(truthy) {
                return;
            }
            let text = json_dumps(block.get("content").unwrap_or(&Value::Null), None);
            if text.contains(id) {
                let publish = Publish {
                    time: entry
                        .get("timestamp")
                        .filter(|time| truthy(time))
                        .cloned()
                        .unwrap_or(json!("")),
                    source: source_of(call, &text),
                    cwd: self.cwd.clone().unwrap_or(Value::Null),
                };
                self.found.push(publish);
            }
        }
    }

    fn read_entry(&mut self, entry: &Value, id: &str) {
        if let Some(cwd) = entry.get("cwd").filter(|cwd| truthy(cwd)) {
            self.cwd = Some(cwd.clone());
        }
        let content = entry
            .get("message")
            .and_then(|message| message.get("content"));
        for block in content.and_then(Value::as_array).into_iter().flatten() {
            if block.is_object() {
                self.read_block(entry, block, id);
            }
        }
    }
}

fn publishes_in(path: &Path, id: &str) -> Vec<Publish> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let mut reader = BufReader::new(file);
    let mut transcript = Transcript::default();
    let mut line = Vec::new();
    loop {
        line.clear();
        match reader.read_until(b'\n', &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let text = String::from_utf8_lossy(&line);
        if let Ok(entry) = serde_json::from_str::<Value>(&text) {
            transcript.read_entry(&entry, id);
        }
    }
    transcript.found
}

fn entries(dir: &Path) -> impl Iterator<Item = PathBuf> + use<> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
}

fn transcripts(projects: &Path) -> impl Iterator<Item = PathBuf> {
    entries(projects)
        .filter(|path| path.is_dir())
        .flat_map(|project| entries(&project))
        .filter(|path| {
            let name = path.file_name().map(|name| name.to_string_lossy());
            name.is_some_and(|name| name.ends_with(".jsonl")) && path.is_file()
        })
}

pub fn run(link: &str) -> Result<String, String> {
    let projects = env::home_dir()
        .ok_or("no home folder")?
        .join(".claude/projects");
    let id = artifact_id(link);
    let mut latest: Option<(Publish, String)> = None;
    for path in transcripts(&projects) {
        for publish in publishes_in(&path, id) {
            let is_later = latest
                .as_ref()
                .is_none_or(|(best, _)| publish.sort_key() > best.sort_key());
            if is_later {
                let stem = path.file_stem().unwrap_or_default();
                latest = Some((publish, stem.to_string_lossy().into_owned()));
            }
        }
    }
    let (publish, conversation_id) = latest.ok_or(NOT_FOUND)?;
    let result = json!({
        "time": publish.time,
        "source": publish.source,
        "cwd": publish.cwd,
        "conversation_id": conversation_id,
    });
    Ok(json_dumps(&result, Some(2)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_id_comes_from_the_link_or_the_whole_text_is_the_id() {
        assert_eq!(artifact_id("https://claude.ai/artifact/AbC123?x"), "AbC123");
        assert_eq!(artifact_id("some-uuid"), "some-uuid");
    }

    #[test]
    fn publish_tools_are_artifact_and_the_mcp_publishers() {
        assert!(is_publish_tool("Artifact"));
        assert!(is_publish_tool("mcp__claude-artifacts__publish_files"));
        assert!(is_publish_tool("mcp__x_artifacts_y__edit_version"));
        assert!(!is_publish_tool("mcp__claude-artifacts__read_artifact"));
        assert!(!is_publish_tool("Write"));
    }

    #[test]
    fn asset_uploads_and_other_actions_are_not_page_publishes() {
        assert!(is_page_publish("Artifact", &json!({})));
        assert!(is_page_publish(
            "Artifact",
            &json!({"action": "publish", "asset": ""})
        ));
        assert!(!is_page_publish("Artifact", &json!({"action": "share"})));
        assert!(!is_page_publish("Artifact", &json!({"asset": "a.png"})));
        assert!(is_page_publish(
            "mcp__a_artifacts__edit_version",
            &json!({"asset": 1})
        ));
    }

    #[test]
    fn source_prefers_the_published_line_then_the_call() {
        let text = "Published /a/b.html at https://claude.ai/artifact/x";
        assert_eq!(
            source_of(&json!({"file_path": "/c"}), text),
            json!("/a/b.html")
        );
        assert_eq!(source_of(&json!({"file_path": "/c"}), ""), json!("/c"));
        let files =
            json!({"files": [{"as": "a.css", "path": "/x"}, {"as": "index.html", "path": "/y"}]});
        assert_eq!(source_of(&files, ""), json!("/y"));
        assert_eq!(source_of(&json!({}), ""), Value::Null);
    }
}
