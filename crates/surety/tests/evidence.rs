//! Slice 2, acceptance test 2: the evidence record. Every fixture passes
//! with its hand-written performance under the generated tier, the record
//! carries every kind, and a property false only on generated inputs is a
//! finding with the refinement that would close it.

mod common;

use surety_check::{Budget, Checker, PropVerdict, Report};
use surety_syntax::normalize::canonical_file;

fn check(name: &str, budget: Budget) -> Report {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, name);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    let mut c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = budget;
    c.check()
}

#[test]
fn every_fixture_passes_with_a_full_record() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let r = check(name, Budget::default());
        let text = surety::render(&r);
        assert!(r.passed(), "{name}:\n{text}");
        assert!(r.generated_inputs > 0, "{name}: no generated inputs");
        assert!(r.runs > r.example_runs, "{name}: no generated runs");
        for p in &r.properties {
            assert_eq!(p.verdict, PropVerdict::Pass, "{name}: {}", p.text);
            assert_eq!(p.runs, r.runs, "{name}: property run count: {}", p.text);
        }
        for f in &r.frame {
            assert_eq!(f.runs, r.runs, "{name}: frame run count");
        }
        assert_eq!(
            r.determinism.total, r.runs,
            "{name}: every run was run twice"
        );
        assert_eq!(
            r.determinism.matched, r.determinism.total,
            "{name}:\n{text}"
        );
        assert!(r.generated_timeouts.is_empty(), "{name}: timeouts\n{text}");
        assert!(r.generated_raised.is_empty(), "{name}: raised\n{text}");
        assert!(r.generated_boundary.is_empty(), "{name}: boundary\n{text}");
        let i = r
            .inspection
            .as_ref()
            .unwrap_or_else(|| panic!("{name}: no inspection: {:?}", r.inspection_error));
        assert_eq!(i.hash.len(), 64, "{name}: performance hash");
        assert!(i.lines > 0 && i.functions > 0, "{name}: shape");
        let idx = common::fixture_index(&graph, name);
        assert_eq!(r.obligation_hash, canonical_file(&graph, idx).unwrap().hash);
        for line in [
            "  evidence",
            "    types:",
            "    examples:",
            "    properties:",
            "    determinism:",
            "    totality:",
            "    purity:",
            "    shape:",
            "    underdetermination:",
            "    generated:",
        ] {
            assert!(text.contains(line), "{name}: record lacks {line:?}\n{text}");
        }
        match name {
            "p5b" => {
                // The identical-orders residue: `leftover` subtracts by value
                // and `before` finds positions by first occurrence, so two
                // identical orders make "the leftover one is later" false.
                let before = r
                    .properties
                    .iter()
                    .find(|p| p.text.contains("o before l in orders"))
                    .unwrap();
                assert!(before.findings() > 0, "{text}");
                assert_eq!(
                    before.suggest, None,
                    "no refinement fixes identical elements:\n{text}"
                );
                assert!(
                    before
                        .note
                        .as_deref()
                        .unwrap_or("")
                        .contains("repeats an element of `orders`"),
                    "{text}"
                );
                for p in &r.properties {
                    if !p.text.contains("o before l in orders") {
                        assert_eq!(p.findings(), 0, "{name}: {}\n{text}", p.text);
                    }
                }
            }
            // apply_diff's open hole, a sku in both diff.removed and
            // diff.changed, is a coincidence the 500 generated inputs no
            // longer reach now that each list is unique on sku; it is pinned
            // by a hand-built input in the test below.
            // The holes P6, P7 and apply_diff's `added` had are closed in the
            // fixtures and tested on copies below.
            _ => {
                assert_eq!(
                    r.claim_findings(),
                    0,
                    "{name}: no finding expected:\n{text}"
                );
            }
        }
        eprintln!(
            "{name}: findings {} ({} claims, {} raised, {} nondeterministic)",
            r.findings(),
            r.claim_findings(),
            r.generated_raised.len(),
            r.determinism.failures.len()
        );
    }
}

