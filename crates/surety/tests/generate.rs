//! Slice 2, acceptance test 1: generated inputs. For each of the eleven,
//! the inputs are deterministic, pass the input boundary, respect the cap
//! with every top-level list length represented, repeat no example input,
//! and draw on the obligation's own literals.

mod common;

use surety_check::value::{from_json_at, to_json};
use surety_check::{Checker, Value};
use surety_syntax::Type;

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
    assert!(b1.exhaustive, "32 one-element lists fit the bucket");
    assert_eq!(b1.taken, 32);
    let b2 = generated.buckets.iter().find(|b| b.lengths == [2]).unwrap();
    assert!(!b2.exhaustive);
    assert_eq!(b2.space, 32 * 32);
    assert!(
        b2.taken > 100,
        "the quota unused by small buckets carries forward: {}",
        b2.taken
    );
}
