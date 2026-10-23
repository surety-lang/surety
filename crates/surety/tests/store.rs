//! Slice 3, the store and pinning: a definition is an obligation by
//! canonical hash, its performances by target and structural hash, and one
//! record per performance pinned to both; the kernel admits a record only
//! when its claims are backed by recorded runs; `show` answers the model's
//! query; verify reproduces the committed graph.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use surety_check::Budget;
use surety_graph::{PinState, State, Store, pin, show, status, verify};

const ORDER: [&str; 11] = [
    "p1",
    "p2",
    "p3",
    "p4",
    "p5a",
    "p5b",
    "p6",
    "p7",
    "p8",
    "apply_diff",
    "p14",
];

fn scratch(tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("store")
        .join(tag);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn pin_corpus(store: &Store, name: &str) -> surety_graph::Pinned {
    let sur = common::corpus_dir().join(format!("{name}.sur"));
    pin(
        store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap_or_else(|e| panic!("pin {name}: {e}"))
}

/// The eleven, pinned once per test binary and shared read-only.
fn eleven() -> &'static Store {
    static STORE: OnceLock<Store> = OnceLock::new();
    STORE.get_or_init(|| {
        let store = Store::open(&scratch("eleven").join("graph"));
        for name in ORDER {
            let p = pin_corpus(&store, name);
            assert_eq!(p.state, PinState::Verified, "{name}");
        }
        store
    })
}

/// Every file under a directory with its bytes.
fn tree(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.insert(p.clone(), fs::read(&p).unwrap());
            }
        }
    }
    walk(dir, &mut out);
    out
}

/// A copy of a corpus obligation and its performance in a directory of
/// its own, with the obligation's text edited, as a source to pin.
fn scratch_source(tag: &str, name: &str, from: &str, to: &str) -> (PathBuf, PathBuf) {
    let dir = scratch(tag);
    let src = fs::read_to_string(common::corpus_dir().join(format!("{name}.sur"))).unwrap();
    assert!(
        from.is_empty() || src.contains(from),
        "{name}: {from:?} is not in the fixture"
    );
    let sur = dir.join(format!("{name}.sur"));
    fs::write(&sur, src.replace(from, to)).unwrap();
    fs::copy(
        common::corpus_dir().join(format!("{name}.py")),
        dir.join(format!("{name}.py")),
    )
    .unwrap();
    (dir, sur)
}

#[test]
fn the_eleven_pin_and_a_second_pin_writes_nothing() {
    let store = eleven();
    let ns = store.namespace().unwrap();
    assert_eq!(ns.len(), 11);
    for name in ORDER {
        let fun = ns
            .iter()
            .find(|(_, e)| e.source.ends_with(&format!("{name}.sur")))
            .unwrap_or_else(|| panic!("{name} is in the namespace"));
        let hash = &fun.1.obligation;
        assert!(store.has_node(hash));
        assert_eq!(store.presentations(hash).unwrap().len(), 1);
        assert_eq!(store.performances(hash, "python").unwrap().len(), 1);
        assert_eq!(store.records(hash, "python").unwrap().len(), 1);
    }
    let rows = status(store).unwrap();
    assert!(rows.iter().all(|r| r.state == State::Verified), "{rows:?}");
    let before = tree(&store.root);
    for name in ["p2", "p5a", "p14"] {
        let again = pin_corpus(store, name);
        assert!(!again.wrote, "{name}: a second pin wrote something");
    }
    assert_eq!(
        tree(&store.root),
        before,
        "the tree changed under a second pin"
    );
}

