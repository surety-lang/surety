//! Saving an obligation before its performance exists. A graph reference
//! the file's own performance would supply has nothing to run: the module
//! is missing, so no Python starts, the clause is unchecked rather than
//! catching every mutant, and the run takes no longer than any other. A
//! module that fails to import starts once and is remembered for the run.
//! One test per file: the count of Python starts is per process.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use surety_check::Budget;

const SUR: &str = "type Event = { id: Id, at: Text }\ntype Bucket = { month: Text, events: Event[] }\n\nbucket(events: Event[]) -> Bucket[]\n\tall e in events: exactly_one b in out: month_of(e.at) == b.month\n\tall b in out: len(b.events) >= 1\n";

/// Small enough that the old behaviour, a new Python per call, finishes in
/// seconds and fails on the count rather than running for an hour.
fn small() -> Budget {
    Budget {
        generated: 30,
        search_inputs: 2,
        search_candidates: 40,
        mutants: true,
    }
}

fn scratch(dir: &str, py: Option<&str>) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("no-performance")
        .join(dir);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bucket.sur");
    fs::write(&path, SUR).unwrap();
    if let Some(py) = py {
        fs::write(dir.join("bucket.py"), py).unwrap();
    }
    path
}

#[test]
fn a_missing_or_broken_performance_starts_python_at_most_once() {
    // No performance: nothing starts, the clause is unchecked, and the run is quick.
    let path = scratch("missing", None);
    let before = surety_py::starts();
    let started = Instant::now();
    let r = surety::mutants_file(&path, None, small()).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(
        surety_py::starts(),
        before,
        "no Python for a missing module"
    );
    let (clause, why) = r
        .unevaluable
        .iter()
        .find(|(c, _)| c.line() == Some(5))
        .expect("the clause calling month_of is unchecked");
    assert!(why.starts_with("no performance at "), "{why}");
    assert!(
        !r.mutants.iter().any(|m| m
            .kills
            .iter()
            .any(|k| k.clause == *clause && k.message.as_deref() == Some(why.as_str()))),
        "no mutant is credited as caught by the missing module"
    );

    // A module that fails to import: one start for the whole run.
    let path = scratch("broken", Some("def month_of(s:\n"));
    let before = surety_py::starts();
    let r = surety::mutants_file(&path, None, small()).unwrap();
    assert_eq!(
        surety_py::starts(),
        before + 1,
        "one start, remembered as broken"
    );
    assert!(
        r.unevaluable
            .iter()
            .any(|(c, why)| c.line() == Some(5) && why.contains("SyntaxError")),
        "{:?}",
        r.unevaluable
    );

    // A module that imports: one process for the run, and the clause is checked.
    let path = scratch("working", Some("def month_of(s):\n    return s[:7]\n"));
    let before = surety_py::starts();
    let r = surety::mutants_file(&path, None, small()).unwrap();
    assert_eq!(
        surety_py::starts(),
        before + 1,
        "one process kept for the run"
    );
    assert!(
        !r.unevaluable.iter().any(|(c, _)| c.line() == Some(5)),
        "{:?}",
        r.unevaluable
    );
}
