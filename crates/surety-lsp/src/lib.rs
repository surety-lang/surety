//! A language server over stdio on the existing parser. One source of
//! truth: the server calls what `surety parse` calls, so a diagnostic is
//! exactly the error `parse` would print, on the line it carries, the range
//! being the whole line. Parse on every change; nothing here runs the
//! checker. The protocol is JSON-RPC with Content-Length framing, written
//! by hand: the surface this server speaks is a dozen methods, and no
//! crate is needed for it.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value as Json, json};
use surety_syntax::Error;

pub mod query;
pub mod siblings;

use siblings::Siblings;

/// The errors `parse` reports for a file with this text and its siblings
/// on disk: at most one, since the parser and the resolver stop at the
/// first. A sibling's error is the sibling's and is not reported here.
pub fn diagnostics_for(path: &Path, text: &str) -> Vec<Error> {
    diagnostics_with(&mut Siblings::default(), path, text)
}

/// The same, the siblings' parses kept in `siblings` between calls.
pub fn diagnostics_with(siblings: &mut Siblings, path: &Path, text: &str) -> Vec<Error> {
    match siblings.load(path, text) {
        Err(errs) => errs
            .into_iter()
            .filter(|(p, _)| p == path)
            .map(|(_, e)| e)
            .collect(),
        Ok((graph, idx)) => match graph.check_file(idx) {
            Ok(()) => Vec::new(),
            Err(e) => vec![e],
        },
    }
}

/// The whole document replaced by its rendering: prose wrapped at the
/// column, every comment where it was, the obligation unchanged. Null when
/// the text does not parse, since there is nothing to render then.
pub fn formatting(text: &str) -> Json {
    match surety_syntax::parse(text) {
        Ok(file) => {
            let rendered = surety_syntax::render::file(&file, false);
            if rendered == text {
                return json!([]);
            }
            let end = text.lines().count() + usize::from(text.ends_with('\n'));
            json!([{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": end, "character": 0 } },
                "newText": rendered,
            }])
        }
        Err(_) => Json::Null,
    }
}

/// A hint, never an error, on each sentence or comment line past the
/// column: formatting wraps it.
pub fn long_prose_lines(text: &str) -> Vec<Json> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            (t.starts_with('#') || t.starts_with("//"))
                && surety_syntax::render::columns(l) > surety_syntax::render::WIDTH
        })
        .map(|(i, l)| {
            let indent = l.chars().take_while(|c| c.is_whitespace()).count();
            json!({
                "range": { "start": { "line": i, "character": indent }, "end": { "line": i, "character": l.encode_utf16().count() } },
                "severity": 4,
                "code": "long_line",
                "source": "surety",
                "message": format!("past {} columns; formatting wraps it", surety_syntax::render::WIDTH),
            })
        })
        .collect()
}

/// An error as an LSP diagnostic: the columns it is about when it knows
/// them, else its line from the first character after the indentation to
/// the end; severity Error, the kind as the code.
pub fn diagnostic(e: &Error, text: &str) -> Json {
    let line = e.line.saturating_sub(1);
    let l = text.lines().nth(line).unwrap_or_default();
    let len = l.chars().count();
    let indent = l.chars().take_while(|c| c.is_whitespace()).count();
    // A fix names the term it mends: underline that when no span is known.
    let term = e.fix.as_ref().and_then(|f| term_at(l, &f.find));
    let (start, len) = match e.span.or(term) {
        Some((from, to)) if to <= len => (from, to),
        _ => (if indent < len { indent } else { 0 }, len),
    };
    let mut d = json!({
        "range": { "start": { "line": line, "character": start }, "end": { "line": line, "character": len } },
        "severity": 1,
        "code": e.kind.name(),
        "source": "surety",
        "message": e.message,
    });
    if let Some(f) = &e.fix {
        d["data"] = json!({ "fix": { "title": f.title, "find": f.find, "replace": f.replace } });
    }
    d
}

/// The columns of the first whole term `find` on a line: not inside a
/// longer name or field chain.
pub fn term_at(line: &str, find: &str) -> Option<(usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    let want: Vec<char> = find.chars().collect();
    let joins = |c: Option<&char>| c.is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '.');
    (0..=chars.len().checked_sub(want.len())?).find_map(|at| {
        let end = at + want.len();
        let whole = chars[at..end] == want[..]
            && !joins(at.checked_sub(1).and_then(|i| chars.get(i)))
            && !joins(chars.get(end));
        whole.then_some((at, end))
    })
}

/// The quick fixes for diagnostics that carry one: the edit on the line the
/// diagnostic is on, where the term it names stands now.
pub fn code_actions(uri: &str, text: &str, diagnostics: &[Json]) -> Json {
    let mut actions = Vec::new();
    for d in diagnostics {
        let fix = &d["data"]["fix"];
        let (Some(find), Some(replace)) = (fix["find"].as_str(), fix["replace"].as_str()) else {
            continue;
        };
        let line = d["range"]["start"]["line"].as_u64().unwrap_or(0) as usize;
        let Some((from, to)) = text.lines().nth(line).and_then(|l| term_at(l, find)) else {
            continue;
        };
        actions.push(json!({
            "title": fix["title"],
            "kind": "quickfix",
            "diagnostics": [d],
            "isPreferred": true,
            "edit": { "changes": { uri: [{
                "range": { "start": { "line": line, "character": from }, "end": { "line": line, "character": to } },
                "newText": replace,
            }] } },
        }));
    }
    Json::Array(actions)
}

