//! Slice 5, acceptance test 9: the scalar family has teeth. For an `Int`
//! output the library is no longer two mutants: the constants, and each
//! `Int` the inputs afford with one more, one less, negated and times 100,
//! are wrong performances the tolerance line catches. For a record output
//! the family edits one scalar of the skeleton, so the arithmetic clauses
//! about `total_pages` and `has_next` catch something and leave the
//! look-here section.

use std::fs;
use std::path::PathBuf;

use surety_check::{Budget, LookKind, Mutant, MutantVerdict};

fn scratch(name: &str, sur: &str, py: &str) -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scalar");
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

#[test]
fn an_int_output_gets_the_wrong_performances_the_inputs_afford() {
    let (s, p) = scratch("invoice_total", INVOICE, HALF_EVEN);
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    assert!(r.passed() && r.findings() == 0, "{text}");
    let m = r.mutants.as_ref().unwrap();
    let names: Vec<String> = m.mutants.iter().map(|x| x.mutant.name()).collect();
    for expected in [
        "out = 1",
        "out = -1",
        "out = sum(items.amount_cents)",
        "out = sum(items.amount_cents) + 1",
        "out = sum(items.amount_cents) - 1",
        "out = 0 - sum(items.amount_cents)",
        "out = sum(items.amount_cents) * 100",
        "out = tax_rate_bps + 1",
        "out = tax_rate_bps - 1",
        "out = 0 - tax_rate_bps",
        "out = tax_rate_bps * 100",
    ] {
        assert!(
            names.iter().any(|n| n == expected),
            "{expected} is missing: {names:?}"
        );
    }
    assert!(
        !names.iter().any(|n| n == "out = tax_rate_bps"),
        "the input itself is `identity`: {names:?}"
    );
    assert!(
        m.applicable() > 2,
        "two applied before the family: {}",
        m.applicable()
    );
    for run in m
        .mutants
        .iter()
        .filter(|x| matches!(x.mutant, Mutant::Scalar(_)))
    {
        assert!(
            matches!(run.verdict, MutantVerdict::Caught),
            "{} slipped:\n{}",
            run.mutant.name(),
            surety::render_mutants(m, false)
        );
    }
    let sum = m
        .mutants
        .iter()
        .find(|x| x.mutant.name() == "out = sum(items.amount_cents)")
        .unwrap();
    assert_eq!(sum.mutant.describe(), "returns sum(items.amount_cents)");
}

const PAGINATE: &str = "type Item = { id: Id }
type Page = { items: Item[], total_pages: Int, has_next: Bool }
type PageError = { code: Text }

page_range_err: PageError = { code: \"e-page-range\" }

paginate(items: Item[], page: Int, size: Int) -> Result<Page, PageError>
	# An error is the range error.
	all Err(e) in out: e == page_range_err
	# The output is an error exactly when the page is negative, the size is less than 1, or page times size is more than the item count.
	(page < 0 or size < 1 or page * size > len(items)) <-> any Err(e) in out: e == page_range_err
	# The page's items are the items from position page times size up to the next page's start.
	all Ok(p) in out: p.items == items[page * size .. (page + 1) * size]
	# total_pages times size covers the items, and one page fewer does not.
	all Ok(p) in out: p.total_pages * size >= len(items) and (p.total_pages - 1) * size < len(items)
	# has_next is true exactly when the next page starts before the end.
	all Ok(p) in out: p.has_next <-> (page + 1) * size < len(items)
	e:
		in: ([{id: \"a\"}, {id: \"b\"}, {id: \"c\"}], 0, 2)
		out: Ok({items: [{id: \"a\"}, {id: \"b\"}], total_pages: 2, has_next: true})
	e:
		in: ([{id: \"a\"}], 2, 1)
		out: Err(page_range_err)
";

const PAGINATE_PY: &str = "def paginate(items, page, size):
    if page < 0 or size < 1 or page * size > len(items):
        return {\"err\": {\"code\": \"e-page-range\"}}
    n = len(items)
    total = (n + size - 1) // size
    return {\"ok\": {\"items\": items[page * size:(page + 1) * size], \"total_pages\": total, \"has_next\": (page + 1) * size < n}}
";

#[test]
fn a_record_output_has_each_scalar_edited_and_the_clauses_leave_look_here() {
    let (s, p) = scratch("paginate", PAGINATE, PAGINATE_PY);
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    let text = surety::render(&r);
    assert!(r.passed() && r.findings() == 0, "{text}");
    let m = r.mutants.as_ref().unwrap();
    let names: Vec<String> = m.mutants.iter().map(|x| x.mutant.name()).collect();
    for expected in [
        "total_pages = 1",
        "total_pages = -1",
        "total_pages = page",
        "total_pages = size + 1",
        "total_pages = size - 1",
        "total_pages = 0 - page",
        "total_pages = size * 100",
        "has_next = true",
    ] {
        let run = m
            .mutants
            .iter()
            .find(|x| x.mutant.name() == expected)
            .unwrap_or_else(|| panic!("{expected} is missing: {names:?}"));
        assert!(
            matches!(run.verdict, MutantVerdict::Caught),
            "{expected} slipped:\n{}",
            surety::render_mutants(m, false)
        );
    }
    let one = m
        .mutants
        .iter()
        .find(|x| x.mutant.name() == "total_pages = 1")
        .unwrap();
    assert_eq!(
        one.mutant.describe(),
        "returns the skeleton, or the empty value, with total_pages = 1"
    );
    // The arithmetic clauses catch something, so they are not in the
    // look-here section and no clause is uncaught.
    assert!(
        m.uncaught.is_empty(),
        "{}",
        surety::render_mutants(m, false)
    );
    for l in &r.look_here {
        if let LookKind::Uncaught(c) = &l.kind {
            panic!("uncaught: {c:?}\n{text}");
        }
    }
}

/// `true` and `false` are values: an example can state a `Bool` field or
/// output, and a line can compare with one. Problem 13's `has_next` needs
/// the first.
#[test]
fn a_bool_literal_is_a_value() {
    let text = "positive(x: Int) -> Bool\n\t# The output says whether x is positive.\n\tout <-> x > 0\n\tx == 0 -> out == false\n\te:\n\t\tin: 1\n\t\tout: true\n\te:\n\t\tin: 0\n\t\tout: false\n";
    let f = surety_syntax::parse(text).unwrap();
    let g = surety_syntax::Graph::single("positive", f.clone());
    g.check_file(0).unwrap();
    let c = surety_syntax::normalize::canonical_function(&g, 0, g.functions_in(0)[0]).unwrap();
    assert!(c.contains("(== out (bool false))"), "{c}");
    let rendered = surety_syntax::render::file(&f, false);
    assert!(
        rendered.contains("out: true") && rendered.contains("out == false"),
        "{rendered}"
    );
    let (s, p) = scratch("positive", text, "def positive(x):\n    return x > 0\n");
    let r = surety::check_file_with(&s, &p, Budget::default()).unwrap();
    assert!(r.passed() && r.findings() == 0, "{}", surety::render(&r));
    let e = surety_syntax::parse("f(x: Int) -> Int\n\tout == true\n").unwrap();
    let e = surety_syntax::Graph::single("f", e)
        .check_file(0)
        .unwrap_err();
    assert!(e.message.contains("cannot compare"), "{e}");
}
