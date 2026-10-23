//! A property may call its own obligation: the metamorphic form. In `check`
//! the call runs the performance under test, so the clause relates two of
//! its outputs; under `mutants` the call means the mutant, so a mutant
//! consistent with itself passes the clause, which is why the clause alone
//! catches nothing and the report lists it under "look here". Problem 11
//! (invoice total) is the fixture: the total of the items is the total of
//! one item carrying their sum, so tax is applied to the subtotal once.

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, Checker, LookKind};
use surety_syntax::load::load_with_siblings;

mod common;

use common::{METAMORPHIC_PY as PERFORMANCE, METAMORPHIC_SUR as OBLIGATION};

fn fixture() -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("metamorphic");
    fs::create_dir_all(&dir).unwrap();
    let sur = dir.join("invoice_total.sur");
    let py = dir.join("invoice_total.py");
    fs::write(&sur, OBLIGATION).unwrap();
    fs::write(&py, PERFORMANCE).unwrap();
    (sur, py)
}

#[test]
fn a_property_may_call_its_own_obligation() {
    let (sur, py) = fixture();
    let (graph, idx) = load_with_siblings(&sur).unwrap_or_else(|e| panic!("{e:?}"));
    let dir = sur.parent().unwrap().to_path_buf();
    let mut c = Checker::new(&graph, idx, &py, &dir, surety::TIMEOUT).unwrap();
    c.budget = Budget::default();

    // In check, the call runs the performance under test. The binding and
    // generated tiers pass; the one finding is the scalar mutant that
    // forgets the tax, `out = sum(items.amount_cents)`: the self-call
    // clause is consistent with it and the two bounds admit it, so only
    // the examples catch it. That hole is why the clause never stands
    // alone; with the tolerance line beside it (`scalar.rs`) it is caught.
    let r = c.check();
    let text = surety::render(&r);
    assert!(r.passed(), "{text}");
    assert_eq!(r.findings(), 1, "{text}");
    assert!(
        text.contains("1 mutant slipped (out = sum(items.amount_cents))"),
        "{text}"
    );
    assert!(text.contains("caught by examples 3, 4 only"), "{text}");
    let self_call = &r.properties[0];
    assert!(self_call.text.contains("invoice_total("), "{text}");
    assert_eq!(self_call.runs, r.runs, "evaluated on every run:\n{text}");
    assert_eq!(self_call.findings(), 0, "{text}");
    assert!(
        r.graph_refs
            .iter()
            .any(|g| g.starts_with("invoice_total ->") && g.ends_with("invoice_total.py")),
        "the reference is to the performance under test: {:?}",
        r.graph_refs
    );

    // Under mutants, the call means the mutant: a mutant consistent with
    // itself passes the clause, so the clause alone catches nothing, the
    // companions do, and the report points the reviewer at it.
    let m = c.mutants();
    let mtext = surety::render_mutants(&m, true);
    assert!(m.applicable() >= 2, "{mtext}");
    let slipped: Vec<String> = m.slipped().iter().map(|s| s.mutant.name()).collect();
    assert_eq!(
        slipped,
        ["out = sum(items.amount_cents)"],
        "the companions catch every mutant but the tax forgotten:\n{mtext}"
    );
    for run in m.mutants.iter().filter(|r| r.inputs_tried > 0) {
        assert!(
            run.slipped
                .iter()
                .any(|c| c.text().contains("invoice_total(")),
            "{}: the self-call clause is never false for a mutant\n{mtext}",
            run.mutant.name()
        );
        assert!(
            !run.kills
                .iter()
                .any(|k| k.clause.text().contains("invoice_total(")),
            "{}\n{mtext}",
            run.mutant.name()
        );
    }
    assert!(
        m.uncaught
            .iter()
            .any(|c| c.text().contains("invoice_total(")),
        "{mtext}"
    );
    assert!(
        r.look_here.iter().any(
            |l| matches!(&l.kind, LookKind::Uncaught(c) if c.text().contains("invoice_total("))
        ),
        "look here names the metamorphic clause:\n{text}"
    );
    assert!(text.contains("no mutant can violate it"), "{text}");
}
