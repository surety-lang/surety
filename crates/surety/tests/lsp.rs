//! Slice 4, acceptance test 1: the language server's diagnostics are
//! `parse`'s. Driven over stdio, the way an editor drives it: for every
//! file under `corpus/bad/`, one diagnostic with the `.err` file's kind and
//! line; for the eleven with their siblings, none.

mod common;

use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value as Json, json};

struct Lsp {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Lsp {
    fn start() -> Lsp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_surety"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("surety lsp starts");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut lsp = Lsp {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let reply = lsp.request("initialize", json!({ "capabilities": {} }));
        assert_eq!(reply["result"]["capabilities"]["textDocumentSync"], 1);
        lsp.notify("initialized", json!({}));
        lsp
    }

    fn send(&mut self, msg: &Json) {
        surety_lsp::write_message(&mut self.stdin, msg).unwrap();
    }

    fn read(&mut self) -> Json {
        surety_lsp::read_message(&mut self.stdout)
            .unwrap()
            .expect("the server answers")
    }

    fn request(&mut self, method: &str, params: Json) -> Json {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let msg = self.read();
            if msg["id"] == id {
                return msg;
            }
        }
    }

    fn notify(&mut self, method: &str, params: Json) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    /// Open a file as the editor would and return the diagnostics published for it.
    fn open(&mut self, path: &Path) -> Vec<Json> {
        let uri = surety_lsp::path_to_uri(&fs::canonicalize(path).unwrap());
        let text = fs::read_to_string(path).unwrap();
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "surety", "version": 1, "text": text } }),
        );
        loop {
            let msg = self.read();
            if msg["method"] == "textDocument/publishDiagnostics" && msg["params"]["uri"] == uri {
                return msg["params"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
            }
        }
    }

    fn change(&mut self, path: &Path, text: &str) -> Vec<Json> {
        let uri = surety_lsp::path_to_uri(&fs::canonicalize(path).unwrap());
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": text }] }),
        );
        loop {
            let msg = self.read();
            if msg["method"] == "textDocument/publishDiagnostics" && msg["params"]["uri"] == uri {
                return msg["params"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
            }
        }
    }

    fn stop(mut self) {
        let reply = self.request("shutdown", Json::Null);
        assert!(reply["result"].is_null());
        self.notify("exit", Json::Null);
        let status = self.child.wait().unwrap();
        assert!(status.success(), "the server exits cleanly on `exit`");
    }
}

#[test]
fn every_bad_fixture_gets_parses_diagnostic_and_the_corpus_gets_none() {
    let mut lsp = Lsp::start();
    let bad = common::corpus_dir().join("bad");
    let mut checked = 0;
    for entry in fs::read_dir(&bad).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "sur") {
            continue;
        }
        let expected = fs::read_to_string(path.with_extension("err")).unwrap();
        let mut parts = expected.split_whitespace();
        let kind = parts.next().unwrap();
        let line: u64 = parts.next().unwrap().parse().unwrap();
        let diagnostics = lsp.open(&path);
        assert_eq!(
            diagnostics.len(),
            1,
            "{}: one diagnostic, as parse prints one error",
            path.display()
        );
        let d = &diagnostics[0];
        assert_eq!(d["code"], kind, "{}: the kind", path.display());
        assert_eq!(
            d["range"]["start"]["line"].as_u64().unwrap() + 1,
            line,
            "{}: the line",
            path.display()
        );
        // The range starts after the indentation, where the line's text does.
        let text = fs::read_to_string(&path).unwrap();
        let at = text.lines().nth(line as usize - 1).unwrap_or_default();
        let indent = at.chars().take_while(|c| c.is_whitespace()).count();
        if indent < at.chars().count() {
            assert_eq!(
                d["range"]["start"]["character"].as_u64().unwrap() as usize,
                indent,
                "{}: the first character",
                path.display()
            );
        }
        assert_eq!(d["severity"], 1);
        assert_eq!(d["source"], "surety");
        assert!(d["message"].as_str().is_some_and(|m| !m.is_empty()));
        checked += 1;
    }
    assert_eq!(checked, 18, "the eighteen negative fixtures");
    let mut hints = 0;
    for name in common::FIXTURES {
        let path = common::corpus_dir().join(format!("{name}.sur"));
        let diagnostics = lsp.open(&path);
        let errors: Vec<&Json> = diagnostics.iter().filter(|d| d["severity"] != 4).collect();
        assert!(errors.is_empty(), "{name}: {errors:?}");
        // A sentence or comment past the column is a hint, never an error.
        for d in diagnostics.iter().filter(|d| d["severity"] == 4) {
            assert_eq!(d["code"], "long_line", "{name}: {d}");
            hints += 1;
        }
    }
    assert!(
        hints >= 5,
        "the corpus has prose lines past the column: {hints}"
    );
    lsp.stop();
}

