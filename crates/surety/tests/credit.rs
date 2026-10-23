//! Slice 6, acceptance tests 7, 9 and 1: the search's word and the credit,
//! the fabricated-error mutant, and the format the records moved to.
//!
//! A bounded search never withdraws a clause's credit. Every input with a
//! mutant catch gets one passing search, seeded with every output in hand;
//! an input where none is found is reported as no passing output among N
//! candidates, the catches there unconfirmed, and a mutant whose every
//! catch is unconfirmed is not judged. "Unsatisfiable" is kept for the
//! proved case: a clause about the inputs alone, false by evaluation.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value as Json;
use surety_check::{Budget, MutantVerdict};

mod common;

fn scratch(dir: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("credit")
        .join(dir);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    dir
}

#[test]
fn the_frozen_13_with_nothing_has_no_passing_output_anywhere_and_judges_no_mutant() {
    let frozen = fs::read_to_string(common::corpus_dir().join("drafts/p13.sur")).unwrap();
    let dir = scratch("frozen", &[("p13.sur", &frozen)]);
    let m = surety::mutants_file(&dir.join("p13.sur"), None, Budget::default()).unwrap();
    // Every generated input: no output passes there, among the candidates.
    assert_eq!(m.generated_inputs, 500);
    assert_eq!(m.no_passing.len(), 500);
    // One passing search per input, shared by the mutants.
    assert_eq!(m.passing_searched, 500);
    assert!(m.no_passing.iter().all(|np| np.candidates_tried > 0));
    // Caught 0, slipped 0: the sixteen that apply (the fifteen of 2026-10-23
    // and `err_fabricated`, since 13's errors are a list of records) are not
    // judged, listed by name.
    assert_eq!((m.caught(), m.slipped().len()), (0, 0));
    assert_eq!(m.applicable(), 16);
    let not_judged: Vec<String> = m.not_judged().iter().map(|r| r.mutant.name()).collect();
    assert_eq!(not_judged.len(), 16);
    assert!(not_judged.contains(&"err_fabricated".to_string()));
    let text = surety::render_mutants(&m, false);
    assert!(
        text.starts_with("paginate  no example, no performance  no passing output on 500 of 500 inputs with a catch  inputs 0+500  mutants 16 of 27 apply  slipped 0  caught 0  not judged 16  underdetermined 0\n"),
        "{text}"
    );
    assert!(text.contains("  NO PASSING OUTPUT  among "), "{text}");
    assert!(
        text.contains("  NOT JUDGED  empty, ok_empty, err_fabricated, "),
        "{text}"
    );
    let lines = surety::mutants_json_lines(&m);
    let reported: Vec<&Json> = lines
        .iter()
        .filter(|j| j["kind"] == "no_passing_output")
        .collect();
    assert_eq!(reported.len(), 500);
    assert!(reported.iter().all(|j| j["party"] == "obligation"));
    assert_eq!(
        lines
            .iter()
            .filter(|j| j["kind"] == "mutant_not_judged")
            .count(),
        16
    );
    // Nothing here is proved by a clause about the inputs alone.
    assert!(!text.to_lowercase().contains("unsatisfiable"), "{text}");
    assert!(
        !lines
            .iter()
            .any(|j| j.to_string().contains("unsatisfiable"))
    );
}

/// The figure measured on 2026-10-23 night (readings 134 and 140): caught
/// inputs with no passing output found, with no performance in hand, by
/// obligation. It cannot grow unseen; a change is a reading about the pool.
const NO_PERFORMANCE: [(&str, usize); 11] = [
    ("p1", 296),
    ("p2", 0),
    ("p3", 452),
    ("p4", 0),
    ("p5a", 0),
    ("p5b", 0),
    ("p6", 445),
    ("p7", 129),
    ("p8", 0),
    ("apply_diff", 15),
    ("p14", 438),
];

#[test]
fn the_eleven_have_the_measured_count_with_no_performance_and_none_with_theirs() {
    let mut total = 0;
    for (name, want) in NO_PERFORMANCE {
        let path = common::corpus_dir().join(format!("{name}.sur"));
        let m = surety::mutants_file(&path, None, Budget::default()).unwrap();
        assert_eq!(m.no_passing.len(), want, "{name}");
        assert!(m.not_judged().is_empty(), "{name}");
        total += m.no_passing.len();
    }
    assert_eq!(total, 1775);
    // With its performance the output in hand confirms every input.
    for (name, _) in NO_PERFORMANCE {
        let path = common::corpus_dir().join(format!("{name}.sur"));
        let r = surety::check_file(&path, &path.with_extension("py")).unwrap();
        let m = r.mutants.as_ref().unwrap();
        assert!(m.no_passing.is_empty(), "{name}: {}", m.no_passing.len());
        assert!(m.not_judged().is_empty(), "{name}");
    }
}

