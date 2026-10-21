//! `accept <boolean over the inputs>`: a deliberate non-decision (decision
//! note, 2026-10-21). An underdetermined input an `accept` covers is
//! reported ACCEPTED and passes `--strict`; one no `accept` covers stays
//! UNDERDETERMINED. A dead accept is a finding; `accept true` is refused;
//! the line is in the hash.

mod common;

use surety_check::{Budget, Checker, Report};
use surety_syntax::normalize::canonical_file;

const P5A_ACCEPT: &str = "\taccept any a in orders: any b in orders where b != a and b.customer_email == a.customer_email: b.ordered_at == a.ordered_at\n";

fn check(name: &str) -> Report {
    let graph = common::corpus();
    let idx = common::fixture_index(&graph, name);
    let perf = common::corpus_dir().join(format!("{name}.py"));
    let mut c = Checker::new(&graph, idx, &perf, &common::corpus_dir(), surety::TIMEOUT).unwrap();
    c.budget = Budget::default();
    c.check()
}

#[test]
fn p5a_ties_are_accepted_and_pass_strict() {
    let r = check("p5a");
    let text = surety::render(&r);
    let m = r.mutants.as_ref().unwrap();
    assert!(!m.search.is_empty(), "the ties are still found:\n{text}");
    assert_eq!(m.open().len(), 0, "every tie is covered:\n{text}");
    assert_eq!(m.accepted().len(), m.search.len());
    assert!(text.contains("  ACCEPTED  in:"), "{text}");
    assert!(!text.contains("  UNDERDETERMINED"), "{text}");
    assert!(text.contains(", open 0"), "{text}");
    assert_eq!(r.accepts.len(), 1);
    assert!(r.accepts[0].satisfied > 0, "{text}");
    assert!(
        text.contains(&format!(
            "accepts:          1; line {} fires on {} inputs",
            r.accepts[0].line, r.accepts[0].satisfied
        )),
        "{text}"
    );
    assert_eq!(r.findings(), 0, "{text}");
    assert!(r.strict_passed(), "{text}");
    let json = surety::render_json(&r);
    assert!(json.contains("\"kind\":\"accepted\""), "{json}");
    assert!(!json.contains("\"kind\":\"underdetermined\""), "{json}");
    assert!(json.contains("\"accepted\":"), "{json}");
    let mutants = surety::render_mutants(m, false);
    assert!(
        mutants.contains("  accepted  in:") && mutants.contains("underdetermined 0  accepted"),
        "{mutants}"
    );
}

#[test]
fn without_the_accept_the_ties_are_open() {
    let r = common::check_copy(
        "p5a",
        "\taccept any a in orders",
        "\t// accept any a in orders",
        &[],
    );
    let text = surety::render(&r);
    let m = r.mutants.as_ref().unwrap();
    assert!(!m.open().is_empty(), "{text}");
    assert!(m.accepted().is_empty());
    assert!(text.contains("  UNDERDETERMINED  in:"), "{text}");
    assert!(r.accepts.is_empty());
    assert!(r.passed() && !r.strict_passed(), "{text}");
}

#[test]
fn a_dead_accept_is_a_finding() {
    let r = common::check_copy(
        "p2",
        "\tout same_order_as orders\n",
        "\tout same_order_as orders\n\taccept any o in orders: o.amount < 0\n",
        &[],
    );
    let text = surety::render(&r);
    assert_eq!(r.accepts.len(), 1);
    assert!(r.accepts[0].dead(), "{text}");
    assert_eq!(r.dead_accepts(), 1);
    assert!(text.contains("a dead accept"), "{text}");
    assert!(
        r.passed() && !r.strict_passed(),
        "a dead accept fails strict:\n{text}"
    );
    let json = surety::render_json(&r);
    assert!(json.contains("\"kind\":\"dead_accept\""), "{json}");
    assert!(json.contains("\"dead\":true"), "{json}");
}

#[test]
fn accept_true_and_accept_over_out_are_refused() {
    let src = std::fs::read_to_string(common::corpus_dir().join("p2.sur")).unwrap();
    let marker = "\tout same_order_as orders\n";
    let refused = [
        ("\taccept true\n", "true"),
        ("\taccept 1 == 1\n", "must name an input"),
        ("\taccept len(out) == 0\n", "out"),
        ("\taccept orders\n", "Bool"),
    ];
    for (line, expect) in refused {
        let text = src.replace(marker, &format!("{marker}{line}"));
        let result = surety_syntax::parse(&text)
            .map_err(|e| e.to_string())
            .and_then(|f| {
                let g = surety_syntax::Graph::single("p2", f);
                g.check_file(0).map_err(|e| e.to_string())
            });
        let err = result.expect_err(&format!("{line:?} should be refused"));
        assert!(err.contains(expect), "{line:?}: {err}");
    }
    let fine = src.replace(marker, &format!("{marker}\taccept len(orders) == 0\n"));
    let f = surety_syntax::parse(&fine).unwrap();
    assert_eq!(f.decls.len(), 2);
    surety_syntax::Graph::single("p2", f).check_file(0).unwrap();
}

#[test]
fn the_accept_is_in_the_hash_and_its_position_is_not() {
    let read =
        |n: &str| std::fs::read_to_string(common::corpus_dir().join(format!("{n}.sur"))).unwrap();
    let src = read("p5a");
    assert!(src.contains(P5A_ACCEPT));
    let hash_of = |text: &str| {
        let f = surety_syntax::parse(text).unwrap();
        let g = surety_syntax::Graph::single("p5a", f);
        g.check_file(0).unwrap();
        canonical_file(&g, 0).unwrap()
    };
    let with = hash_of(&src);
    assert!(with.text.contains("(accepts "), "{}", with.text);
    let without = hash_of(&src.replace(P5A_ACCEPT, ""));
    assert!(!without.text.contains("(accepts"));
    assert_ne!(
        with.hash, without.hash,
        "the accept is part of the obligation"
    );
    // The same accept line right after the signature: same hash.
    let signature = "get_first_customer_orders(orders: Order[]) -> Order[]\n";
    let moved = src
        .replace(P5A_ACCEPT, "")
        .replace(signature, &format!("{signature}{P5A_ACCEPT}"));
    assert_eq!(
        hash_of(&moved).hash,
        with.hash,
        "position among the properties does not matter"
    );
    // Every other fixture hashes as before: no accepts section.
    let graph = common::corpus();
    for name in common::FIXTURES {
        if name == "p5a" {
            continue;
        }
        let idx = common::fixture_index(&graph, name);
        assert!(
            !canonical_file(&graph, idx)
                .unwrap()
                .text
                .contains("(accepts"),
            "{name}"
        );
    }
}
