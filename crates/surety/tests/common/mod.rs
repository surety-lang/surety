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

/// A copy of a fixture with one text edit, as a graph of its own (plus the
/// siblings it needs for types), checked against the fixture's own
/// performance under the default budget. The fixture is never edited.
pub fn check_copy(name: &str, from: &str, to: &str, siblings: &[&str]) -> surety_check::Report {
    let read = |n: &str| std::fs::read_to_string(corpus_dir().join(format!("{n}.sur"))).unwrap();
    let src = read(name);
    assert!(src.contains(from), "{name}: {from:?} is not in the fixture");
    let file = surety_syntax::parse(&src.replace(from, to)).unwrap();
    let mut files = vec![(name.to_string(), file)];
    for s in siblings {
        files.push((s.to_string(), surety_syntax::parse(&read(s)).unwrap()));
    }
    let graph = Graph::new(files);
    let perf = corpus_dir().join(format!("{name}.py"));
    let mut c =
        surety_check::Checker::new(&graph, 0, &perf, &corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = surety_check::Budget::default();
    c.check()
}
