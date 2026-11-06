#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

PROJECTS = Path.home() / ".claude" / "projects"
SHORT_ID = re.compile(r"artifact/([A-Za-z0-9]+)")
PUBLISHED = re.compile(r"Published (\S+) at (https://claude\.ai/\S+)")


def artifact_ids(link):
    match = SHORT_ID.search(link)
    return {match.group(1)} if match else {link}


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
            if block.get("type") == "tool_use" and block.get("name") == "Artifact":
                artifact_calls[block["id"]] = block.get("input", {})
            elif block.get("type") == "tool_result" and block.get("tool_use_id") in artifact_calls:
                call = artifact_calls[block["tool_use_id"]]
                if call.get("action", "publish") != "publish" or call.get("asset"):
                    continue
                text = json.dumps(block.get("content"))
                if any(i in text for i in ids):
                    source = PUBLISHED.search(text)
                    found.append({
                        "time": entry.get("timestamp") or "",
                        "source": source.group(1) if source else call.get("file_path"),
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
