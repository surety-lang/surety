//! Slice 3, item 4: the behavioural diff. Two performances of one obligation
//! run on the same example and generated inputs; the report is the inputs on
//! which their outputs differ under the order claims, each classed against
//! the obligation. Never a text diff.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use surety_check::{Budget, Class, Clause, DiffReport, Outcome};

fn scratch_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diff");
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn scratch(name: &str, source: &str) -> PathBuf {
    let path = scratch_dir().join(name);
    fs::write(&path, source).unwrap();
    path
}

fn corpus(name: &str) -> PathBuf {
    common::corpus_dir().join(format!("{name}.py"))
}

fn diff(name: &str, old: &Path, new: &Path) -> DiffReport {
    let sur = common::corpus_dir().join(format!("{name}.sur"));
    surety::diff_files(&sur, old, new, Budget::default()).unwrap()
}

const KEEPS_LAST_ON_TIE: &str = r#"def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or o["ordered_at"] <= earliest[email]["ordered_at"]:
            earliest[email] = o
    return list(earliest.values())
"#;

#[test]
fn the_same_performance_twice_agrees_everywhere() {
    let r = diff("p5a", &corpus("p5a"), &corpus("p5a"));
    assert!(r.same(), "{}", surety::render_diff(&r, true));
    assert_eq!((r.examples, r.generated), (3, 500));
    assert_eq!(r.agree, r.inputs());
    assert_eq!(r.old_hash, r.new_hash);
    let text = surety::render_diff(&r, false);
    assert!(
        text.starts_with("get_first_customer_orders  SAME  old and new agree on all 503 inputs"),
        "{text}"
    );
}

#[test]
fn a_mutant_is_a_regression_and_the_examples_come_first() {
    let r = diff(
        "p5a",
        &corpus("p5a"),
        &common::corpus_dir().join("mutants/p5a_keeps_latest.py"),
    );
    assert!(!r.same());
    assert_eq!(r.count(Class::Regression), r.differences.len());
    let first = &r.differences[0];
    assert_eq!(first.example, Some(1), "the example inputs come first");
    assert!(first.old_fails.is_empty());
    assert!(
        first
            .new_fails
            .iter()
            .any(|c| matches!(c, Clause::Property { line: 7, .. })),
        "the leftover clause names the regression: {:?}",
        first.new_fails
    );
    let text = surety::render_diff(&r, false);
    assert!(
        text.starts_with("get_first_customer_orders  DIFFERS  on "),
        "{text}"
    );
    assert!(text.contains("REGRESSION  new fails line 7 ("), "{text}");
    assert!(text.contains("# example 1"), "{text}");
    assert!(text.contains("3 of "), "three shown by default: {text}");
    assert_eq!(
        surety::render_diff(&r, true).matches("REGRESSION").count(),
        r.differences.len(),
        "--full shows every difference"
    );
}

#[test]
fn order_claims_decide_whether_an_order_difference_counts() {
    // P5a's `out unordered`: reversing the output is the same behaviour.
    let reversed = scratch(
        "p5a_reversed.py",
        r#"def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or o["ordered_at"] < earliest[email]["ordered_at"]:
            earliest[email] = o
    return list(reversed(list(earliest.values())))
"#,
    );
    let r = diff("p5a", &corpus("p5a"), &reversed);
    assert!(r.same(), "{}", surety::render_diff(&r, true));

    // P2's `out same_order_as orders`: reversing the output is a regression.
    let reversed = scratch(
        "p2_reversed.py",
        "def get_small_orders(orders):\n    return [o for o in orders if o[\"amount\"] <= 100][::-1]\n",
    );
    let r = diff("p2", &corpus("p2"), &reversed);
    assert!(!r.same());
    assert_eq!(r.count(Class::Regression), r.differences.len());
    assert!(r.differences.iter().all(|d| {
        d.new_fails
            .iter()
            .any(|c| matches!(c, Clause::Property { text, .. } if text.contains("same_order_as")))
    }));
}

