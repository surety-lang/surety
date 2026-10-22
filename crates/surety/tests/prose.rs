//! Prose wrapping. Consecutive `#` lines above a property or an accept line
//! are one pinned sentence, joined by single spaces, so a sentence can be
//! wrapped at any space without touching the obligation: the canonical form
//! and the hash never see prose. The renderer wraps sentences and comments
//! at 100 columns and keeps every comment anchored to the line that follows
//! it, which is what makes it a formatter.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use surety_syntax::load::load_with_siblings;
use surety_syntax::normalize::canonical_file;
use surety_syntax::{Decl, File, PinKind};

/// A directory holding the corpus with one file replaced by `text`.
fn corpus_with(name: &str, text: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("prose")
        .join(format!("{name}-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for entry in fs::read_dir(common::corpus_dir()).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|x| x == "sur") {
            fs::copy(&p, dir.join(p.file_name().unwrap())).unwrap();
        }
    }
    fs::write(dir.join(format!("{name}.sur")), text).unwrap();
    dir.join(format!("{name}.sur"))
}

fn hash_of(file: &Path) -> String {
    let (graph, idx) = load_with_siblings(file).unwrap_or_else(|e| panic!("{file:?}: {e:?}"));
    canonical_file(&graph, idx)
        .unwrap_or_else(|e| panic!("{}: {e}", file.display()))
        .hash
}

/// Every `#` line with a space in its text, split at the space nearest its
/// middle into two `#` lines with the same indent.
fn split_every_sentence(text: &str) -> String {
    let mut out = Vec::new();
    for line in text.lines() {
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let rest = &line[indent.len()..];
        if let Some(s) = rest.strip_prefix('#') {
            let s = s.trim();
            let spaces: Vec<usize> = s.match_indices(' ').map(|(i, _)| i).collect();
            if let Some(&at) = spaces.iter().min_by_key(|&&i| i.abs_diff(s.len() / 2)) {
                out.push(format!("{indent}# {}", &s[..at]));
                out.push(format!("{indent}# {}", s[at + 1..].trim()));
                continue;
            }
        }
        out.push(line.to_string());
    }
    out.join("\n") + "\n"
}

/// The sentences of every property and accept, in order.
fn sentences_of(f: &File) -> Vec<Option<String>> {
    let mut out = Vec::new();
    for d in &f.decls {
        if let Decl::Function(fun) = d {
            out.extend(fun.properties.iter().map(|p| p.sentence.clone()));
            out.extend(fun.accepts.iter().map(|p| p.sentence.clone()));
        }
    }
    out
}

fn sentence_pins(f: &File) -> Vec<String> {
    f.pins
        .iter()
        .filter(|p| p.kind == PinKind::Sentence)
        .map(|p| p.text.clone())
        .collect()
}

#[test]
fn consecutive_sentence_lines_are_one_sentence_and_the_hash_ignores_the_split() {
    let mut split_lines = 0;
    for name in common::FIXTURES {
        let file = common::corpus_dir().join(format!("{name}.sur"));
        let text = fs::read_to_string(&file).unwrap();
        let original = hash_of(&file);
        let parsed = surety_syntax::parse(&text).unwrap();
        let split = split_every_sentence(&text);
        split_lines += split.lines().count() - text.lines().count();
        let reparsed = surety_syntax::parse(&split).unwrap_or_else(|e| panic!("{name} split: {e}"));
        assert_eq!(
            sentences_of(&reparsed),
            sentences_of(&parsed),
            "{name}: every property reads its sentence joined"
        );
        assert_eq!(
            sentence_pins(&reparsed),
            sentence_pins(&parsed),
            "{name}: one pin per sentence, joined"
        );
        assert_eq!(
            hash_of(&corpus_with(name, &split, "split")),
            original,
            "{name}: the hash never sees prose"
        );
    }
    assert!(
        split_lines > 20,
        "the corpus has sentences to split: {split_lines}"
    );
}

#[test]
fn a_blank_line_a_comment_or_a_line_of_code_ends_a_sentence() {
    let text = "type T = { a: Int }\n\nf(x: T) -> Int\n\t# one\n\t# still one\n\n\t# two\n\t// a comment\n\t# three\n\tout == x.a\n\t# four,\n\t# still four\n\tout >= 0\n";
    let file = surety_syntax::parse(text).unwrap();
    assert_eq!(
        sentence_pins(&file),
        vec!["one still one", "two", "three", "four, still four"]
    );
    assert_eq!(
        sentences_of(&file),
        vec![
            Some("three".to_string()),
            Some("four, still four".to_string())
        ]
    );
}
