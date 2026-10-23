//! Slice 2, acceptance test 1: generated inputs. For each of the eleven,
//! the inputs are deterministic, pass the input boundary, respect the cap
//! with every top-level list length represented, repeat no example input,
//! and draw on the obligation's own literals.

mod common;

use surety_check::value::{from_json_at, to_json};
use surety_check::{Checker, Value};
use surety_syntax::Type;

/// Slice 5, acceptance test 4: a threshold written as arithmetic over the
/// inputs, `page * size > len(items)`, is crossed from both sides by
/// generated inputs with no example, since the base integers are the
/// lengths the generator gives lists; and through `check`, a performance
/// that branches on it passes while the mutant returning 0 is caught, so
/// both branches of the implication were taken.
#[test]
fn a_computed_threshold_is_crossed_without_an_example() {
    let text = "paginate(items: Int[], page: Int, size: Int) -> Int\n\tpage * size > len(items) -> out == 1\n\tpage * size <= len(items) -> out == 0\n";
    let graph = surety_syntax::Graph::single("paginate", surety_syntax::parse(text).unwrap());
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("threshold");
    std::fs::create_dir_all(&dir).unwrap();
    let perf = dir.join("paginate.py");
    std::fs::write(
        &perf,
        "def paginate(items, page, size):\n    return 1 if page * size > len(items) else 0\n",
    )
    .unwrap();
    let c = Checker::new(&graph, 0, &perf, &dir, surety::TIMEOUT).unwrap();
    let generated = c.generated_inputs();
    let (mut over, mut under) = (0, 0);
    for inputs in &generated.inputs {
        let (Value::List(items), Value::Int(page), Value::Int(size)) =
            (&inputs[0], &inputs[1], &inputs[2])
        else {
            panic!("{inputs:?}");
        };
        if page * size > items.len() as i128 {
            over += 1;
        } else {
            under += 1;
        }
    }
    assert!(over > 0 && under > 0, "past the threshold {over}, within it {under}");
    let r = surety::check_file_with(&dir.join("paginate.sur"), &perf, surety_check::Budget::default())
        .or_else(|_| {
            std::fs::write(dir.join("paginate.sur"), text).unwrap();
            surety::check_file_with(&dir.join("paginate.sur"), &perf, surety_check::Budget::default())
        })
        .unwrap();
    let report = surety::render(&r);
    // With no examples the binding tier is skipped, so the verdict line,
    // not `passed`, says the generated tier was clean.
    assert!(surety::verdict_line(&r).contains("ACCEPTED"), "{report}");
    assert_eq!(r.findings(), 0, "{report}");
    let m = r.mutants.as_ref().unwrap();
    let empty = m
        .mutants
        .iter()
        .find(|x| x.mutant == surety_check::Mutant::Empty)
        .unwrap();
    assert!(empty.caught_inputs > 0, "{report}");
}

fn checker<'g>(graph: &'g surety_syntax::Graph, name: &str) -> Checker<'g> {
    let idx = common::fixture_index(graph, name);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    Checker::new(graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap()
}

#[test]
fn generated_inputs_are_deterministic_valid_and_bounded() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let c = checker(&graph, name);
        let a = c.generated_inputs();
        let b = c.generated_inputs();
        assert_eq!(
            a.inputs.len(),
            b.inputs.len(),
            "{name}: two generations differ in size"
        );
        for (x, y) in a.inputs.iter().zip(&b.inputs) {
            assert!(x == y, "{name}: two generations differ in order or content");
        }
        assert!(!a.inputs.is_empty(), "{name}: nothing generated");
        assert!(
            a.inputs.len() <= c.budget.generated,
            "{name}: {} inputs over the cap of {}",
            a.inputs.len(),
            c.budget.generated
        );
        assert_eq!(
            a.buckets.iter().map(|b| b.taken).sum::<usize>(),
            a.inputs.len(),
            "{name}: the buckets do not account for the inputs"
        );
        for inputs in &a.inputs {
            assert_eq!(inputs.len(), c.scope.inputs.len());
            for (v, (n, t)) in inputs.iter().zip(&c.scope.inputs) {
                from_json_at(&to_json(v), t, n).unwrap_or_else(|e| {
                    panic!("{name}: a generated input fails the boundary: {e}")
                });
            }
        }
        // Every top-level list length from 0 to 4 is in the check's input
        // set: the examples and the generated inputs together (P2's empty
        // list is an example, and examples are not generated again).
        let examples = c.example_input_set();
        let all: Vec<&Vec<Value>> = examples.iter().chain(a.inputs.iter()).collect();
        for (i, (n, t)) in c.scope.inputs.iter().enumerate() {
            if let Type::List(..) = t {
                for len in 0..=4 {
                    assert!(
                        all.iter()
                            .any(|row| matches!(&row[i], Value::List(xs) if xs.len() == len)),
                        "{name}: no input with `{n}` of length {len}"
                    );
                }
            }
        }
        for e in &examples {
            assert!(
                !a.inputs.iter().any(|g| g == e),
                "{name}: an example input was generated again"
            );
        }
    }
}

