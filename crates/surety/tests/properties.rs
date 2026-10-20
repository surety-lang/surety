//! Acceptance test 4: every property in the eleven files passes against its
//! hand-written performance, evaluated over the example inputs.

mod common;

use surety_check::{Checker, PropVerdict, Verdict};

#[test]
fn every_property_passes_against_its_performance() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let idx = common::fixture_index(&graph, name);
        let perf = common::corpus_dir().join(format!("{name}.py"));
        let mut checker =
            Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
        let report = checker.check();
        for e in &report.examples {
            assert_eq!(
                e.verdict,
                Verdict::Pass,
                "{name} example {} (line {})",
                e.index + 1,
                e.line
            );
        }
        assert!(!report.properties.is_empty(), "{name}: no properties");
        for p in &report.properties {
            assert_eq!(
                p.verdict,
                PropVerdict::Pass,
                "{name} property at line {}: {}",
                p.line,
                p.text
            );
        }
    }
}

#[test]
fn graph_references_resolve_to_the_right_modules() {
    let graph = common::corpus();
    let refs = |name: &str| {
        let idx = common::fixture_index(&graph, name);
        let perf = common::corpus_dir().join(format!("{name}.py"));
        let mut checker =
            Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
        checker.check().graph_refs
    };
    // P8 stubs `well_formed_email` in its own module.
    assert_eq!(refs("p8"), vec!["well_formed_email -> p8.py"]);
    // P14 calls another obligation's performance.
    assert_eq!(refs("p14"), vec!["apply_diff -> apply_diff.py"]);
    // P5b calls itself.
    assert_eq!(
        refs("p5b"),
        vec!["get_first_occurrence_per_email -> p5b.py"]
    );
    for name in ["p1", "p2", "p3", "p4", "p5a", "p6", "p7", "apply_diff"] {
        assert!(refs(name).is_empty(), "{name} makes no graph reference");
    }
}

#[test]
fn a_missing_graph_reference_fails_at_check_time_with_the_line() {
    let src = "type Payload = { email: Text }
validate(p: Payload) -> Result<Payload, Text[]>
\tnot looks_like_an_email(p.email) <-> exactly_one Err(_) in out
\te:
\t\tin: { email: \"x\" }
\t\tout: Err([\"bad\"])
";
    let file = surety_syntax::parse(src).unwrap();
    let graph = surety_syntax::Graph::single("missing_ref", file);
    graph
        .check_file(0)
        .expect("an unknown callee is a graph reference, not a parse error");
    let dir = common::corpus_dir().join("mutants");
    let mut checker = Checker::new(
        &graph,
        0,
        &dir.join("missing_ref.py"),
        &dir,
        surety::TIMEOUT,
    )
    .unwrap();
    let report = checker.check();
    assert_eq!(report.examples[0].verdict, Verdict::Pass);
    match &report.properties[0].verdict {
        PropVerdict::Error(m) => assert!(
            m.contains("no performance for `looks_like_an_email` (line 3)"),
            "{m}"
        ),
        other => panic!("expected an error verdict, got {other:?}"),
    }
}
