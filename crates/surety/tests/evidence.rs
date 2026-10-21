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
        if name != "apply_diff" {
            assert!(r.generated_boundary.is_empty(), "{name}: boundary\n{text}");
        }
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
            "p7" => {
                // `users: User[unique id]` closed the users case; the orders
                // case is the same shape, and the report names its fix.
                let user_name = r
                    .properties
                    .iter()
                    .find(|p| p.text.contains("else o.user_name == None"))
                    .unwrap();
                assert_eq!(user_name.findings(), 0, "p7 users case is closed:\n{text}");
                let by_id = r
                    .properties
                    .iter()
                    .find(|p| p.text.contains("exactly_one i_o in orders"))
                    .unwrap();
                assert!(by_id.findings() > 0, "p7 orders case is open:\n{text}");
                assert_eq!(
                    by_id.suggest.as_deref(),
                    Some("orders: Order[unique id]"),
                    "{text}"
                );
                assert!(
                    r.frame
                        .iter()
                        .all(|f| f.suggest.as_deref() == Some("orders: Order[unique id]")),
                    "{text}"
                );
            }
            "p6" => {
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
            }
            "p5b" => {
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
            }
            "apply_diff" => {
                let changed = r
                    .properties
                    .iter()
                    .find(|p| p.text.contains("exactly_one i in res where i.sku == c.sku"))
                    .unwrap();
                // A sku in both diff.removed and diff.changed must be absent
                // from the result and present in it: no one-list refinement
                // closes that, so the report offers none.
                assert!(changed.findings() > 0, "{text}");
                assert_eq!(changed.suggest, None, "{text}");
                assert_eq!(changed.note, None, "{text}");
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
                    r.passed() && !r.strict_passed(),
                    "a forced boundary failure is a finding, binding only under --strict"
                );
            }
            "p1" | "p2" | "p3" | "p4" | "p5a" | "p8" | "p14" => {
                assert_eq!(
                    r.claim_findings(),
                    0,
                    "{name}: no finding expected:\n{text}"
                );
            }
            _ => {}
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

/// P7 before its author's fix: the refinement the report suggests is the
/// one the author applied.
#[test]
fn p7_without_its_refinement_is_unsatisfiable_on_shared_ids() {
    let src = std::fs::read_to_string(common::corpus_dir().join("p7.sur")).unwrap();
    let before = "users: User[unique id]";
    assert!(src.contains(before));
    let weakened = src.replace(before, "users: User[]");
    let file = surety_syntax::parse(&weakened).unwrap();
    let graph = surety_syntax::Graph::new(vec![("p7".to_string(), file)]);
    let perf = common::corpus_dir().join("p7.py");
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::default();
    let r = c.check();
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
