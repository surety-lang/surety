//! Slice 5, acceptance test 3: the positional slice `xs[a .. b]` is the
//! elements from position a up to but not including b, clamped to the list
//! and never an error; a negative start clamps to 0 and does not count
//! from the end; the result is the element list with its refinements
//! dropped; and `Item[0..3]` in a type position is still the length
//! refinement.

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, Verdict};
use surety_syntax::normalize::canonical_function;
use surety_syntax::{Expr, Graph, parse, render};

#[test]
fn a_slice_parses_prints_and_keeps_apart_from_the_refinement() {
    let text = "type Item = { id: Id }\n\nf(xs: Item[0..3], page: Int, size: Int) -> Item[]\n\tout == xs[page * size .. (page + 1) * size]\n";
    let f = parse(text).unwrap();
    let g = Graph::single("slice", f.clone());
    g.check_file(0).unwrap();
    let fun = g.functions_in(0)[0];
    assert!(matches!(
        &fun.properties[0].expr,
        Expr::Binary(_, _, b) if matches!(**b, Expr::Slice { .. })
    ));
    let c = canonical_function(&g, 0, fun).unwrap();
    assert!(
        c.contains("(slice xs (* page size) (* (+ page (int 1)) size))"),
        "{c}"
    );
    assert_eq!(
        render::expr(&fun.properties[0].expr),
        "out == xs[page * size .. (page + 1) * size]"
    );
    let rendered = render::file(&f, false);
    assert!(
        rendered.contains("f(xs: Item[0..3], page: Int, size: Int) -> Item[]"),
        "the length refinement stays a refinement:\n{rendered}"
    );
    assert_eq!(
        render::file(&parse(&rendered).unwrap(), false),
        rendered,
        "the rendering reads back the same"
    );
    // A negative start and a slice of a slice parse; the canonical nests.
    let f = parse("g(xs: Int[]) -> Int[]\n\tout == xs[-1 .. 2][0 .. 1]\n").unwrap();
    let g = Graph::single("slice2", f);
    g.check_file(0).unwrap();
    let c = canonical_function(&g, 0, g.functions_in(0)[0]).unwrap();
    assert!(
        c.contains("(slice (slice xs (int -1) (int 2)) (int 0) (int 1))"),
        "{c}"
    );
    // The slice has the list's element type.
    let f = parse("h(xs: Int[]) -> Text[]\n\tout == xs[0 .. 1]\n").unwrap();
    let e = Graph::single("slice3", f).check_file(0).unwrap_err();
    assert!(e.message.contains("cannot compare"), "{e}");
    // `..` is required.
    let e = parse("h(xs: Int[]) -> Int[]\n\tout == xs[0]\n").unwrap_err();
    assert!(e.message.starts_with("expected `..` in a slice"), "{e}");
}

const WINDOW: &str = "window(xs: Int[], a: Int, b: Int) -> Int[]
	# The output is the elements of xs from position a up to but not including b, clamped to the list.
	out == xs[a .. b]
	e:
		in: ([1, 2, 3, 4, 5], 1, 3)
		out: [2, 3]
	e:
		in: ([1, 2, 3], 1, 10)
		out: [2, 3]
	e:
		in: ([1, 2, 3], 5, 7)
		out: []
	e:
		in: ([1, 2, 3], 2, 1)
		out: []
	e:
		in: ([1, 2, 3], -1, 2)
		out: [1, 2]
";

fn scratch(name: &str, py: &str) -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("slice");
    fs::create_dir_all(&dir).unwrap();
    let s = dir.join(format!("{name}.sur"));
    let p = dir.join(format!("{name}.py"));
    fs::write(
        &s,
        WINDOW
            .replace("window(", &format!("{name}("))
            .replace("window", name),
    )
    .unwrap();
    fs::write(&p, py).unwrap();
    (s, p)
}

/// Inside the list, past the end, start past the end, b below a, negative
/// a: the clamped performance passes every example and every generated
/// input, and the slice raises on none.
#[test]
fn the_slice_is_clamped_and_never_an_error() {
    let (s, p) = scratch(
        "clamped",
        "def clamped(xs, a, b):\n    n = len(xs)\n    a = min(max(a, 0), n)\n    b = min(max(b, a), n)\n    return xs[a:b]\n",
    );
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    assert!(r.passed() && r.findings() == 0, "{text}");
    assert_eq!(
        r.properties[0].runs, r.runs,
        "evaluated on every run:\n{text}"
    );
}

/// Python's own slice counts a negative start from the end: that
/// performance fails the fifth example and nothing else.
#[test]
fn a_negative_start_does_not_count_from_the_end() {
    let (s, p) = scratch("pythonic", "def pythonic(xs, a, b):\n    return xs[a:b]\n");
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    assert!(!r.passed(), "{text}");
    let failing: Vec<usize> = r
        .examples
        .iter()
        .filter(|e| e.verdict != Verdict::Pass)
        .map(|e| e.index + 1)
        .collect();
    assert_eq!(failing, vec![5], "{text}");
}