#[test]
fn the_kernel_refuses_hand_edited_records() {
    let base = Store::open(&scratch("kernel").join("graph"));
    pin_corpus(&base, "p2");
    let ok = verify(&base, false, surety::rendered).unwrap();
    assert!(
        ok.iter().all(|v| v.result.is_ok()),
        "the checker's own record is admitted"
    );
    let hash = base.namespace().unwrap()["get_small_orders"]
        .obligation
        .clone();
    let perf = base.performances(&hash, "python").unwrap()[0].clone();

    let copy = |tag: &str| -> Store {
        let root = scratch(tag).join("graph");
        for (p, bytes) in tree(&base.root) {
            let rel = p.strip_prefix(&base.root).unwrap();
            fs::create_dir_all(root.join(rel).parent().unwrap()).unwrap();
            fs::write(root.join(rel), bytes).unwrap();
        }
        Store::open(&root)
    };
    let refusal = |store: &Store| -> String {
        let vs = verify(store, false, surety::rendered).unwrap();
        assert_eq!(vs.len(), 1);
        vs[0].result.clone().expect_err("the kernel should refuse")
    };

    // A claim's count, changed by hand.
    let s = copy("kernel-count");
    let mut rec = s.record(&hash, "python", &perf).unwrap().unwrap();
    let lines = rec["evidence"]["report"].as_array_mut().unwrap();
    let ev = lines.iter_mut().find(|l| l["kind"] == "evidence").unwrap();
    ev["examples"]["passed"] = serde_json::json!(0);
    fs::write(
        s.record_path(&hash, "python", &perf),
        serde_json::to_string_pretty(&rec).unwrap(),
    )
    .unwrap();
    assert!(refusal(&s).contains("examples"), "{}", refusal(&s));

    // The performance copy, edited.
    let s = copy("kernel-perf");
    let p = s.performance_path(&hash, "python", &perf);
    let mut text = fs::read_to_string(&p).unwrap();
    text.push_str("\n\ndef extra():\n    return 1\n");
    fs::write(&p, text).unwrap();
    assert!(refusal(&s).contains("performance hash"), "{}", refusal(&s));

    // A run-log entry, removed.
    let s = copy("kernel-runs");
    let mut rec = s.record(&hash, "python", &perf).unwrap().unwrap();
    rec["evidence"]["runs"].as_array_mut().unwrap().pop();
    fs::write(
        s.record_path(&hash, "python", &perf),
        serde_json::to_string_pretty(&rec).unwrap(),
    )
    .unwrap();
    assert!(refusal(&s).contains("run log"), "{}", refusal(&s));
}

#[test]
fn stale_by_construction_and_the_old_node_still_answers() {
    let (_dir, sur) = scratch_source("stale", "p5a", "", "");
    let store = Store::open(&scratch("stale-graph").join("graph"));
    let first = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(first.state, PinState::Verified);
    let old = first.obligation.clone();
    // The author changes a property: the file now hashes elsewhere.
    let text = fs::read_to_string(&sur).unwrap();
    assert!(text.contains("o.ordered_at <= l_o.ordered_at"));
    fs::write(
        &sur,
        text.replace(
            "o.ordered_at <= l_o.ordered_at",
            "o.ordered_at < l_o.ordered_at",
        ),
    )
    .unwrap();
    let rows = status(&store).unwrap();
    assert_eq!(rows.len(), 1);
    match &rows[0].state {
        State::Stale { now } => assert_ne!(now, &old),
        other => panic!("expected stale, got {other:?}"),
    }
    let view = show(&store, &old[..12]).unwrap();
    assert_eq!(view.hash, old, "the old node answers by hash");
    assert_eq!(view.names, vec!["get_first_customer_orders".to_string()]);
    assert!(matches!(view.state, State::Stale { .. }));
    assert_eq!(view.performances.len(), 1);
    assert!(
        view.performances[0].verified,
        "the record under the old hash is untouched"
    );
    let text = surety::render_show(&view);
    assert!(
        text.starts_with("get_first_customer_orders  STALE (NOW "),
        "{text}"
    );
}

#[test]
fn one_hop_each_way() {
    let store = eleven();
    let p14 = show(store, "snapshot_diff").unwrap();
    assert_eq!(
        p14.callees,
        vec![("apply_diff".to_string(), State::Verified)]
    );
    // Type providers are transitive: Diff comes from apply_diff, whose
    // DiffError is validate_signup's ValidationError.
    assert_eq!(
        p14.uses,
        vec![
            ("apply_diff".to_string(), State::Verified),
            ("validate_signup".to_string(), State::Verified)
        ]
    );
    assert!(p14.callers.is_empty());
    let apply = show(store, "apply_diff").unwrap();
    assert_eq!(
        apply.callers,
        vec![("snapshot_diff".to_string(), State::Verified)]
    );
    assert_eq!(
        apply.uses,
        vec![("validate_signup".to_string(), State::Verified)]
    );
    assert!(apply.callees.is_empty());
    let text = surety::render_show(&apply);
    assert!(
        text.contains("called by: snapshot_diff (verified)"),
        "{text}"
    );
    assert!(
        text.contains("uses types from: validate_signup (verified)"),
        "{text}"
    );
    let json = surety::show_json(&p14);
    assert_eq!(json["calls"][0]["name"], "apply_diff");
    assert_eq!(json["state"], "verified");
}

