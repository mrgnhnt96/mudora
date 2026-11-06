#!/usr/bin/env python3
import difflib
import re
import sys
from html.parser import HTMLParser

USAGE = "usage: page_text.py page.html | page_text.py last-reviewed.html now.html"

BLOCK = {
    "p", "div", "section", "article", "header", "footer", "main", "aside", "nav",
    "li", "ul", "ol", "tr", "table", "details", "summary", "figure", "figcaption",
    "br", "hr", "blockquote", "pre", "dt", "dd", "h1", "h2", "h3", "h4", "h5", "h6",
}
SKIP = {"script", "style", "noscript", "template", "svg"}
HEADINGS = {"h1", "h2", "h3", "h4", "h5", "h6"}
TIP_ATTRS = ("title", "aria-label", "data-tip", "data-tooltip")


class VisibleText(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.lines, self.cur, self.skip_depth = [], [], 0

    def end_line(self):
        text = re.sub(r"\s+", " ", "".join(self.cur)).strip()
        if text:
            self.lines.append(text)
        self.cur = []

    def handle_starttag(self, tag, attrs):
        if tag in SKIP:
            self.skip_depth += 1
            return
        if self.skip_depth:
            return
        if tag in BLOCK:
            self.end_line()
        if tag in HEADINGS:
            self.cur.append("#" * int(tag[1]) + " ")
        elif tag == "summary":
            self.cur.append("▸ ")
        elif tag == "li":
            self.cur.append("- ")
        attr = dict(attrs)
        for name in TIP_ATTRS:
            if attr.get(name):
                self.end_line()
                self.lines.append(f"[tip: {attr[name].strip()}]")

    def handle_endtag(self, tag):
        if tag in SKIP:
            self.skip_depth = max(0, self.skip_depth - 1)
        elif not self.skip_depth and tag in BLOCK:
            self.end_line()

    def handle_data(self, data):
        if not self.skip_depth:
            self.cur.append(data)


def page_lines(path):
    parser = VisibleText()
    with open(path, encoding="utf-8") as f:
        parser.feed(f.read())
    parser.end_line()
    return parser.lines


def text_diff(old_path, new_path):
    return difflib.unified_diff(
        page_lines(old_path), page_lines(new_path), "last reviewed", "now", n=3, lineterm=""
    )


def main(args):
    if len(args) == 1:
        print("\n".join(page_lines(args[0])))
    elif len(args) == 2:
        print("\n".join(text_diff(*args)))
    else:
        sys.exit(USAGE)


if __name__ == "__main__":
    main(sys.argv[1:])
