//! Slice 2, acceptance tests 3 to 7: the mutant library and the search.

mod common;

use surety_check::mutants::{Clause, LIBRARY, Mutant, MutantReport, MutantVerdict, applies};
use surety_check::{Budget, Checker, Value};

fn checker<'g>(graph: &'g surety_syntax::Graph, name: &str) -> Checker<'g> {
    let idx = common::fixture_index(graph, name);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    Checker::new(graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap()
}

fn report(name: &str) -> MutantReport {
    let graph = common::corpus();
    checker(&graph, name).mutants()
}

fn kill_text(r: &MutantReport, m: Mutant) -> Vec<String> {
    r.run(m)
        .map(|run| run.kills.iter().map(|k| k.clause.text()).collect())
        .unwrap_or_default()
}

/// Test 3: which entries apply to which fixture, by the shape of the
/// signature alone.
#[test]
fn applicability_matches_the_table() {
    const ALL: &[&str] = &[
        "empty",
        "identity",
        "drop_one",
        "duplicate_one",
        "append_fabricated",
        "swap_adjacent",
    ];
    const TABLE: &[(&str, &[&str])] = &[
        ("p1", &["empty"]),
        ("p2", ALL),
        ("p3", &["empty"]),
        ("p4", ALL),
        ("p5a", ALL),
        ("p5b", ALL),
        ("p6", &["empty"]),
        ("p7", &["empty"]),
        ("p8", &["empty", "ok_empty", "identity"]),
        (
            "apply_diff",
            &[
                "empty",
                "ok_empty",
                "identity",
                "drop_one",
                "duplicate_one",
                "append_fabricated",
                "swap_adjacent",
            ],
        ),
        ("p14", &["empty"]),
    ];
    let graph = common::corpus();
    for (name, expected) in TABLE {
        let c = checker(&graph, name);
        for m in LIBRARY {
            assert_eq!(
                applies(m, &c.scope),
                expected.contains(&m.name()),
                "{name}: {} applicability",
                m.name()
            );
        }
    }
    // P4's identity on four or five scores fails `Score[0..3]`: caught by the type.
    let p4 = report("p4");
    let identity = p4.run(Mutant::Identity).unwrap();
    assert_eq!(identity.verdict, MutantVerdict::Caught);
    let by_type = identity
        .kills
        .iter()
        .find(|k| k.clause == Clause::Boundary)
        .expect("a boundary kill");
    assert!(by_type.message.as_deref().unwrap_or("").contains("0..3"));
    for name in ["p1", "p3", "p6", "p7", "p14"] {
        let r = report(name);
        assert_eq!(r.applicable(), 1, "{name}: only `empty` applies");
    }
}

/// Test 4: where a library entry corresponds to a hand-written mutant of
/// slice 1, the same property catches it.
#[test]
fn library_entries_are_caught_where_the_hand_mutants_were() {
    let p2 = report("p2");
    assert!(
        kill_text(&p2, Mutant::Empty)
            .iter()
            .any(|t| t.contains("leftover")),
        "{:?}",
        kill_text(&p2, Mutant::Empty)
    );
    assert!(
        kill_text(&p2, Mutant::DropOne)
            .iter()
            .any(|t| t.contains("leftover"))
    );
    assert!(
        kill_text(&p2, Mutant::DuplicateOne)
            .iter()
            .any(|t| t.contains("same_order_as"))
    );
    let p4 = report("p4");
    assert!(
        kill_text(&p4, Mutant::DuplicateOne)
            .iter()
            .any(|t| t.contains("drawn_from"))
    );
    assert!(
        kill_text(&p4, Mutant::SwapAdjacent)
            .iter()
            .any(|t| t.contains("sorted_by"))
    );
    let ad = report("apply_diff");
    assert!(
        kill_text(&ad, Mutant::Empty)
            .iter()
            .any(|t| t.contains("exactly_one Ok(_) in out"))
    );
    assert!(
        kill_text(&ad, Mutant::AppendFabricated)
            .iter()
            .any(|t| t.contains("sk in old.sku or sk in diff.added.sku"))
    );
    for m in LIBRARY {
        if let Some(run) = ad.run(m) {
            assert_ne!(
                run.verdict,
                MutantVerdict::Slipped,
                "apply_diff: {} slipped",
                m.name()
            );
        }
    }
}

/// Test 5: the demo v2 names. Without "nothing lost", the always-empty
/// performance passes every property of the filter obligation and only an
/// example catches it; with it, the property catches it on the example and
/// on generated inputs alike.
#[test]
fn the_demo_v2_names() {
    let src = std::fs::read_to_string(common::corpus_dir().join("p2.sur")).unwrap();
    let line = "\tall o leftover in orders: o.amount > 100\n";
    assert!(src.contains(line));
    let weakened = src.replace(line, "");
    let file = surety_syntax::parse(&weakened).unwrap();
    let graph = surety_syntax::Graph::new(vec![("p2".to_string(), file)]);
    let perf = common::corpus_dir().join("p2.py");
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    let r = c.mutants();
    let text = surety::render_mutants(&r, true);
    let empty = r.run(Mutant::Empty).unwrap();
    assert_eq!(empty.verdict, MutantVerdict::Slipped, "{text}");
    assert_eq!(empty.example_catches, vec![1], "{text}");
    assert_eq!(
        empty.slipped.len(),
        3,
        "the three remaining properties:\n{text}"
    );
    assert!(text.contains("slipped  empty"), "{text}");
    assert!(text.contains("caught by example 1 only"), "{text}");
    assert!(
        text.contains("      out: [") && text.matches("      out: ").count() >= 2,
        "out shown twice:\n{text}"
    );
    assert!(r.underdetermined());

    let full = report("p2");
    let empty = full.run(Mutant::Empty).unwrap();
    assert_eq!(empty.verdict, MutantVerdict::Caught);
    let k = empty
        .kills
        .iter()
        .find(|k| k.clause.text().contains("leftover"))
        .unwrap();
    assert!(
        k.count > 1,
        "caught on the example and on generated inputs: {}",
        k.count
    );
    assert!(!full.underdetermined() || !full.search.is_empty());
}

