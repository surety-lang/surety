//! Acceptance test 6: the frame rule. The report lists, per obligation,
//! the fields preserved by default.

mod common;

use surety_check::{Checker, PropVerdict, Report};

fn check(fixture: &str, perf: &str) -> Report {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, fixture);
    let perf = common::corpus_dir().join(perf);
    let mut checker =
        Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    checker.check()
}

#[test]
fn preserved_fields_are_reported() {
    assert_eq!(
        check("p7", "p7.py").frame_preserved(),
        ["amount", "customer_email", "user_id"]
    );
    assert_eq!(check("p1", "p1.py").frame_preserved(), ["email"]);
    assert_eq!(
        check("p6", "p6.py").frame_preserved(),
        ["amount", "item_id"]
    );
}

#[test]
fn no_pair_forms_where_the_amendment_says_none() {
    // P3's pair shares only `customer_email`, which the match names.
    assert!(check("p3", "p3.py").frame_preserved().is_empty());
    // P8's output is a Result, not a record: no single-input pair.
    assert!(check("p8", "p8.py").frame_preserved().is_empty());
    for name in ["p2", "p4", "p5a", "p5b", "p14", "apply_diff"] {
        assert!(
            check(name, &format!("{name}.py"))
                .frame_preserved()
                .is_empty(),
            "{name}"
        );
    }
}

#[test]
fn implicit_frame_properties_hold_for_the_real_performances() {
    for name in ["p1", "p6", "p7"] {
        let report = check(name, &format!("{name}.py"));
        assert!(!report.frame.is_empty(), "{name}");
        for f in &report.frame {
            assert_eq!(f.verdict, PropVerdict::Pass, "{name}: {}", f.description);
        }
    }
}

#[test]
fn a_performance_that_breaks_the_frame_fails_the_implicit_property() {
    let report = check("p1", "mutants/p1_changes_email.py");
    assert!(
        report
            .frame
            .iter()
            .any(|f| matches!(f.verdict, PropVerdict::Fail { .. })),
        "{}",
        surety::render(&report)
    );
    // The stated properties still hold: only the frame catches it.
    for p in &report.properties {
        assert_eq!(p.verdict, PropVerdict::Pass);
    }
}