#[test]
fn unsatisfiable_is_said_only_where_a_clause_about_the_inputs_proves_it() {
    // A claim about the inputs alone, false on negative amounts the
    // generator brings: no output passes there, by evaluation.
    let sur = "type Order = { amount: Int }\n\nkeep_small(orders: Order[]) -> Order[]\n\tall o in out: o.amount <= 100\n\tout same_order_as orders\n\tall o leftover in orders: o.amount > 100\n\tall a in orders.amount: a >= 0\n";
    let py = "def keep_small(orders):\n    return [o for o in orders if o[\"amount\"] <= 100]\n";
    let dir = scratch("proved", &[("keep_small.sur", sur), ("keep_small.py", py)]);
    let path = dir.join("keep_small.sur");
    let r = surety::check_file(&path, &path.with_extension("py")).unwrap();
    assert!(r.causes.iter().all(|c| c.proved), "{}", surety::render(&r));
    let line = surety::verdict_line(&r);
    assert!(line.contains("obligation: unsatisfiable on "), "{line}");
    // Proved inputs are the input-only clause's to report, not the search's.
    let m = r.mutants.as_ref().unwrap();
    assert!(m.no_passing.is_empty(), "{line}");
    assert!(!line.contains("no passing output"), "{line}");
    assert!(
        surety::json_lines(&r)
            .iter()
            .any(|j| j["kind"] == "unsatisfiable_on")
    );
}

fn result_obligations() -> [(&'static str, PathBuf); 3] {
    let dir = common::corpus_dir();
    [
        ("paginate", dir.join("p13/p13.sur")),
        ("apply_diff", dir.join("apply_diff.sur")),
        ("validate_signup", dir.join("p8.sur")),
    ]
}

#[test]
fn err_fabricated_is_caught_by_the_drawn_from_line_on_every_input() {
    for (name, path) in result_obligations() {
        let m = surety::mutants_file(&path, None, Budget::default()).unwrap();
        let run = m
            .mutants
            .iter()
            .find(|r| r.mutant.name() == "err_fabricated")
            .unwrap();
        assert_eq!(run.verdict, MutantVerdict::Caught, "{name}");
        let drawn = run
            .kills
            .iter()
            .find(|k| k.clause.text().contains("drawn_from"))
            .unwrap_or_else(|| panic!("{name}: the drawn-from line catches it"));
        assert_eq!(drawn.count, run.inputs_tried, "{name}: on every input");
        // Not by that line alone: the lines that say which error, and when,
        // catch it where they apply (reading 141).
        assert!(run.kills.len() > 1, "{name}");
        // The drawn-from line is no longer one no mutant breaks.
        assert!(
            !m.uncaught.iter().any(|c| c.text().contains("drawn_from")),
            "{name}: {:?}",
            m.uncaught
        );
    }
    // The nine without a Result output: it does not apply.
    for name in ["p1", "p2", "p3", "p4", "p5a", "p5b", "p6", "p7", "p14"] {
        let path = common::corpus_dir().join(format!("{name}.sur"));
        let m = surety::mutants_file(&path, None, Budget::default()).unwrap();
        let run = m
            .mutants
            .iter()
            .find(|r| r.mutant.name() == "err_fabricated")
            .unwrap();
        assert_eq!(run.verdict, MutantVerdict::NotApplicable, "{name}");
        assert!(m.mutants.len() >= 14, "{name}");
    }
}

#[test]
fn every_committed_record_is_at_format_3_and_says_14() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../graph/nodes");
    let mut records = 0;
    for node in fs::read_dir(&root).unwrap() {
        let evidence = node.unwrap().path().join("evidence/python");
        let Ok(files) = fs::read_dir(&evidence) else {
            continue;
        };
        for f in files {
            let r: Json =
                serde_json::from_str(&fs::read_to_string(f.unwrap().path()).unwrap()).unwrap();
            assert_eq!(r["provenance"]["format"], 3, "{}", r["evidence"]["verdict"]);
            let ev = r["evidence"]["report"]
                .as_array()
                .unwrap()
                .iter()
                .find(|j| j["kind"] == "evidence")
                .unwrap();
            let u = &ev["underdetermination"];
            assert!(u["total"].as_u64().unwrap() >= 14, "{u}");
            assert!(u.get("gaps").is_some() && u.get("causes").is_none(), "{u}");
            assert_eq!(ev["examples_against"]["contradictions"], 0);
            records += 1;
        }
    }
    assert_eq!(records, 13);
}

/// A self-call clause is set aside in both searches, performance or none:
/// a performance judging candidates for its own obligation breaks the
/// referee rule. 11's metamorphic line used to pin every tie to the
/// performance's own answer; set aside, its half-cent ties are gaps, the
/// three the author wrote examples for (reading 143).
#[test]
fn a_self_call_does_not_judge_candidates_for_its_own_obligation() {
    let path = common::corpus_dir().join("p11/p11.sur");
    let r = surety::check_file(&path, &path.with_extension("py")).unwrap();
    let ties = |m: &surety_check::MutantReport| {
        let mut t: Vec<(Option<usize>, String, String)> = m
            .search
            .iter()
            .map(|u| (u.example, u.a.to_string(), u.b.to_string()))
            .collect();
        t.sort();
        t
    };
    let want = vec![
        (Some(3), "138".to_string(), "137".to_string()),
        (Some(4), "16".to_string(), "17".to_string()),
        (Some(8), "-16".to_string(), "-17".to_string()),
    ];
    assert_eq!(ties(r.mutants.as_ref().unwrap()), want);
    // The same with no performance in hand.
    let bare = surety::mutants_file(&path, None, Budget::default()).unwrap();
    assert_eq!(ties(&bare), want);
    let line = surety::verdict_line(&r);
    assert_eq!(
        line,
        "invoice_total  NOT ACCEPTED  obligation: 3 underdetermined inputs, 1 gap  performance: passes"
    );
    // The self-call is still the performance's test over its runs.
    assert!(r.passed(), "{line}");
}