#[test]
fn formatting_returns_the_rendering_and_the_hints_go_away() {
    let mut lsp = Lsp::start();
    let path = common::corpus_dir().join("p7.sur");
    let text = fs::read_to_string(&path).unwrap();
    let before = lsp.open(&path);
    assert!(
        before.iter().any(|d| d["severity"] == 4),
        "p7 has a prose line past the column"
    );
    let uri = surety_lsp::path_to_uri(&fs::canonicalize(&path).unwrap());
    let reply = lsp.request(
        "textDocument/formatting",
        json!({ "textDocument": { "uri": uri }, "options": { "tabSize": 4, "insertSpaces": false } }),
    );
    let edits = reply["result"].as_array().cloned().expect("one edit");
    assert_eq!(edits.len(), 1, "{reply}");
    assert_eq!(
        edits[0]["range"]["start"],
        json!({ "line": 0, "character": 0 })
    );
    let rendered = edits[0]["newText"].as_str().unwrap().to_string();
    let parsed = surety_syntax::parse(&text).unwrap();
    assert_eq!(rendered, surety_syntax::render::file(&parsed, false));
    let after = lsp.change(&path, &rendered);
    assert!(
        after.is_empty(),
        "formatted, p7 has no diagnostic at all: {after:?}"
    );
    // Formatted text formats to itself: no edit.
    let again = lsp.request(
        "textDocument/formatting",
        json!({ "textDocument": { "uri": uri }, "options": { "tabSize": 4, "insertSpaces": false } }),
    );
    assert_eq!(again["result"], json!([]));
    lsp.stop();
}

#[test]
fn a_change_reparses_against_the_siblings_on_disk() {
    let mut lsp = Lsp::start();
    let path = common::corpus_dir().join("p14.sur");
    assert!(lsp.open(&path).is_empty());
    // Unsaved text: a reference to a type the siblings do not define.
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace("old: Item[unique sku]", "old: Itm[unique sku]");
    let diagnostics = lsp.change(&path, &text);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["code"], "unknown_type");
    assert_eq!(
        diagnostics[0]["range"]["start"]["line"], 0,
        "the signature is line 1"
    );
    // Back to the saved text: clean again, and the siblings still resolve `Item`.
    let diagnostics = lsp.change(&path, &fs::read_to_string(&path).unwrap());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    lsp.stop();
}

/// The 0-based line and character of the first occurrence of `needle`.
fn position_of(text: &str, needle: &str) -> (usize, usize) {
    for (n, line) in text.lines().enumerate() {
        if let Some(c) = line.find(needle) {
            return (n, line[..c].chars().count());
        }
    }
    panic!("{needle:?} is not in the text");
}

impl Lsp {
    fn start_with_graph() -> Lsp {
        let graph = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../graph");
        let mut child = Command::new(env!("CARGO_BIN_EXE_surety"))
            .arg("lsp")
            .arg("--graph")
            .arg(&graph)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("surety lsp starts");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut lsp = Lsp {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        lsp.request("initialize", json!({ "capabilities": {} }));
        lsp.notify("initialized", json!({}));
        lsp
    }

    fn at(&mut self, method: &str, path: &Path, needle: &str) -> Json {
        let uri = surety_lsp::path_to_uri(&fs::canonicalize(path).unwrap());
        let text = fs::read_to_string(path).unwrap();
        let (line, character) = position_of(&text, needle);
        let reply = self.request(
            method,
            json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character } }),
        );
        reply["result"].clone()
    }
}

