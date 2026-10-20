//! Extract obligations from the authored markdown.
//!
//! A fence may begin with header lines and end with authoring notes, all at
//! column 0 and none of them declarations. Extraction keeps the obligation,
//! from the first declaration through the last example block, and drops the
//! rest, reporting every dropped line.

use crate::lexer::is_decl_start;

#[derive(Debug, Clone)]
pub struct Extracted {
    /// Fixture name without extension: `p1`, `p5a`, `apply_diff`.
    pub name: String,
    /// The obligation text, verbatim, ending in a newline.
    pub text: String,
    /// Non-blank lines of the fence that were dropped, with their line
    /// numbers in the markdown.
    pub dropped: Vec<(usize, String)>,
    /// The markdown line number of the first kept line.
    pub first_line: usize,
}

/// Bracket depth after scanning one physical line, ignoring strings.
fn scan_depth(line: &str, mut depth: i32) -> i32 {
    let mut in_str = false;
    for c in line.chars() {
        if in_str {
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            _ => {}
        }
    }
    depth
}

fn is_comment_or_sentence(line: &str) -> bool {
    line.starts_with("//") || line.starts_with('#')
}

fn problem_name(header: &[&str], body: &[&str]) -> Option<String> {
    if body
        .iter()
        .any(|l| l.starts_with("apply_diff(") || l.starts_with("apply_diff "))
    {
        return Some("apply_diff".to_string());
    }
    for h in header {
        if let Some(rest) = h.strip_prefix("Problem") {
            let rest = rest.trim_start();
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.is_empty() {
                continue;
            }
            let suffix = rest[digits.len()..]
                .chars()
                .next()
                .filter(|c| *c == 'a' || *c == 'b')
                .map(|c| c.to_string())
                .unwrap_or_default();
            return Some(format!("p{digits}{suffix}"));
        }
    }
    None
}

/// Extract every obligation fence from a markdown document.
pub fn extract(markdown: &str) -> Vec<Extracted> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "```" {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < lines.len() && lines[end].trim() != "```" {
            end += 1;
        }
        let fence = &lines[start..end];
        if let Some(ex) = extract_fence(fence, start) {
            out.push(ex);
        }
        i = end + 1;
    }
    out
}

fn extract_fence(fence: &[&str], offset: usize) -> Option<Extracted> {
    // The obligation starts at the first column-0 declaration.
    let first = fence
        .iter()
        .position(|l| !l.starts_with([' ', '\t']) && is_decl_start(l))?;
    // Walk forward with bracket depth; stop at a column-0 line that is not
    // a declaration, a comment, a sentence, or blank.
    let mut depth = 0;
    let mut last_kept = first;
    let mut j = first;
    while j < fence.len() {
        let line = fence[j];
        if depth == 0 {
            let at_col0 = !line.starts_with([' ', '\t']) && !line.trim().is_empty();
            if at_col0 && !is_decl_start(line) && !is_comment_or_sentence(line) {
                break;
            }
            if !line.trim().is_empty() {
                last_kept = j;
            }
            if !is_comment_or_sentence(line.trim_start()) {
                depth = scan_depth(line, depth);
            }
        } else {
            last_kept = j;
            if !is_comment_or_sentence(line.trim_start()) {
                depth = scan_depth(line, depth);
            }
        }
        j += 1;
    }
    let kept = &fence[first..=last_kept];
    let header: Vec<&str> = fence[..first].to_vec();
    let name = problem_name(&header, kept).unwrap_or_else(|| format!("fence_{}", offset + 1));
    let mut text = kept.join("\n");
    text.push('\n');
    let mut dropped = Vec::new();
    for (k, l) in fence.iter().enumerate() {
        if (k < first || k > last_kept) && !l.trim().is_empty() {
            dropped.push((offset + k + 1, l.to_string()));
        }
    }
    Some(Extracted {
        name,
        text,
        dropped,
        first_line: offset + first + 1,
    })
}
