//! Slice 2, acceptance test 8: the performance hash. Formatting, comments,
//! docstrings and local names do not change it; an operator does. The
//! obligation hash in every report is `surety hash`'s.

mod common;

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, Checker};
use surety_syntax::normalize::canonical_file;

fn tmp(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("hash");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn locals_and_formatting_do_not_change_the_hash_but_an_operator_does() {
    let original = common::corpus_dir().join("p2.py");
    let same = tmp(
        "p2_renamed.py",
        "\"\"\"The small orders.\"\"\"\n\n\ndef get_small_orders(orders):\n    \"\"\"Keep the orders at 100 or under.\"\"\"\n    # the locals are renamed and the layout changed\n    return [order   for order   in orders\n            if order[\"amount\"] <= 100 ]\n",
    );
    let different = tmp(
        "p2_strict.py",
        "def get_small_orders(orders):\n    return [o for o in orders if o[\"amount\"] < 100]\n",
    );
    let a = surety_py::inspect(&original).unwrap();
    let b = surety_py::inspect(&same).unwrap();
    let c = surety_py::inspect(&different).unwrap();
    assert_eq!(a.hash.len(), 64);
    assert_eq!(
        a.hash, b.hash,
        "renamed locals and changed layout hash the same"
    );
    assert_ne!(a.hash, c.hash, "`<` for `<=` changes the hash");
    assert_eq!(a.imports, Vec::<String>::new());
    assert_eq!(a.functions, 1);
    assert_eq!(a.lines, 2);
    assert_eq!(
        b.lines, 8,
        "size is the source's, not the normalized form's"
    );
}

#[test]
fn imports_and_dependencies_are_listed() {
    let p5a = surety_py::inspect(&common::corpus_dir().join("p5a.py")).unwrap();
    assert_eq!(p5a.imports, vec!["datetime".to_string()]);
    assert!(p5a.dependencies.is_empty(), "datetime is standard library");
    let third_party = tmp(
        "deps.py",
        "import re\nimport numpy as np\nfrom requests.adapters import HTTPAdapter\n\n\ndef f(x):\n    return x\n",
    );
    let i = surety_py::inspect(&third_party).unwrap();
    assert_eq!(i.imports, vec!["re", "numpy", "requests"]);
    assert_eq!(i.dependencies, vec!["numpy", "requests"]);
}

#[test]
fn the_obligation_hash_in_a_report_is_the_canonical_one() {
    let graph = common::corpus();
    for name in ["p2", "p8", "p14"] {
        let idx = common::fixture_index(&graph, name);
        let perf = common::corpus_dir().join(format!("{name}.py"));
        let mut c =
            Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
        c.budget = Budget {
            generated: 20,
            search_inputs: 0,
            search_candidates: 0,
        };
        let report = c.check();
        let canonical = canonical_file(&graph, idx).unwrap().hash;
        assert_eq!(report.obligation_hash, canonical, "{name}: check");
        let mutants = c.mutants();
        assert_eq!(mutants.obligation_hash, canonical, "{name}: mutants");
        assert_eq!(
            report.inspection.as_ref().unwrap().hash,
            surety_py::inspect(&perf).unwrap().hash,
            "{name}: the performance hash is the inspector's"
        );
    }
}