#[test]
fn two_presentations_one_node() {
    let store = Store::open(&scratch("presentations").join("graph"));
    let first = pin_corpus(&store, "p3");
    let parsed =
        surety_syntax::parse(&fs::read_to_string(common::corpus_dir().join("p3.sur")).unwrap())
            .unwrap();
    let fresh = surety_syntax::render::file(&parsed, true);
    let (_dir, sur) = scratch_source("presentations-src", "p3", "", "");
    fs::write(&sur, fresh).unwrap();
    let second = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(
        second.obligation, first.obligation,
        "renamed binders, the same node"
    );
    assert_eq!(store.presentations(&first.obligation).unwrap().len(), 2);
    let view = show(&store, "group_by_customer").unwrap();
    assert_eq!(view.presentations.len(), 2);
    assert_eq!(view.performances.len(), 1, "one performance, one record");
}

#[test]
fn open_then_verified() {
    let dir = scratch("open");
    let sur = dir.join("p2.sur");
    fs::copy(common::corpus_dir().join("p2.sur"), &sur).unwrap();
    let store = Store::open(&dir.join("graph"));
    let open = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(open.state, PinState::Open);
    assert!(open.performance.is_none());
    assert_eq!(status(&store).unwrap()[0].state, State::Open);
    let view = show(&store, "get_small_orders").unwrap();
    assert!(view.performances.is_empty());
    assert!(surety::render_show(&view).contains("performances: none (open)"));
    fs::copy(common::corpus_dir().join("p2.py"), dir.join("p2.py")).unwrap();
    let verified = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(verified.state, PinState::Verified);
    assert_eq!(status(&store).unwrap()[0].state, State::Verified);
    assert_eq!(
        surety::pin_line(&open),
        format!("pinned get_small_orders {}: open", open.obligation)
    );
}

#[test]
fn findings_are_a_state() {
    let (_dir, sur) = scratch_source(
        "findings",
        "p5a",
        "\taccept any a in orders: any b in orders where b != a and b.customer_email == a.customer_email: b.ordered_at == a.ordered_at\n",
        "",
    );
    let store = Store::open(&scratch("findings-graph").join("graph"));
    let p = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(p.state, PinState::Findings, "{}", surety::pin_line(&p));
    assert!(p.report.as_ref().unwrap().findings() > 0);
    assert_eq!(status(&store).unwrap()[0].state, State::Findings);
    let view = show(&store, "get_first_customer_orders").unwrap();
    assert_eq!(view.performances.len(), 1);
    assert!(!view.performances[0].verified);
    let verdict = &view.performances[0].verdict;
    assert!(
        verdict.contains("NOT ACCEPTED") && verdict.contains("underdetermined"),
        "{verdict}"
    );
    let vs = verify(&store, false, surety::rendered).unwrap();
    assert!(
        vs[0].result.is_ok(),
        "a record with findings is admitted: {:?}",
        vs[0].result
    );
}

/// The hardest one: the committed graph at the repository root, re-run
/// record by record, every evidence part byte-identical, here and in CI.
/// The eleven, and 13 and 11 each from its own directory of the corpus
/// (slice 6).
#[test]
fn verify_reproduces_the_committed_graph() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../graph");
    let store = Store::open(&root);
    assert!(store.exists(), "graph/ is committed at the repository root");
    let rows = status(&store).unwrap();
    assert_eq!(rows.len(), 13);
    assert!(
        rows.iter().all(|r| r.state == State::Verified),
        "re-pin the corpus (surety pin) after changing an obligation or a performance:\n{}",
        surety::render_status(&rows)
    );
    let vs = verify(&store, true, surety::rendered).unwrap();
    assert_eq!(vs.len(), 13);
    for v in &vs {
        assert!(
            v.result.is_ok(),
            "{} {}: {:?}",
            v.name,
            &v.node[..12],
            v.result
        );
    }
}