/// Test 6: the search finds P5a's tie. Two orders with one email and one
/// `ordered_at`: keep-first and keep-second both pass every clause.
#[test]
fn the_search_finds_p5a_tie() {
    let r = report("p5a");
    let text = surety::render_mutants(&r, true);
    assert!(
        !r.search.is_empty(),
        "no underdetermined input found:\n{text}"
    );
    let tie = r.search.iter().find(|u| {
        let Value::List(orders) = &u.inputs[0] else {
            return false;
        };
        orders.iter().enumerate().any(|(i, a)| {
            orders[..i].iter().any(|b| {
                a.field("customer_email") == b.field("customer_email")
                    && a.field("ordered_at") == b.field("ordered_at")
                    && a != b
            })
        })
    });
    let tie =
        tie.unwrap_or_else(|| panic!("no finding with a tie on email and ordered_at:\n{text}"));
    assert!(tie.a != tie.b);
    assert!(text.contains("underdetermined"), "{text}");
    assert!(text.matches("      out: ").count() >= 2, "{text}");
    let json = surety::render_mutants_json(&r);
    // Since the author's `accept` line, P5a's ties are reported as accepted;
    // the search still finds them.
    assert!(
        json.contains("\"kind\":\"accepted\"") || json.contains("\"kind\":\"underdetermined\""),
        "{json}"
    );
}

/// Test 7: a slipping mutant is a report, not a bug. The same mutant slips
/// on an obligation that does not care about order and is caught on one
/// that does.
#[test]
fn a_slipping_mutant_is_a_report_not_a_bug() {
    let src = "shuffle(xs: Int[]) -> Int[]\n\tout same_elements_as xs\n\tout unordered\n";
    let file = surety_syntax::parse(src).unwrap();
    let graph = surety_syntax::Graph::single("shuffle", file);
    let perf = common::corpus_dir().join("shuffle.py");
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget {
        search_candidates: 0,
        ..Budget::default()
    };
    let r = c.mutants();
    let text = surety::render_mutants(&r, true);
    let swap = r.run(Mutant::SwapAdjacent).unwrap();
    assert_eq!(swap.verdict, MutantVerdict::Slipped, "{text}");
    assert!(swap.example_catches.is_empty());
    assert_eq!(
        r.run(Mutant::Identity).unwrap().verdict,
        MutantVerdict::Slipped
    );
    assert_eq!(
        r.run(Mutant::DropOne).unwrap().verdict,
        MutantVerdict::Caught
    );
    let p4 = report("p4");
    let swap = p4.run(Mutant::SwapAdjacent).unwrap();
    assert_eq!(swap.verdict, MutantVerdict::Caught);
    assert!(
        swap.kills
            .iter()
            .any(|k| k.clause.text().contains("sorted_by"))
    );
}

/// P5b before the author's fix: its first line said every output email is
/// an input's, which the always-empty performance satisfies vacuously, so
/// `empty` slipped through every property and only examples caught it.
/// Flipped to in-to-out, the line catches it.
#[test]
fn p5b_without_nothing_lost_admits_the_empty_performance() {
    let src = std::fs::read_to_string(common::corpus_dir().join("p5b.sur")).unwrap();
    let fixed = "\tall o in orders: o.customer_email in out.customer_email\n";
    let hole = "\tall o in out: o.customer_email in orders.customer_email\n";
    assert!(src.contains(fixed), "the fixture carries the fix");
    let file = surety_syntax::parse(&src.replace(fixed, hole)).unwrap();
    let graph = surety_syntax::Graph::new(vec![("p5b".to_string(), file)]);
    let perf = common::corpus_dir().join("p5b.py");
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    let r = c.mutants();
    let text = surety::render_mutants(&r, true);
    let empty = r.run(Mutant::Empty).unwrap();
    assert_eq!(empty.verdict, MutantVerdict::Slipped, "{text}");
    assert_eq!(empty.example_catches, vec![1, 3], "{text}");
    assert_eq!(
        empty.slipped.len(),
        5,
        "every property, the self-call included:\n{text}"
    );
    assert!(
        !r.search.is_empty(),
        "an output missing an email passes every clause:\n{text}"
    );
    assert!(r.underdetermined());

    let fixed_report = report("p5b");
    let empty = fixed_report.run(Mutant::Empty).unwrap();
    assert_eq!(empty.verdict, MutantVerdict::Caught);
    assert!(
        empty.kills.iter().any(|k| k
            .clause
            .text()
            .contains("all o in orders: o.customer_email in out.customer_email")),
        "the in-to-out line is the killer: {:?}",
        kill_text(&fixed_report, Mutant::Empty)
    );
    // Two identical orders yield one output value: the search compares by
    // value, so it reports no choice between them.
    assert!(
        fixed_report.search.is_empty(),
        "no underdetermined input on the fixed obligation:\n{}",
        surety::render_mutants(&fixed_report, true)
    );
}
