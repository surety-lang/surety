//! Slice 6, acceptance test 3: the reach count. An example reaches a clause
//! when, on the example's own input and output, flipping some of the
//! clause's output-reading parts at some of their evaluations makes the
//! clause false; a part never evaluated on that pair cannot be flipped. Each
//! case below is a clause and the examples, by number, that reach it.

use std::fs;
use std::path::{Path, PathBuf};

use surety_check::{Against, Checker, Clause};
use surety_syntax::load::load_with_siblings;

mod common;

use common::corpus_dir;

fn scratch(dir: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("reach")
        .join(dir);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    dir
}

/// The examples against the lines for a file, with no performance.
fn against(path: &Path) -> Against {
    let (graph, idx) = load_with_siblings(path).unwrap_or_else(|e| panic!("{e:?}"));
    graph.check_file(idx).unwrap();
    let dir = path.parent().unwrap();
    let perf = path.with_extension("py");
    Checker::new(&graph, idx, &perf, dir, surety::TIMEOUT)
        .unwrap()
        .against()
}

fn against_src(dir: &str, src: &str) -> Against {
    let dir = scratch(dir, &[("t.sur", src)]);
    against(&dir.join("t.sur"))
}

/// The examples that reach the clause whose text is `text`.
fn reached(a: &Against, text: &str) -> Vec<usize> {
    let k = a
        .clauses
        .iter()
        .position(|c| matches!(&c.clause, Clause::Property { text: t, .. } if t == text))
        .unwrap_or_else(|| panic!("no clause {text:?}"));
    a.reached_by(k)
}

/// The frozen 13 with two examples: 1 is the has_next boundary (four items,
/// size 2, page 1: `(page + 1) * size` equals the count), 2 the size error.
fn thirteen(dir: &str, edit: impl Fn(String) -> String) -> Against {
    let src = fs::read_to_string(corpus_dir().join("drafts/p13.sur")).unwrap();
    let src = edit(src)
        + "\te:\n\t\tin: ([{id: \"a\"}, {id: \"b\"}, {id: \"c\"}, {id: \"d\"}], 1, 2)\n\t\tout: Ok({items: [{id: \"c\"}, {id: \"d\"}], total_pages: 2, has_next: false})\n"
        + "\te:\n\t\tin: ([{id: \"a\"}, {id: \"b\"}, {id: \"c\"}], 0, 0)\n\t\tout: Err([{code: \"e-page-size\", display: \"Page size must be at least 1\"}])\n";
    against_src(dir, &src)
}

const HAS_NEXT: &str = "(page + 1) * size < len(items) -> p.has_next == true";

#[test]
fn has_next_on_the_boundary_example_is_unreached_in_either_form() {
    // The antecedent is about the inputs and false at the boundary, so no
    // flip of `p.has_next == true` can make the line false there.
    let a = thirteen("has-next", |s| s);
    assert_eq!(
        reached(&a, &format!("exactly_one Ok(p) in out: {HAS_NEXT}")),
        Vec::<usize>::new()
    );
    let a = thirteen("has-next-all", |s| {
        s.replace(
            "exactly_one Ok(p) in out: (page + 1)",
            "all Ok(p) in out: (page + 1)",
        )
    });
    assert_eq!(
        reached(&a, &format!("all Ok(p) in out: {HAS_NEXT}")),
        Vec::<usize>::new()
    );
}

#[test]
fn the_checks_line_is_reached_by_every_example() {
    // A bodiless quantifier over `out` is one flippable leaf.
    let a = thirteen("checks", |s| s);
    assert_eq!(
        reached(&a, "checks <-> (exactly_one Ok(_) in out)"),
        vec![1, 2]
    );
}

#[test]
fn an_error_iff_is_reached_by_the_error_examples_and_not_the_ok_ones() {
    let line = "size < 1 <-> (exactly_one Err(errs) in out: size_error in errs)";
    let a = thirteen("error-iff", |s| {
        s.replace(
            "size < 1 <-> (exactly_one Err(err) in out: err == size_error)",
            line,
        )
    });
    assert_eq!(reached(&a, line), vec![2]);
}

#[test]
fn an_error_side_implication_on_an_ok_example_is_unreached() {
    // The shape of out2's line, on an Ok example (1), the size error (2),
    // and another error (3), where the antecedent is false.
    let src = "type PageError = { code: Text, display: Text }\n\nsize_error: PageError = {code: \"e-page-size\", display: \"Page size must be at least 1\"}\n\npage_of(size: Int) -> Result<Int, PageError>\n\tall Err(err) in out: size < 1 -> err == size_error\n\te:\n\t\tin: 3\n\t\tout: Ok(3)\n\te:\n\t\tin: 0\n\t\tout: Err({code: \"e-page-size\", display: \"Page size must be at least 1\"})\n\te:\n\t\tin: 2\n\t\tout: Err({code: \"e-other\", display: \"another error\"})\n";
    let a = against_src("error-side", src);
    assert_eq!(
        reached(&a, "all Err(err) in out: size < 1 -> err == size_error"),
        vec![2]
    );
}

#[test]
fn a_disjunction_and_its_implication_get_the_same_answer() {
    // Page −1 (1): the input decides both, so neither is reached; page 2
    // (2): the Ok side's body is evaluated, and both are.
    let src = "page_at(page: Int) -> Result<Int, Text>\n\tpage < 0 or (all Ok(p) in out: p >= 0)\n\tpage >= 0 -> (all Ok(p) in out: p >= 0)\n\te:\n\t\tin: -1\n\t\tout: Err(\"negative\")\n\te:\n\t\tin: 2\n\t\tout: Ok(2)\n";
    let a = against_src("or-implies", src);
    assert_eq!(
        reached(&a, "page < 0 or (all Ok(p) in out: p >= 0)"),
        vec![2]
    );
    assert_eq!(
        reached(&a, "page >= 0 -> (all Ok(p) in out: p >= 0)"),
        vec![2]
    );
}