#[test]
fn pin_refuses_a_file_that_still_carries_the_draft_mark() {
    let dir = scratch("draft-mark");
    let sur = dir.join("p2.sur");
    let src = fs::read_to_string(common::corpus_dir().join("p2.sur")).unwrap();
    fs::write(&sur, format!("// draft\n{src}")).unwrap();
    fs::copy(common::corpus_dir().join("p2.py"), dir.join("p2.py")).unwrap();
    let store = Store::open(&dir.join("graph"));
    let err = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .expect_err("the mark is refused");
    assert!(err.contains("draft mark"), "{err}");
    assert!(store.namespace().unwrap().is_empty(), "nothing was pinned");
    // The mark removed, the same file pins.
    fs::write(&sur, src).unwrap();
    let p = pin(
        &store,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(p.state, PinState::Verified);
}

/// A second presentation of a corpus obligation: a header comment, the
/// properties and the examples in reverse order, every binder renamed.
fn other_presentation(name: &str) -> String {
    let mut parsed = surety_syntax::parse(
        &fs::read_to_string(common::corpus_dir().join(format!("{name}.sur"))).unwrap(),
    )
    .unwrap();
    for d in &mut parsed.decls {
        if let surety_syntax::Decl::Function(f) = d {
            f.properties.reverse();
            f.examples.reverse();
        }
    }
    format!(
        "// a second presentation of the same obligation\n{}",
        surety_syntax::render::file(&parsed, true)
    )
}

#[test]
fn evidence_is_a_function_of_the_hashes_not_the_presentation() {
    // Half one: two presentations of one node, one performance, in two
    // stores of their own: byte-identical evidence.
    let a = Store::open(&scratch("invariant-a").join("graph"));
    let first = pin_corpus(&a, "p5a");
    let (_dir, sur) = scratch_source("invariant-src", "p5a", "", "");
    fs::write(&sur, other_presentation("p5a")).unwrap();
    let b = Store::open(&scratch("invariant-b").join("graph"));
    let second = pin(
        &b,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    assert_eq!(second.obligation, first.obligation, "the same node");
    let perf = a.performances(&first.obligation, "python").unwrap()[0].clone();
    let rec_a = a
        .record(&first.obligation, "python", &perf)
        .unwrap()
        .unwrap();
    let rec_b = b
        .record(&first.obligation, "python", &perf)
        .unwrap()
        .unwrap();
    assert_ne!(
        rec_a["provenance"]["presentation"], rec_b["provenance"]["presentation"],
        "two presentations"
    );
    assert_eq!(
        serde_json::to_string(&rec_a["evidence"]).unwrap(),
        serde_json::to_string(&rec_b["evidence"]).unwrap(),
        "one evidence part"
    );
    let text = serde_json::to_string(&rec_a["evidence"]).unwrap();
    assert!(
        !text.contains("\"line\""),
        "no presentation line in the evidence"
    );
    assert!(
        text.contains("\"canonical\""),
        "clauses carry their canonical print"
    );
    assert_eq!(
        rec_a["provenance"]["format"],
        serde_json::json!(surety_graph::FORMAT)
    );

    // Half two: the record, with the other presentation named in its
    // provenance, verifies when materialized under it.
    let both = Store::open(&scratch("invariant-both").join("graph"));
    pin_corpus(&both, "p5a");
    pin(
        &both,
        &sur,
        None,
        Budget::default(),
        surety::TIMEOUT,
        surety::rendered,
    )
    .unwrap();
    let presentations = both.presentations(&first.obligation).unwrap();
    assert_eq!(presentations.len(), 2);
    let mut rec = both
        .record(&first.obligation, "python", &perf)
        .unwrap()
        .unwrap();
    let current = rec["provenance"]["presentation"]
        .as_str()
        .unwrap()
        .to_string();
    let other = presentations
        .iter()
        .find(|p| **p != current)
        .unwrap()
        .clone();
    rec["provenance"]["presentation"] = serde_json::json!(other);
    fs::write(
        both.record_path(&first.obligation, "python", &perf),
        serde_json::to_string_pretty(&rec).unwrap(),
    )
    .unwrap();
    let vs = verify(&both, true, surety::rendered).unwrap();
    assert_eq!(vs.len(), 1);
    assert!(vs[0].result.is_ok(), "{:?}", vs[0].result);
}

#[test]
fn verify_says_format_moved_for_an_old_record() {
    let store = Store::open(&scratch("format").join("graph"));
    let p = pin_corpus(&store, "p2");
    let perf = p.performance.clone().unwrap();
    let mut rec = store
        .record(&p.obligation, "python", &perf)
        .unwrap()
        .unwrap();
    rec["provenance"]["format"] = serde_json::json!(1);
    fs::write(
        store.record_path(&p.obligation, "python", &perf),
        serde_json::to_string_pretty(&rec).unwrap(),
    )
    .unwrap();
    let vs = verify(&store, false, surety::rendered).unwrap();
    let err = vs[0]
        .result
        .clone()
        .expect_err("an old format is named, not compared");
    assert!(err.contains("format moved"), "{err}");
}