#[test]
fn a_side_that_raises_is_a_difference_not_a_crash() {
    let raises = scratch(
        "p5a_raises.py",
        r#"def get_first_customer_orders(orders):
    if len(orders) >= 2:
        raise ValueError("two is too many")
    return list(orders)
"#,
    );
    let r = diff("p5a", &corpus("p5a"), &raises);
    assert!(!r.same());
    assert!(r.differences.iter().all(|d| d.class == Class::Regression));
    assert!(
        r.differences
            .iter()
            .any(|d| matches!(d.new, Outcome::Raised(_))),
        "{}",
        surety::render_diff(&r, true)
    );
    let text = surety::render_diff(&r, false);
    assert!(
        text.contains("REGRESSION  new raised; old passes"),
        "{text}"
    );
    assert!(
        text.contains("new: raised: ValueError: two is too many"),
        "{text}"
    );
}

#[test]
fn both_within_the_obligation_is_accepted_when_an_accept_line_covers_it() {
    let last = scratch("p5a_keeps_last_on_tie.py", KEEPS_LAST_ON_TIE);
    let r = diff("p5a", &corpus("p5a"), &last);
    assert!(!r.same(), "ties exist among the generated inputs");
    assert_eq!(r.count(Class::Within), r.differences.len());
    assert_eq!(r.accepted(), r.differences.len());
    assert_eq!(r.undecided(), 0);
    assert!(r.differences.iter().all(|d| d.accept == Some(15)));
    let text = surety::render_diff(&r, false);
    assert!(
        text.contains("WITHIN THE OBLIGATION  both outputs pass every clause; accept (line 15) covers this input"),
        "{text}"
    );
    assert!(text.contains("accepted, 0 undecided"), "{text}");
}

#[test]
fn both_within_the_obligation_is_undecided_without_an_accept_line() {
    // P5a without its accept line, as a scratch copy: the same tie inputs
    // are then ones the obligation does not decide.
    let src = fs::read_to_string(common::corpus_dir().join("p5a.sur")).unwrap();
    let without: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("accept "))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_ne!(src, without, "the fixture has an accept line to remove");
    let sur = scratch("p5a.sur", &without);
    let last = scratch("p5a_keeps_last_on_tie.py", KEEPS_LAST_ON_TIE);
    let r = surety::diff_files(&sur, &corpus("p5a"), &last, Budget::default()).unwrap();
    assert!(!r.same());
    assert_eq!(r.count(Class::Within), r.differences.len());
    assert_eq!(r.undecided(), r.differences.len());
    assert_eq!(r.accepted(), 0);
    let text = surety::render_diff(&r, false);
    assert!(
        text.contains(
            "no accept line covers this input: the obligation does not decide between them"
        ),
        "{text}"
    );
}

#[test]
fn json_has_one_object_per_difference_and_a_summary() {
    let r = diff(
        "p5a",
        &corpus("p5a"),
        &common::corpus_dir().join("mutants/p5a_keeps_latest.py"),
    );
    let lines = surety::diff_json_lines(&r);
    assert_eq!(lines.len(), r.differences.len() + 1);
    let first = &lines[0];
    assert_eq!(first["kind"], "difference");
    assert_eq!(first["class"], "regression");
    assert_eq!(first["example"], 1);
    assert!(first["old"]["output"].is_array());
    assert!(first["new"]["output"].is_array());
    assert_eq!(first["new_fails"][0]["line"], 7);
    let summary = lines.last().unwrap();
    assert_eq!(summary["kind"], "summary");
    assert_eq!(summary["verdict"], "differs");
    assert_eq!(summary["regressions"], r.differences.len());
    assert_eq!(summary["inputs"], 503);
    assert_eq!(summary["agree"], r.agree);
    assert_eq!(summary["old"]["hash"], serde_json::json!(r.old_hash));
    let same = diff("p5a", &corpus("p5a"), &corpus("p5a"));
    let lines = surety::diff_json_lines(&same);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["verdict"], "same");
}

