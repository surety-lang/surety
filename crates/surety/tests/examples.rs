//! Acceptance test 3: every example in the eleven files passes against its
//! hand-written performance, with order-insensitive comparison exactly
//! where `unordered` says.

mod common;

use surety_check::examples::{Step, compare, unordered_paths};
use surety_check::{Checker, Value, Verdict};

#[test]
fn every_example_passes_against_its_performance() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let idx = common::fixture_index(&graph, name);
        let perf = common::corpus_dir().join(format!("{name}.py"));
        let mut checker =
            Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
        let reports = checker.check_examples();
        assert!(!reports.is_empty(), "{name}: no examples");
        for r in &reports {
            assert_eq!(
                r.verdict,
                Verdict::Pass,
                "{name} example {} (line {})",
                r.index + 1,
                r.line
            );
        }
    }
}

#[test]
fn unordered_paths_are_exactly_where_the_corpus_says() {
    let graph = common::corpus();
    let path = |name: &str| {
        let idx = common::fixture_index(&graph, name);
        unordered_paths(graph.functions_in(idx)[0])
    };
    let f = |s: &str| Step::Field(s.to_string());
    assert_eq!(path("p3"), vec![vec![], vec![Step::Each, f("orders")]]);
    assert_eq!(path("p5a"), vec![vec![]]);
    assert_eq!(path("p8"), vec![vec![Step::Err]]);
    assert_eq!(path("apply_diff"), vec![vec![Step::Err], vec![Step::Ok]]);
    assert_eq!(
        path("p14"),
        vec![vec![f("added")], vec![f("removed")], vec![f("changed")]]
    );
    for name in ["p1", "p2", "p4", "p5b", "p6", "p7"] {
        assert!(
            path(name).is_empty(),
            "{name} makes no order claim of `unordered`"
        );
    }
}

#[test]
fn order_is_enforced_where_nothing_says_unordered() {
    let a = Value::List(vec![Value::Int(1), Value::Int(2)]);
    let b = Value::List(vec![Value::Int(2), Value::Int(1)]);
    assert!(compare(&a, &b, &[]).is_err());
    assert!(compare(&a, &b, &[vec![]]).is_ok());
    // A nested unordered list under an ordered one.
    let rec = |xs: Vec<i128>| {
        Value::record(vec![(
            "orders".into(),
            Value::List(xs.into_iter().map(Value::Int).collect()),
        )])
    };
    let x = Value::List(vec![rec(vec![1, 2]), rec(vec![3])]);
    let y = Value::List(vec![rec(vec![2, 1]), rec(vec![3])]);
    let z = Value::List(vec![rec(vec![3]), rec(vec![2, 1])]);
    let inner = vec![vec![Step::Each, Step::Field("orders".into())]];
    assert!(compare(&x, &y, &inner).is_ok());
    assert!(compare(&x, &z, &inner).is_err());
    assert!(compare(&x, &z, &[vec![], inner[0].clone()]).is_ok());
}

#[test]
fn a_performance_with_the_wrong_order_fails_p4_but_not_p8() {
    let graph = common::corpus();
    let dir = common::corpus_dir().join("mutants");
    // P8 with the error list reversed: `errs unordered`, so it passes.
    let idx = common::fixture_index(&graph, "p8");
    let mut c = Checker::new(
        &graph,
        idx,
        &dir.join("p8_errors_reordered.py"),
        &common::corpus_dir(),
        surety::TIMEOUT,
    )
    .unwrap();
    for r in c.check_examples() {
        assert_eq!(
            r.verdict,
            Verdict::Pass,
            "p8 reordered example {}",
            r.index + 1
        );
    }
    // P4 with the output reversed: order is claimed, so it fails.
    let idx = common::fixture_index(&graph, "p4");
    let mut c = Checker::new(
        &graph,
        idx,
        &dir.join("p4_reversed.py"),
        &common::corpus_dir(),
        surety::TIMEOUT,
    )
    .unwrap();
    let reports = c.check_examples();
    assert!(
        reports
            .iter()
            .any(|r| matches!(r.verdict, Verdict::Mismatch(_))),
        "p4 reversed should mismatch on an ordered comparison"
    );
}
