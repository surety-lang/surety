//! Example-wanted lines (slice 6, piece 4). The draft writes, for each case
//! the sentences leave to an example, a comment `// example wanted: <the
//! case and what decides it>`, wrapped over the comment lines below it,
//! and the input as a literal in the signature's types on its own `// in:
//! (...)` line directly beneath. Both are comments, never part of the
//! obligation and never hashed. The renderer, the long-line hint, the
//! literal's check and the editor's code action find them by position,
//! through this one reading.

/// One example-wanted block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    /// The line of `// example wanted:`, from 1.
    pub line: usize,
    /// The last line of the case: the first line, or its last continuation.
    pub case_end: usize,
    /// The case and what decides it, its wrapped lines joined by spaces.
    pub case: String,
    /// The `// in:` line directly beneath and the literal on it.
    pub literal: Option<(usize, String)>,
}

impl Wanted {
    /// The block's last line: the literal's, or the case's.
    pub fn end(&self) -> usize {
        self.literal.as_ref().map_or(self.case_end, |(l, _)| *l)
    }
}

const HEAD: &str = "example wanted:";

/// Comments a case does not run on into: another case, or a mark the draft
/// writes for itself.
fn starts_other(text: &str) -> bool {
    text.starts_with(HEAD)
        || text.starts_with("drafted")
        || text.starts_with("draft")
        || text.starts_with("reaching for")
}

/// The blocks among comments given as `(line, text after "//", trimmed)`,
/// in source order. A comment on the very next line continues the case,
/// unless it is the `in:` line, which ends the block, or starts another.
pub fn find(comments: &[(usize, &str)]) -> Vec<Wanted> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < comments.len() {
        let (line, text) = comments[i];
        i += 1;
        let Some(rest) = text.strip_prefix(HEAD) else {
            continue;
        };
        let mut w = Wanted {
            line,
            case_end: line,
            case: rest.trim().to_string(),
            literal: None,
        };
        while let Some(&(next, t)) = comments.get(i) {
            if next != w.end() + 1 {
                break;
            }
            if let Some(lit) = t.strip_prefix("in:") {
                w.literal = Some((next, lit.trim().to_string()));
                i += 1;
                break;
            }
            if starts_other(t) {
                break;
            }
            w.case.push(' ');
            w.case.push_str(t);
            w.case_end = next;
            i += 1;
        }
        out.push(w);
    }
    out
}

/// The blocks in source text, for a document that may not parse.
pub fn in_text(text: &str) -> Vec<Wanted> {
    let comments: Vec<(usize, &str)> = text
        .lines()
        .enumerate()
        .filter_map(|(i, l)| Some((i + 1, l.trim_start().strip_prefix("//")?.trim())))
        .collect();
    find(&comments)
}

/// The blocks among a parsed file's comments.
pub fn in_file(file: &crate::File) -> Vec<Wanted> {
    let comments: Vec<(usize, &str)> = file
        .pins
        .iter()
        .filter(|p| p.kind == crate::PinKind::Comment)
        .map(|p| (p.line, p.text.as_str()))
        .collect();
    find(&comments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wrapped_case_and_its_literal_are_one_block() {
        let text = "f(x: Int) -> Int\n\t// example wanted: zero: the\n\t// boundary\n\t// in: 0\n\t// example wanted: one\n\t// a note of the author's\n";
        let blocks = in_text(text);
        assert_eq!(
            blocks,
            vec![
                Wanted {
                    line: 2,
                    case_end: 3,
                    case: "zero: the boundary".into(),
                    literal: Some((4, "0".into())),
                },
                Wanted {
                    line: 5,
                    case_end: 6,
                    case: "one a note of the author's".into(),
                    literal: None,
                },
            ]
        );
    }

    #[test]
    fn a_draft_mark_or_a_gap_ends_the_case() {
        let text =
            "\t// example wanted: zero\n\t// drafted\n\n\t// example wanted: one\n\n\t// in: 1\n";
        let blocks = in_text(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].end(), 1);
        assert_eq!(
            blocks[1].literal, None,
            "an `in:` after a blank line is not this block's"
        );
    }
}
