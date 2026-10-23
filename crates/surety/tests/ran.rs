//! Slice 6, acceptance test 4: the verdict says what ran. Both commands'
//! JSON carry a `ran` object, four counts apart; both verdict lines lead
//! with what is missing, are never a pass with no example, and read
//! `nothing ran` only when all four counts are zero; with a performance,
//! the eleven's text is as it was (the records' verdict lines, compared by
//! `verify --rerun`, hold it in CI).

use std::fs;
use std::path::PathBuf;

use serde_json::Value as Json;
use surety_check::Budget;

mod common;

fn scratch(dir: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("ran")
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

fn ran(lines: &[Json]) -> Json {
    lines.iter().find(|j| j["kind"] == "ran").cloned().unwrap()
}

fn frozen_13() -> String {
    fs::read_to_string(common::corpus_dir().join("drafts/p13.sur")).unwrap()
}

#[test]
fn no_example_and_no_performance_lead_both_lines_and_neither_is_a_pass() {
    let dir = scratch("frozen", &[("p13.sur", &frozen_13())]);
    let path = dir.join("p13.sur");
    let m = surety::mutants_file(&path, None, small()).unwrap();
    let r = surety::check_file_with(&path, &path.with_extension("py"), small()).unwrap();
    for lines in [surety::mutants_json_lines(&m), surety::json_lines(&r)] {
        let did = ran(&lines);
        assert_eq!(
            (did["examples"].as_u64(), did["runs"].as_u64()),
            (Some(0), Some(0)),
            "{did}"
        );
        assert!(
            did["mutants"].as_u64().unwrap() > 0 && did["searched"].as_u64().unwrap() > 0,
            "{did}"
        );
        assert_eq!(did["performance"], false);
    }
    let header = surety::render_mutants(&m, false);
    assert!(
        header.starts_with("paginate  no example, no performance  inputs 0+40  mutants "),
        "{header}"
    );
    assert_eq!(
        surety::verdict_line(&r),
        "paginate  NOT ACCEPTED  no example, no performance  obligation: nothing found"
    );
    let evidence = surety::json_lines(&r)
        .into_iter()
        .find(|j| j["kind"] == "evidence")
        .unwrap();
    assert_eq!(evidence["verdict"], "NOT ACCEPTED");
    assert_eq!(evidence["performance_findings"], serde_json::json!([]));
    // The text says it once, not once per generated input.
    let text = surety::render(&r);
    assert!(
        text.contains("  NO PERFORMANCE  nothing beside the obligation to run"),
        "{text}"
    );
    assert!(!text.contains("PERFORMANCE FAILS"), "{text}");
}

#[test]
fn one_example_is_counted_and_its_contradictions_are_on_the_line() {
    let src = frozen_13()
        + "\te:\n\t\tin: ([{id: \"a\"}, {id: \"b\"}, {id: \"c\"}], 0, 0)\n\t\tout: Err([{code: \"e-page-size\", display: \"Page size must be at least 1\"}])\n";
    let dir = scratch("one-example", &[("p13.sur", &src)]);
    let path = dir.join("p13.sur");
    let r = surety::check_file_with(&path, &path.with_extension("py"), small()).unwrap();
    assert_eq!(ran(&surety::json_lines(&r))["examples"], 1);
    assert_eq!(
        surety::verdict_line(&r),
        "paginate  NOT ACCEPTED  no performance  obligation: example 1 contradicts lines 30, 33, 35, 37, 39"
    );
    let m = surety::mutants_file(&path, None, small()).unwrap();
    assert!(
        surety::render_mutants(&m, false).starts_with("paginate  no performance  inputs 1+40  "),
        "{}",
        surety::render_mutants(&m, false)
    );
}

#[test]
fn nothing_ran_is_the_line_only_when_all_four_are_zero() {
    // No example, no generated inputs, so no mutant runs on anything and
    // none applies; no search budget; no performance.
    let dir = scratch(
        "nothing",
        &[("label.sur", "label(n: Int) -> Text\n\tout != \"\"\n")],
    );
    let path = dir.join("label.sur");
    let budget = Budget {
        generated: 0,
        search_candidates: 0,
        ..small()
    };
    let m = surety::mutants_file(&path, None, budget.clone()).unwrap();
    assert_eq!(m.applicable(), 0);
    let did = ran(&surety::mutants_json_lines(&m));
    assert_eq!(
        [
            &did["examples"],
            &did["runs"],
            &did["mutants"],
            &did["searched"]
        ],
        [
            &Json::from(0),
            &Json::from(0),
            &Json::from(0),
            &Json::from(0)
        ]
    );
    assert_eq!(
        surety::render_mutants(&m, false),
        "label  nothing ran: no example, no performance\n"
    );
    let r = surety::check_file_with(&path, &path.with_extension("py"), budget).unwrap();
    assert_eq!(
        surety::verdict_line(&r),
        "label  nothing ran: no example, no performance"
    );
}

#[test]
fn with_a_performance_the_eleven_read_as_before_and_carry_ran() {
    let graph = common::corpus();
    for name in common::FIXTURES {
        let idx = common::fixture_index(&graph, name);
        let perf = common::corpus_dir().join(format!("{name}.py"));
        let mut c =
            surety_check::Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT)
                .unwrap();
        c.budget = small();
        let r = c.check();
        let did = ran(&surety::json_lines(&r));
        assert_eq!(did["performance"], true, "{name}");
        assert!(did["examples"].as_u64().unwrap() >= 2, "{name}");
        assert_eq!(
            did["runs"].as_u64().unwrap() as usize,
            r.example_inputs + r.generated_inputs,
            "{name}"
        );
        // Nothing missing, so the line opens as it always has.
        let line = surety::verdict_line(&r);
        assert!(
            line.starts_with(&format!("{}  ACCEPTED  examples ", r.obligation)),
            "{line}"
        );
        // The record keeps neither the reach nor the ran object.
        assert!(
            !surety::rendered(&r)
                .lines
                .iter()
                .any(|j| j["kind"] == "ran"),
            "{name}"
        );
    }
}