#[test]
fn names_resolve_and_hover_says_what_a_thing_is() {
    let mut lsp = Lsp::start_with_graph();
    let p5b = common::corpus_dir().join("p5b.sur");
    let p14 = common::corpus_dir().join("p14.sur");
    let p3 = common::corpus_dir().join("p3.sur");
    for p in [&p5b, &p14, &p3] {
        assert!(lsp.open(p).iter().all(|d| d["severity"] == 4), "no error");
    }

    // Go to definition on `Order` in p5b's signature lands on the type.
    let def = lsp.at("textDocument/definition", &p5b, "Order[]");
    let uri = surety_lsp::path_to_uri(&fs::canonicalize(&p5b).unwrap());
    assert_eq!(def["uri"], uri);
    assert_eq!(def["range"]["start"]["line"], 0, "type Order is line 1");

    // On `apply_diff` in p14's property, across the directory, on apply_diff's signature.
    let def = lsp.at("textDocument/definition", &p14, "apply_diff(old, out)");
    let apply = common::corpus_dir().join("apply_diff.sur");
    assert_eq!(
        def["uri"],
        surety_lsp::path_to_uri(&fs::canonicalize(&apply).unwrap())
    );
    let apply_text = fs::read_to_string(&apply).unwrap();
    let (sig_line, _) = position_of(&apply_text, "apply_diff(old: ");
    assert_eq!(def["range"]["start"]["line"], sig_line);

    // Hover on a property line shows the sentence above it.
    let hover = lsp.at("textDocument/hover", &p3, "exactly_one g in out:");
    let value = hover["contents"]["value"].as_str().unwrap_or_default();
    assert!(
        value.contains("there is exactly one group with that email"),
        "the sentence above the property: {value}"
    );

    // Hover on a sibling obligation shows its signature and, with the
    // committed graph, its state.
    let hover = lsp.at("textDocument/hover", &p14, "apply_diff(old, out)");
    let value = hover["contents"]["value"].as_str().unwrap_or_default();
    assert!(value.contains("apply_diff(old: "), "{value}");
    assert!(value.contains("in `apply_diff.sur`"), "{value}");
    assert!(value.contains("state: **verified**"), "{value}");

    // Hover on a type shows its declaration.
    let hover = lsp.at("textDocument/hover", &p5b, "Order[]");
    let value = hover["contents"]["value"].as_str().unwrap_or_default();
    assert!(
        value.contains("type Order = { amount: Int, customer_email: Text }"),
        "{value}"
    );

    // Symbols: the type, the obligation, its properties by sentence, its examples.
    let uri3 = surety_lsp::path_to_uri(&fs::canonicalize(&p3).unwrap());
    let symbols = lsp.request(
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": uri3 } }),
    );
    let list = symbols["result"].as_array().unwrap();
    let names: Vec<&str> = list.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(
        names.contains(&"Order") && names.contains(&"OrderGroup"),
        "{names:?}"
    );
    let fun = list
        .iter()
        .find(|s| {
            s["name"]
                .as_str()
                .is_some_and(|n| n.starts_with("group_by_customer("))
        })
        .unwrap();
    let children: Vec<&str> = fun["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    assert!(
        children
            .iter()
            .any(|c| c.starts_with("For every distinct customer_email")),
        "{children:?}"
    );
    assert!(
        children.contains(&"example 1") && children.contains(&"example 2"),
        "{children:?}"
    );
    lsp.stop();
}

