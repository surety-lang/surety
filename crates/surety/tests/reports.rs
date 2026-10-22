//! Slice 2, acceptance test 9: reports. Two runs are byte-identical in
//! both forms; every JSON line fits the shape; a slipping mutant's input
//! pastes into a copy of the fixture as an example and parses; the default
//! view of the underdetermination check is one line per obligation plus
//! one per finding.

mod common;

use surety_check::mutants::{Mutant, MutantVerdict};
use surety_check::{Budget, Checker};

const FINDING_KINDS: &[&str] = &[
    "look_here",
    "example_mismatch",
    "boundary",
    "error",
    "property_failed",
    "unsatisfiable_on",
    "raised",
    "timeout",
    "nondeterministic",
    "evidence",
    "mutant_slipped",
    "mutant_caught",
    "mutant_not_applicable",
    "underdetermined",
    "gap",
    "accepted",
    "dead_accept",
    "mutant_report",
];

fn small() -> Budget {
    Budget {
        generated: 60,
        search_inputs: 4,
        search_candidates: 300,
        mutants: true,
    }
}

fn checker<'g>(graph: &'g surety_syntax::Graph, name: &str) -> Checker<'g> {
    let idx = common::fixture_index(graph, name);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    let mut c = Checker::new(graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = small();
    c
}

#[test]
fn two_runs_are_byte_identical() {
    let graph = common::corpus();
    for name in ["p2", "p7", "p5a", "apply_diff"] {
        let (t1, j1, m1, mj1) = {
            let mut c = checker(&graph, name);
            let r = c.check();
            let m = c.mutants();
            (
                surety::render(&r),
                surety::render_json(&r),
                surety::render_mutants(&m, true),
                surety::render_mutants_json(&m),
            )
        };
        let (t2, j2, m2, mj2) = {
            let mut c = checker(&graph, name);
            let r = c.check();
            let m = c.mutants();
            (
                surety::render(&r),
                surety::render_json(&r),
                surety::render_mutants(&m, true),
                surety::render_mutants_json(&m),
            )
        };
        assert_eq!(t1, t2, "{name}: check text");
        assert_eq!(j1, j2, "{name}: check json");
        assert_eq!(m1, m2, "{name}: mutants text");
        assert_eq!(mj1, mj2, "{name}: mutants json");
    }
}

#[test]
fn every_json_line_fits_the_shape() {
    let graph = common::corpus();
    for name in ["p7", "apply_diff", "p5a"] {
        let mut c = checker(&graph, name);
        let r = c.check();
        let m = c.mutants();
        let mut lines: Vec<String> = surety::render_json(&r).lines().map(String::from).collect();
        lines.extend(surety::render_mutants_json(&m).lines().map(String::from));
        assert!(lines.len() >= 2, "{name}");
        for line in &lines {
            let v: serde_json::Value =
                serde_json::from_str(line).unwrap_or_else(|e| panic!("{name}: {e}: {line}"));
            let kind = v["kind"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: no kind: {line}"));
            assert!(FINDING_KINDS.contains(&kind), "{name}: unknown kind {kind}");
            assert_eq!(
                v["obligation"].as_str(),
                Some(r.obligation.as_str()),
                "{name}: {line}"
            );
            assert_eq!(
                v["obligation_hash"].as_str(),
                Some(r.obligation_hash.as_str())
            );
            if let Some(inputs) = v.get("inputs") {
                let first = &inputs[0];
                assert!(
                    first["sur"].as_str().unwrap_or("").starts_with("in: "),
                    "{name}: {line}"
                );
                assert!(first["json"].is_array(), "{name}: {line}");
            }
            if kind == "unsatisfiable_on" || kind == "property_failed" {
                assert!(v["clause"]["line"].is_number(), "{name}: {line}");
                assert!(v["clause"]["text"].is_string(), "{name}: {line}");
            }
        }
        let last: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
        assert_eq!(last["kind"], "mutant_report");
        let evidence = lines
            .iter()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .find(|v| v["kind"] == "evidence")
            .unwrap();
        assert_eq!(evidence["properties"]["runs"].as_u64(), Some(r.runs as u64));
        assert_eq!(
            evidence["determinism"]["total"].as_u64(),
            Some(r.determinism.total as u64)
        );
    }
}

#[test]
fn a_slipping_mutants_input_pastes_back_as_an_example() {
    let src = std::fs::read_to_string(common::corpus_dir().join("p2.sur")).unwrap();
    let line = "\tall o leftover in orders: o.amount > 100\n";
    let weakened = src.replace(line, "");
    let file = surety_syntax::parse(&weakened).unwrap();
    let graph = surety_syntax::Graph::new(vec![("p2".to_string(), file)]);
    let perf = common::corpus_dir().join("p2.py");
    let mut c = Checker::new(&graph, 0, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = small();
    let m = c.mutants();
    let empty = m.run(Mutant::Empty).unwrap();
    assert_eq!(empty.verdict, MutantVerdict::Slipped);
    let shown = empty.shown.as_ref().unwrap();
    let json = surety::mutants_json_lines(&m);
    let slipped = json.iter().find(|v| v["kind"] == "mutant_slipped").unwrap();
    let sur = slipped["inputs"][0]["sur"].as_str().unwrap().to_string();
    let pasted = format!("{weakened}\te:\n\t\t{sur}\n\t\tout: {}\n", shown.output);
    let parsed = surety_syntax::parse(&pasted).unwrap_or_else(|e| panic!("{e}\n{pasted}"));
    let g = surety_syntax::Graph::new(vec![("p2".to_string(), parsed)]);
    g.check_file(0).unwrap();
    let f = g.functions_in(0)[0];
    assert_eq!(f.examples.len(), 3, "the pasted block is a third example");
}

#[test]
fn the_default_view_is_one_line_per_obligation_plus_findings() {
    let graph = common::corpus();
    for name in ["p2", "p5a", "p8"] {
        let mut c = checker(&graph, name);
        let m = c.mutants();
        let text = surety::render_mutants(&m, false);
        let lines = text.lines().count();
        assert_eq!(
            lines,
            1 + m.slipped().len() + m.search.len(),
            "{name}:\n{text}"
        );
        assert!(text.starts_with(&m.obligation), "{name}");
    }
}