/// A copy of a fixture with one text edit, as a graph of its own (plus the
/// siblings it needs for types), checked against the fixture's own
/// performance under the default budget. The fixture is never edited.
fn check_copy(name: &str, from: &str, to: &str, siblings: &[&str]) -> Report {
    let read =
        |n: &str| std::fs::read_to_string(common::corpus_dir().join(format!("{n}.sur"))).unwrap();
    let src = read(name);
    assert!(src.contains(from), "{name}: {from:?} is not in the fixture");
    let file = surety_syntax::parse(&src.replace(from, to)).unwrap();
    let mut files = vec![(name.to_string(), file)];
    for s in siblings {
        files.push((s.to_string(), surety_syntax::parse(&read(s)).unwrap()));
    }
    let graph = surety_syntax::Graph::new(files);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::default();
    c.check()
}

/// P7 before the author's first fix: the refinement the report suggests
/// is the one the author applied.
#[test]
fn p7_without_its_refinement_is_unsatisfiable_on_shared_ids() {
    let r = check_copy("p7", "users: User[unique id]", "users: User[]", &[]);
    let text = surety::render(&r);
    assert!(r.passed(), "examples still pass:\n{text}");
    assert!(!r.strict_passed(), "strict mode fails on the finding");
    let p = r
        .properties
        .iter()
        .find(|p| p.text.contains("else o.user_name == None"))
        .unwrap();
    assert!(
        p.findings() > 0,
        "the user_name property is false on some generated input:\n{text}"
    );
    assert_eq!(
        p.suggest.as_deref(),
        Some("users: User[unique id]"),
        "{text}"
    );
    assert!(text.contains("suggest users: User[unique id]"));
    let json = surety::render_json(&r);
    assert!(json.contains("\"unsatisfiable_on\""), "{json}");
    assert!(
        json.contains("\"suggest\":\"users: User[unique id]\""),
        "{json}"
    );
}

/// P7 before the author's second fix: two orders sharing an id break both
/// `exactly_one` properties and both frame pairs; the report names the
/// refinement on `orders`.
#[test]
fn p7_without_its_orders_refinement_is_unsatisfiable_on_shared_order_ids() {
    let r = check_copy("p7", "orders: Order[unique id]", "orders: Order[]", &[]);
    let text = surety::render(&r);
    assert!(r.passed() && !r.strict_passed(), "{text}");
    let by_id = r
        .properties
        .iter()
        .find(|p| p.text.contains("exactly_one i_o in orders"))
        .unwrap();
    assert!(by_id.findings() > 0, "{text}");
    assert_eq!(
        by_id.suggest.as_deref(),
        Some("orders: Order[unique id]"),
        "{text}"
    );
    let other_way = r
        .properties
        .iter()
        .find(|p| p.text.contains("exactly_one o in out"))
        .unwrap();
    assert!(other_way.findings() > 0, "{text}");
    assert_eq!(
        other_way.suggest.as_deref(),
        Some("orders: Order[unique id]"),
        "{text}"
    );
    assert!(!r.frame.is_empty());
    for f in &r.frame {
        assert!(f.findings() > 0, "{text}");
        assert_eq!(
            f.suggest.as_deref(),
            Some("orders: Order[unique id]"),
            "{text}"
        );
    }
    let user_name = r
        .properties
        .iter()
        .find(|p| p.text.contains("else o.user_name == None"))
        .unwrap();
    assert_eq!(
        user_name.findings(),
        0,
        "the users case stays closed:\n{text}"
    );
}

/// P6 before the author's fix: two items sharing an id break both
/// `exactly_one` properties; the refinement is on a field of a named type.
#[test]
fn p6_without_its_refinement_is_unsatisfiable_on_shared_item_ids() {
    let r = check_copy(
        "p6",
        "items: OrderItemRow[unique id]",
        "items: OrderItemRow[]",
        &[],
    );
    let text = surety::render(&r);
    assert!(r.passed() && !r.strict_passed(), "{text}");
    let by_id = r
        .properties
        .iter()
        .find(|p| p.text.contains("exactly_one i_row in order.items"))
        .unwrap();
    assert!(by_id.findings() > 0, "{text}");
    assert_eq!(
        by_id.suggest.as_deref(),
        Some("items: OrderItemRow[unique id] in type Order"),
        "{text}"
    );
    let other_way = r
        .properties
        .iter()
        .find(|p| p.text.contains("exactly_one r in out"))
        .unwrap();
    assert!(other_way.findings() > 0, "{text}");
    assert_eq!(
        other_way.suggest.as_deref(),
        Some("items: OrderItemRow[unique id] in type Order"),
        "{text}"
    );
}

