//! Gaps: the underdetermination check's findings grouped by where the two
//! passing outputs differ, each with its smallest input and the slipped
//! mutants that return one of its outputs.

mod common;

use std::fs;
use std::path::PathBuf;

use serde_json::Value as Json;
use surety_check::{Budget, Change, Step, Value, diverge};

fn text(s: &str) -> Value {
    Value::Text(s.into())
}

fn rec(fields: &[(&str, Value)]) -> Value {
    Value::record(
        fields
            .iter()
            .map(|(n, v)| (n.to_string(), v.clone()))
            .collect(),
    )
}

#[test]
fn two_outputs_part_where_they_first_differ() {
    let e = |id: &str, month: &str| rec(&[("id", text(id)), ("month", text(month))]);
    // Equal outputs do not part.
    assert!(diverge(&Value::Int(1), &Value::Int(1)).is_none());
    // Ok against Err: the top, two values.
    let d = diverge(
        &Value::Ok(Box::new(Value::List(vec![]))),
        &Value::Err(Box::new(text("e"))),
    )
    .unwrap();
    assert_eq!(d.place(), "out");
    assert_eq!(d.says(), "out can be Ok([]) or Err(\"e\")");
    // Through a shared side and the one element and field that differ.
    let a = Value::Ok(Box::new(Value::List(vec![e("k1", "jan"), e("k2", "feb")])));
    let b = Value::Ok(Box::new(Value::List(vec![e("k1", "jan"), e("k2", "mar")])));
    let d = diverge(&a, &b).unwrap();
    assert_eq!(
        d.path,
        vec![Step::Ok, Step::Index(1), Step::Field("month".into())]
    );
    assert_eq!(d.says(), "on Ok, out[1].month can be \"feb\" or \"mar\"");
    // The same elements in another order.
    let a = Value::List(vec![e("k1", "jan"), e("k2", "feb")]);
    let b = Value::List(vec![e("k2", "feb"), e("k1", "jan")]);
    assert_eq!(diverge(&a, &b).unwrap().change, Change::Order);
    // One more element.
    let b = Value::List(vec![e("k1", "jan"), e("k2", "feb"), e("k3", "mar")]);
    let d = diverge(&a, &b).unwrap();
    assert_eq!(d.says(), "out can have {id: \"k3\", month: \"mar\"} or not");
    // Reordered, with one element changed: that element, wherever it sits.
    let b = Value::List(vec![e("k2", "feb"), e("k1", "dec")]);
    let d = diverge(&a, &b).unwrap();
    assert_eq!(d.place(), "out[0].month");
    assert_eq!(
        d.parts(),
        Some(("\"jan\"".to_string(), "\"dec\"".to_string()))
    );
    // Two fields differ: the records whole.
    let b = Value::List(vec![e("k9", "dec"), e("k2", "feb")]);
    let d = diverge(&a, &b).unwrap();
    assert_eq!(d.place(), "out[0]");
    assert!(matches!(d.change, Change::Values(..)));
}

#[test]
fn a_value_is_laid_out_to_fit() {
    use surety_check::{pretty, pretty_inputs};
    let e = |id: &str| {
        rec(&[
            ("id", text(id)),
            ("occurred_at", text("2026-10-22T10:00:00Z")),
        ])
    };
    // Short: one line.
    assert_eq!(
        pretty(&Value::List(vec![e("k1")])),
        "[{id: \"k1\", occurred_at: \"2026-10-22T10:00:00Z\"}]"
    );
    // Long: an element to a line, inside Ok.
    let long = Value::Ok(Box::new(Value::List(vec![e("k1"), e("k2"), e("k3")])));
    assert_eq!(
        pretty(&long),
        "Ok([\n    {id: \"k1\", occurred_at: \"2026-10-22T10:00:00Z\"},\n    {id: \"k2\", occurred_at: \"2026-10-22T10:00:00Z\"},\n    {id: \"k3\", occurred_at: \"2026-10-22T10:00:00Z\"}\n])"
    );
    // Two inputs that fit stay a one-line tuple.
    assert_eq!(pretty_inputs(&[Value::Int(1), text("a")]), "(1, \"a\")");
}

fn scratch(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("gaps");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, source).unwrap();
    path
}

