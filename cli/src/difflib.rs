//! Python's `difflib.unified_diff` with its default `SequenceMatcher`. Other diff algorithms
//! pick different hunks, and the reviewer has to see the same diff the script produced.

use std::collections::HashMap;

const POPULAR_MIN_LINES: usize = 200;

#[derive(Clone, Copy, PartialEq)]
enum Tag {
    Equal,
    Replace,
    Delete,
    Insert,
}

#[derive(Clone, Copy)]
struct Opcode {
    tag: Tag,
    i1: usize,
    i2: usize,
    j1: usize,
    j2: usize,
}

struct Matcher<'a> {
    a: &'a [String],
    b: &'a [String],
    b2j: HashMap<&'a str, Vec<usize>>,
}

impl<'a> Matcher<'a> {
    fn new(a: &'a [String], b: &'a [String]) -> Self {
        let mut b2j: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, line) in b.iter().enumerate() {
            b2j.entry(line).or_default().push(index);
        }
        if b.len() >= POPULAR_MIN_LINES {
            let limit = b.len() / 100 + 1;
            b2j.retain(|_, indices| indices.len() <= limit);
        }
        Self { a, b, b2j }
    }

    fn find_longest_match(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
    ) -> (usize, usize, usize) {
        let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0);
        let mut j2len: HashMap<usize, usize> = HashMap::new();
        for i in alo..ahi {
            let mut next: HashMap<usize, usize> = HashMap::new();
            for &j in self.b2j.get(self.a[i].as_str()).into_iter().flatten() {
                if j < blo {
                    continue;
                }
                if j >= bhi {
                    break;
                }
                let before = j.checked_sub(1).and_then(|at| j2len.get(&at));
                let k = before.copied().unwrap_or(0) + 1;
                next.insert(j, k);
                if k > bestsize {
                    (besti, bestj, bestsize) = (i + 1 - k, j + 1 - k, k);
                }
            }
            j2len = next;
        }
        while besti > alo && bestj > blo && self.a[besti - 1] == self.b[bestj - 1] {
            (besti, bestj, bestsize) = (besti - 1, bestj - 1, bestsize + 1);
        }
        while besti + bestsize < ahi
            && bestj + bestsize < bhi
            && self.a[besti + bestsize] == self.b[bestj + bestsize]
        {
            bestsize += 1;
        }
        (besti, bestj, bestsize)
    }

    fn matching_blocks(&self) -> Vec<(usize, usize, usize)> {
        let (la, lb) = (self.a.len(), self.b.len());
        let mut queue = vec![(0, la, 0, lb)];
        let mut blocks = Vec::new();
        while let Some((alo, ahi, blo, bhi)) = queue.pop() {
            let (i, j, k) = self.find_longest_match(alo, ahi, blo, bhi);
            if k > 0 {
                blocks.push((i, j, k));
                if alo < i && blo < j {
                    queue.push((alo, i, blo, j));
                }
                if i + k < ahi && j + k < bhi {
                    queue.push((i + k, ahi, j + k, bhi));
                }
            }
        }
        blocks.sort_unstable();

        let (mut i1, mut j1, mut k1) = (0, 0, 0);
        let mut merged = Vec::new();
        for (i2, j2, k2) in blocks {
            if i1 + k1 == i2 && j1 + k1 == j2 {
                k1 += k2;
            } else {
                if k1 > 0 {
                    merged.push((i1, j1, k1));
                }
                (i1, j1, k1) = (i2, j2, k2);
            }
        }
        if k1 > 0 {
            merged.push((i1, j1, k1));
        }
        merged.push((la, lb, 0));
        merged
    }

    fn opcodes(&self) -> Vec<Opcode> {
        let (mut i, mut j) = (0, 0);
        let mut codes = Vec::new();
        for (ai, bj, size) in self.matching_blocks() {
            let tag = match (i < ai, j < bj) {
                (true, true) => Some(Tag::Replace),
                (true, false) => Some(Tag::Delete),
                (false, true) => Some(Tag::Insert),
                (false, false) => None,
            };
            if let Some(tag) = tag {
                codes.push(Opcode {
                    tag,
                    i1: i,
                    i2: ai,
                    j1: j,
                    j2: bj,
                });
            }
            (i, j) = (ai + size, bj + size);
            if size > 0 {
                codes.push(Opcode {
                    tag: Tag::Equal,
                    i1: ai,
                    i2: i,
                    j1: bj,
                    j2: j,
                });
            }
        }
        codes
    }

    fn grouped_opcodes(&self, context: usize) -> Vec<Vec<Opcode>> {
        let mut codes = self.opcodes();
        if codes.is_empty() {
            codes.push(Opcode {
                tag: Tag::Equal,
                i1: 0,
                i2: 1,
                j1: 0,
                j2: 1,
            });
        }
        if let Some(first) = codes.first_mut().filter(|code| code.tag == Tag::Equal) {
            first.i1 = first.i1.max(first.i2.saturating_sub(context));
            first.j1 = first.j1.max(first.j2.saturating_sub(context));
        }
        if let Some(last) = codes.last_mut().filter(|code| code.tag == Tag::Equal) {
            last.i2 = last.i2.min(last.i1 + context);
            last.j2 = last.j2.min(last.j1 + context);
        }

        let mut groups = Vec::new();
        let mut group = Vec::new();
        for mut code in codes {
            if code.tag == Tag::Equal && code.i2 - code.i1 > context * 2 {
                group.push(Opcode {
                    i2: code.i2.min(code.i1 + context),
                    j2: code.j2.min(code.j1 + context),
                    ..code
                });
                groups.push(std::mem::take(&mut group));
                code.i1 = code.i1.max(code.i2.saturating_sub(context));
                code.j1 = code.j1.max(code.j2.saturating_sub(context));
            }
            group.push(code);
        }
        let only_equal = group.len() == 1 && group[0].tag == Tag::Equal;
        if !group.is_empty() && !only_equal {
            groups.push(group);
        }
        groups
    }
}

