//! Slice 6, acceptance test 8: hints and errors that carry their edit. The
//! standalone-side hint, `pattern_as_value` on `out == Err(x)` and
//! `quantifier_mid_line` on a top-level `and`, `or` or `<->`, from one
//! function (`mend::diagnose`) behind the editor and `surety parse --json`.
//! Every offered edit is applied here and the file parsed again: each new
//! line's tree prints back as the line the edit wrote, the meaning it was
//! built from. The draft's round is in `draft.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value as Json;
use surety_syntax::mend::Diagnosis;
use surety_syntax::{Edit, ErrorKind};

mod common;

/// What the editor gets for this text: parse's error, through `diagnose`.
fn diagnosed(text: &str) -> Diagnosis {
    surety_lsp::diagnostics_for(Path::new("/nonexistent/scratch.sur"), text)
}

/// The text with the edit applied: its lines replaced whole.
fn applied(text: &str, e: &Edit) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<&str> = lines[..e.from - 1].to_vec();
    out.extend(e.text.lines());
    out.extend(&lines[e.to..]);
    out.join("\n") + "\n"
}

/// Apply the edit, parse the whole file again, and require each line the
/// edit wrote to be a property whose tree prints back as that line: the
/// reparse reached the intended canonical form.
fn reparses(text: &str, e: &Edit, want: &[&str]) {
    let got: Vec<&str> = e.text.lines().map(|l| l.trim_start()).collect();
    assert_eq!(got, want, "{}", e.text);
    let new = applied(text, e);
    let file = surety_syntax::parse(&new).unwrap_or_else(|err| panic!("{err}\n{new}"));
    let f = file
        .decls
        .iter()
        .find_map(|d| match d {
            surety_syntax::Decl::Function(f) => Some(f),
            _ => None,
        })
        .unwrap();
    for (i, line) in want.iter().enumerate() {
        let at = e.from + i;
        let p = f
            .properties
            .iter()
            .find(|p| p.line == at)
            .unwrap_or_else(|| panic!("no property at line {at}:\n{new}"));
        assert_eq!(surety_syntax::render::expr(&p.expr), *line, "{new}");
    }
}

const PAGINATE: &str = "type Item = { id: Id }\ntype Page = { items: Item[], total_pages: Int }\ntype PageError = { code: Text }\n\nsize_error: PageError = {code: \"e-page-size\"}\nerrs: PageError[] = [{code: \"e-page-size\"}, {code: \"e-page-negative\"}]\n\npaginate(items: Item[], page: Int, size: Int) -> Result<Page, PageError>\n\tchecks = size >= 1 and page >= 0\n";

fn paginate(line: &str) -> String {
    format!("{PAGINATE}\t{line}\n")
}