#[test]
fn the_domain_carries_the_obligations_literals() {
    let graph = common::corpus();
    let p2 = checker(&graph, "p2").domain();
    for i in [4, 52, 99, 100, 42143] {
        assert!(p2.ints.contains(&i), "p2 Int literal {i} missing");
    }
    assert!(p2.texts.contains(&"test@test.com".to_string()));
    let generated = checker(&graph, "p2").generated_inputs();
    for i in [4, 52, 99, 100, 42143] {
        let seen = generated.inputs.iter().any(|row| match &row[0] {
            Value::List(xs) => xs.iter().any(|o| o.field("amount") == Some(&Value::Int(i))),
            _ => false,
        });
        assert!(seen, "p2: no generated order with amount {i}");
    }
    let p4 = checker(&graph, "p4").domain();
    for f in [4.0, 4.3, 4.5] {
        assert!(p4.floats.contains(&f), "p4 Float literal {f} missing");
    }
    let p5a = checker(&graph, "p5a").domain();
    let lits: Vec<i128> = [
        "2026-10-19T14:19:01Z",
        "2026-09-19T14:19:01Z",
        "2025-09-19T14:19:01Z",
    ]
    .iter()
    .map(|t| surety_check::value::parse_rfc3339(t).unwrap())
    .collect();
    for l in &lits {
        assert!(
            p5a.times.iter().any(|(i, _)| i == l),
            "p5a Time literal missing"
        );
    }
    let min = p5a.times.iter().map(|(i, _)| *i).min().unwrap();
    let max = p5a.times.iter().map(|(i, _)| *i).max().unwrap();
    assert!(min < *lits.iter().min().unwrap(), "no earlier instant");
    assert!(max > *lits.iter().max().unwrap(), "no later instant");
    let p8 = checker(&graph, "p8").domain();
    for i in [12, 13, 120, 121] {
        assert!(p8.ints.contains(&i), "p8 boundary literal {i} missing");
    }
    assert!(
        p8.texts.contains(&"e-email-shape".to_string()),
        "p8 named value's code missing from the Text domain"
    );
    let p7 = checker(&graph, "p7").domain();
    for id in ["o1", "u1a", "u2b", "k0", "k3"] {
        assert!(p7.ids.contains(&id.to_string()), "p7 Id {id} missing");
    }
}

#[test]
fn small_buckets_are_enumerated_exhaustively() {
    let graph = common::corpus();
    let generated = checker(&graph, "p2").generated_inputs();
    let b0 = generated.buckets.iter().find(|b| b.lengths == [0]).unwrap();
    assert!(b0.exhaustive);
    assert_eq!(
        b0.taken, 0,
        "the empty list is an example and is not repeated"
    );
    let b1 = generated.buckets.iter().find(|b| b.lengths == [1]).unwrap();
    // P2's integers: the base 0 to 4; its literal 100 with its neighbours
    // 99 and 101; the example amounts 4, 42143, 52 and 99 (4 and 99 once);
    // ten in all, times four texts.
    assert!(b1.exhaustive, "40 one-element lists fit the bucket");
    assert_eq!(b1.taken, 40);
    let b2 = generated.buckets.iter().find(|b| b.lengths == [2]).unwrap();
    assert!(!b2.exhaustive);
    assert_eq!(b2.space, 40 * 40);
    assert!(
        b2.taken > 100,
        "the quota unused by small buckets carries forward: {}",
        b2.taken
    );
}
