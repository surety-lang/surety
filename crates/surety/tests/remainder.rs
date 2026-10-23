//! `%` on `Int`, decided by the author on 2026-10-23 evening (spec f6d00db)
//! to name problem 11's ties. No fixture has it, so it has its own test:
//! exact, floored (the remainder takes the divisor's sign, as Python's
//! does), at `*`'s precedence and left-associative with it, canonical
//! `(% a b)`, round-tripping through the renderer; `% 0` the obligation's
//! error like an overflow; a `Float` operand refused with what it waits
//! on; `/` still refused (`arith.rs`).

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, PropVerdict};
use surety_syntax::normalize::canonical_function;
use surety_syntax::{Graph, parse, render};

fn with_property(body: &str) -> String {
    format!("f(a: Int, b: Int, c: Int, xs: Int[]) -> Int\n\t{body}\n")
}

fn checked(text: &str) -> Graph {
    let f = parse(text).unwrap_or_else(|e| panic!("{text}: {e}"));
    let g = Graph::single("remainder", f);
    g.check_file(0).unwrap_or_else(|e| panic!("{text}: {e}"));
    g
}

fn scratch(name: &str, sur: &str, py: &str) -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("remainder");
    fs::create_dir_all(&dir).unwrap();
    let s = dir.join(format!("{name}.sur"));
    let p = dir.join(format!("{name}.py"));
    fs::write(&s, sur).unwrap();
    fs::write(&p, py).unwrap();
    (s, p)
}

#[test]
fn the_remainder_parses_at_the_products_precedence_and_round_trips() {
    // Source, the canonical print, the renderer's spelling with the fewest
    // parentheses.
    let table = [
        ("out == a % b", "(== out (% a b))", "out == a % b"),
        (
            "out == a + b % c",
            "(== out (+ a (% b c)))",
            "out == a + b % c",
        ),
        (
            "out == (a + b) % c",
            "(== out (% (+ a b) c))",
            "out == (a + b) % c",
        ),
        (
            "out == a * b % c",
            "(== out (% (* a b) c))",
            "out == a * b % c",
        ),
        (
            "out == (a * b) % c",
            "(== out (% (* a b) c))",
            "out == a * b % c",
        ),
        (
            "out == a % (b * c)",
            "(== out (% a (* b c)))",
            "out == a % (b * c)",
        ),
        (
            "out == a % b * c",
            "(== out (* (% a b) c))",
            "out == a % b * c",
        ),
        (
            "out == a % b % c",
            "(== out (% (% a b) c))",
            "out == a % b % c",
        ),
        (
            "out == a % (b % c)",
            "(== out (% a (% b c)))",
            "out == a % (b % c)",
        ),
        ("out == a % -2", "(== out (% a (int -2)))", "out == a % -2"),
        // 11's half-even line, as the author writes it.
        (
            "(sum(xs) * (10000 + a)) % 10000 == 5000 -> out % 2 == 0",
            "(-> (== (% (* (call sum xs) (+ (int 10000) a)) (int 10000)) (int 5000)) (== (% out (int 2)) (int 0)))",
            "sum(xs) * (10000 + a) % 10000 == 5000 -> out % 2 == 0",
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

/// Floored on every sign, as Python's `%`: the examples state it, and a
/// performance that is Python's own `%` passes every generated input,
/// boundary values and negatives among them.
#[test]
fn the_remainder_takes_the_divisors_sign_as_pythons_does() {
    let mut sur = "rem(a: Int, b: Int) -> Int\n\t# Off zero, the output is a's remainder on b.\n\tb != 0 -> out == a % b\n".to_string();
    for (a, b, r) in [
        (7, 3, 1),
        (-7, 3, 2),
        (7, -3, -2),
        (-7, -3, -1),
        (0, 5, 0),
        (6, -3, 0),
        (-165000, 10000, 5000),
        (165000, -10000, -5000),
    ] {
        sur.push_str(&format!("\te:\n\t\tin: ({a}, {b})\n\t\tout: {r}\n"));
    }
    let py = "def rem(a, b):\n    return a % b if b else 0\n";
    let (s, p) = scratch("rem", &sur, py);
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    assert!(r.against.clean(), "{text}");
    assert_eq!(r.against.evaluated(), 8, "{text}");
    assert_eq!(r.properties[0].verdict, PropVerdict::Pass, "{text}");
    assert_eq!(r.properties[0].findings(), 0, "{text}");
    assert!(
        r.generated_inputs > 0 && r.properties[0].runs == r.runs,
        "{text}"
    );
    assert!(r.passed(), "{text}");
}

/// `% 0` has no remainder: like an overflow it is the obligation's error,
/// found on the example's own pair, never the performance's.
#[test]
fn a_remainder_on_zero_is_the_obligations_error() {
    let sur = "zero_rem(x: Int) -> Int\n\t# The output is x's remainder on 0.\n\tout == x % 0\n\te:\n\t\tin: 5\n\t\tout: 0\n";
    let (s, p) = scratch("zero_rem", sur, "def zero_rem(x):\n    return 0\n");
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    let faults = r.against.contradictions();
    let Some(m) = faults.first().and_then(|c| c.fault) else {
        panic!("{text}");
    };
    assert!(surety_check::is_obligation_fault(m), "{m}");
    assert!(m.contains("divides by zero: 5 % 0"), "{m}");
    let line = surety::verdict_line(&r);
    let (obligation, performance) = line.split_once("  performance: ").unwrap();
    assert!(
        obligation.contains(
            "cannot evaluate line 3 on example 1: the obligation's arithmetic divides by zero"
        ),
        "{line}"
    );
    assert_eq!(performance, "passes", "{line}");
    assert!(
        surety::json_lines(&r)
            .iter()
            .filter(|j| j["kind"] == "example_unevaluable")
            .all(|j| j["party"] == "obligation")
    );
}

#[test]
fn a_remainder_on_anything_but_int_names_what_it_waits_on() {
    let text = "type R = { rate: Float }\n\ng(r: R, s: Text) -> Int\n\tr.rate % 2 == 0\n";
    let e = Graph::single("g", parse(text).unwrap())
        .check_file(0)
        .unwrap_err();
    assert!(
        e.message
            .starts_with("`%` on Float waits for the tolerance decision"),
        "{e}"
    );
    let text = "g(s: Text) -> Int\n\tlen(s) % 2 == 0 and s % 2 == 0\n";
    let e = Graph::single("g", parse(text).unwrap())
        .check_file(0)
        .unwrap_err();
    assert!(
        e.message.starts_with("`%` on Text: no arithmetic on text"),
        "{e}"
    );
}
