#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

PROJECTS = Path.home() / ".claude" / "projects"
SHORT_ID = re.compile(r"artifact/([A-Za-z0-9]+)")
PUBLISHED = re.compile(r"Published (\S+) at (https://claude\.ai/\S+)")
MCP_PUBLISH = re.compile(r"^mcp__.*artifacts.*__(publish_version|publish_files|edit_version)$")


def artifact_ids(link):
    match = SHORT_ID.search(link)
    return {match.group(1)} if match else {link}


def is_publish_tool(name):
    return name == "Artifact" or bool(MCP_PUBLISH.match(name or ""))


def is_page_publish(name, call):
    if name == "Artifact":
        return call.get("action", "publish") == "publish" and not call.get("asset")
    return True


def source_of(call, result_text):
    published = PUBLISHED.search(result_text)
    if published:
        return published.group(1)
    if call.get("file_path"):
        return call["file_path"]
    files = call.get("files")
    if isinstance(files, list):
        for file in files:
            if isinstance(file, dict) and file.get("as") == "index.html":
                return file.get("path")
    return None


def publishes_in(transcript, ids):
    artifact_calls, found, cwd = {}, [], None
    for line in transcript.open(encoding="utf-8", errors="replace"):
        try:
            entry = json.loads(line)
        except ValueError:
            continue
        cwd = entry.get("cwd") or cwd
        content = (entry.get("message") or {}).get("content")
        if not isinstance(content, list):
            continue
        for block in content:
            if not isinstance(block, dict):
                continue
            if block.get("type") == "tool_use" and is_publish_tool(block.get("name")):
                artifact_calls[block["id"]] = (block["name"], block.get("input", {}))
            elif block.get("type") == "tool_result" and block.get("tool_use_id") in artifact_calls:
                name, call = artifact_calls[block["tool_use_id"]]
                if not is_page_publish(name, call) or block.get("is_error"):
                    continue
                text = json.dumps(block.get("content"))
                if any(i in text for i in ids):
                    found.append({
                        "time": entry.get("timestamp") or "",
                        "source": source_of(call, text),
                        "cwd": cwd,
                    })
    return found


def main(args):
    if len(args) != 1:
        sys.exit("usage: find_doc_session.py <artifact link>")
    ids = artifact_ids(args[0])
    latest = None
    for transcript in PROJECTS.glob("*/*.jsonl"):
        for publish in publishes_in(transcript, ids):
            if latest is None or publish["time"] > latest["time"]:
                latest = {**publish, "conversation_id": transcript.stem}
    if latest is None:
        sys.exit("no session published this artifact")
    print(json.dumps(latest, indent=2))


if __name__ == "__main__":
    main(sys.argv[1:])
