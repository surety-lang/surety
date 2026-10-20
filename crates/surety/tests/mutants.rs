//! Acceptance test 5: hand-written mutants. The checker must fail each on
//! at least one property, and the failure names the property. The control
//! (P8's errors in a different order) must pass.

mod common;

use surety_check::{Checker, PropVerdict, Report};

fn check(fixture: &str, mutant: &str) -> Report {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, fixture);
    let perf = common::corpus_dir()
        .join("mutants")
        .join(format!("{mutant}.py"));
    let mut checker =
        Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    checker.check()
}

/// (fixture, mutant file, the property line that must kill it, as a text fragment)
const MUTANTS: &[(&str, &str, &str)] = &[
    ("p2", "p2_always_empty", "leftover"),
    ("p2", "p2_each_twice", "same_order_as"),
    (
        "p3",
        "p3_extra_empty_group",
        "g.customer_email in orders.customer_email",
    ),
    ("p4", "p4_any_three_among_ties", "leftover"),
    ("p4", "p4_doubled_element", "drawn_from"),
    ("p4", "p4_keeps_first_three_seen", "sorted_by"),
    ("p5a", "p5a_keeps_latest", "o.ordered_at <= l_o.ordered_at"),
    ("p5b", "p5b_keeps_last", "o before l in orders"),
    ("p6", "p6_always_empty", "exactly_one r in out"),
    ("p7", "p7_first_user_name", "else o.user_name == None"),
    ("p8", "p8_ok_modified_on_failed", "p_o == p"),
    ("p8", "p8_extra_unknown_error", "drawn_from valid_errors"),
    ("p14", "p14_unchanged_in_changed", "c.old_qty != c.new_qty"),
    ("p14", "p14_always_empty", "n in out.added"),
    (
        "apply_diff",
        "apply_diff_stops_at_first",
        "added_already_present_sku_err in errs",
    ),
    (
        "apply_diff",
        "apply_diff_always_err_empty",
        "exactly_one Ok(_) in out",
    ),
];

#[test]
fn every_mutant_fails_a_named_property() {
    for (fixture, mutant, killer) in MUTANTS {
        let report = check(fixture, mutant);
        let failed: Vec<_> = report
            .properties
            .iter()
            .filter(|p| matches!(p.verdict, PropVerdict::Fail { .. }))
            .collect();
        assert!(
            !failed.is_empty(),
            "{mutant}: no property failed\n{}",
            surety::render(&report)
        );
        for p in &report.properties {
            assert!(
                !matches!(p.verdict, PropVerdict::Error(_)),
                "{mutant}: property at line {} errored instead of failing: {:?}",
                p.line,
                p.verdict
            );
        }
        assert!(
            failed.iter().any(|p| p.text.contains(killer)),
            "{mutant}: expected the failure to name a property containing {killer:?}; failed: {:?}",
            failed
                .iter()
                .map(|p| (p.line, p.text.as_str()))
                .collect::<Vec<_>>()
        );
        for p in &failed {
            assert!(
                p.line > 0 && !p.text.is_empty(),
                "{mutant}: failure names the property"
            );
        }
    }
}

#[test]
fn the_control_mutant_passes() {
    let report = check("p8", "p8_errors_reordered");
    assert!(
        report.passed(),
        "errors in a different order must pass:\n{}",
        surety::render(&report)
    );
}
