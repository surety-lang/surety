//! Slice 5, acceptance tests 2, 10 and 11: integer `+ - *` and `abs` parse
//! and print with precedence and round-trip through the renderer; `/` and
//! `%` are refused with the sheet's reason; Float, Time, Text and list
//! operands are type errors naming what each waits on; a tolerance claim
//! is one `abs` line and catches the zero mutant; and an overflow of the
//! obligation's own arithmetic is the obligation's error in the verdict
//! line and the JSON, never the performance's.

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, Mutant, MutantVerdict, PropVerdict};
use surety_syntax::normalize::canonical_function;
use surety_syntax::{Graph, parse, render};

fn with_property(body: &str) -> String {
    format!("f(a: Int, b: Int, c: Int, xs: Int[]) -> Int\n\t{body}\n")
}

fn checked(text: &str) -> Graph {
    let f = parse(text).unwrap_or_else(|e| panic!("{text}: {e}"));
    let g = Graph::single("arith", f);
    g.check_file(0).unwrap_or_else(|e| panic!("{text}: {e}"));
    g
}

#[test]
fn arithmetic_parses_and_prints_with_precedence() {
    // Source, the canonical print of the property, the renderer's spelling
    // with the fewest parentheses.
    let table = [
        (
            "out == a + b * c",
            "(== out (+ a (* b c)))",
            "out == a + b * c",
        ),
        (
            "out == (a + b) * c",
            "(== out (* (+ a b) c))",
            "out == (a + b) * c",
        ),
        (
            "out == a - b - c",
            "(== out (- (- a b) c))",
            "out == a - b - c",
        ),
        (
            "out == (a - b) - c",
            "(== out (- (- a b) c))",
            "out == a - b - c",
        ),
        (
            "out == a - (b - c)",
            "(== out (- a (- b c)))",
            "out == a - (b - c)",
        ),
        (
            "out == a * b * c",
            "(== out (* (* a b) c))",
            "out == a * b * c",
        ),
        (
            "out == a * (b * c)",
            "(== out (* a (* b c)))",
            "out == a * (b * c)",
        ),
        ("out == -1 * a", "(== out (* (int -1) a))", "out == -1 * a"),
        ("out == a - -1", "(== out (- a (int -1)))", "out == a - -1"),
        (
            "a * b > len(xs)",
            "(> (* a b) (call len xs))",
            "a * b > len(xs)",
        ),
        (
            "out == sum(xs) * (10000 + a)",
            "(== out (* (call sum xs) (+ (int 10000) a)))",
            "out == sum(xs) * (10000 + a)",
        ),
        (
            "abs(a - b) <= 5",
            "(<= (call abs (- a b)) (int 5))",
            "abs(a - b) <= 5",
        ),
        (
            "out * 10000 <= sum(xs) * (10000 + a) + 5000",
            "(<= (* out (int 10000)) (+ (* (call sum xs) (+ (int 10000) a)) (int 5000)))",
            "out * 10000 <= sum(xs) * (10000 + a) + 5000",
        ),
        (
            "a + b > c and a - b < c",
            "(and (> (+ a b) c) (< (- a b) c))",
            "a + b > c and a - b < c",
        ),
    ];
    for (source, canonical, spelled) in table {
        let g = checked(&with_property(source));
        let fun = g.functions_in(0)[0];
        let c = canonical_function(&g, 0, fun).unwrap();
        assert!(c.contains(canonical), "{source}: canonical is {c}");
        let printed = render::expr(&fun.properties[0].expr);
        assert_eq!(printed, spelled, "{source}");
        let again = checked(&with_property(&printed));
        let c2 = canonical_function(&again, 0, again.functions_in(0)[0]).unwrap();
        assert_eq!(
            c2, c,
            "{source}: the rendering parses back to the same form"
        );
    }
}

#[test]
fn division_and_modulo_are_refused_with_the_reason() {
    let e = parse(&with_property("out == a / b")).unwrap_err();
    assert!(
        e.message.starts_with(
            "`/` is not in the language: every division carries a rounding decision that is an obligation's own"
        ),
        "{e}"
    );
    assert_eq!(e.line, 2);
    let e = parse(&with_property("out == a % b")).unwrap_err();
    assert!(e.message.starts_with("`%` is not in the language"), "{e}");
    // Two terms side by side are still named as that, with the product spelled out.
    let e = parse(&with_property("a b > 0")).unwrap_err();
    assert_eq!(
        e.message,
        "unexpected `b` after `a`: two terms cannot stand side by side; a product is written `a * b`"
    );
    // A minus sign before anything but a number is a negation, written `0 - x`.
    let e = parse(&with_property("out == -a")).unwrap_err();
    assert_eq!(
        e.message,
        "a minus sign goes before a number; a negation is written `0 - x`"
    );
}

