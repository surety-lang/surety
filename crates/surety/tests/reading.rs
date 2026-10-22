//! Slice 3, the second round of report-reading items. The test is the P7
//! and P5b snapshots in the closed-loop run folder, the morning's
//! obligations before the day's fixes, against the model-written
//! performances: a cold reader should say which party each finding belongs
//! to without reading past the first line.

mod common;

use std::path::{Path, PathBuf};

use surety_check::{Budget, Checker, Party};

fn run_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/closed-loop/runs/20261021-081212-claude-opus-5-5")
}

fn snapshot(name: &str) -> surety_check::Report {
    let dir = run_dir();
    let graph = surety::load_dir(&dir).expect("the run folder parses");
    let idx = graph.index_of(name).unwrap();
    let perf = dir.join(format!("{name}.py"));
    let mut c = Checker::new(&graph, idx, &perf, &dir, surety::TIMEOUT).unwrap();
    c.budget = Budget::default();
    c.check()
}

#[test]
fn p7_snapshot_blames_the_obligation_in_one_line() {
    let r = snapshot("p7");
    let text = surety::render(&r);
    let first = text.lines().next().unwrap();
    assert!(
        first.starts_with("enrich_orders  NOT ACCEPTED  obligation: unsatisfiable on "),
        "{first}"
    );
    assert!(
        first.contains("closed by orders: Order[unique id]"),
        "{first}"
    );
    assert!(first.ends_with("  performance: passes"), "{first}");
    // Four clauses, one cause, one block.
    assert_eq!(r.causes.len(), 1, "{text}");
    assert_eq!(r.causes[0].party, Party::Obligation);
    assert_eq!(r.causes[0].clauses.len(), 4);
    assert_eq!(
        text.matches("  UNSATISFIABLE ON  cause 1:").count(),
        1,
        "{text}"
    );
    assert_eq!(
        text.matches("cause 1)").count(),
        4,
        "four claim lines point at the one cause:\n{text}"
    );
    assert!(text.contains("4 clauses: property (line 12), property (line 17), frame o ~ i_o (line 13), frame i_o ~ o (line 18)"), "{text}");
    assert!(
        text.contains("no output among"),
        "the attribution says it searched:\n{text}"
    );
    // The record agrees with the body.
    assert!(text.contains("    properties:       2 of 6 clean over 503 runs; 4 with findings, 1 cause, the obligation's"), "{text}");
    assert!(text.contains("    generated:        500 inputs: lists of length 0 to 4 over 2 list inputs, 25 length combinations"), "{text}");
    assert!(
        !text.contains("0x0:"),
        "no bucket dump without --full:\n{text}"
    );
    assert!(
        surety::render_with(&r, true).contains("    buckets:          0x0: all 1"),
        "the dump is there with --full"
    );
    let json = surety::render_json(&r);
    assert!(
        json.contains("\"kind\":\"unsatisfiable_on\"") && json.contains("\"party\":\"obligation\""),
        "{json}"
    );
    assert!(json.contains("\"verdict\":\"NOT ACCEPTED\""), "{json}");
}

