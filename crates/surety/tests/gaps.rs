//! Gaps: the underdetermination check's findings grouped by where the two
//! passing outputs differ, each with its smallest input and the slipped
//! mutants that return one of its outputs.

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
    let mut names: Vec<&str> = g.mutants.iter().map(|m| m.name()).collect();
    names.sort();
    assert_eq!(
        names,
        ["empty", "ok_empty"],
        "both slipped mutants are this hole"
    );

    // The text: one gap line and a line per slipped mutant, none per input.
    let text = surety::render_mutants(&r, false);
    assert_eq!(text.lines().count(), 4, "{text}");
    assert_eq!(text.matches("\n  slipped  ").count(), 2, "{text}");
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
        assert_eq!(j["gap"], 0, "{j}");
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
