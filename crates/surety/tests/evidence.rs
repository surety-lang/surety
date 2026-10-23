//! Slice 2, acceptance test 2: the evidence record. Every fixture passes
//! with its hand-written performance under the generated tier, the record
//! carries every kind, and a property false only on generated inputs is a
//! finding with the refinement that would close it.

mod common;

use surety_check::{Budget, Checker, PropVerdict, Report, Value};
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
        // Every hole the corpus had is closed in the fixtures and tested
        // on copies below; apply_diff's conflict has its own error now.
        assert_eq!(
            r.claim_findings(),
            0,
            "{name}: no finding expected:\n{text}"
        );
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
fn p7_without_its_refinement_has_no_passing_output_on_shared_ids() {
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
    assert!(text.contains("closed by users: User[unique id]"), "{text}");
    assert!(
        text.starts_with("enrich_orders  NOT ACCEPTED  obligation: no passing output on"),
        "{text}"
    );
    assert!(
        text.lines()
            .next()
            .unwrap()
            .ends_with("performance: passes"),
        "{text}"
    );
    let json = surety::render_json(&r);
    // Found by a bounded search, not proved: the type is `no_passing_output`.
    assert!(json.contains("\"kind\":\"no_passing_output\""), "{json}");
    assert!(!json.contains("\"unsatisfiable_on\""), "{json}");
    assert!(
        json.contains("\"suggest\":\"users: User[unique id]\""),
        "{json}"
    );
}

/// P7 before the author's second fix: two orders sharing an id break both
/// `exactly_one` properties and both frame pairs; the report names the
/// refinement on `orders`.
#[test]
fn p7_without_its_orders_refinement_has_no_passing_output_on_shared_order_ids() {
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
fn p6_without_its_refinement_has_no_passing_output_on_shared_item_ids() {
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
        text.contains("closed by added: Item[unique sku] in type Diff"),
        "{text}"
    );
    assert!(
        text.contains("  FORCED OUTSIDE  the output type:"),
        "{text}"
    );
    let json = surety::render_json(&r);
    assert!(json.contains("\"kind\":\"boundary\""), "{json}");
}

/// apply_diff's seventh example, a sku in both diff.removed and
/// diff.changed, is answered by the fourth error since the author's fix.
/// `Err([conflicting_sku_err])` holds every property on that input; an
/// `Ok` of either shape and an empty `Err` each fail one.
#[test]
fn apply_diff_answers_a_sku_both_removed_and_changed_with_an_error() {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "apply_diff");
    let perf = common::corpus_dir().join("apply_diff.py");
    let mut c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::none();
    let fun = c.fun;
    let conflict = fun
        .examples
        .iter()
        .find(|e| e.note.as_deref() == Some("a sku in both removed and changed"))
        .expect("the author's example");
    let inputs = c.example_inputs(conflict).unwrap();
    let answer = Value::Err(Box::new(Value::List(vec![
        c.named_value("conflicting_sku_err").unwrap(),
    ])));
    for p in &fun.properties {
        assert_eq!(
            c.holds(&p.expr, &inputs, &answer),
            Ok(true),
            "the answer fails {}",
            p.text
        );
    }
    let item = Value::record(vec![
        ("sku".into(), Value::Text("A".into())),
        ("qty".into(), Value::Int(6)),
    ]);
    let wrong = [
        (
            "Ok([])",
            Value::Ok(Box::new(Value::List(vec![]))),
            "exactly_one Ok(_) in out",
        ),
        (
            "Ok([A: 6])",
            Value::Ok(Box::new(Value::List(vec![item]))),
            "exactly_one Ok(_) in out",
        ),
        (
            "Err([])",
            Value::Err(Box::new(Value::List(vec![]))),
            "conflicting_sku_err in errs",
        ),
    ];
    for (label, out, fails_text) in wrong {
        let fails = fun
            .properties
            .iter()
            .find(|p| p.text.contains(fails_text))
            .unwrap();
        assert_eq!(
            c.holds(&fails.expr, &inputs, &out),
            Ok(false),
            "{label} should fail {fails_text}"
        );
    }
    let r = c.check();
    assert!(
        r.passed(),
        "the performance answers the example:\n{}",
        surety::render(&r)
    );
}