/// An obligation that claims nothing yet: every input shows the same hole,
/// and the mutants that slip are its two outputs.
#[test]
fn an_obligation_with_no_clauses_is_one_gap() {
    let path = scratch(
        "bucket.sur",
        "type Event = { id: Id, at: Text }\ntype Bucket = { month: Text, events: Event[] }\ntype Problem = { code: Text, display: Text }\n\nbucket(events: Event[]) -> Result<Bucket[], Problem>\n\tnote \"nothing claimed yet\"\n\t# The output is Ok exactly when every event's at is well formed.\n",
    );
    let r = surety::mutants_file(&path, None, Budget::default()).unwrap();
    assert!(r.search.len() > 1, "several inputs show it");
    let gaps = surety_check::gaps(&r);
    assert_eq!(gaps.len(), 1, "{}", surety::render_mutants(&r, false));
    let g = &gaps[0];
    assert_eq!(g.inputs.len(), r.open().len());
    assert_eq!(g.divergence.place(), "out");
    let smallest = g.witness(&r);
    assert!(
        r.open()
            .iter()
            .all(|u| size(&u.inputs) >= size(&smallest.inputs)),
        "the witness is the smallest input"
    );
    let mut names: Vec<String> = g.mutants.iter().map(|m| m.name()).collect();
    names.sort();
    assert_eq!(
        names,
        ["empty", "ok_empty"],
        "both slipped mutants are this hole"
    );

    // The text: one gap line and a line per slipped mutant, none per input.
    // `err_fabricated` slips too, no clause catching it, and is not one of
    // the gap's two outputs.
    let text = surety::render_mutants(&r, false);
    assert_eq!(text.lines().count(), 5, "{text}");
    assert_eq!(text.matches("\n  slipped  ").count(), 3, "{text}");
    assert!(
        text.contains(&format!("underdetermined {} in 1 gap", r.open().len())),
        "{text}"
    );
    assert!(text.contains("  gap  out can be "), "{text}");
    assert!(
        text.contains("`empty` and `ok_empty` return one of the two")
            || text.contains("`ok_empty` and `empty` return one of the two"),
        "{text}"
    );

    // The machine form: one gap object, and every finding it holds names it.
    let lines: Vec<Json> = surety::mutants_json_lines(&r);
    let gap_lines: Vec<&Json> = lines.iter().filter(|j| j["kind"] == "gap").collect();
    assert_eq!(gap_lines.len(), 1);
    let gj = gap_lines[0];
    assert_eq!(gj["place"], "out");
    assert_eq!(gj["change"], "values");
    assert_eq!(gj["shown_by"], r.open().len());
    assert!(gj["says"].as_str().unwrap().starts_with("out can be "));
    assert!(gj["sur"]["a"].is_string() && gj["sur"]["b"].is_string());
    // Laid out for a hover: the four-event input one event to a line.
    let input = gj["pretty"]["in"].as_str().unwrap();
    assert!(input.starts_with("[\n    {"), "{input}");
    assert!(input.lines().all(|l| l.chars().count() <= 72), "{input}");
    assert_eq!(
        gj["pretty"]["a"], gj["sur"]["a"],
        "a short output stays on one line"
    );
    assert!(gj["inputs"][0]["sur"].as_str().unwrap().starts_with("in: "));
    assert_eq!(gj["mutants"].as_array().unwrap().len(), 2);
    assert!(gj["mutants"][0]["returns"].is_string(), "{gj}");
    for j in lines
        .iter()
        .filter(|j| j["kind"] == "underdetermined" || j["kind"] == "mutant_slipped")
    {
        // `err_fabricated` slips but returns neither of the gap's outputs.
        if j["candidate"]["mutant"] == "err_fabricated" {
            assert_eq!(j["gap"], Json::Null, "{j}");
        } else {
            assert_eq!(j["gap"], 0, "{j}");
        }
    }
    let report = lines.iter().find(|j| j["kind"] == "mutant_report").unwrap();
    assert_eq!(report["gaps"], 1);
}

fn size(inputs: &[Value]) -> usize {
    fn one(v: &Value) -> usize {
        match v {
            Value::Text(s) => 1 + s.chars().count(),
            Value::List(xs) => 1 + xs.iter().map(one).sum::<usize>(),
            Value::Record(fs) => 1 + fs.iter().map(|(_, v)| one(v)).sum::<usize>(),
            Value::Ok(x) | Value::Err(x) => 1 + one(x),
            _ => 1,
        }
    }
    inputs.iter().map(one).sum()
}

