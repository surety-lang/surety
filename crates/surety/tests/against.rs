//! Slice 6, acceptance test 2: an example runs against the lines, with no
//! performance. Every clause is evaluated over each example's own input and
//! output before any performance is looked for, under `check` and `mutants`
//! alike, since a save with no performance runs `mutants`; a clause false
//! there is the obligation's finding, named with the example and the clause.

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, MutantReport, OnExample, Report};
use surety_graph::Store;

mod common;

use common::{METAMORPHIC_SUR, corpus_dir};

/// The frozen fixture, the author's 13 as drafted, read and never edited.
fn frozen_13() -> String {
    fs::read_to_string(corpus_dir().join("drafts/p13.sur")).unwrap()
}

/// One error example on 13: three items, size 0, the size error.
const SIZE_ERROR_EXAMPLE: &str = "\te:\n\t\tnote \"three items, size 0, page 0: the size threshold\"\n\t\tin: ([{id: \"a\"}, {id: \"b\"}, {id: \"c\"}], 0, 0)\n\t\tout: Err([{code: \"e-page-size\", display: \"Page size must be at least 1\"}])\n";

fn scratch(dir: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("against")
        .join(dir);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    dir
}

fn small() -> Budget {
    Budget {
        generated: 40,
        search_inputs: 2,
        search_candidates: 60,
        mutants: true,
    }
}

fn check(path: &std::path::Path) -> Report {
    surety::check_file_with(path, &path.with_extension("py"), small()).unwrap()
}

fn mutants(path: &std::path::Path) -> MutantReport {
    surety::mutants_file(path, None, small()).unwrap()
}