#[test]
fn the_standalone_side_hint_and_its_edit_to_all() {
    for (line, want) in [
        (
            "exactly_one Ok(p) in out: p.total_pages >= 1",
            "all Ok(p) in out: p.total_pages >= 1",
        ),
        (
            "any Err(e) in out: e in errs",
            "all Err(e) in out: e in errs",
        ),
    ] {
        let text = paginate(line);
        let d = diagnosed(&text);
        assert!(d.errors.is_empty(), "{line}: {:?}", d.errors);
        let [h] = &d.hints[..] else {
            panic!("{line}: {:?}", d.hints)
        };
        assert_eq!((h.code, h.line), ("standalone_side", 10), "{line}");
        let side = if line.contains("Ok(") { "Ok" } else { "Err" };
        assert!(
            h.message
                .contains(&format!("claims the `{side}` side on every input"))
                && h.message.contains("alone is `all`"),
            "{}",
            h.message
        );
        reparses(&text, h.edit.as_ref().unwrap(), &[want]);
    }
    // None inside an iff, none for `all`, none with no body, none over a
    // list: the standalone form is a claim of its own there.
    for line in [
        "checks <-> exactly_one Ok(_) in out",
        "all Ok(p) in out: p.total_pages >= 1",
        "size < 1 <-> (exactly_one Err(e) in out: e == size_error)",
        "exactly_one Ok(_) in out",
        "exactly_one i in items where i.id == i.id: i.id == i.id",
    ] {
        let d = diagnosed(&paginate(line));
        assert!(d.errors.is_empty() && d.hints.is_empty(), "{line}: {d:?}");
    }
    // The frozen 13: its four `exactly_one Ok(p)` lines and its `any Err`
    // line, each with the edit; its iff lines none.
    let frozen = fs::read_to_string(common::corpus_dir().join("drafts/p13.sur")).unwrap();
    let d = diagnosed(&frozen);
    let lines: Vec<usize> = d.hints.iter().map(|h| h.line).collect();
    assert_eq!(lines, [30, 33, 35, 37, 39]);
    for h in &d.hints {
        let e = h.edit.as_ref().unwrap();
        let was = frozen.lines().nth(h.line - 1).unwrap().trim_start();
        let now = was
            .replacen("exactly_one ", "all ", 1)
            .replacen("any ", "all ", 1);
        reparses(&frozen, e, &[now.as_str()]);
    }
    // The eleven, `apply_diff` among them, and the corpus 13: none.
    for name in common::FIXTURES {
        let text = fs::read_to_string(common::corpus_dir().join(format!("{name}.sur"))).unwrap();
        assert!(diagnosed(&text).hints.is_empty(), "{name}");
    }
    let p13 = fs::read_to_string(common::corpus_dir().join("p13/p13.sur")).unwrap();
    assert!(diagnosed(&p13).hints.is_empty());
}

const CHECK: &str = "type Item = { amount: Int }\ntype ListError = { code: Text }\n\nbad: ListError = {code: \"e-bad\"}\n\n";

fn check(inputs: &str, line: &str) -> String {
    format!("{CHECK}check({inputs}) -> Result<Item[], ListError>\n\t{line}\n")
}

/// The one error on the line, with its edit when one was verified.
fn error(text: &str, kind: ErrorKind) -> surety_syntax::Error {
    let d = diagnosed(text);
    let [e] = &d.errors[..] else {
        panic!("{text}{:?}", d.errors)
    };
    assert_eq!(e.kind, kind, "{e}");
    e.clone()
}

#[test]
fn out_equals_a_pattern_is_rewritten_in_place_or_swapped_or_named() {
    let ins = "items: Item[], limit: Int";
    let q = "(exactly_one Err(e) in out: e == bad)";
    for (line, want) in [
        // Standing alone, and as the right operand of an iff: in place.
        (
            "out == Err(bad)",
            "exactly_one Err(e) in out: e == bad".to_string(),
        ),
        (
            "limit < 0 <-> out == Err(bad)",
            format!("limit < 0 <-> {q}"),
        ),
        // As the left operand of an iff: the sides swapped.
        (
            "out == Err(bad) <-> limit < 0",
            format!("limit < 0 <-> {q}"),
        ),
        // The `Ok` mirror, with `p`.
        (
            "out == Ok(items)",
            "exactly_one Ok(p) in out: p == items".to_string(),
        ),
    ] {
        let text = check(ins, line);
        let e = error(&text, ErrorKind::PatternAsValue);
        assert!(
            e.message.contains("compares the output with a pattern")
                && e.message.contains("the claim is written `exactly_one "),
            "{}",
            e.message
        );
        reparses(&text, e.edit.as_ref().unwrap(), &[want.as_str()]);
    }
    // With a quantifier elsewhere on the line: neither form parses to the
    // meaning, so no edit, and the message shows the abbreviation form with
    // no name invented.
    let text = check(ins, "out == Err(bad) <-> any i in items: i.amount < 0");
    let e = error(&text, ErrorKind::PatternAsValue);
    assert!(e.edit.is_none(), "{e:?}");
    assert!(
        e.message.contains(&format!(
            "name the other side, `name = any i in items: i.amount < 0`, and write `name <-> {q}`"
        )),
        "{}",
        e.message
    );
    // The binder is fresh: `e` an input's name, `err`; both, `e2`.
    for (ins, binder) in [
        ("items: Item[], e: Int", "err"),
        ("items: Item[], e: Int, err: Int", "e2"),
    ] {
        let text = check(ins, "e < 0 <-> out == Err(bad)");
        let edit = error(&text, ErrorKind::PatternAsValue).edit.unwrap();
        let want = format!("e < 0 <-> (exactly_one Err({binder}) in out: {binder} == bad)");
        reparses(&text, &edit, &[want.as_str()]);
    }
    // Not `out == Err(x)`: the error as it was, no edit.
    let bad =
        fs::read_to_string(common::corpus_dir().join("bad/pattern_as_value_not.sur")).unwrap();
    let e = error(&bad, ErrorKind::PatternAsValue);
    assert!(e.edit.is_none() && e.message.contains("is a pattern; a value is expected here"));
}