fn hover_text(reply: Json) -> String {
    reply["contents"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn hover_writes_out_spreads_and_gives_every_callee_its_interface() {
    let mut lsp = Lsp::start_with_graph();
    let p7 = common::corpus_dir().join("p7.sur");
    let p8 = common::corpus_dir().join("p8.sur");
    let p14 = common::corpus_dir().join("p14.sur");
    for p in [&p7, &p8, &p14] {
        assert!(lsp.open(p).iter().all(|d| d["severity"] == 4), "no error");
    }

    // A record that spreads another is written out field by field: the
    // inherited under the type they came from, the own under `own`, and a
    // link to the spread type's line.
    let value = hover_text(lsp.at("textDocument/hover", &p7, "EnrichedOrder[]"));
    assert!(
        value.contains("type EnrichedOrder = {\n    // from Order\n    id: Id,"),
        "{value}"
    );
    assert!(
        value.contains("    // own\n    user_name: Option<Text>\n}"),
        "{value}"
    );
    let p7_uri = surety_lsp::path_to_uri(&fs::canonicalize(&p7).unwrap());
    assert!(
        value.contains(&format!("[`Order`]({p7_uri}#L2)")),
        "{value}"
    );

    // An alias is followed by what it names, across the directory.
    let apply = common::corpus_dir().join("apply_diff.sur");
    assert!(
        lsp.open(&apply).iter().all(|d| d["severity"] == 4),
        "no error"
    );
    let value = hover_text(lsp.at("textDocument/hover", &apply, "DiffError = "));
    assert!(
        value.contains(
            "type DiffError = ValidationError\ntype ValidationError = { code: Text, display: Text }"
        ),
        "{value}"
    );

    // An obligation shows its interface: the signature, the note, and the
    // types the signature names with what they alias.
    let value = hover_text(lsp.at("textDocument/hover", &p14, "snapshot_diff(old"));
    assert!(
        value.contains("snapshot_diff(old: Item[unique sku], new: Item[unique sku]) -> Diff"),
        "{value}"
    );
    assert!(
        value.contains("return the added, removed, and changed items"),
        "{value}"
    );
    assert!(
        value.contains("type Item = { sku: Id, qty: Int }\ntype Diff = {"),
        "{value}"
    );
    let value = hover_text(lsp.at("textDocument/hover", &p14, "apply_diff(old, out)"));
    assert!(
        value.contains("type DiffError = ValidationError\ntype ValidationError = "),
        "{value}"
    );

    // A bodiless predicate is a graph reference: typed as the file calls it.
    let value = hover_text(lsp.at("textDocument/hover", &p8, "well_formed_email(p.email)"));
    assert!(value.contains("well_formed_email(Text) -> Bool"), "{value}");
    assert!(value.contains("A graph reference"), "{value}");

    // A built-in says so.
    let value = hover_text(lsp.at("textDocument/hover", &p8, "len(password)"));
    assert!(value.contains("`len`: built-in"), "{value}");

    // Outside the corpus: an obligation sharing a pinned name does not
    // borrow that node's state, and a call to an obligation only the graph
    // holds shows its interface, read from its source with the siblings.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("lsp-hover");
    fs::create_dir_all(&dir).unwrap();
    let elsewhere = dir.join("elsewhere.sur");
    fs::write(
        &elsewhere,
        "type Item = { sku: Id, qty: Int }\n\napply_diff(old: Item[]) -> Item[]\n\tall i in out: i in old\n\tsnapshot_diff(old, out) == snapshot_diff(out, old)\n",
    )
    .unwrap();
    lsp.open(&elsewhere);
    let value = hover_text(lsp.at("textDocument/hover", &elsewhere, "apply_diff(old: "));
    assert!(
        value.contains(
            "the graph pins `apply_diff` from `crates/surety/tests/corpus/apply_diff.sur`"
        ),
        "{value}"
    );
    assert!(!value.contains("state:"), "{value}");
    // An obligation named like the start of a node's hash is not that node:
    // its state comes from the namespace by name, never from a hash prefix.
    let prefix: String =
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../graph/nodes"))
            .unwrap()
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .find(|h| h.chars().take(3).all(|c| ('a'..='f').contains(&c)))
            .map(|h| h[..3].to_string())
            .expect("a node whose hash starts with three letters");
    let hashlike = dir.join("hashlike.sur");
    fs::write(&hashlike, format!("{prefix}(x: Int) -> Int\n\tout == x\n")).unwrap();
    lsp.open(&hashlike);
    let value = hover_text(lsp.at("textDocument/hover", &hashlike, &format!("{prefix}(x")));
    assert!(
        value.contains(&format!("{prefix}(x: Int) -> Int")),
        "{value}"
    );
    assert!(
        !value.contains("state:") && !value.contains("pins"),
        "{value}"
    );
    let value = hover_text(lsp.at("textDocument/hover", &elsewhere, "snapshot_diff(old, out)"));
    assert!(
        value.contains("snapshot_diff(old: Item[unique sku], new: Item[unique sku]) -> Diff"),
        "{value}"
    );
    assert!(value.contains("type Diff = {"), "{value}");
    assert!(
        value.contains(
            "in the graph, pinned from `crates/surety/tests/corpus/p14.sur` · state: **verified**"
        ),
        "{value}"
    );
    lsp.stop();
}

impl Lsp {
    /// Completion where the editor holds `text` (unsaved), at the end of
    /// the first `needle` in it: the labels, with each item.
    fn complete(&mut self, path: &Path, text: &str, needle: &str) -> Vec<Json> {
        self.change(path, text);
        let uri = surety_lsp::path_to_uri(&fs::canonicalize(path).unwrap());
        let (line, character) = position_of(text, needle);
        let reply = self.request(
            "textDocument/completion",
            json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character + needle.chars().count() } }),
        );
        reply["result"].as_array().cloned().unwrap_or_default()
    }
}

