//! Whose a generated-input failure is, when the obligation has no examples:
//! no example speaks against the claim, so the search decides, as it does
//! when the examples pass. Before, a claim with no example run was read as
//! contradicted, and every such failure was put on the performance with no
//! search and a line saying a passing output existed.

use std::fs;
use std::path::PathBuf;

use surety_check::Party;

fn scratch(dir: &str, sur: &str, py: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("attribution")
        .join(dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("keep_small.py"), py).unwrap();
    let path = dir.join("keep_small.sur");
    fs::write(&path, sur).unwrap();
    path
}

const CLAUSES: &str = "type Order = { amount: Int }\n\nkeep_small(orders: Order[]) -> Order[]\n\tall o in out: o.amount <= 100\n\tout same_order_as orders\n\tall o leftover in orders: o.amount > 100\n";

const RIGHT: &str =
    "def keep_small(orders):\n    return [o for o in orders if o[\"amount\"] <= 100]\n";

/// A claim about the inputs alone, false on the generated negative amounts
/// whatever the output: the obligation's, found by a search that comes up
/// empty, and the right performance passes.
#[test]
fn no_examples_and_no_passing_output_is_the_obligations() {
    let sur = format!("{CLAUSES}\tall a in orders.amount: a >= 0\n");
    let path = scratch("unsatisfiable", &sur, RIGHT);
    let r = surety::check_file(&path, &path.with_extension("py")).unwrap();
    let text = surety::render(&r);
    assert_eq!(r.causes.len(), 1, "{text}");
    let c = &r.causes[0];
    assert_eq!(c.party, Party::Obligation, "{text}");
    assert!(
        c.passing.is_none() && c.candidates_tried > 0,
        "searched and found none:\n{text}"
    );
    let first = text.lines().next().unwrap();
    assert!(first.ends_with("performance: passes"), "{first}");
    assert!(text.contains("  UNSATISFIABLE ON  cause 1: "), "{text}");
    assert!(!text.contains("PERFORMANCE FAILS"), "{text}");
}

/// A wrong performance on the same clauses, still no examples: the search
/// finds an output that passes, so the failure is the performance's, and
/// only then does the report say a passing output exists.
#[test]
fn no_examples_and_a_passing_output_is_the_performances() {
    let path = scratch(
        "wrong",
        CLAUSES,
        "def keep_small(orders):\n    return orders\n",
    );
    let r = surety::check_file(&path, &path.with_extension("py")).unwrap();
    let text = surety::render(&r);
    assert!(!r.causes.is_empty(), "{text}");
    for c in &r.causes {
        assert_eq!(c.party, Party::Performance, "{text}");
        assert!(
            c.passing.is_some(),
            "the search found the passing output:\n{text}"
        );
    }
    assert!(
        text.contains("; an output that passes every clause exists\n"),
        "{text}"
    );
}

/// The outputs already in hand come before the pool: a list of nine is
/// past what the drawn candidates enumerate and what the pool builds, so
/// before slice 6 no passing output was found for `out == xs` on it; the
/// identity mutant's output is that list, and it passes.
#[test]
fn a_mutant_output_that_passes_is_a_passing_output() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("attribution")
        .join("seeded");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("same.sur");
    fs::write(&path, "same(xs: Int[]) -> Int[]\n\tout == xs\n").unwrap();
    let (graph, idx) = surety::load_with_siblings(&path).unwrap();
    let mut c = surety::checker_for(&graph, idx, &path, &path.with_extension("py")).unwrap();
    let nine = surety_check::Value::List((1..=9).map(surety_check::Value::Int).collect());
    let (found, _) = c.find_passing(std::slice::from_ref(&nine));
    assert_eq!(found, Some(nine));
}

/// The example's own output and the performance's are in hand too. combine
/// joins two fields into a name, which neither the pool nor the library
/// builds: on p1's example input the example's output is found with no
/// performance, and once `check` has run the performance, every generated
/// input it answered has a passing output.
#[test]
fn the_examples_and_the_performances_outputs_are_passing_outputs() {
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let sur = corpus.join("p1.sur");
    let (graph, idx) = surety::load_with_siblings(&sur).unwrap();
    let mut c = surety::checker_for(&graph, idx, &sur, &corpus.join("absent.py")).unwrap();
    let fun = c.fun;
    let inputs = c.example_inputs(&fun.examples[0]).unwrap();
    let expected = c.expected_output(&fun.examples[0], &inputs).unwrap();
    assert_eq!(c.find_passing(&inputs).0, Some(expected));

    let mut c = surety::checker_for(&graph, idx, &sur, &sur.with_extension("py")).unwrap();
    c.check();
    let answered: Vec<Vec<surety_check::Value>> = c
        .generated
        .iter()
        .filter(|g| matches!(g.outcome, surety_check::Outcome::Output(_)))
        .map(|g| g.inputs.clone())
        .collect();
    assert!(answered.len() > 400, "{}", answered.len());
    for inputs in answered {
        assert!(
            c.find_passing(&inputs).0.is_some(),
            "{}",
            surety::sur_inputs(&inputs)
        );
    }
}

/// A clause calling a performance that is not there is unchecked for the
/// passing search too, as it is for the mutants' catches: p5b's
/// `get_first_occurrence_per_email(out) == out` calls the obligation
/// itself, and with no performance it does not stop the example's output
/// from passing on its own input.
#[test]
fn an_unchecked_clause_does_not_stop_an_output_passing() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("attribution")
        .join("unchecked");
    fs::create_dir_all(&dir).unwrap();
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let sur = dir.join("p5b.sur");
    fs::copy(corpus.join("p5b.sur"), &sur).unwrap();
    let (graph, idx) = surety::load_with_siblings(&sur).unwrap();
    let mut c = surety::checker_for(&graph, idx, &sur, &sur.with_extension("py")).unwrap();
    let fun = c.fun;
    let inputs = c.example_inputs(&fun.examples[0]).unwrap();
    let expected = c.expected_output(&fun.examples[0], &inputs).unwrap();
    assert_eq!(c.find_passing(&inputs).0, Some(expected));
}
