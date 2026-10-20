//! Acceptance test 2: layout, binder names, property order, and prose do
//! not change the hash; prose does change the pinned notes.

mod common;

use std::fs;

use surety_syntax::normalize::canonical_file;
use surety_syntax::{Graph, PinKind, parse};

fn p3_text() -> String {
    fs::read_to_string(common::corpus_dir().join("p3.sur")).unwrap()
}

/// Hash a variant of p3 as its own single-file graph.
fn hash_of(text: &str) -> (String, Vec<String>) {
    let file = parse(text).unwrap_or_else(|e| panic!("{e}\n{text}"));
    let pins: Vec<String> = file
        .pins
        .iter()
        .filter(|p| p.kind == PinKind::Sentence)
        .map(|p| p.text.clone())
        .collect();
    let g = Graph::single("p3", file);
    g.check_file(0).unwrap();
    (canonical_file(&g, 0).unwrap().hash, pins)
}

const VERTICAL: &str =
    "\tall g in out:\n\t\tall o in g.orders:\n\t\t\to.customer_email == g.customer_email\n";
const INLINE: &str = "\tall g in out: all o in g.orders: o.customer_email == g.customer_email\n";

#[test]
fn inline_and_vertical_bodies_hash_the_same() {
    let vertical = p3_text();
    assert!(
        vertical.contains(VERTICAL),
        "p3's S5 line is written vertically in the fixture"
    );
    let inline = vertical.replace(VERTICAL, INLINE);
    assert_ne!(vertical, inline);
    assert_eq!(hash_of(&vertical).0, hash_of(&inline).0);
}

#[test]
fn reordered_properties_hash_the_same() {
    let original = p3_text();
    // Move `unique out.customer_email` (with its sentence) to the end of the
    // property list, just before the first example.
    let unique_block = "\t# No two groups share a customer_email\n\tunique out.customer_email\n";
    assert!(original.contains(unique_block));
    let moved = original.replacen(unique_block, "", 1).replacen(
        "\te:\n",
        &format!("{unique_block}\te:\n"),
        1,
    );
    assert_ne!(original, moved);
    assert_eq!(hash_of(&original).0, hash_of(&moved).0);
}

#[test]
fn renamed_binder_hashes_the_same() {
    let original = p3_text();
    // `o` is a binder in p3; rename it everywhere it is bound and used.
    let renamed = original
        .replace("all o in orders", "all order in orders")
        .replace("all o in g.orders", "all order in g.orders")
        .replace("o.customer_email", "order.customer_email");
    assert_ne!(original, renamed);
    assert_eq!(hash_of(&original).0, hash_of(&renamed).0);
}

#[test]
fn changed_sentence_hashes_the_same_but_changes_the_pins() {
    let original = p3_text();
    let changed = original.replace(
        "# No two groups share a customer_email",
        "# Customer emails are unique across groups",
    );
    assert_ne!(original, changed);
    let (h1, pins1) = hash_of(&original);
    let (h2, pins2) = hash_of(&changed);
    assert_eq!(h1, h2);
    assert_ne!(pins1, pins2);
    assert!(
        pins2
            .iter()
            .any(|p| p == "Customer emails are unique across groups")
    );
}

#[test]
fn a_real_change_changes_the_hash() {
    let original = p3_text();
    let changed = original.replace(
        "g.total_amount == sum(g.orders.amount)",
        "g.total_amount == len(g.orders)",
    );
    assert_ne!(hash_of(&original).0, hash_of(&changed).0);
}

#[test]
fn every_fixture_has_a_stable_hash() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let idx = common::fixture_index(&graph, name);
        let a = canonical_file(&graph, idx).unwrap();
        let b = canonical_file(&graph, idx).unwrap();
        assert_eq!(a, b, "{name}");
        assert_eq!(a.hash.len(), 64, "{name}");
    }
}