#[test]
fn a_quantifier_mid_line_is_split_or_rewritten_with_the_quantifier_last() {
    let ins = "items: Item[], limit: Int";
    let q = "all Ok(os) in out: len(os) <= len(items)";
    for (line, want) in [
        // A top-level `and`: one line per claim.
        (
            format!("({q}) and limit >= 0"),
            vec![q.to_string(), "limit >= 0".to_string()],
        ),
        (
            "limit >= 0 and exactly_one Ok(_) in out and len(items) > 0".to_string(),
            vec![
                "limit >= 0".to_string(),
                "exactly_one Ok(_) in out".to_string(),
                "len(items) > 0".to_string(),
            ],
        ),
        // A top-level `or`: `Q or A` and `(Q) or X` as `not A -> Q`.
        (
            format!("({q}) or limit < 0"),
            vec![format!("not limit < 0 -> ({q})")],
        ),
        (
            "exactly_one Ok(_) in out or limit < 0".to_string(),
            vec!["not limit < 0 -> (exactly_one Ok(_) in out)".to_string()],
        ),
        // A chain whose quantifier is not last: `A or (Q) or B`.
        (
            format!("limit < 0 or ({q}) or len(items) == 0"),
            vec![format!("not len(items) == 0 -> limit < 0 or ({q})")],
        ),
        // A top-level iff with the quantifier first: the sides swapped.
        (
            "exactly_one Ok(_) in out <-> limit >= 0".to_string(),
            vec!["limit >= 0 <-> (exactly_one Ok(_) in out)".to_string()],
        ),
    ] {
        let text = check(ins, &line);
        let e = error(&text, ErrorKind::QuantifierMidLine);
        assert!(
            e.message
                .starts_with("a quantifier must be the last operand on its line: "),
            "{}",
            e.message
        );
        let want: Vec<&str> = want.iter().map(String::as_str).collect();
        reparses(&text, e.edit.as_ref().unwrap(), &want);
    }
    // `A or Q` and `A and Q` with the quantifier last parse as they stand:
    // no error, so nothing to mend (reading 144).
    for line in [format!("limit < 0 or {q}"), format!("limit >= 0 and {q}")] {
        let d = diagnosed(&check(ins, &line));
        assert!(d.errors.is_empty(), "{line}: {:?}", d.errors);
    }
    // The negative fixture: 13's own form, `checks <-> (exactly_one Ok(_) in out)`.
    let bad = fs::read_to_string(common::corpus_dir().join("bad/quantifier_mid_line.sur")).unwrap();
    let e = error(&bad, ErrorKind::QuantifierMidLine);
    reparses(
        &bad,
        e.edit.as_ref().unwrap(),
        &["checks <-> (exactly_one Ok(_) in out)"],
    );
}

/// A census draft, copied verbatim with one comment line added last naming
/// the library, version and license its docstring came from, so the lines
/// the brief cites keep their numbers.
fn census(name: &str) -> String {
    fs::read_to_string(
        common::corpus_dir()
            .join("census")
            .join(format!("more_itertools__more__{name}.sur")),
    )
    .unwrap()
}

