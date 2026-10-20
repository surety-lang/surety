#![allow(dead_code)]

use std::path::{Path, PathBuf};

use surety_syntax::Graph;

pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus")
}

pub const FIXTURES: [&str; 11] = [
    "p1",
    "p2",
    "p3",
    "p4",
    "p5a",
    "p5b",
    "p6",
    "p7",
    "p8",
    "p14",
    "apply_diff",
];

/// The whole corpus directory as one graph.
pub fn corpus() -> Graph {
    match surety::load_dir(&corpus_dir()) {
        Ok(g) => g,
        Err(errs) => {
            for (p, e) in &errs {
                eprintln!("{}: {e}", p.display());
            }
            panic!("corpus does not parse");
        }
    }
}

pub fn fixture_index(graph: &Graph, name: &str) -> usize {
    graph
        .index_of(name)
        .unwrap_or_else(|| panic!("no fixture {name}"))
}
