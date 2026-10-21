//! Acceptance test 1: all eleven fixtures parse; every `bad/` file fails
//! with the expected error kind and line. Plus unit tests for the forms the
//! brief lists that no fixture uses.

mod common;

use std::fs;

use surety_syntax::{Graph, parse};

#[test]
fn all_eleven_parse_and_check() {
    let graph = common::corpus();
    assert_eq!(graph.files.len(), 11, "expected eleven fixtures");
    for name in common::FIXTURES {
        let idx = common::fixture_index(&graph, name);
        graph
            .check_file(idx)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            graph.functions_in(idx).len(),
            1,
            "{name}: one obligation per file"
        );
    }
}

#[test]
fn fixture_shapes_match_the_corpus() {
    let graph = common::corpus();
    // (fixture, properties, examples, local abbreviations) counted by hand.
    let expected = [
        ("p1", 2, 2, 0),
        ("p2", 4, 2, 0),
        ("p3", 8, 2, 0),
        ("p4", 4, 5, 0),
        ("p5a", 5, 3, 0),
        ("p5b", 5, 3, 0),
        ("p6", 5, 2, 0),
        ("p7", 4, 3, 0),
        ("p8", 7, 12, 1),
        ("apply_diff", 14, 8, 5),
        ("p14", 16, 3, 0),
    ];
    for (name, props, examples, abbrevs) in expected {
        let idx = common::fixture_index(&graph, name);
        let f = graph.functions_in(idx)[0];
        assert_eq!(f.properties.len(), props, "{name}: properties");
        assert_eq!(f.examples.len(), examples, "{name}: examples");
        assert_eq!(f.abbrevs.len(), abbrevs, "{name}: local abbreviations");
    }
}

#[test]
fn every_bad_fixture_fails_as_expected() {
    let dir = common::corpus_dir().join("bad");
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "sur") {
            continue;
        }
        count += 1;
        let err_path = path.with_extension("err");
        let expected = fs::read_to_string(&err_path)
            .unwrap_or_else(|_| panic!("missing {}", err_path.display()));
        let mut parts = expected.split_whitespace();
        let kind = parts.next().unwrap();
        let line: usize = parts.next().unwrap().parse().unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy();
        let result = parse(&text).and_then(|f| {
            let g = Graph::single("bad", f);
            g.check_file(0)
        });
        match result {
            Ok(()) => panic!("{name}: expected error `{kind}` at line {line}, but it was accepted"),
            Err(e) => {
                assert_eq!(e.kind.name(), kind, "{name}: error kind ({e})");
                assert_eq!(e.line, line, "{name}: error line ({e})");
            }
        }
    }
    assert!(
        count >= 10,
        "expected at least the brief's negative fixtures, found {count}"
    );
}

/// Forms in the brief that no fixture exercises: `any`, `none`, `count`,
/// `max`, `->`, `<`, `[0..3, unique f]`, `Bool`, `UUID`.
#[test]
fn brief_only_forms_parse() {
    let src = "type Row = { id: UUID, flag: Bool, n: Int }
pick(rows: Row[], limit: Int) -> Row[0..3, unique id]
\tnone r in out: r.n < 0
\tany r in out: r.flag == r.flag
\tcount(out) <= max(limit, 3)
\tlen(out) == 0 -> len(rows) == 0
\tnone Err(_) in out
\te:
\t\tin: ([{ id: \"a\", flag: 1, n: 1 }], 3)
\t\tout: [{ id: \"a\", flag: 1, n: 1 }]
";
    // Line by line: everything parses; the checker then rejects the Int in
    // a Bool slot (there is no Bool literal in the corpus).
    let file = parse(src).expect("parses");
    let g = Graph::single("forms", file);
    let err = g.check_file(0).expect_err("an Int literal in a Bool field");
    assert_eq!(err.line, 7, "{err}");
    // `none Err(_) in out` needs a Result domain; `out` is a list of records.
    let src = src.replace("none Err(_) in out", "none Err(_) in out.n");
    let file = parse(&src).expect("parses");
    let g = Graph::single("forms", file);
    let err = g.check_file(0).expect_err("pattern on a non-Result domain");
    assert_eq!(err.line, 7, "{err}");
    // Without the Bool field and the pattern line, the file checks.
    let src = src
        .replace(", flag: Bool", "")
        .replace(", flag: 1", "")
        .replace(
            "\tany r in out: r.flag == r.flag\n",
            "\tany r in out: r.n == r.n\n",
        )
        .replace("\tnone Err(_) in out.n\n", "");
    let file = parse(&src).expect("parses");
    let g = Graph::single("forms", file);
    g.check_file(0).expect("checks");
}

#[test]
fn hash_and_structure_survive_reextraction() {
    // The authored markdown is not in the repository; when it is present
    // locally, extraction must reproduce the committed fixtures byte for byte.
    let md = common::corpus_dir().join("../../../../surety-docs/Implement 1-9 in surety.md");
    let Ok(text) = fs::read_to_string(&md) else {
        eprintln!("skipping: {} not present", md.display());
        return;
    };
    let extracted = surety_syntax::extract::extract(&text);
    assert_eq!(extracted.len(), 11);
    for ex in extracted {
        let path = common::corpus_dir().join(format!("{}.sur", ex.name));
        let committed = fs::read_to_string(&path).unwrap();
        assert_eq!(ex.text, committed, "{} drifted from the markdown", ex.name);
    }
}