#[test]
fn p5b_snapshot_collapses_the_hole_and_shows_both_outputs() {
    let r = snapshot("p5b");
    let text = surety::render(&r);
    let first = text.lines().next().unwrap();
    assert!(
        first.starts_with(
            "get_first_occurrence_per_email  NOT ACCEPTED  obligation: 1 mutant slipped (empty); "
        ),
        "{first}"
    );
    assert!(first.contains("underdetermined inputs, 1 gap"), "{first}");
    assert!(first.ends_with("  performance: passes"), "{first}");
    let m = r.mutants.as_ref().unwrap();
    assert!(m.open().len() >= 2);
    assert!(
        m.open()
            .iter()
            .all(|u| u.same_as == Some(surety_check::Mutant::Empty)),
        "{text}"
    );
    assert_eq!(
        text.matches("  UNDERDETERMINED  ").count(),
        1,
        "one block for one hole:\n{text}"
    );
    // The gap says where the two outputs differ, how many inputs show it,
    // and that the slipped mutant is the same hole.
    assert!(
        text.contains("  UNDERDETERMINED  out can have ")
            && text.contains(" inputs show it; `empty` returns one of the two; the smallest:\n"),
        "{text}"
    );
    // Both outputs, in full, on their own lines, under the smallest input, uncut.
    let block_at = text.find("  UNDERDETERMINED  ").unwrap();
    let block = &text[block_at
        ..text[block_at..]
            .find("  evidence")
            .map_or(text.len(), |e| block_at + e)];
    let lines: Vec<&str> = block.lines().collect();
    assert!(lines[1].starts_with("        in: ["), "{block}");
    assert!(
        lines[2].starts_with("        out: ") && lines[3].starts_with("        out: "),
        "{block}"
    );
    assert!(lines[3].ends_with("# also passes every clause"), "{block}");
    assert!(!block.contains("\u{2026}"), "nothing cut:\n{block}");
    // The slipped mutant shows the example's output beside its own.
    assert!(text.contains("  SLIPPED  empty:"), "{text}");
    assert!(text.contains("        out: []    # empty"), "{text}");
    assert!(
        text.contains("    properties:       5 of 5 clean over 503 runs\n"),
        "{text}"
    );
    assert!(
        text.contains("    underdetermination: 6 of 13 mutants apply; 1 slipped (empty); "),
        "{text}"
    );
}

#[test]
fn p2_snapshot_is_accepted_in_one_line() {
    let r = snapshot("p2");
    let text = surety::render(&r);
    let first = text.lines().next().unwrap();
    assert!(first.starts_with("get_small_orders  ACCEPTED  examples 2/2; 4 clauses clean over 502 runs; 6 of 13 mutants apply, none slipped"), "{first}");
    assert!(r.strict_passed());
}

/// A performance that passes every example and is wrong where the examples
/// do not reach: P8 with age 2 accepted. The obligation is satisfiable
/// there (an `Err` of the right errors passes every clause), so the report
/// blames the performance and shows the output that passes.
#[test]
fn a_hidden_performance_bug_is_the_performances() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("reading");
    std::fs::create_dir_all(&dir).unwrap();
    let perf = dir.join("p8_age_two.py");
    std::fs::write(&perf, r#"import re

EMAIL = {"code": "e-email-shape", "display": "Please provide a valid email"}
AGE = {"code": "e-age-invalid", "display": "You are not old enough or did not enter a valid birthday."}
PASS = {"code": "e-pass-length", "display": "Your password must be at least 12 characters"}


def well_formed_email(e):
    return re.match(r"^[^@\s]+@[^@\s]+\.[^@\s]+$", e) is not None


def validate_signup(p):
    errors = []
    if not well_formed_email(p["email"]):
        errors.append(EMAIL)
    if not (13 <= p["age"] <= 120) and p["age"] != 2:
        errors.append(AGE)
    if len(p["password"]) < 12:
        errors.append(PASS)
    return {"err": errors} if errors else {"ok": p}
"#).unwrap();
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "p8");
    let mut c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::default();
    let r = c.check();
    let text = surety::render(&r);
    assert!(r.passed(), "every example passes:\n{text}");
    assert!(!r.causes.is_empty(), "age 2 is generated:\n{text}");
    let first = text.lines().next().unwrap();
    assert!(
        first.starts_with(
            "validate_signup  NOT ACCEPTED  obligation: nothing found  performance: fails property"
        ),
        "{first}"
    );
    assert!(first.contains("generated inputs"), "{first}");
    for cause in &r.causes {
        assert_eq!(cause.party, Party::Performance, "{text}");
        assert!(
            cause.passing.is_some(),
            "a passing output is shown:\n{text}"
        );
    }
    assert!(text.contains("  PERFORMANCE FAILS  cause 1:"), "{text}");
    assert!(text.contains("    # passes every clause"), "{text}");
    let json = surety::render_json(&r);
    assert!(
        json.contains("\"kind\":\"property_failed\"") && json.contains("\"party\":\"performance\""),
        "{json}"
    );
    assert!(!json.contains("\"kind\":\"unsatisfiable_on\""), "{json}");
}