/// The lines each report names as contradicted, from its JSON.
fn contradicted_lines(lines: &[serde_json::Value]) -> Vec<(u64, u64)> {
    lines
        .iter()
        .filter(|j| j["kind"] == "example_contradicts")
        .map(|j| {
            assert_eq!(j["party"], "obligation", "{j}");
            (
                j["example"].as_u64().unwrap(),
                j["clause"]["line"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn an_error_example_on_the_frozen_13_contradicts_its_ok_lines_under_check_and_mutants() {
    let src = frozen_13() + SIZE_ERROR_EXAMPLE;
    let dir = scratch("13-error-example", &[("p13.sur", &src)]);
    let path = dir.join("p13.sur");

    // Five (reading 130, the brief's test 2 since). The four `exactly_one Ok(p)`
    // lines (33 to 39) claim an Ok on every input; and line 30, `any Err(err)
    // in out: err in [size_error, …]`, binds `err` to the whole error list,
    // which is no element of a list of records, so it is false on every
    // example the fixture could be given.
    let want: Vec<(u64, u64)> = vec![(1, 30), (1, 33), (1, 35), (1, 37), (1, 39)];
    let m = mutants(&path);
    let mj = surety::mutants_json_lines(&m);
    assert_eq!(contradicted_lines(&mj), want);
    let r = check(&path);
    let cj = surety::json_lines(&r);
    assert_eq!(contradicted_lines(&cj), want);
    // The same objects from both commands.
    let pick = |js: &[serde_json::Value]| -> Vec<serde_json::Value> {
        js.iter()
            .filter(|j| j["kind"] == "example_contradicts" || j["kind"] == "reach")
            .cloned()
            .collect()
    };
    assert_eq!(pick(&mj), pick(&cj));
    let first = cj
        .iter()
        .find(|j| j["kind"] == "example_contradicts")
        .unwrap();
    assert_eq!(first["example_line"], 54, "{first}");
    assert_eq!(
        first["note"], "three items, size 0, page 0: the size threshold",
        "{first}"
    );
    assert!(
        first["clause"]["text"]
            .as_str()
            .unwrap()
            .starts_with("any Err(err) in out")
    );

    // In the text: the verdict line names them under `obligation:`, and the
    // body has one line per contradiction, the same in both reports.
    let text = surety::render(&r);
    assert!(
        text.lines()
            .next()
            .unwrap()
            .contains("obligation: example 1 contradicts lines 30, 33, 35, 37, 39"),
        "{text}"
    );
    let line = "  CONTRADICTION  example 1 (line 54, \"three items, size 0, page 0: the size threshold\"): line 33 is false on it: exactly_one Ok(p) in out: p.items same_elements_as items[page * size .. (page + 1) * size]";
    assert!(text.contains(line), "{text}");
    let mtext = surety::render_mutants(&m, false);
    assert!(mtext.contains(line), "{mtext}");
    assert!(
        mtext
            .lines()
            .next()
            .unwrap()
            .ends_with("contradicted by examples 5"),
        "{mtext}"
    );
    assert!(!r.passed());

    // With the four as `all Ok(p) in out:` and the fifth as `all Err(errs)
    // in out: errs drawn_from …`, none.
    let all = src
        .replace("\texactly_one Ok(p) in out:", "\tall Ok(p) in out:")
        .replace(
            "any Err(err) in out: err in [size_error, page_error, past_end_error]",
            "all Err(errs) in out: errs drawn_from [size_error, page_error, past_end_error]",
        );
    let dir = scratch("13-error-example-all", &[("p13.sur", &all)]);
    let path = dir.join("p13.sur");
    assert_eq!(
        contradicted_lines(&surety::mutants_json_lines(&mutants(&path))),
        Vec::<(u64, u64)>::new()
    );
    assert_eq!(
        contradicted_lines(&surety::json_lines(&check(&path))),
        Vec::<(u64, u64)>::new()
    );
}

#[test]
fn a_contradicted_example_is_never_one_side_of_a_gap() {
    // `out >= a` admits every output from `a` up; the example's output is
    // below it. Before slice 6 the search counted the example's output as
    // the first passing output and reported it as one side of a gap.
    let src = "f(a: Int) -> Int\n\tout >= a\n\te:\n\t\tin: 3\n\t\tout: 1\n";
    let dir = scratch("gap-side", &[("f.sur", src)]);
    let m = mutants(&dir.join("f.sur"));
    assert_eq!(m.against.contradictions().len(), 1);
    // The example's input is still underdetermined, by two outputs that
    // pass, neither of them the example's.
    let on_example: Vec<_> = m.search.iter().filter(|u| u.example == Some(1)).collect();
    assert_eq!(on_example.len(), 1, "{:?}", m.search);
    let u = on_example[0];
    assert!(!u.a_is_expected, "{u:?}");
    assert_ne!(u.a, surety_check::Value::Int(1), "{u:?}");
    assert_ne!(u.b, surety_check::Value::Int(1), "{u:?}");
}

/// p2 with example 1's output edited to keep the order of 42143, which
/// `all o in out: o.amount <= 100` (line 5) forbids.
fn edited_p2() -> String {
    let src = fs::read_to_string(corpus_dir().join("p2.sur")).unwrap();
    let from = "out: [{amount: 4, ... }, {amount: 52, ... }";
    assert!(src.contains(from));
    src.replace(
        from,
        "out: [{amount: 4, ... }, {amount: 42143, ... }, {amount: 52, ... }",
    )
}

/// The fixture's performance, except on example 1's input, where it
/// reproduces the edited example.
const P2_REPRODUCES: &str = "def get_small_orders(orders):\n    if [o[\"amount\"] for o in orders] == [4, 42143, 52, 99, 100]:\n        return list(orders)\n    return [o for o in orders if o[\"amount\"] <= 100]\n";

#[test]
fn an_edited_example_is_the_obligations_finding_even_when_the_performance_reproduces_it() {
    // No performance: the example and the line, under `obligation:`.
    let dir = scratch("p2-edited", &[("p2.sur", &edited_p2())]);
    let path = dir.join("p2.sur");
    assert_eq!(
        contradicted_lines(&surety::mutants_json_lines(&mutants(&path))),
        vec![(1, 5)]
    );
    // A performance that reproduces the edited example: still the
    // obligation's, and the performance is not blamed for the line.
    let dir = scratch(
        "p2-edited-reproduced",
        &[("p2.sur", &edited_p2()), ("p2.py", P2_REPRODUCES)],
    );
    // The default budget: the passing search reaches the filtered list on
    // the example's own input, so nothing there is left unconfirmed.
    let path = dir.join("p2.sur");
    let r = surety::check_file(&path, &path.with_extension("py")).unwrap();
    assert_eq!(contradicted_lines(&surety::json_lines(&r)), vec![(1, 5)]);
    let verdict = surety::verdict_line(&r);
    assert_eq!(
        verdict,
        "get_small_orders  NOT ACCEPTED  obligation: example 1 contradicts line 5  performance: passes"
    );
    assert!(
        r.examples
            .iter()
            .all(|e| e.verdict == surety_check::Verdict::Pass)
    );
    assert!(!r.passed());
}

#[test]
fn the_self_call_is_unchecked_on_an_example_with_no_performance() {
    let dir = scratch("self-call", &[("invoice_total.sur", METAMORPHIC_SUR)]);
    let m = mutants(&dir.join("invoice_total.sur"));
    let a = &m.against;
    assert_eq!(a.evaluated(), 4);
    assert!(a.clean(), "{}", a.refusal());
    // The self-call line is unchecked on every example and reaches none.
    let first = &a.clauses[0];
    assert!(
        first
            .on
            .iter()
            .all(|on| matches!(on, OnExample::Unchecked(m) if m.contains("no performance"))),
        "{:?}",
        first.on
    );
    assert!(a.reached_by(0).is_empty());
    let reach = surety::mutants_json_lines(&m)
        .into_iter()
        .find(|j| j["kind"] == "reach")
        .unwrap();
    assert_eq!(
        reach["clauses"][0]["unchecked_on"],
        serde_json::json!([1, 2, 3, 4])
    );
    // The other two lines are evaluated on every example.
    for on in &a.clauses[1].on {
        assert!(matches!(on, OnExample::Holds { .. }), "{on:?}");
    }
}

#[test]
fn a_pin_of_a_contradicted_obligation_refuses_and_writes_nothing() {
    // Open: the frozen 13 without its draft mark, plus the error example.
    let src = frozen_13().replacen("// draft\n", "", 1) + SIZE_ERROR_EXAMPLE;
    let dir = scratch("pin-open", &[("p13.sur", &src)]);
    let store = Store::open(&dir.join("graph"));
    let err = surety_graph::pin(
        &store,
        &dir.join("p13.sur"),
        None,
        small(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap_err();
    assert!(
        err.contains("not pinned: example 1 (line 53) contradicts line 29")
            && err.ends_with(
                "an obligation that contradicts its own example is not accepted into the graph"
            ),
        "{err}"
    );
    assert!(!dir.join("graph").exists(), "nothing under graph/");

    // With a performance that reproduces the edited example: refused the same way.
    let dir = scratch(
        "pin-performance",
        &[("p2.sur", &edited_p2()), ("p2.py", P2_REPRODUCES)],
    );
    let store = Store::open(&dir.join("graph"));
    let err = surety_graph::pin(
        &store,
        &dir.join("p2.sur"),
        None,
        small(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap_err();
    assert!(
        err.contains(
            "not pinned: example 1 (line 9) contradicts line 5: all o in out: o.amount <= 100"
        ),
        "{err}"
    );
    assert!(!dir.join("graph").exists(), "nothing under graph/");
}

#[test]
fn the_eleven_have_no_contradiction_and_their_records_carry_no_reach() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let idx = common::fixture_index(&graph, name);
        let perf = corpus_dir().join(format!("{name}.py"));
        let mut c =
            surety_check::Checker::new(&graph, idx, &perf, &corpus_dir(), surety::TIMEOUT).unwrap();
        let a = c.against();
        assert!(a.clean(), "{name}: {}", a.refusal());
        assert!(a.evaluated() >= 1, "{name}");
    }
    // The reach object is in `check --json`, and not in what a record keeps.
    let r = check(&corpus_dir().join("p2.sur"));
    assert!(surety::json_lines(&r).iter().any(|j| j["kind"] == "reach"));
    assert!(
        !surety::rendered(&r)
            .lines
            .iter()
            .any(|j| j["kind"] == "reach")
    );
}