/// A side's directory: the performance plus whichever siblings it brings.
fn side_dir(name: &str, files: &[(&str, PathBuf)]) -> PathBuf {
    let dir = scratch_dir().join(name);
    fs::create_dir_all(&dir).unwrap();
    for (target, source) in files {
        fs::copy(source, dir.join(target)).unwrap();
    }
    dir
}

#[test]
fn graph_references_resolve_beside_each_performance() {
    // P14's clause at line 44 calls apply_diff, a graph reference resolved
    // to apply_diff.py beside the performance under judgment. The old side
    // is the corpus; the new side is a mutant in a directory of its own.
    let corpus_dir = common::corpus_dir();
    let mutant = corpus_dir.join("mutants/p14_unchanged_in_changed.py");
    let with = side_dir(
        "p14_with_sibling",
        &[
            ("p14.py", mutant.clone()),
            ("apply_diff.py", corpus_dir.join("apply_diff.py")),
        ],
    );
    let r = diff("p14", &corpus("p14"), &with.join("p14.py"));
    assert!(!r.same());
    assert!(
        r.differences.iter().all(|d| d.old_fails.is_empty()),
        "the old side resolves apply_diff in the corpus"
    );
    assert!(
        r.differences.iter().all(|d| !d
            .new_fails
            .iter()
            .any(|c| matches!(c, Clause::Property { line: 44, .. }))),
        "with apply_diff.py beside it, the new side evaluates line 44: {}",
        surety::render_diff(&r, false)
    );
    // Without a sibling, the new side cannot evaluate line 44, and that
    // counts against the new output, not the old.
    let without = side_dir("p14_without_sibling", &[("p14.py", mutant)]);
    let r = diff("p14", &corpus("p14"), &without.join("p14.py"));
    assert!(!r.same());
    assert!(r.differences.iter().all(|d| d.old_fails.is_empty()));
    assert!(
        r.differences.iter().all(|d| d
            .new_fails
            .iter()
            .any(|c| matches!(c, Clause::Property { line: 44, .. }))),
        "{}",
        surety::render_diff(&r, false)
    );
}

fn run_dir(stamp: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/closed-loop/runs")
        .join(format!("{stamp}-claude-opus-5-5"))
}

/// The two committed closed-loop runs, the morning's baseline against the
/// evening's, diffed on the evening's obligations: the model's second
/// answers differ in source on seven of eleven and in behaviour on one,
/// apply_diff, whose obligation gained a fourth error between the runs.
#[test]
fn the_two_runs_differ_in_behaviour_only_where_the_obligation_changed() {
    let base = run_dir("20261021-081212");
    let new = run_dir("20261021-153427");
    for name in common::FIXTURES {
        let sur = new.join(format!("{name}.sur"));
        let r = surety::diff_files(
            &sur,
            &base.join(format!("{name}.py")),
            &new.join(format!("{name}.py")),
            Budget::default(),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        if name != "apply_diff" {
            assert!(r.same(), "{name}:\n{}", surety::render_diff(&r, false));
            continue;
        }
        assert!(!r.same());
        assert_eq!(
            r.count(Class::Fix),
            r.differences.len(),
            "every difference is a fix: the new performance answers the new error"
        );
        assert!(r.differences.iter().all(|d| !d.old_fails.is_empty()));
        assert!(
            r.differences
                .iter()
                .all(|d| matches!(&d.new, Outcome::Output(surety_check::Value::Err(_)))),
            "the new side answers with an Err list on every differing input"
        );
        let first = &r.differences[0];
        assert_eq!(
            first.example,
            Some(7),
            "the author's seventh example, the conflict, comes first"
        );
        assert!(matches!(
            &first.old,
            Outcome::Output(surety_check::Value::Ok(_))
        ));
    }
}