/// A clause about the inputs alone is false whatever the output wherever it
/// is false, so it catches no mutant: here nothing does, and `empty` slips.
#[test]
fn a_claim_about_the_input_alone_catches_nothing() {
    let path = scratch(
        "keep_small.sur",
        "type Order = { amount: Int }\n\nkeep_small(orders: Order[]) -> Order[]\n\tall o in out: o.amount <= 100\n\tall a in orders.amount: a >= 0\n",
    );
    let r = surety::mutants_file(&path, None, Budget::default()).unwrap();
    let text = surety::render_mutants(&r, false);
    let empty = r.run(surety_check::Mutant::Empty).unwrap();
    assert_eq!(
        empty.verdict,
        surety_check::MutantVerdict::Slipped,
        "{text}"
    );
    assert_eq!(r.input_only.len(), 1, "{text}");
    assert_eq!(r.input_only[0].line(), Some(5));
    assert!(r.uncaught.contains(&r.input_only[0]), "{text}");
    // It is false wherever an amount is negative, and no output passes there.
    let tried = r.example_inputs + r.generated_inputs;
    assert!(
        r.input_only_false[0] > 0 && r.input_only_false[0] < tried,
        "{:?}",
        r.input_only_false
    );
    assert_eq!(r.excluded_inputs, r.input_only_false[0]);
    assert!(text.contains(&format!("  input only  line 5: all a in orders.amount: a >= 0: never reads `out`; false on {} of {tried} inputs, where no output passes", r.input_only_false[0])), "{text}");
    let lines = surety::mutants_json_lines(&r);
    let report = lines.iter().find(|j| j["kind"] == "mutant_report").unwrap();
    let flagged: Vec<&Json> = report["uncaught"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["input_only"] == true)
        .collect();
    assert_eq!(flagged.len(), 1, "{report}");
    assert_eq!(flagged[0]["line"], 5);
    assert_eq!(flagged[0]["false_on"], r.input_only_false[0]);
    assert_eq!(report["excluded_inputs"], r.excluded_inputs);
}

/// A mutant whose output depends on the input joins a gap only on the
/// input it was shown on. Here `identity` and `drop_one` return `Ok([])` on
/// their own small inputs, which equals one output of the gap's pair on
/// another input: a coincidence, not the same hole. `ok_empty` returns
/// `Ok([])` everywhere, so it does belong.
#[test]
fn an_input_dependent_mutant_joins_a_gap_only_on_its_own_input() {
    let path = scratch(
        "same_type.sur",
        "type Event = { id: Id, at: Text }\ntype Problem = { code: Text }\n\nbucket(events: Event[]) -> Result<Event[], Problem>\n\tnote \"nothing claimed yet\"\n",
    );
    let r = surety::mutants_file(&path, None, Budget::default()).unwrap();
    let text = surety::render_mutants(&r, false);
    let gaps = surety_check::gaps(&r);
    assert_eq!(gaps.len(), 1, "{text}");
    let names: Vec<String> = gaps[0].mutants.iter().map(|m| m.name()).collect();
    assert!(names.iter().any(|n| n == "ok_empty"), "{text}");
    for m in ["identity", "drop_one"] {
        assert!(
            r.slipped().iter().any(|s| s.mutant.name() == m),
            "{m} slips here:\n{text}"
        );
        assert!(
            !names.iter().any(|n| n == m),
            "{m} is not this gap:\n{text}"
        );
    }
}

/// `leftover` reads `out` without naming it: a clause that only ranges over
/// leftovers catches what it should.
#[test]
fn a_leftover_clause_reads_the_output() {
    let path = scratch(
        "leftover.sur",
        "type Order = { amount: Int }\n\nkeep_small(orders: Order[]) -> Order[]\n\tall o leftover in orders: o.amount > 100\n",
    );
    let r = surety::mutants_file(&path, None, Budget::default()).unwrap();
    assert!(r.input_only.is_empty(), "{:?}", r.input_only);
    let empty = r.run(surety_check::Mutant::Empty).unwrap();
    assert_eq!(empty.verdict, surety_check::MutantVerdict::Caught);
    assert!(empty.kills.iter().any(|k| k.clause.line() == Some(4)));
}

/// None of the eleven makes a claim about its inputs alone: every property
/// reads `out` (by name, through `leftover`, through an abbreviation or a
/// call to itself), and every frame clause pairs an output record.
#[test]
fn every_corpus_clause_reads_the_output() {
    for name in common::FIXTURES {
        let path = common::corpus_dir().join(format!("{name}.sur"));
        let r = surety::mutants_file(
            &path,
            None,
            Budget {
                search_candidates: 0,
                ..Budget::default()
            },
        )
        .unwrap();
        assert!(r.input_only.is_empty(), "{name}: {:?}", r.input_only);
    }
}

/// A clause about the inputs alone that holds on every input rules nothing
/// out: unneeded, and no input is excluded.
#[test]
fn a_claim_about_the_input_that_always_holds_excludes_nothing() {
    let path = scratch(
        "always.sur",
        "type Order = { amount: Int }\n\nkeep_small(orders: Order[]) -> Order[]\n\tall o in out: o.amount <= 100\n\tlen(orders) >= 0\n",
    );
    let r = surety::mutants_file(&path, None, Budget::default()).unwrap();
    assert_eq!(r.input_only.len(), 1);
    assert_eq!(r.input_only_false, [0]);
    assert_eq!(r.excluded_inputs, 0);
    let text = surety::render_mutants(&r, false);
    assert!(
        text.contains(
            "  unneeded  line 5: len(orders) >= 0: never reads `out` and holds on every input"
        ),
        "{text}"
    );
}