/// apply_diff before the author's fix: a sku repeated in `diff.added` forces
/// an output that violates `Item[unique sku]`, a boundary failure the
/// obligation itself causes; the report names the refinement on `added`.
#[test]
fn apply_diff_without_its_added_refinement_forces_a_non_unique_output() {
    let r = check_copy(
        "apply_diff",
        "added: Item[unique sku]",
        "added: Item[]",
        &["p8"],
    );
    let text = surety::render(&r);
    assert!(r.passed() && !r.strict_passed(), "{text}");
    assert!(
        !r.generated_boundary.is_empty(),
        "a repeated sku in diff.added forces a non-unique output:\n{text}"
    );
    assert_eq!(
        r.boundary_suggest.as_deref(),
        Some("added: Item[unique sku] in type Diff"),
        "{text}"
    );
    assert!(
        text.contains("suggest added: Item[unique sku] in type Diff"),
        "{text}"
    );
    let json = surety::render_json(&r);
    assert!(json.contains("\"kind\":\"boundary\""), "{json}");
}

/// apply_diff's open hole: a sku in both diff.removed and diff.changed
/// must be absent from the result and present in it. Every shape of output
/// fails a different property, so no performance can satisfy the
/// obligation on that input. The generated tier reached it twice in 497
/// runs before the author's fix and not at all after, so it is pinned here.
#[test]
fn apply_diff_is_contradictory_when_a_sku_is_both_removed_and_changed() {
    let read =
        |n: &str| std::fs::read_to_string(common::corpus_dir().join(format!("{n}.sur"))).unwrap();
    let src = read("apply_diff")
        + "\n\te:\n\t  note \"sku A removed and changed\"\n\t  in: ([ { sku: \"A\", qty: 5 } ], { added: [], removed: [ { sku: \"A\", qty: 5 } ], changed: [ { sku: \"A\", old_qty: 5, new_qty: 6 } ] })\n\t  out: Ok([])\n";
    let file = surety_syntax::parse(&src).unwrap();
    let p8 = surety_syntax::parse(&read("p8")).unwrap();
    let graph = surety_syntax::Graph::new(vec![
        ("apply_diff".to_string(), file),
        ("p8".to_string(), p8),
    ]);
    graph.check_file(0).unwrap();
    let perf = common::corpus_dir().join("apply_diff.py");
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::none();
    let fun = c.fun;
    let conflict = fun.examples.last().unwrap();
    let inputs = c.example_inputs(conflict).unwrap();
    let outputs = [
        (
            "Ok([])",
            "sk not in res.sku",
            "exactly_one i in res where i.sku == c.sku",
        ),
        (
            "Ok([ { sku: \"A\", qty: 6 } ])",
            "exactly_one i in res where i.sku == c.sku",
            "sk not in res.sku",
        ),
        ("Err([])", "all Ok(res) in out", "exactly_one Ok(_) in out"),
    ];
    for (out, holds_text, fails_text) in outputs {
        let value = {
            let lookup = |n: &str| c.named_value(n);
            let expr = surety_syntax::parse(&format!("f(x: Int) -> Result<Item[unique sku], DiffError[]>\n\te:\n\t\tin: 0\n\t\tout: {out}\n"))
                .map(|f| match &f.decls[0] {
                    surety_syntax::Decl::Function(f) => f.examples[0].output.clone(),
                    _ => unreachable!(),
                })
                .unwrap();
            surety_check::value::from_literal(&expr, c.output_type(), &lookup, None).unwrap()
        };
        let holds = fun
            .properties
            .iter()
            .find(|p| p.text.contains(holds_text))
            .unwrap();
        let fails = fun
            .properties
            .iter()
            .find(|p| p.text.contains(fails_text))
            .unwrap();
        assert_eq!(
            c.holds(&holds.expr, &inputs, &value),
            Ok(true),
            "{out}: {holds_text}"
        );
        assert_eq!(
            c.holds(&fails.expr, &inputs, &value),
            Ok(false),
            "{out}: {fails_text}"
        );
    }
}

#[test]
fn a_zero_budget_is_slice_one() {
    let r = check("p2", Budget::none());
    assert!(r.passed());
    assert_eq!(r.generated_inputs, 0);
    assert_eq!(r.runs, r.example_runs);
    assert_eq!(r.determinism.total, r.example_runs);
    for p in &r.properties {
        assert_eq!(p.runs, r.example_runs);
        assert_eq!(p.findings(), 0);
    }
}