#[test]
fn operands_that_are_not_ints_name_what_they_wait_on() {
    let header = "type R = { rate: Float, t: Time }\n\ng(r: R, s: Text, xs: Int[]) -> Int\n\t";
    let cases = [
        (
            "out == r.rate * 2",
            "on Float waits for the tolerance decision",
        ),
        ("out == r.t + 1", "no time arithmetic"),
        ("out == s + 1", "no arithmetic on text"),
        ("out == xs + 1", "`sum` and `len` turn a list into one Int"),
        (
            "abs(r.rate) > 0",
            "on Float waits for the tolerance decision",
        ),
        ("abs(1.5) > 0", "on Float waits for the tolerance decision"),
        ("abs(xs) > 0", "`sum` and `len` turn a list into one Int"),
        ("out == len(xs[0 .. \"x\"])", "a slice position is an Int"),
    ];
    for (body, expected) in cases {
        let f = parse(&format!("{header}{body}\n")).unwrap_or_else(|e| panic!("{body}: {e}"));
        let e = Graph::single("g", f).check_file(0).unwrap_err();
        assert!(e.message.contains(expected), "{body}: {e}");
    }
}

fn scratch(name: &str, sur: &str, py: &str) -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("arith");
    fs::create_dir_all(&dir).unwrap();
    let s = dir.join(format!("{name}.sur"));
    let p = dir.join(format!("{name}.py"));
    fs::write(&s, sur).unwrap();
    fs::write(&p, py).unwrap();
    (s, p)
}

const INVOICE: &str = "type LineItem = { amount_cents: Int }

invoice_total(items: LineItem[], tax_rate_bps: Int) -> Int
	# The total is the subtotal with the tax applied once, to within half a cent.
	abs(out * 10000 - sum(items.amount_cents) * (10000 + tax_rate_bps)) <= 5000
	e:
		in: ([{amount_cents: 125}], 1000)
		out: 138
	e:
		in: ([{amount_cents: 135}], 1000)
		out: 148
";

const HALF_EVEN: &str = "def invoice_total(items, tax_rate_bps):
    subtotal = sum(item[\"amount_cents\"] for item in items)
    scaled = subtotal * (10000 + tax_rate_bps)
    q, r = divmod(scaled, 10000)
    if r * 2 > 10000 or (r * 2 == 10000 and q % 2 == 1):
        q += 1
    return q
";

/// Problem 11's "tax applied once" as one line: the half-even performance
/// passes it, and the mutant that returns 0 does not.
#[test]
fn a_tolerance_is_one_abs_line() {
    let (sur, py) = scratch("invoice_total", INVOICE, HALF_EVEN);
    let r = surety::check_file_with(&sur, &py, Budget::default()).unwrap();
    let text = surety::render(&r);
    assert!(r.passed() && r.findings() == 0, "{text}");
    let m = r.mutants.as_ref().unwrap();
    let empty = m
        .mutants
        .iter()
        .find(|x| x.mutant == Mutant::Empty)
        .unwrap();
    assert!(matches!(empty.verdict, MutantVerdict::Caught), "{text}");
    assert!(empty.caught_inputs > 0, "{text}");
}

/// 2^43 cubed is past the checker's i128. A performance's integers do not
/// overflow, so the error is the obligation's, named under `obligation:`
/// in the verdict line and carried as the obligation's party in the JSON.
#[test]
fn an_overflow_is_the_obligations_error_not_the_performances() {
    let sur = "cube_zero(x: Int) -> Int
	# Adding the cube of x to the output gives the cube of x: the output is zero.
	out + x * x * x == x * x * x
	e:
		in: 8796093022208
		out: 0
";
    let (s, p) = scratch("cube_zero", sur, "def cube_zero(x):\n    return 0\n");
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    let PropVerdict::Error(m) = &r.properties[0].verdict else {
        panic!("{text}");
    };
    assert!(surety_check::is_obligation_fault(m), "{m}");
    assert!(m.contains("overflows"), "{m}");
    let line = surety::verdict_line(&r);
    let (obligation, performance) = line.split_once("  performance: ").unwrap();
    assert!(
        obligation
            .contains("cannot evaluate property (line 3): the obligation's arithmetic overflows"),
        "{line}"
    );
    assert_eq!(performance, "passes", "{line}");
    let errors: Vec<_> = surety::json_lines(&r)
        .into_iter()
        .filter(|j| j["kind"] == "error")
        .collect();
    assert!(!errors.is_empty(), "{text}");
    assert!(
        errors.iter().all(|j| j["party"] == "obligation"),
        "{errors:?}"
    );
}

/// Values a fixture can reach never overflow: a product of two boundary
/// values and a sum over a list both fit, and the result stays an Int.
#[test]
fn boundary_values_multiply_without_overflow() {
    let sur = "scale(xs: Int[], k: Int) -> Int
	# The output is the sum of the inputs times k.
	out == sum(xs) * k
	e:
		in: ([9223372036854775807, 9223372036854775807], 9223372036854775807)
		out: 0
";
    let (s, p) = scratch("scale", sur, "def scale(xs, k):\n    return 0\n");
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    // The example fails the property (0 is not the product), but the
    // product itself evaluated: a Fail, not an Error.
    assert!(
        matches!(r.properties[0].verdict, PropVerdict::Fail { .. }),
        "{text}"
    );
}