#[test]
fn the_census_cases() {
    // `more__zip_broadcast` line 16: `out == Err(length_error)` on the left
    // of an iff whose right side holds two quantifiers. Swapped, the iff
    // falls inside the second quantifier's body: no edit, the form named.
    let zip = census("zip_broadcast");
    let e = error(&zip, ErrorKind::PatternAsValue);
    assert_eq!(e.line, 16);
    assert!(e.edit.is_none(), "{e:?}");
    assert!(
        e.message.contains("`name = strict and (any a in objects where not a.scalar: any b in objects where not b.scalar: len(a.items) != len(b.items))`, and write `name <-> (exactly_one Err(e) in out: e == length_error)`"),
        "{}",
        e.message
    );
    // `more__intersperse` line 11: `(n >= 1 <-> exactly_one Ok(_) in out)
    // and all Err(x) in out: …`, split into its two claims.
    let inter = census("intersperse");
    let e = error(&inter, ErrorKind::QuantifierMidLine);
    assert_eq!(e.line, 11);
    reparses(
        &inter,
        e.edit.as_ref().unwrap(),
        &[
            "n >= 1 <-> (exactly_one Ok(_) in out)",
            "all Err(x) in out: x == n_error",
        ],
    );
    // Its signature names an input `e`: a rewrite there takes `err`.
    let text = inter.replace(
        "(n >= 1 <-> exactly_one Ok(_) in out) and all Err(x) in out: x == n_error",
        "n < 1 <-> out == Err(n_error)",
    );
    let edit = error(&text, ErrorKind::PatternAsValue).edit.unwrap();
    reparses(
        &text,
        &edit,
        &["n < 1 <-> (exactly_one Err(err) in out: err == n_error)"],
    );
}

fn scratch(name: &str, text: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("edits")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("scratch.sur");
    fs::write(&path, text).unwrap();
    path
}

/// `surety parse --json`: its lines, and its exit code.
fn parse_json(path: &Path) -> (Vec<Json>, Option<i32>) {
    let out = Command::new(env!("CARGO_BIN_EXE_surety"))
        .args(["parse", "--json"])
        .arg(path)
        .output()
        .unwrap();
    let lines = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    (lines, out.status.code())
}

#[test]
fn parse_json_reports_all_three_and_exits_as_parse_does() {
    let ins = "items: Item[], limit: Int";
    for (name, text, kind, code, exit, line) in [
        (
            "hint",
            paginate("exactly_one Ok(p) in out: p.total_pages >= 1"),
            "hint",
            "standalone_side",
            0,
            10,
        ),
        (
            "pattern",
            check(ins, "out == Err(bad)"),
            "error",
            "pattern_as_value",
            1,
            7,
        ),
        (
            "mid-line",
            check(ins, "exactly_one Ok(_) in out <-> limit >= 0"),
            "error",
            "quantifier_mid_line",
            1,
            7,
        ),
    ] {
        let path = scratch(name, &text);
        let (lines, status) = parse_json(&path);
        assert_eq!(status, Some(exit), "{name}: {lines:?}");
        let found = lines
            .iter()
            .find(|j| j["code"] == code)
            .unwrap_or_else(|| panic!("{name}: {lines:?}"));
        assert_eq!(found["kind"], kind, "{name}");
        let edit = &found["edit"];
        assert!(
            edit["title"].is_string() && edit["text"].is_string(),
            "{name}: {found}"
        );
        assert_eq!(
            (edit["from"].as_u64(), edit["to"].as_u64()),
            (Some(line), Some(line)),
            "{name}"
        );
        // The text form is unchanged: a hint is not printed, nor an edit.
        let text_form = Command::new(env!("CARGO_BIN_EXE_surety"))
            .arg("parse")
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(text_form.status.code(), Some(exit), "{name}");
        assert!(
            !String::from_utf8_lossy(&text_form.stdout).contains("\"edit\""),
            "{name}"
        );
    }
    // A file that parses gives one `ok` line per obligation.
    let (lines, status) = parse_json(&common::corpus_dir().join("p2.sur"));
    assert_eq!(status, Some(0));
    assert!(
        lines
            .iter()
            .any(|j| j["kind"] == "ok" && j["obligation"] == "get_small_orders"),
        "{lines:?}"
    );
}
