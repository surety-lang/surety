#![allow(dead_code)]

use std::path::{Path, PathBuf};

use surety_syntax::Graph;

pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus")
}

pub const FIXTURES: [&str; 11] = [
    "p1",
    "p2",
    "p3",
    "p4",
    "p5a",
    "p5b",
    "p6",
    "p7",
    "p8",
    "p14",
    "apply_diff",
];

/// The whole corpus directory as one graph.
pub fn corpus() -> Graph {
    match surety::load_dir(&corpus_dir()) {
        Ok(g) => g,
        Err(errs) => {
            for (p, e) in &errs {
                eprintln!("{}: {e}", p.display());
            }
            panic!("corpus does not parse");
        }
    }
}

pub fn fixture_index(graph: &Graph, name: &str) -> usize {
    graph
        .index_of(name)
        .unwrap_or_else(|| panic!("no fixture {name}"))
}

/// A copy of a fixture with one text edit, as a graph of its own (plus the
/// siblings it needs for types), checked against the fixture's own
/// performance under the default budget. The fixture is never edited.
pub fn check_copy(name: &str, from: &str, to: &str, siblings: &[&str]) -> surety_check::Report {
    let read = |n: &str| std::fs::read_to_string(corpus_dir().join(format!("{n}.sur"))).unwrap();
    let src = read(name);
    assert!(src.contains(from), "{name}: {from:?} is not in the fixture");
    let file = surety_syntax::parse(&src.replace(from, to)).unwrap();
    let mut files = vec![(name.to_string(), file)];
    for s in siblings {
        files.push((s.to_string(), surety_syntax::parse(&read(s)).unwrap()));
    }
    let graph = Graph::new(files);
    let perf = corpus_dir().join(format!("{name}.py"));
    let mut c =
        surety_check::Checker::new(&graph, 0, &perf, &corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = surety_check::Budget::default();
    c.check()
}

/// Problem 11 (invoice total), the metamorphic form: a property that calls
/// its own obligation (`metamorphic.rs`, and the self-call case of piece 1).
pub const METAMORPHIC_SUR: &str = r#"type LineItem = { amount_cents: Int }

invoice_total(items: LineItem[], tax_rate_bps: Int) -> Int
	note "sum the line items in cents, apply the tax rate in basis points once, round half-even to the cent"
	# The total of the items is the total of one item carrying their sum: tax is applied to the subtotal once, never per line.
	out == invoice_total([{amount_cents: sum(items.amount_cents)}], tax_rate_bps)
	# At a zero rate the total is the sum of the amounts.
	tax_rate_bps == 0 -> out == sum(items.amount_cents)
	# At a rate of at least zero and a sum of at least zero, the total is at least the sum.
	(tax_rate_bps >= 0 and sum(items.amount_cents) >= 0) -> out >= sum(items.amount_cents)
	e:
		in: ([], 1000)
		out: 0
	e:
		in: ([{amount_cents: 250}, {amount_cents: 750}], 0)
		out: 1000
	e:
		in: ([{amount_cents: 125}], 1000)
		out: 138
	e:
		in: ([{amount_cents: 135}], 1000)
		out: 148
"#;

pub const METAMORPHIC_PY: &str = r#"def invoice_total(items, tax_rate_bps):
    subtotal = sum(item["amount_cents"] for item in items)
    scaled = subtotal * (10000 + tax_rate_bps)
    q, r = divmod(scaled, 10000)
    if r * 2 > 10000 or (r * 2 == 10000 and q % 2 == 1):
        q += 1
    return q
"#;
