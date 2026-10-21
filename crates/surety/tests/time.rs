//! Slice 3, item 3: Time at the boundary. A `Time` reaches a performance as
//! a timezone-aware UTC datetime, whatever text the obligation wrote it in,
//! and comes back as a datetime, as RFC 3339 text, or tagged.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use surety_check::{Budget, Checker, Report};

fn scratch(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("time");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, source).unwrap();
    path
}

/// P5a, whose Order carries a Time, checked against a scratch performance
/// under the default budget, so generated Times cross the boundary too.
fn check_p5a(perf: &Path) -> Report {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "p5a");
    let mut c = Checker::new(&graph, idx, perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::default();
    c.check()
}

fn clean(report: &Report) {
    assert!(
        report.passed() && report.findings() == 0,
        "expected a clean report, got\n{}",
        surety::render(report)
    );
}

/// The performance checks what it is handed and returns the same datetimes.
const DATETIMES: &str = r#"from datetime import datetime, timedelta


def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        t = o["ordered_at"]
        assert isinstance(t, datetime), type(t).__name__
        assert t.utcoffset() == timedelta(0), t
        email = o["customer_email"]
        if email not in earliest or t < earliest[email]["ordered_at"]:
            earliest[email] = o
    return list(earliest.values())
"#;

#[test]
fn a_time_arrives_as_an_aware_utc_datetime_and_returns_as_one() {
    let perf = scratch("datetimes.py", DATETIMES);
    let report = check_p5a(&perf);
    clean(&report);
    assert!(
        report.generated_inputs > 0,
        "generated Times crossed the boundary too"
    );
}

#[test]
fn a_time_may_return_as_rfc3339_text_or_tagged() {
    let text = scratch(
        "text_out.py",
        r#"def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or o["ordered_at"] < earliest[email]["ordered_at"]:
            earliest[email] = o
    return [{**o, "ordered_at": o["ordered_at"].isoformat()} for o in earliest.values()]
"#,
    );
    clean(&check_p5a(&text));
    let tagged = scratch(
        "tagged_out.py",
        r#"def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or o["ordered_at"] < earliest[email]["ordered_at"]:
            earliest[email] = o
    return [{**o, "ordered_at": {"$time": o["ordered_at"].isoformat()}} for o in earliest.values()]
"#,
    );
    clean(&check_p5a(&tagged));
}

#[test]
fn a_naive_datetime_returned_is_read_as_utc() {
    let naive = scratch(
        "naive_out.py",
        r#"def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or o["ordered_at"] < earliest[email]["ordered_at"]:
            earliest[email] = o
    return [{**o, "ordered_at": o["ordered_at"].replace(tzinfo=None)} for o in earliest.values()]
"#,
    );
    clean(&check_p5a(&naive));
}

#[test]
fn an_offset_in_the_obligation_is_the_same_instant() {
    // The first example's input written two hours east of UTC: the same
    // instant, so the corpus performance still returns the expected output.
    let report = common::check_copy(
        "p5a",
        "\"2026-10-19T14:19:01Z\"",
        "\"2026-10-19T16:19:01+02:00\"",
        &[],
    );
    clean(&report);
}

#[test]
fn a_performance_that_still_parses_text_raises() {
    let parses = scratch(
        "parses_text.py",
        r#"from datetime import datetime


def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        when = datetime.fromisoformat(o["ordered_at"].replace("Z", "+00:00"))
        email = o["customer_email"]
        if email not in earliest or when < earliest[email]:
            earliest[email] = when
    return [o for o in orders if o["ordered_at"] == earliest[o["customer_email"]]]
"#,
    );
    let report = check_p5a(&parses);
    assert!(!report.passed(), "a Time is no longer text: {report:#?}");
}