/// `file:///a/b%20c.sur` → `/a/b c.sur`.
pub fn uri_to_path(uri: &str) -> PathBuf {
    let raw = uri.strip_prefix("file://").unwrap_or(uri);
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&raw[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    PathBuf::from(String::from_utf8_lossy(&out).into_owned())
}

pub fn path_to_uri(path: &Path) -> String {
    let mut s = String::from("file://");
    for b in path.to_string_lossy().bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => {
                s.push(b as char)
            }
            _ => s.push_str(&format!("%{b:02X}")),
        }
    }
    s
}

pub struct Server {
    docs: HashMap<String, String>,
    siblings: Siblings,
    /// The graph directory `show` answers from, when one was given.
    pub graph: Option<PathBuf>,
    pub exited: bool,
}

impl Server {
    pub fn new(graph: Option<PathBuf>) -> Server {
        Server {
            docs: HashMap::new(),
            siblings: Siblings::default(),
            graph,
            exited: false,
        }
    }

    /// The sibling files parsed from disk since the server started.
    pub fn parses(&self) -> usize {
        self.siblings.parses()
    }

    fn publish(&mut self, uri: &str) -> Json {
        let text = self.docs.get(uri).cloned().unwrap_or_default();
        let path = uri_to_path(uri);
        let diagnostics: Vec<Json> = if self.docs.contains_key(uri) {
            let mut all: Vec<Json> = diagnostics_with(&mut self.siblings, &path, &text)
                .iter()
                .map(|e| diagnostic(e, &text))
                .collect();
            all.extend(long_prose_lines(&text));
            all
        } else {
            Vec::new()
        };
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": diagnostics },
        })
    }

    fn response(id: &Json, result: Json) -> Json {
        json!({ "jsonrpc": "2.0", "id": id, "result": result })
    }

    /// Handle one message; the messages to send back, in order.
    pub fn handle(&mut self, msg: &Json) -> Vec<Json> {
        let method = msg["method"].as_str().unwrap_or_default();
        let params = &msg["params"];
        let id = msg.get("id").cloned();
        let uri = params["textDocument"]["uri"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        match (method, id) {
            ("initialize", Some(id)) => vec![Self::response(
                &id,
                json!({
                    "capabilities": {
                        "textDocumentSync": 1,
                        "hoverProvider": true,
                        "definitionProvider": true,
                        "documentSymbolProvider": true,
                        "documentFormattingProvider": true,
                        "completionProvider": { "triggerCharacters": ["."] },
                        "codeActionProvider": true,
                    },
                    "serverInfo": { "name": "surety-lsp", "version": env!("CARGO_PKG_VERSION") },
                }),
            )],
            ("shutdown", Some(id)) => vec![Self::response(&id, Json::Null)],
            ("exit", _) => {
                self.exited = true;
                Vec::new()
            }
            ("textDocument/didOpen", _) => {
                let text = params["textDocument"]["text"].as_str().unwrap_or_default();
                self.docs.insert(uri.clone(), text.to_string());
                vec![self.publish(&uri)]
            }
            ("textDocument/didChange", _) => {
                if let Some(last) = params["contentChanges"].as_array().and_then(|c| c.last()) {
                    if let Some(text) = last["text"].as_str() {
                        self.docs.insert(uri.clone(), text.to_string());
                    }
                }
                vec![self.publish(&uri)]
            }
            ("textDocument/didSave", _) => {
                if let Some(text) = params["text"].as_str() {
                    self.docs.insert(uri.clone(), text.to_string());
                }
                vec![self.publish(&uri)]
            }
            ("textDocument/didClose", _) => {
                self.docs.remove(&uri);
                vec![self.publish(&uri)]
            }
            ("textDocument/hover", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let result = query::hover(
                    &mut self.siblings,
                    &uri_to_path(&uri),
                    &text,
                    &params["position"],
                    self.graph.as_deref(),
                );
                vec![Self::response(&id, result)]
            }
            ("textDocument/definition", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let result = query::definition(
                    &mut self.siblings,
                    &uri_to_path(&uri),
                    &text,
                    &params["position"],
                );
                vec![Self::response(&id, result)]
            }
            ("textDocument/codeAction", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let diagnostics = params["context"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                vec![Self::response(&id, code_actions(&uri, &text, &diagnostics))]
            }
            ("surety/unverified", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let result = query::unverified(
                    &mut self.siblings,
                    &uri_to_path(&uri),
                    &text,
                    self.graph.as_deref(),
                );
                vec![Self::response(&id, result)]
            }
            ("textDocument/completion", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let result = query::completion(
                    &mut self.siblings,
                    &uri_to_path(&uri),
                    &text,
                    &params["position"],
                );
                vec![Self::response(&id, result)]
            }
            ("textDocument/documentSymbol", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                vec![Self::response(&id, query::symbols(&text))]
            }
            ("textDocument/formatting", Some(id)) => {
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                vec![Self::response(&id, formatting(&text))]
            }
            (_, Some(id)) => vec![json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("method not found: {method}") },
            })],
            _ => Vec::new(),
        }
    }
}

/// One framed message from the reader, or None at end of input.
pub fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Json>> {
    let mut length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            length = v.trim().parse().ok();
        }
    }
    let Some(n) = length else {
        return Err(io::Error::other("a message without Content-Length"));
    };
    let mut body = vec![0u8; n];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| io::Error::other(format!("bad JSON-RPC body: {e}")))
}

pub fn write_message(writer: &mut impl Write, msg: &Json) -> io::Result<()> {
    let body = msg.to_string();
    write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    writer.flush()
}

/// Serve stdin and stdout until `exit`.
pub fn run(graph: Option<PathBuf>) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut server = Server::new(graph);
    while let Some(msg) = read_message(&mut reader)? {
        for reply in server.handle(&msg) {
            write_message(&mut writer, &reply)?;
        }
        if server.exited {
            break;
        }
    }
    Ok(())
}