/// The binders decision: on two identical orders, P5b's "every same-email
/// leftover is later" holds, because `leftover` assigns the output's order
/// the first occurrence and the leftover is the second. Three copies too.
#[test]
fn p5b_holds_on_identical_orders_under_occurrence_semantics() {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "p5b");
    let perf = common::corpus_dir().join("p5b.py");
    let mut c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::none();
    let fun = c.fun;
    let before = fun
        .properties
        .iter()
        .find(|p| p.text.contains("o before l in orders"))
        .unwrap();
    let x = Value::record(vec![
        ("amount".into(), Value::Int(2)),
        ("customer_email".into(), Value::Text("j@test.com".into())),
    ]);
    for copies in [2usize, 3] {
        let orders = Value::List(vec![x.clone(); copies]);
        let out = Value::List(vec![x.clone()]);
        assert_eq!(
            c.holds(&before.expr, std::slice::from_ref(&orders), &out),
            Ok(true),
            "{copies} identical orders"
        );
        for p in &fun.properties {
            assert_eq!(
                c.holds(&p.expr, std::slice::from_ref(&orders), &out),
                Ok(true),
                "{copies} copies: {}",
                p.text
            );
        }
    }
    // Keeping a later copy is still wrong: the kept order's position is
    // the earliest occurrence, and the leftover is later than it, so the
    // property cannot tell the copies apart by value; it is the `same_order_as`
    // and the other clauses that pin which copy. Two different orders, kept
    // in the wrong order, still fail.
    let y = Value::record(vec![
        ("amount".into(), Value::Int(4)),
        ("customer_email".into(), Value::Text("j@test.com".into())),
    ]);
    let orders = Value::List(vec![x.clone(), y.clone()]);
    assert_eq!(
        c.holds(
            &before.expr,
            std::slice::from_ref(&orders),
            &Value::List(vec![x.clone()])
        ),
        Ok(true)
    );
    assert_eq!(
        c.holds(&before.expr, &[orders], &Value::List(vec![y])),
        Ok(false),
        "keeping the later order is caught"
    );
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

/// Slice 3, report-reading fix 1: every pass line says how many runs and
/// that nothing was found, so "0 findings" is stated rather than implied.
#[test]
fn pass_lines_carry_the_findings_count() {
    let r = check("p2", Budget::default());
    let text = surety::render(&r);
    for p in &r.properties {
        let line = format!(
            "property (line {}): {}  ({} runs, 0 findings)",
            p.line, p.text, p.runs
        );
        assert!(text.contains(&line), "{line:?} not in:\n{text}");
    }
    assert!(
        !text.contains(" runs)\n"),
        "no pass line without a findings count:\n{text}"
    );
    let r = check("p7", Budget::default());
    let text = surety::render(&r);
    for f in &r.frame {
        assert!(
            text.contains(&format!("({} runs, 0 findings)", f.runs)),
            "{text}"
        );
    }
}

/// Slice 3, report-reading fix 2: `check` runs the underdetermination check
/// and carries its verdict, so one report says what the properties leave
/// open. A slipping mutant is a finding there, binding under `--strict`.
#[test]
fn check_carries_the_mutant_verdict() {
    let r = check("p2", Budget::default());
    let m = r.mutants.as_ref().expect("the budget runs the mutants");
    assert_eq!(m.slipped().len(), 0);
    let text = surety::render(&r);
    assert!(
        text.contains(
            "    underdetermination: 6 of 14 mutants apply; 0 slipped; 0 underdetermined inputs"
        ),
        "{text}"
    );
    assert!(
        text.starts_with("get_small_orders  ACCEPTED  examples 2/2; 4 clauses clean over"),
        "{text}"
    );
    assert!(r.strict_passed(), "{text}");

    let r = check_copy(
        "p2",
        "\tall o leftover in orders: o.amount > 100\n",
        "",
        &[],
    );
    let text = surety::render(&r);
    assert!(text.contains("  SLIPPED  empty:"), "{text}");
    assert!(text.contains("caught by example 1 only"), "{text}");
    assert!(text.contains("; 1 slipped (empty); "), "{text}");
    assert!(
        text.starts_with("get_small_orders  NOT ACCEPTED  obligation: 1 mutant slipped (empty)"),
        "{text}"
    );
    assert_eq!(
        r.underdetermination_findings(),
        1 + r.mutants.as_ref().unwrap().search.len()
    );
    assert!(
        r.passed() && !r.strict_passed(),
        "a slipping mutant is a finding under --strict"
    );
    let json = surety::render_json(&r);
    let slipped_at = json
        .find("\"kind\":\"mutant_slipped\"")
        .expect("a mutant_slipped object");
    let evidence_at = json.find("\"kind\":\"evidence\"").unwrap();
    assert!(slipped_at < evidence_at, "findings come before the record");
    assert!(json.contains("\"underdetermination\":{"), "{json}");
    assert!(
        !json.contains("\"kind\":\"mutant_caught\""),
        "caught mutants belong to surety mutants, not check"
    );

    let r = check("p2", Budget::none());
    assert!(r.mutants.is_none());
    assert!(surety::render(&r).contains("underdetermination: not run"));
}