fn labels(items: &[Json]) -> Vec<&str> {
    items.iter().filter_map(|i| i["label"].as_str()).collect()
}

#[test]
fn completion_offers_the_words_the_language_has_and_the_names_in_scope() {
    let mut lsp = Lsp::start();
    let init = lsp.request("initialize", json!({ "capabilities": {} }));
    assert_eq!(
        init["result"]["capabilities"]["completionProvider"]["triggerCharacters"],
        json!(["."])
    );
    let p2 = common::corpus_dir().join("p2.sur");
    let saved = fs::read_to_string(&p2).unwrap();
    assert!(lsp.open(&p2).is_empty());

    // A relation half typed, on a line that does not parse yet.
    let text = saved.replace("out same_order_as orders", "out same_o");
    let items = lsp.complete(&p2, &text, "out same_o");
    let relation = items
        .iter()
        .find(|i| i["label"] == "same_order_as")
        .expect("same_order_as");
    assert_eq!(relation["kind"], 14, "a keyword");
    assert_eq!(
        relation["detail"],
        "drawn from the right side, in its order"
    );
    let all = labels(&items);
    for want in [
        "same_elements_as",
        "drawn_from",
        "orders",
        "out",
        "Order",
        "len",
        "Int",
    ] {
        assert!(all.contains(&want), "{want}: {all:?}");
    }
    assert!(!all.contains(&"same_o"), "not the word being typed");
    let input = items.iter().find(|i| i["label"] == "orders").unwrap();
    assert_eq!(input["detail"], "orders: Order[]");

    // After a binder's `.`: the fields of its type, read from its quantifier.
    let text = saved.replace("all o in out: o.amount <= 100", "all o in out: o.");
    let items = lsp.complete(&p2, &text, "all o in out: o.");
    let mut fields = labels(&items);
    fields.sort();
    assert_eq!(fields, ["amount", "customer_email"]);

    // After an `Ok` pattern's binder: the fields of the Ok side's payload.
    let p8 = common::corpus_dir().join("p8.sur");
    let saved8 = fs::read_to_string(&p8).unwrap();
    assert!(lsp.open(&p8).is_empty());
    let text = saved8.replace("all Ok(p_o) in out: p_o == p", "all Ok(p_o) in out: p_o.");
    let mut fields = labels(&lsp.complete(&p8, &text, "all Ok(p_o) in out: p_o."))
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    fields.sort();
    assert_eq!(fields, ["age", "email", "password"]);

    // A half-typed head over a vertical body: the file as saved still says
    // what `c_out` ranges over.
    let p14 = common::corpus_dir().join("p14.sur");
    let saved14 = fs::read_to_string(&p14).unwrap();
    lsp.open(&p14);
    let text = saved14.replacen(
        "all c_out in out.changed:",
        "all c_out in out.changed: c_out.",
        1,
    );
    let mut fields = labels(&lsp.complete(&p14, &text, "all c_out in out.changed: c_out."))
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    fields.sort();
    assert_eq!(fields, ["new_qty", "old_qty", "sku"]);
    lsp.stop();
}
