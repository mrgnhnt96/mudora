#!/usr/bin/env python3
"""Copy base.css into every starter that inlines it (the <style> block marked "base.css — keep in sync").
Run after any change to base.css. Prints which starters changed."""
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
MARK = "/* base.css — keep in sync */"
base = (HERE / "base.css").read_text(encoding="utf-8").strip()
for page in sorted(HERE.glob("*.html")):
    text = page.read_text(encoding="utf-8")
    if MARK not in text:
        continue
    new = re.sub(re.escape(MARK) + r"\n.*?(?=\n</style>)", lambda m: MARK + "\n" + base, text, count=1, flags=re.S)
    if new != text:
        page.write_text(new, encoding="utf-8")
        print("updated", page.name)
    else:
        print("in sync", page.name)