fn format_range(start: usize, stop: usize) -> String {
    let length = stop - start;
    match length {
        0 => format!("{start},0"),
        1 => format!("{}", start + 1),
        _ => format!("{},{length}", start + 1),
    }
}

pub fn unified_diff(a: &[String], b: &[String], from: &str, to: &str) -> Vec<String> {
    let mut out = Vec::new();
    let groups = Matcher::new(a, b).grouped_opcodes(3);
    if !groups.is_empty() {
        out.push(format!("--- {from}"));
        out.push(format!("+++ {to}"));
    }
    for group in groups {
        let (first, last) = (group[0], group[group.len() - 1]);
        out.push(format!(
            "@@ -{} +{} @@",
            format_range(first.i1, last.i2),
            format_range(first.j1, last.j2)
        ));
        for code in group {
            if code.tag == Tag::Equal {
                out.extend(a[code.i1..code.i2].iter().map(|line| format!(" {line}")));
                continue;
            }
            if code.tag != Tag::Insert {
                out.extend(a[code.i1..code.i2].iter().map(|line| format!("-{line}")));
            }
            if code.tag != Tag::Delete {
                out.extend(b[code.j1..code.j2].iter().map(|line| format!("+{line}")));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    fn diff(a: &str, b: &str) -> Vec<String> {
        unified_diff(&lines(a), &lines(b), "old", "new")
    }

    #[test]
    fn same_text_has_no_diff() {
        assert!(diff("a b c", "a b c").is_empty());
        assert!(diff("", "").is_empty());
    }

    #[test]
    fn replaced_line_keeps_three_lines_of_context() {
        let out = diff("a b c d e f g h i", "a b c d X f g h i");
        assert_eq!(
            out,
            [
                "--- old",
                "+++ new",
                "@@ -2,7 +2,7 @@",
                " b",
                " c",
                " d",
                "-e",
                "+X",
                " f",
                " g",
                " h",
            ]
        );
    }

    #[test]
    fn empty_side_uses_a_zero_length_range() {
        assert_eq!(
            diff("", "a b"),
            ["--- old", "+++ new", "@@ -0,0 +1,2 @@", "+a", "+b"]
        );
        assert_eq!(diff("a", ""), ["--- old", "+++ new", "@@ -1 +0,0 @@", "-a"]);
    }

    #[test]
    fn prefers_the_longest_block_over_a_common_prefix() {
        assert_eq!(
            diff("a b", "a c a b"),
            [
                "--- old",
                "+++ new",
                "@@ -1,2 +1,4 @@",
                "+a",
                "+c",
                " a",
                " b"
            ]
        );
    }

    #[test]
    fn distant_changes_become_separate_hunks() {
        let a = "1 2 3 4 5 6 7 8 9 10 11 12";
        let b = "X 2 3 4 5 6 7 8 9 10 11 Y";
        let out = diff(a, b);
        assert_eq!(out.iter().filter(|line| line.starts_with("@@")).count(), 2);
    }
}
