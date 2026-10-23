//! Slice 3: the skeleton mutants, built from the frame pairs. The cofounder's
//! acceptance: on the five obligations the first library was blind to (P1,
//! P3, P6, P7, P14), the count of clauses that catch nothing must drop, and
//! every skeleton mutant must die on a named clause.

mod common;

use surety_check::mutants::{Clause, LIBRARY, Mutant, MutantReport, MutantVerdict};
use surety_check::{Budget, Checker};
use surety_syntax::Type;

const BLIND: [&str; 5] = ["p1", "p3", "p6", "p7", "p14"];

fn report(name: &str) -> MutantReport {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, name);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    let mut c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget {
        search_candidates: 0,
        ..Budget::default()
    };
    c.mutants()
}

/// Clauses no mutant in `names` catches.
fn uncaught_by(r: &MutantReport, names: &[&str]) -> usize {
    let total = r
        .mutants
        .iter()
        .flat_map(|m| m.kills.iter().map(|k| k.clause.clone()))
        .chain(r.mutants.iter().flat_map(|m| m.slipped.iter().cloned()))
        .fold(Vec::<Clause>::new(), |mut acc, c| {
            if !acc.contains(&c) {
                acc.push(c);
            }
            acc
        });
    total
        .iter()
        .filter(|c| {
            !r.mutants
                .iter()
                .filter(|m| names.contains(&m.mutant.name().as_str()))
                .any(|m| m.kills.iter().any(|k| k.clause == **c))
        })
        .count()
}

#[test]
fn the_blind_five_see_more_and_every_skeleton_dies_on_a_named_clause() {
    for name in BLIND {
        let r = report(name);
        let text = surety::render_mutants(&r, true);
        let skeletons: Vec<_> = r
            .mutants
            .iter()
            .filter(|m| m.mutant.is_skeleton() && m.verdict != MutantVerdict::NotApplicable)
            .collect();
        assert!(
            !skeletons.is_empty(),
            "{name}: no skeleton applies:\n{text}"
        );
        for m in &skeletons {
            assert_eq!(
                m.verdict,
                MutantVerdict::Caught,
                "{name}: {} slipped:\n{text}",
                m.mutant.name()
            );
            // A named clause, or the output type naming the refinement it
            // failed (P14's `changed` is unique on sku, so a duplicated
            // entry dies at the boundary).
            assert!(
                m.kills.iter().all(|k| k.clause != Clause::Boundary
                    || k.message
                        .as_deref()
                        .is_some_and(|msg| msg.contains("the type says"))),
                "{name}: {} died on nothing named:\n{text}",
                m.mutant.name()
            );
        }
        let before = uncaught_by(&r, &["empty"]);
        let library: Vec<String> = LIBRARY.iter().map(|m| m.name()).collect();
        let after = uncaught_by(&r, &library.iter().map(String::as_str).collect::<Vec<_>>());
        eprintln!(
            "{name}: clauses catching nothing: {before} with `empty` alone, {after} with the library"
        );
        if before > 0 {
            assert!(
                after < before,
                "{name}: the count of clauses that catch nothing must drop ({before} -> {after}):\n{text}"
            );
        } else {
            assert_eq!(
                after, 0,
                "{name}: nothing to drop, nothing may rise:\n{text}"
            );
        }
    }
}

/// P3's per-key skeleton is the wrong group-by: one empty group per
/// distinct email. The nothing-lost clause catches it; the correspondence
/// and uniqueness clauses let it through, which is what they should do.
#[test]
fn p3_per_key_skeleton_is_caught_by_nothing_lost() {
    let r = report("p3");
    let m = r.run(Mutant::SkeletonPerKey).unwrap();
    assert_eq!(m.verdict, MutantVerdict::Caught);
    let kills: Vec<String> = m.kills.iter().map(|k| k.clause.text()).collect();
    assert!(
        kills
            .iter()
            .any(|t| t.contains("flatten(out.orders) same_elements_as orders")),
        "{kills:?}"
    );
    let slipped: Vec<String> = m.slipped.iter().map(|c| c.text()).collect();
    assert!(
        slipped.iter().any(|t| t.contains("exactly_one g in out")),
        "{slipped:?}"
    );
    assert!(
        slipped
            .iter()
            .any(|t| t.contains("unique out.customer_email")),
        "{slipped:?}"
    );
    // The plain skeleton makes one group per order: uniqueness catches it
    // as soon as two orders share an email.
    let plain = r.run(Mutant::Skeleton).unwrap();
    assert!(
        plain
            .kills
            .iter()
            .any(|k| k.clause.text().contains("unique out.customer_email"))
    );
}

/// P7's skeleton copies every order and leaves `user_name` empty: the
/// lookup clause catches it, the correspondence clauses do not.
#[test]
fn p7_skeleton_is_caught_by_the_lookup() {
    let r = report("p7");
    let m = r.run(Mutant::Skeleton).unwrap();
    assert_eq!(m.verdict, MutantVerdict::Caught);
    assert!(
        m.kills
            .iter()
            .any(|k| k.clause.text().contains("o.user_name == u.user_name"))
    );
    assert!(
        m.slipped
            .iter()
            .any(|c| c.text().contains("exactly_one i_o in orders"))
    );
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, "p7");
    let perf = common::corpus_dir().join("p7.py");
    let c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    assert_eq!(c.plan.len(), 1, "one output path");
    assert_eq!(
        c.plan[0].matched,
        vec![("id".to_string(), "id".to_string())]
    );
    assert_eq!(
        c.plan[0].preserved,
        vec!["customer_email", "amount", "user_id"]
    );
    assert!(matches!(c.scope.output.as_ref().unwrap(), Type::List(..)));
}

/// P6's correspondence matches on different field names, and P14's lives
/// inside a record output.
#[test]
fn correspondences_follow_paths_and_renamed_matches() {
    let graph = common::corpus();
    let plan_of = |name: &str| {
        let idx = common::fixture_index(&graph, name);
        let perf = common::corpus_dir().join(format!("{name}.py"));
        Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT)
            .unwrap()
            .plan
    };
    let p6 = plan_of("p6");
    assert_eq!(
        p6[0].matched,
        vec![("line_id".to_string(), "id".to_string())]
    );
    assert_eq!(
        p6[0].input_path,
        vec![surety_check::examples::Step::Field("items".into())]
    );
    assert_eq!(p6[0].preserved, vec!["amount", "item_id"]);
    let p14 = plan_of("p14");
    assert!(
        p14.iter().any(|c| c.out_path
            == vec![surety_check::examples::Step::Field("changed".into())]
            && c.matched == vec![("sku".to_string(), "sku".to_string())]),
        "{p14:?}"
    );
    let p1 = plan_of("p1");
    assert_eq!(p1.len(), 1);
    assert!(
        p1[0].out_path.is_empty() && p1[0].matched.is_empty() && p1[0].preserved == vec!["email"]
    );
}
