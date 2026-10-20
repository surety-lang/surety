//! Acceptance test 7: boundary failures are their own verdict.

mod common;

use surety_check::value::from_json_at;
use surety_check::{Checker, PropVerdict, Verdict};
use surety_syntax::resolve::Scope;

#[test]
fn four_scores_fail_at_the_output_boundary_not_in_a_property() {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "p4");
    let perf = common::corpus_dir().join("mutants/p4_four_scores.py");
    let mut checker =
        Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    let report = checker.check();
    let boundary: Vec<_> = report
        .examples
        .iter()
        .filter(|e| matches!(e.verdict, Verdict::Boundary(_)))
        .collect();
    assert!(!boundary.is_empty(), "{}", surety::render(&report));
    for e in &boundary {
        let Verdict::Boundary(m) = &e.verdict else {
            unreachable!()
        };
        assert!(m.contains("0..3"), "names the refinement: {m}");
    }
    // The example with four tied scores returns four.
    assert!(matches!(report.examples[4].verdict, Verdict::Boundary(_)));
    for p in &report.properties {
        assert!(
            !matches!(p.verdict, PropVerdict::Fail { .. }),
            "no property fails; the boundary verdict stands alone: {}",
            p.text
        );
    }
}

#[test]
fn a_repeated_sku_is_rejected_at_the_input_boundary() {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "p14");
    let f = graph.functions_in(idx)[0];
    let scope = Scope::for_function(&graph, idx, f).unwrap();
    let (name, old_ty) = &scope.inputs[0];
    assert_eq!(name, "old");
    let bad = serde_json::json!([{ "sku": "A", "qty": 5 }, { "sku": "A", "qty": 6 }]);
    let err = from_json_at(&bad, old_ty, "old").expect_err("repeated sku");
    assert!(err.to_string().contains("unique sku"), "{err}");
    let good = serde_json::json!([{ "sku": "A", "qty": 5 }, { "sku": "B", "qty": 6 }]);
    from_json_at(&good, old_ty, "old").expect("distinct skus");
}

#[test]
fn a_graph_reference_call_checks_the_callee_input_boundary() {
    // An obligation that hands `snapshot_diff` an `old` with a repeated sku
    // through a graph reference: the call is rejected at the input boundary.
    // `Item` resolves across the graph to apply_diff's definition.
    let src = "twice(items: Item[]) -> Item[]
\tlen(snapshot_diff(out, items).added) == 0
\te:
\t\tin: [{ sku: \"A\", qty: 1 }]
\t\tout: [{ sku: \"A\", qty: 1 }, { sku: \"A\", qty: 1 }]
";
    let mut files = vec![("twice".to_string(), surety_syntax::parse(src).unwrap())];
    let p14 = std::fs::read_to_string(common::corpus_dir().join("p14.sur")).unwrap();
    files.push(("p14".to_string(), surety_syntax::parse(&p14).unwrap()));
    let apply = std::fs::read_to_string(common::corpus_dir().join("apply_diff.sur")).unwrap();
    files.push((
        "apply_diff".to_string(),
        surety_syntax::parse(&apply).unwrap(),
    ));
    let graph = surety_syntax::Graph::new(files);
    graph.check_file(0).unwrap();
    let perf = common::corpus_dir().join("mutants/twice.py");
    let mut checker =
        Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    let report = checker.check();
    assert_eq!(report.examples[0].verdict, Verdict::Pass);
    match &report.properties[0].verdict {
        PropVerdict::Error(m) => assert!(
            m.contains("input boundary of `snapshot_diff`") && m.contains("unique sku"),
            "{m}"
        ),
        other => panic!("expected an input-boundary error, got {other:?}"),
    }
}
