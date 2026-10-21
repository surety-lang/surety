//! Hover, go to definition and document symbols, answered from the file's
//! syntax tree and its siblings on disk. A line belongs to a presentation,
//! so everything here reads the text the editor holds and nothing reads
//! the graph except a sibling's state on hover, when a graph is given.

use std::path::{Path, PathBuf};

use serde_json::{Value as Json, json};
use surety_syntax::load::load_with_siblings_text;
use surety_syntax::{Decl, File, Function, Graph, render};

use crate::path_to_uri;

/// The identifier under a position, if the cursor is on one.
fn word_at(text: &str, line: usize, character: usize) -> Option<String> {
    let l = text.lines().nth(line)?;
    let chars: Vec<char> = l.chars().collect();
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if character > chars.len() {
        return None;
    }
    let mut start = character;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let mut end = character;
    while end < chars.len() && is_word(chars[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some(chars[start..end].iter().collect())
}

fn line_range(line: usize, text: &str) -> Json {
    let l = line.saturating_sub(1);
    let len = text.lines().nth(l).map(|x| x.chars().count()).unwrap_or(0);
    json!({ "start": { "line": l, "character": 0 }, "end": { "line": l, "character": len } })
}

fn location(path: &Path, line: usize, text: &str) -> Json {
    json!({ "uri": path_to_uri(path), "range": line_range(line, text) })
}

fn sibling_path(file: &Path, graph: &Graph, fidx: usize) -> PathBuf {
    file.parent()
        .unwrap_or(Path::new("."))
        .join(format!("{}.sur", graph.files[fidx].name))
}

/// Where a name is declared: a type, a named value, an abbreviation (file
/// or function level) or an obligation, in this file or a sibling.
enum Found {
    Type(usize, String),
    Value(usize, String),
    Abbrev(usize, String),
    Obligation(usize, String),
}

fn find_in(file: &File, name: &str) -> Option<Found> {
    for d in &file.decls {
        match d {
            Decl::Type(t) if t.name == name => {
                return Some(Found::Type(t.line, format!("type {} = {}", t.name, t.ty)));
            }
            Decl::NamedValue(nv) if nv.name == name => {
                let text = match &nv.ty {
                    Some(t) => format!("{}: {t} = {}", nv.name, render::expr(&nv.value)),
                    None => format!("{} = {}", nv.name, render::expr(&nv.value)),
                };
                return Some(Found::Value(nv.line, text));
            }
            Decl::Abbrev(a) if a.name == name => {
                return Some(Found::Abbrev(a.line, render::abbrev(a)));
            }
            Decl::Function(f) => {
                if f.name == name {
                    return Some(Found::Obligation(f.line, signature(f)));
                }
                for a in &f.abbrevs {
                    if a.name == name {
                        return Some(Found::Abbrev(a.line, render::abbrev(a)));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn signature(f: &Function) -> String {
    let ins: Vec<String> = f.inputs.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    format!("{}({}) -> {}", f.name, ins.join(", "), f.output)
}

/// The name's declaration: (the file index it is in, what was found).
fn resolve(graph: &Graph, idx: usize, name: &str) -> Option<(usize, Found)> {
    if let Some(f) = find_in(graph.file(idx), name) {
        return Some((idx, f));
    }
    for (fidx, gf) in graph.files.iter().enumerate() {
        if fidx != idx {
            if let Some(f) = find_in(&gf.file, name) {
                return Some((fidx, f));
            }
        }
    }
    None
}

fn position(p: &Json) -> (usize, usize) {
    (
        p["line"].as_u64().unwrap_or(0) as usize,
        p["character"].as_u64().unwrap_or(0) as usize,
    )
}

pub fn definition(path: &Path, text: &str, pos: &Json) -> Json {
    let (line, character) = position(pos);
    let Some(word) = word_at(text, line, character) else {
        return Json::Null;
    };
    let Ok((graph, idx)) = load_with_siblings_text(path, text) else {
        return Json::Null;
    };
    let Some((fidx, found)) = resolve(&graph, idx, &word) else {
        return Json::Null;
    };
    let decl_line = match found {
        Found::Type(l, _) | Found::Value(l, _) | Found::Abbrev(l, _) | Found::Obligation(l, _) => l,
    };
    if fidx == idx {
        location(path, decl_line, text)
    } else {
        let sibling = sibling_path(path, &graph, fidx);
        let sibling_text = std::fs::read_to_string(&sibling).unwrap_or_default();
        location(&sibling, decl_line, &sibling_text)
    }
}

fn markdown(value: String) -> Json {
    json!({ "contents": { "kind": "markdown", "value": value } })
}

pub fn hover(path: &Path, text: &str, pos: &Json, graph_dir: Option<&Path>) -> Json {
    let (line, character) = position(pos);
    let parsed = load_with_siblings_text(path, text);
    if let Some(word) = word_at(text, line, character) {
        if let Ok((graph, idx)) = &parsed {
            if let Some((fidx, found)) = resolve(graph, *idx, &word) {
                let mut value = match &found {
                    Found::Type(_, t) | Found::Value(_, t) | Found::Abbrev(_, t) => {
                        format!("```surety\n{t}\n```")
                    }
                    Found::Obligation(_, t) => format!("```surety\n{t}\n```"),
                };
                if fidx != *idx {
                    value.push_str(&format!("\n\nin `{}.sur`", graph.files[fidx].name));
                }
                if let (Found::Obligation(..), Some(dir)) = (&found, graph_dir) {
                    let store = surety_graph::Store::open(dir);
                    if let Ok(view) = surety_graph::show(&store, &word) {
                        value.push_str(&format!("\n\nstate: **{}**", view.state.word()));
                    }
                }
                return markdown(value);
            }
        }
    }
    // Not a name: a property line, or a continuation line of a vertical
    // body, shows the sentence above the property.
    if let Ok((graph, idx)) = &parsed {
        let cursor = line + 1;
        for f in graph.functions_in(*idx) {
            let first_example = f
                .examples
                .iter()
                .map(|e| e.line)
                .min()
                .unwrap_or(usize::MAX);
            if cursor >= first_example {
                continue;
            }
            let mut items: Vec<_> = f.properties.iter().chain(&f.accepts).collect();
            items.sort_by_key(|p| p.line);
            let at = items
                .iter()
                .enumerate()
                .find(|(i, p)| {
                    p.line <= cursor && items.get(i + 1).is_none_or(|next| next.line > cursor)
                })
                .map(|(_, p)| *p);
            if let Some(p) = at {
                return match &p.sentence {
                    Some(s) => markdown(s.clone()),
                    None => Json::Null,
                };
            }
        }
    }
    Json::Null
}

fn symbol(name: String, kind: u32, line: usize, text: &str, children: Vec<Json>) -> Json {
    let range = line_range(line, text);
    let mut s = json!({ "name": name, "kind": kind, "range": range, "selectionRange": range });
    if !children.is_empty() {
        s["children"] = Json::Array(children);
    }
    s
}

fn short(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

/// The types, the signature with each property by its sentence (its text
/// when it has none), the accept lines, and the examples by note.
pub fn symbols(text: &str) -> Json {
    const STRUCT: u32 = 23;
    const CONSTANT: u32 = 14;
    const FUNCTION: u32 = 12;
    const PROPERTY: u32 = 7;
    const EVENT: u32 = 24;
    let Ok(file) = surety_syntax::parse(text) else {
        return json!([]);
    };
    let mut out = Vec::new();
    for d in &file.decls {
        match d {
            Decl::Type(t) => out.push(symbol(t.name.clone(), STRUCT, t.line, text, Vec::new())),
            Decl::NamedValue(nv) => {
                out.push(symbol(nv.name.clone(), CONSTANT, nv.line, text, Vec::new()))
            }
            Decl::Abbrev(a) => out.push(symbol(a.name.clone(), FUNCTION, a.line, text, Vec::new())),
            Decl::Function(f) => {
                let mut children = Vec::new();
                for a in &f.abbrevs {
                    children.push(symbol(a.name.clone(), FUNCTION, a.line, text, Vec::new()));
                }
                for p in &f.properties {
                    let name = p.sentence.clone().unwrap_or_else(|| short(&p.text, 60));
                    children.push(symbol(name, PROPERTY, p.line, text, Vec::new()));
                }
                for a in &f.accepts {
                    children.push(symbol(
                        format!("accept {}", short(&a.text, 50)),
                        PROPERTY,
                        a.line,
                        text,
                        Vec::new(),
                    ));
                }
                for (i, e) in f.examples.iter().enumerate() {
                    let name = e
                        .note
                        .clone()
                        .unwrap_or_else(|| format!("example {}", i + 1));
                    children.push(symbol(name, EVENT, e.line, text, Vec::new()));
                }
                out.push(symbol(signature(f), FUNCTION, f.line, text, children));
            }
        }
    }
    Json::Array(out)
}
