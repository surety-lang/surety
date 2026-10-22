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

/// Two lists after `in` (a domain typed twice) is named as that, not as
/// the binder the bodyless reading would leave unused.
#[test]
fn a_second_domain_is_named() {
    let text = fs::read_to_string(common::corpus_dir().join("bad/two_domains.sur")).unwrap();
    let e = parse(&text).unwrap_err();
    assert_eq!(
        e.message,
        "two lists after `in`: `out` and `orders.amount`; a quantifier ranges over one"
    );
}

/// A minus sign before a number is part of it, wherever a number goes: a
/// named value, a property, a float, an example, and the line the editor's
/// quick fix writes for a gap whose input is negative. Rendered, it reads
/// back the same. Before anything but a number it is refused, since there
/// is no arithmetic yet.
#[test]
fn a_number_may_be_negative() {
    let text = "type Item = { amount: Int, rate: Float }\n\nfloor: Int = -5\n\ntotal(items: Item[], tax: Int) -> Int\n\tall i in items: i.amount > -1000 and i.rate >= -2.5\n\ttax == -1 -> out == floor\n\n\te:\n\t\tin: ([{amount: -125, rate: -0.5}], -1)\n\t\tout: -5\n\n\te:\n\t\tin: ([], -1)\n\t\tout: 0\n";
    let f = parse(text).unwrap();
    let g = Graph::single("neg", f.clone());
    g.check_file(0).unwrap();
    let rendered = surety_syntax::render::file(&f, false);
    assert!(rendered.contains("floor: Int = -5"), "{rendered}");
    assert!(
        rendered.contains("i.amount > -1000 and i.rate >= -2.5"),
        "{rendered}"
    );
    assert!(
        rendered.contains("in: ([{amount: -125, rate: -0.5}], -1)"),
        "{rendered}"
    );
    assert_eq!(
        surety_syntax::render::file(&parse(&rendered).unwrap(), false),
        rendered,
        "it reads back the same"
    );
    let e = parse("total(xs: Int[]) -> Int\n\tout == -xs\n").unwrap_err();
    assert_eq!(
        e.message,
        "a minus sign goes before a number; the language has no arithmetic yet"
    );
    assert_eq!(e.span, Some((8, 9)), "the minus sign");
}

/// A field on something that is not a record says what the thing is.
#[test]
fn a_field_on_a_non_record_says_what_it_is() {
    let text = fs::read_to_string(common::corpus_dir().join("bad/not_a_record.sur")).unwrap();
    let f = parse(&text).unwrap();
    let e = Graph::single("bad", f).check_file(0).unwrap_err();
    assert_eq!(
        e.message,
        "`out` is Int, not a record, so it has no field `amount`"
    );
}

/// An error that knows its tokens names their columns, so the editor
/// underlines the term and not the line: each span here is checked against
/// the text it should cover.
#[test]
fn an_error_names_the_columns_it_is_about() {
    let covered = |name: &str| {
        let text =
            fs::read_to_string(common::corpus_dir().join(format!("bad/{name}.sur"))).unwrap();
        let e = parse(&text).unwrap_err();
        let (from, to) = e.span.unwrap_or_else(|| panic!("{name}: no span on {e}"));
        let line = text.lines().nth(e.line - 1).unwrap();
        line.chars().skip(from).take(to - from).collect::<String>()
    };
    assert_eq!(covered("two_domains"), "orders.amount");
    assert_eq!(covered("empty_leftover"), "leftover in out");
    assert_eq!(covered("unused_binder"), "Err(erors)");
    assert_eq!(covered("unresolved_choice"), "<<<<<<<");
    // A head ending in `:` with nothing under it: the head.
    let e = parse("type A = { x: Int }\nf(a: A) -> Int\n\tall o in [a]:\n").unwrap_err();
    assert_eq!(e.span, Some((1, 14)), "{e}");
    // A token the parser did not expect, and a character the lexer cannot read.
    let e = parse("type A = { x: Int }\nf(a: A) -> Int\n\tall o of [a]: o.x > 0\n").unwrap_err();
    assert_eq!(e.span, Some((7, 9)), "`of`: {e}");
    let e = parse("type A = { x: Int }\nf(a: A) -> Int\n\tout > 0 ; 1\n").unwrap_err();
    assert_eq!(e.span, Some((9, 10)), "`;`: {e}");
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