#[test]
fn two_true_disjuncts_on_the_output_are_reached() {
    // No single flip makes the line false; flipping both does.
    let src = "pair(x: Int) -> {a: Int, b: Int}\n\tout.a > 0 or out.b > 0\n\te:\n\t\tin: 1\n\t\tout: {a: 1, b: 1}\n";
    let a = against_src("two-disjuncts", src);
    assert_eq!(reached(&a, "out.a > 0 or out.b > 0"), vec![1]);
}

#[test]
fn a_tautology_is_reached() {
    let src = "same(xs: Int[]) -> Int[]\n\tlen(out) >= 0\n\te:\n\t\tin: [1]\n\t\tout: [1]\n";
    let a = against_src("tautology", src);
    assert_eq!(reached(&a, "len(out) >= 0"), vec![1]);
}

#[test]
fn apply_diffs_where_line_is_reached_only_where_a_changed_sku_meets_the_output() {
    // `exactly_one i in res where i.sku == c.sku: i.qty == c.new_qty`, as
    // `(exactly_one i in res: w) and (all i in res: w -> b)`. Example 1 is
    // the one Ok example whose `changed` has an entry, B, sharing its sku
    // with an output item; 2 and 3 have no change, 4 to 8 are errors.
    let dir = scratch(
        "apply-diff",
        &[
            (
                "apply_diff.sur",
                &fs::read_to_string(corpus_dir().join("apply_diff.sur")).unwrap(),
            ),
            (
                "p8.sur",
                &fs::read_to_string(corpus_dir().join("p8.sur")).unwrap(),
            ),
        ],
    );
    let a = against(&dir.join("apply_diff.sur"));
    let k = a
        .clauses
        .iter()
        .position(|c| matches!(&c.clause, Clause::Property { text, .. } if text.contains("exactly_one i in res where i.sku == c.sku")))
        .unwrap();
    assert_eq!(a.reached_by(k), vec![1]);
}

#[test]
fn p7s_else_line_is_reached_by_an_order_whose_user_does_not_match() {
    // `all o in out: all u in users where u.id == o.user_id: … else
    // o.user_name == None`: the `else` is one evaluation in the outer scope,
    // for an order no user matches.
    let src = "type User = { id: Id, user_name: Text }\ntype Order = { id: Id, user_id: Option<Id> }\ntype EnrichedOrder = { ...Order, user_name: Option<Text> }\n\nenrich(orders: Order[unique id], users: User[unique id]) -> EnrichedOrder[]\n\tall o in out: all u in users where u.id == o.user_id: o.user_name == u.user_name else o.user_name == None\n\te:\n\t\tin: ([{id: \"o1\", user_id: \"u9\"}], [{id: \"u1\", user_name: \"Ann\"}])\n\t\tout: [{id: \"o1\", user_id: \"u9\", user_name: None}]\n\te:\n\t\tin: ([], [{id: \"u1\", user_name: \"Ann\"}])\n\t\tout: []\n";
    let a = against_src("else", src);
    assert_eq!(
        reached(
            &a,
            "all o in out: all u in users where u.id == o.user_id: o.user_name == u.user_name else o.user_name == None"
        ),
        vec![1]
    );
}

#[test]
fn the_reach_counts_are_in_the_json_of_both_commands() {
    let src = "same(xs: Int[]) -> Int[]\n\tlen(out) >= 0\n\tall x in out: x > 0\n\te:\n\t\tin: [1]\n\t\tout: [1]\n\te:\n\t\tin: []\n\t\tout: []\n";
    let dir = scratch("json", &[("t.sur", src)]);
    let budget = surety_check::Budget {
        generated: 10,
        search_inputs: 1,
        search_candidates: 10,
        mutants: true,
    };
    let path = dir.join("t.sur");
    let m = surety::mutants_file(&path, None, budget.clone()).unwrap();
    let r = surety::check_file_with(&path, &path.with_extension("py"), budget).unwrap();
    let reach =
        |lines: Vec<serde_json::Value>| lines.into_iter().find(|j| j["kind"] == "reach").unwrap();
    let want = serde_json::json!([
        { "line": 2, "text": "len(out) >= 0", "reached_by": [1, 2] },
        { "line": 3, "text": "all x in out: x > 0", "reached_by": [1] },
    ]);
    assert_eq!(reach(surety::mutants_json_lines(&m))["clauses"], want);
    assert_eq!(reach(surety::json_lines(&r))["clauses"], want);
    assert_eq!(reach(surety::json_lines(&r))["examples"], 2);
}

#[test]
fn an_abbreviation_is_opened_so_a_quantifier_over_out_inside_it_can_be_vacuous() {
    // A local abbreviation and a top-level one called with `out`, each a
    // quantifier over the Ok side: an error example (1) leaves it vacuous,
    // an Ok one (2) reaches it, as if it were written in the line.
    let src = "nonneg(r: Result<Int, Text>) = all Ok(p) in r: p >= 0\n\npage_at(page: Int) -> Result<Int, Text>\n\tpositive = all Ok(p) in out: p >= 0\n\tpositive\n\tnonneg(out)\n\te:\n\t\tin: -1\n\t\tout: Err(\"negative\")\n\te:\n\t\tin: 2\n\t\tout: Ok(2)\n";
    let a = against_src("abbreviation", src);
    assert_eq!(reached(&a, "positive"), vec![2]);
    assert_eq!(reached(&a, "nonneg(out)"), vec![2]);
}
