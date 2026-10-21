//! Slice 4, acceptance test 1: the language server's diagnostics are
//! `parse`'s. Driven over stdio, the way an editor drives it: for every
//! file under `corpus/bad/`, one diagnostic with the `.err` file's kind and
//! line; for the eleven with their siblings, none.

mod common;

use std::fs;
use std::io::{BufReader, Write};
use std::path::Path;
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
                return msg["params"]["diagnostics"].as_array().cloned().unwrap_or_default();
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
                return msg["params"]["diagnostics"].as_array().cloned().unwrap_or_default();
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
        assert_eq!(diagnostics.len(), 1, "{}: one diagnostic, as parse prints one error", path.display());
        let d = &diagnostics[0];
        assert_eq!(d["code"], kind, "{}: the kind", path.display());
        assert_eq!(d["range"]["start"]["line"].as_u64().unwrap() + 1, line, "{}: the line", path.display());
        assert_eq!(d["severity"], 1);
        assert_eq!(d["source"], "surety");
        assert!(d["message"].as_str().is_some_and(|m| !m.is_empty()));
        checked += 1;
    }
    assert_eq!(checked, 15, "the fifteen negative fixtures");
    for name in common::FIXTURES {
        let path = common::corpus_dir().join(format!("{name}.sur"));
        let diagnostics = lsp.open(&path);
        assert!(diagnostics.is_empty(), "{name}: {diagnostics:?}");
    }
    lsp.stop();
}

#[test]
fn a_change_reparses_against_the_siblings_on_disk() {
    let mut lsp = Lsp::start();
    let path = common::corpus_dir().join("p14.sur");
    assert!(lsp.open(&path).is_empty());
    // Unsaved text: a reference to a type the siblings do not define.
    let text = fs::read_to_string(&path).unwrap().replace("old: Item[unique sku]", "old: Itm[unique sku]");
    let diagnostics = lsp.change(&path, &text);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["code"], "unknown_type");
    assert_eq!(diagnostics[0]["range"]["start"]["line"], 0, "the signature is line 1");
    // Back to the saved text: clean again, and the siblings still resolve `Item`.
    let diagnostics = lsp.change(&path, &fs::read_to_string(&path).unwrap());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    lsp.stop();
}
