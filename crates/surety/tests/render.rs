//! Slice 3, the store, acceptance test 7: the renderer round-trips the
//! corpus. Parsing what it prints hashes the same as parsing the file, with
//! and without fresh binder names.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use surety_syntax::load::load_with_siblings;
use surety_syntax::normalize::canonical_file;

/// A directory holding the corpus with one file replaced by `text`.
fn corpus_with(name: &str, text: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("render")
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

#[test]
fn the_renderer_round_trips_the_corpus() {
    for name in common::FIXTURES {
        let file = common::corpus_dir().join(format!("{name}.sur"));
        let original = hash_of(&file);
        let parsed = surety_syntax::parse(&fs::read_to_string(&file).unwrap()).unwrap();
        let plain = surety_syntax::render::file(&parsed, false);
        let fresh = surety_syntax::render::file(&parsed, true);
        assert_eq!(
            hash_of(&corpus_with(name, &plain, "plain")),
            original,
            "{name}: rendered plain, the hash changed:\n{plain}"
        );
        assert_eq!(
            hash_of(&corpus_with(name, &fresh, "fresh")),
            original,
            "{name}: rendered with fresh names, the hash changed:\n{fresh}"
        );
    }
}

#[test]
fn fresh_names_are_in_order_of_appearance_and_scoped() {
    let file = common::corpus_dir().join("p5a.sur");
    let parsed = surety_syntax::parse(&fs::read_to_string(&file).unwrap()).unwrap();
    let fresh = surety_syntax::render::file(&parsed, true);
    assert!(
        fresh.contains("all b0 in orders: b0.customer_email in out.customer_email"),
        "{fresh}"
    );
    assert!(
        fresh.contains("all b1 in out: all b2 leftover in orders where b1.customer_email == b2.customer_email: b1.ordered_at <= b2.ordered_at"),
        "{fresh}"
    );
    assert!(
        !fresh.contains(" o in "),
        "no original binder survives: {fresh}"
    );
}
