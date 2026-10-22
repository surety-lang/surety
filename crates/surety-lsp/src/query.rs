//! Hover, go to definition and document symbols, answered from the file's
//! syntax tree and its siblings on disk. A line belongs to a presentation,
//! so everything here reads the text the editor holds and nothing reads
//! the graph except a sibling's state on hover, and an obligation only the
//! graph holds, when a graph is given.

use std::path::{Path, PathBuf};

use serde_json::{Value as Json, json};
use surety_syntax::load::{load_with_siblings, load_with_siblings_text};
use surety_syntax::parser::{BUILTINS, PRIMITIVES, RESERVED};
use surety_syntax::resolve::Scope;
use surety_syntax::{
    Abbrev, BinOp, Decl, Expr, File, Function, Graph, NamedValue, Type, TypeDecl, render,
};

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

/// Whether the identifier under a position is called: `(` right after it.
fn called_at(text: &str, line: usize, character: usize) -> bool {
    let Some(l) = text.lines().nth(line) else {
        return false;
    };
    let chars: Vec<char> = l.chars().collect();
    let mut end = character.min(chars.len());
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    chars.get(end) == Some(&'(')
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

/// The path of the file at `fidx`, given the path of the one at `idx`.
fn file_path(path: &Path, graph: &Graph, idx: usize, fidx: usize) -> PathBuf {
    if fidx == idx {
        path.to_path_buf()
    } else {
        sibling_path(path, graph, fidx)
    }
}

/// Where a name is declared: a type, a named value, an abbreviation (file
/// or function level) or an obligation, in this file or a sibling.
enum Found<'a> {
    Type(&'a TypeDecl),
    Value(&'a NamedValue),
    Abbrev(&'a Abbrev),
    Obligation(&'a Function),
}

impl Found<'_> {
    fn line(&self) -> usize {
        match self {
            Found::Type(t) => t.line,
            Found::Value(nv) => nv.line,
            Found::Abbrev(a) => a.line,
            Found::Obligation(f) => f.line,
        }
    }
}

fn find_in<'a>(file: &'a File, name: &str) -> Option<Found<'a>> {
    for d in &file.decls {
        match d {
            Decl::Type(t) if t.name == name => return Some(Found::Type(t)),
            Decl::NamedValue(nv) if nv.name == name => return Some(Found::Value(nv)),
            Decl::Abbrev(a) if a.name == name => return Some(Found::Abbrev(a)),
            Decl::Function(f) => {
                if f.name == name {
                    return Some(Found::Obligation(f));
                }
                if let Some(a) = f.abbrevs.iter().find(|a| a.name == name) {
                    return Some(Found::Abbrev(a));
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
fn resolve<'g>(graph: &'g Graph, idx: usize, name: &str) -> Option<(usize, Found<'g>)> {
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
    if fidx == idx {
        location(path, found.line(), text)
    } else {
        let sibling = sibling_path(path, &graph, fidx);
        let sibling_text = std::fs::read_to_string(&sibling).unwrap_or_default();
        location(&sibling, found.line(), &sibling_text)
    }
}

fn markdown(value: String) -> Json {
    json!({ "contents": { "kind": "markdown", "value": value } })
}

fn code(text: &str) -> String {
    format!("```surety\n{text}\n```")
}

fn push_unique(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|n| n == name) {
        names.push(name.to_string());
    }
}

/// The type names a type mentions: named types and spreads, at any depth.
fn named_in(ty: &Type, out: &mut Vec<String>) {
    match ty {
        Type::Named(n) => push_unique(out, n),
        Type::Record { spreads, fields } => {
            for s in spreads {
                push_unique(out, s);
            }
            for (_, t) in fields {
                named_in(t, out);
            }
        }
        Type::List(t, _) | Type::Option(t) => named_in(t, out),
        Type::Result(t, e) => {
            named_in(t, out);
            named_in(e, out);
        }
        _ => {}
    }
}

/// One field of a record with its spreads flattened: its type as written,
/// and the spreads it came through, outermost first (empty for the
/// record's own fields).
struct Field {
    name: String,
    ty: Type,
    via: Vec<String>,
}

/// A record's fields in order, spreads flattened; None when a spread does
/// not resolve to a record.
fn fields_of(
    graph: &Graph,
    from: usize,
    ty: &Type,
    line: usize,
    depth: usize,
) -> Option<Vec<Field>> {
    if depth > 32 {
        return None;
    }
    match ty {
        Type::Named(n) => {
            let (idx, decl) = graph.find_type(from, n, line).ok()?;
            fields_of(graph, idx, &decl.ty, decl.line, depth + 1)
        }
        Type::Record { spreads, fields } => {
            let mut out = Vec::new();
            for s in spreads {
                let (idx, decl) = graph.find_type(from, s, line).ok()?;
                for mut f in fields_of(graph, idx, &decl.ty, decl.line, depth + 1)? {
                    f.via.insert(0, s.clone());
                    out.push(f);
                }
            }
            for (n, t) in fields {
                out.push(Field {
                    name: n.clone(),
                    ty: t.clone(),
                    via: Vec::new(),
                });
            }
            Some(out)
        }
        _ => None,
    }
}

/// The comment over a group of fields: the type that declares them, and
/// the spreads between, or `own`.
fn origin(via: &[String]) -> String {
    match via {
        [] => "own".to_string(),
        [one] => format!("from {one}"),
        [between @ .., last] => format!("from {last}, through {}", between.join(", ")),
    }
}

/// A record that spreads others, written out a field to a line, each run
/// of fields under a comment saying where it came from.
fn expanded(name: &str, fields: &[Field]) -> String {
    let mut s = format!("type {name} = {{\n");
    let mut group: Option<&[String]> = None;
    for (i, f) in fields.iter().enumerate() {
        if group != Some(f.via.as_slice()) {
            s.push_str(&format!("    // {}\n", origin(&f.via)));
            group = Some(&f.via);
        }
        let comma = if i + 1 < fields.len() { "," } else { "" };
        s.push_str(&format!("    {}: {}{comma}\n", f.name, f.ty));
    }
    s.push('}');
    s
}

/// A type's declaration for a hover: a record that spreads others written
/// out field by field, an alias followed by what it names, anything else
/// as declared. Every type name the text mentions goes into `mentioned`.
fn type_block(
    graph: &Graph,
    fidx: usize,
    t: &TypeDecl,
    mentioned: &mut Vec<String>,
    depth: usize,
) -> String {
    let declared = format!("type {} = {}", t.name, t.ty);
    match &t.ty {
        Type::Named(n) if depth < 32 => {
            push_unique(mentioned, n);
            match graph.find_type(fidx, n, t.line) {
                Ok((j, target)) => format!(
                    "{declared}\n{}",
                    type_block(graph, j, target, mentioned, depth + 1)
                ),
                Err(_) => declared,
            }
        }
        Type::Record { spreads, .. } if !spreads.is_empty() => {
            match fields_of(graph, fidx, &t.ty, t.line, 0) {
                Some(fields) => {
                    for f in &fields {
                        for v in &f.via {
                            push_unique(mentioned, v);
                        }
                        named_in(&f.ty, mentioned);
                    }
                    expanded(&t.name, &fields)
                }
                None => {
                    named_in(&t.ty, mentioned);
                    declared
                }
            }
        }
        _ => {
            named_in(&t.ty, mentioned);
            declared
        }
    }
}

/// A link to a type's declaration, which an editor opens at the line.
fn type_link(graph: &Graph, path: &Path, idx: usize, from: usize, name: &str) -> Option<String> {
    let (j, decl) = graph.find_type(from, name, 0).ok()?;
    let target = file_path(path, graph, idx, j);
    Some(format!(
        "[`{name}`]({}#L{})",
        path_to_uri(&target),
        decl.line
    ))
}

fn type_text(graph: &Graph, path: &Path, idx: usize, fidx: usize, t: &TypeDecl) -> String {
    let mut mentioned = Vec::new();
    let mut s = code(&type_block(graph, fidx, t, &mut mentioned, 0));
    let links: Vec<String> = mentioned
        .iter()
        .filter(|n| **n != t.name)
        .filter_map(|n| type_link(graph, path, idx, fidx, n))
        .collect();
    if !links.is_empty() {
        s.push_str(&format!("\n\ntypes: {}", links.join(" · ")));
    }
    s
}

/// The declarations of the types a signature names, and of the types those
/// spread or alias, in the order they are reached.
fn signature_types<'g>(graph: &'g Graph, fidx: usize, f: &Function) -> Vec<&'g TypeDecl> {
    let mut names = Vec::new();
    for (_, t) in &f.inputs {
        named_in(t, &mut names);
    }
    named_in(&f.output, &mut names);
    let mut queue: Vec<(usize, String)> = names.into_iter().map(|n| (fidx, n)).collect();
    let mut out: Vec<(usize, &TypeDecl)> = Vec::new();
    let mut i = 0;
    while let Some((from, name)) = queue.get(i).cloned() {
        i += 1;
        let Ok((j, decl)) = graph.find_type(from, &name, f.line) else {
            continue;
        };
        if out.iter().any(|(k, d)| *k == j && d.name == decl.name) {
            continue;
        }
        out.push((j, decl));
        match &decl.ty {
            Type::Named(n) => queue.push((j, n.clone())),
            Type::Record { spreads, .. } => {
                queue.extend(spreads.iter().map(|s| (j, s.clone())));
            }
            _ => {}
        }
    }
    out.into_iter().map(|(_, d)| d).collect()
}

/// An obligation's interface: its signature, its note, and the types the
/// signature names.
fn obligation_text(graph: &Graph, fidx: usize, f: &Function) -> String {
    let mut s = code(&signature(f));
    if let Some(note) = &f.note {
        s.push_str(&format!("\n\n{note}"));
    }
    let types: Vec<String> = signature_types(graph, fidx, f)
        .iter()
        .map(|t| format!("type {} = {}", t.name, t.ty))
        .collect();
    if !types.is_empty() {
        s.push_str(&format!("\n\n{}", code(&types.join("\n"))));
    }
    s
}

fn value_text(nv: &NamedValue) -> String {
    match &nv.ty {
        Some(t) => format!("{}: {t} = {}", nv.name, render::expr(&nv.value)),
        None => format!("{} = {}", nv.name, render::expr(&nv.value)),
    }
}

/// The directory a namespace's source paths are relative to: the graph's
/// parent.
fn graph_base(dir: &Path) -> &Path {
    dir.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

/// What the graph says about an obligation declared in `file`: its state
/// from `show` when the namespace pins the name from that file, else
/// where the name is pinned from, since that state is another file's.
fn graph_line(dir: &Path, name: &str, file: &Path) -> Option<String> {
    let store = surety_graph::Store::open(dir);
    let entry = store.namespace().ok()?.get(name)?.clone();
    let pinned = std::fs::canonicalize(graph_base(dir).join(&entry.source)).ok();
    if pinned.is_none() || pinned != std::fs::canonicalize(file).ok() {
        return Some(format!("the graph pins `{name}` from `{}`", entry.source));
    }
    let view = surety_graph::show(&store, name).ok()?;
    Some(format!("state: **{}**", view.state.word()))
}

fn describe(
    graph: &Graph,
    path: &Path,
    idx: usize,
    fidx: usize,
    found: &Found<'_>,
    graph_dir: Option<&Path>,
) -> String {
    let mut value = match found {
        Found::Type(t) => type_text(graph, path, idx, fidx, t),
        Found::Value(nv) => code(&value_text(nv)),
        Found::Abbrev(a) => code(&render::abbrev(a)),
        Found::Obligation(f) => obligation_text(graph, fidx, f),
    };
    let mut meta = Vec::new();
    if fidx != idx {
        meta.push(format!("in `{}.sur`", graph.files[fidx].name));
    }
    if let (Found::Obligation(f), Some(dir)) = (found, graph_dir) {
        if let Some(line) = graph_line(dir, &f.name, &file_path(path, graph, idx, fidx)) {
            meta.push(line);
        }
    }
    if !meta.is_empty() {
        value.push_str(&format!("\n\n{}", meta.join(" · ")));
    }
    value
}

/// A call to a name with no declaration: its line, the type of each
/// argument where the scope it sits in can say, and whether it stands
/// where a truth value goes.
struct Use {
    line: usize,
    args: Vec<Type>,
    truth: bool,
}

fn calls_to(scope: &Scope<'_>, e: &Expr, name: &str, truth: bool, out: &mut Vec<Use>) {
    match e {
        Expr::Call(n, args, line) => {
            if n == name {
                out.push(Use {
                    line: *line,
                    args: args
                        .iter()
                        .map(|a| scope.infer(a, *line).unwrap_or(Type::Unknown))
                        .collect(),
                    truth,
                });
            }
            for a in args {
                calls_to(scope, a, name, false, out);
            }
        }
        Expr::Not(x) => calls_to(scope, x, name, true, out),
        Expr::Binary(op, a, b) => {
            let t = matches!(op, BinOp::Iff | BinOp::Implies | BinOp::Or | BinOp::And);
            calls_to(scope, a, name, t, out);
            calls_to(scope, b, name, t, out);
        }
        Expr::Quantifier(q) => {
            calls_to(scope, &q.domain, name, false, out);
            let elem = scope.domain_elem(q).unwrap_or(Type::Unknown);
            let inner = match &q.binder.name {
                Some(n) => scope.bind(n, elem),
                None => scope.clone(),
            };
            for part in [&q.where_, &q.body, &q.else_].into_iter().flatten() {
                calls_to(&inner, part, name, true, out);
            }
        }
        Expr::Field(x, _, _)
        | Expr::Ok(x)
        | Expr::Err(x)
        | Expr::Unordered(x)
        | Expr::Unique(x)
        | Expr::SortedBy(x, _) => calls_to(scope, x, name, false, out),
        Expr::List(xs) | Expr::Tuple(xs) => {
            for x in xs {
                calls_to(scope, x, name, false, out);
            }
        }
        Expr::Record(fields, _) => {
            for (_, x) in fields {
                calls_to(scope, x, name, false, out);
            }
        }
        Expr::Relation(_, a, b) => {
            calls_to(scope, a, name, false, out);
            calls_to(scope, b, name, false, out);
        }
        Expr::Before { a, b, list } => {
            for x in [a, b, list] {
                calls_to(scope, x, name, false, out);
            }
        }
        Expr::Int(_) | Expr::Float(_) | Expr::Text(_) | Expr::None | Expr::Name(..) => {}
    }
}

fn with_params<'g>(scope: &Scope<'g>, graph: &Graph, idx: usize, a: &Abbrev) -> Scope<'g> {
    let mut s = scope.clone();
    for (n, t) in &a.params {
        s = s.bind(n, graph.expand(idx, t, a.line).unwrap_or(Type::Unknown));
    }
    s
}

/// Every call to `name` in the file: in abbreviations, properties and
/// accept lines.
fn uses_of(graph: &Graph, idx: usize, name: &str) -> Vec<Use> {
    let mut out = Vec::new();
    for d in &graph.file(idx).decls {
        match d {
            Decl::Abbrev(a) => {
                let scope = with_params(&Scope::top_level(graph, idx), graph, idx, a);
                calls_to(&scope, &a.body, name, false, &mut out);
            }
            Decl::Function(f) => {
                let Ok(scope) = Scope::for_function(graph, idx, f) else {
                    continue;
                };
                for a in &f.abbrevs {
                    let inner = with_params(&scope, graph, idx, a);
                    calls_to(&inner, &a.body, name, false, &mut out);
                }
                for p in f.properties.iter().chain(&f.accepts) {
                    calls_to(&scope, &p.expr, name, true, &mut out);
                }
            }
            _ => {}
        }
    }
    out
}

fn builtin(name: &str) -> Option<&'static str> {
    Some(match name {
        "sum" => "the sum of a list of numbers, of the list's element type",
        "len" => "the length of a list or a `Text`, an `Int`",
        "count" => "the number of elements in a list, an `Int`",
        "min" => "the least of two or more values of one ordered type",
        "max" => "the greatest of two or more values of one ordered type",
        "flatten" => "a list of lists as one list, in order",
        _ => return None,
    })
}

/// An obligation the graph holds and no file in this directory declares:
/// its interface read from its source with that file's siblings, or from
/// the pinned presentation when the source is gone; where it is pinned
/// from, and its state.
fn from_store(dir: &Path, name: &str) -> Option<String> {
    let store = surety_graph::Store::open(dir);
    let entry = store.namespace().ok()?.get(name)?.clone();
    let view = surety_graph::show(&store, name).ok()?;
    let graph = match load_with_siblings(&graph_base(dir).join(&entry.source)) {
        Ok((graph, _)) => graph,
        Err(_) => {
            let text = store
                .presentation(&view.hash, view.presentations.first()?)
                .ok()?;
            Graph::single(name, surety_syntax::parse(&text).ok()?)
        }
    };
    let (fidx, f) = graph.find_function(name)?;
    Some(format!(
        "{}\n\nin the graph, pinned from `{}` · state: **{}**",
        obligation_text(&graph, fidx, f),
        entry.source,
        view.state.word()
    ))
}

/// A call to a name nothing in the directory declares: a built-in, an
/// obligation only the graph holds, or a graph reference with no obligation
/// yet, typed as this file calls it.
fn describe_call(
    graph: &Graph,
    idx: usize,
    name: &str,
    cursor: usize,
    graph_dir: Option<&Path>,
) -> Option<String> {
    if BUILTINS.contains(&name) {
        return builtin(name).map(|what| format!("`{name}`: built-in, {what}."));
    }
    if let Some(s) = graph_dir.and_then(|dir| from_store(dir, name)) {
        return Some(s);
    }
    let uses = uses_of(graph, idx, name);
    let here = uses
        .iter()
        .find(|u| u.line == cursor)
        .or_else(|| uses.first())?;
    let args: Vec<String> = (0..here.args.len())
        .map(|i| {
            uses.iter()
                .filter(|u| u.args.len() == here.args.len())
                .map(|u| &u.args[i])
                .find(|t| !t.is_unknown())
                .unwrap_or(&here.args[i])
                .to_string()
        })
        .collect();
    let output = if uses.iter().any(|u| u.truth) {
        "Bool"
    } else {
        "?"
    };
    Some(format!(
        "{}\n\nA graph reference: no obligation in this directory declares `{name}`, so this is its shape as this file calls it. The performance supplies it, unchecked, until a sibling obligation declares it.",
        code(&format!("{name}({}) -> {output}", args.join(", ")))
    ))
}

pub fn hover(path: &Path, text: &str, pos: &Json, graph_dir: Option<&Path>) -> Json {
    let (line, character) = position(pos);
    let Ok((graph, idx)) = load_with_siblings_text(path, text) else {
        return Json::Null;
    };
    if let Some(word) = word_at(text, line, character) {
        if let Some((fidx, found)) = resolve(&graph, idx, &word) {
            return markdown(describe(&graph, path, idx, fidx, &found, graph_dir));
        }
        if called_at(text, line, character) {
            if let Some(value) = describe_call(&graph, idx, &word, line + 1, graph_dir) {
                return markdown(value);
            }
        }
    }
    // Not a name: a property line, or a continuation line of a vertical
    // body, shows the sentence above the property.
    let cursor = line + 1;
    for f in graph.functions_in(idx) {
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

// ---- completion ----

// LSP CompletionItemKind.
const TEXT: u32 = 1;
const FUNCTION: u32 = 3;
const FIELD: u32 = 5;
const VARIABLE: u32 = 6;
const CLASS: u32 = 7;
const KEYWORD: u32 = 14;
const CONSTANT: u32 = 21;
const STRUCT: u32 = 22;

/// What the relations and `unordered` claim, from the spec's fragment.
fn keyword_detail(word: &str) -> Option<&'static str> {
    Some(match word {
        "drawn_from" => "a sub-multiset of the right side",
        "same_order_as" => "drawn from the right side, in its order",
        "same_elements_as" => "the same elements as the right side, any order",
        "unordered" => "no claim about the order",
        "sorted_by" => "sorted by a lexicographic key",
        "unique" => "no element appears twice",
        "leftover" => "the domain's elements the output does not take",
        "before" => "an earlier position in the list",
        _ => return None,
    })
}

fn item(label: &str, kind: u32, detail: Option<String>, rank: u8) -> Json {
    let mut i = json!({ "label": label, "kind": kind, "sortText": format!("{rank}{label}") });
    if let Some(d) = detail {
        i["detail"] = json!(d);
    }
    i
}

/// The identifiers of the code in a text: sentences, comments and strings
/// left out, as prose is not a name.
fn code_words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with('#') || t.starts_with("//") {
            continue;
        }
        let code = t.split("//").next().unwrap_or_default();
        let mut quoted = false;
        let mut word = String::new();
        for c in code.chars().chain(std::iter::once(' ')) {
            if c == '"' {
                quoted = !quoted;
            }
            if !quoted && (c.is_alphanumeric() || c == '_') {
                word.push(c);
            } else {
                if word
                    .chars()
                    .next()
                    .is_some_and(|f| f.is_alphabetic() || f == '_')
                {
                    push_unique(&mut out, &word);
                }
                word.clear();
            }
        }
    }
    out
}

/// The dotted names before the word at a position, when a `.` precedes
/// it: `diff.changed.` gives `["diff", "changed"]`.
fn chain_before(text: &str, line: usize, character: usize) -> Option<Vec<String>> {
    let l = text.lines().nth(line)?;
    let chars: Vec<char> = l.chars().take(character).collect();
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut i = chars.len();
    while i > 0 && is_word(chars[i - 1]) {
        i -= 1;
    }
    let mut chain = Vec::new();
    while i > 0 && chars[i - 1] == '.' {
        i -= 1;
        let end = i;
        while i > 0 && is_word(chars[i - 1]) {
            i -= 1;
        }
        if i == end {
            return None;
        }
        chain.insert(0, chars[i..end].iter().collect());
    }
    (!chain.is_empty()).then_some(chain)
}

/// The graph with the file in it: from its text; when that does not parse
/// (a line half typed), from its text with the cursor's line blank; when
/// that does not either (the half-typed line heads a body), from the file
/// as last saved.
fn load_for_completion(path: &Path, text: &str, line: usize) -> Option<(Graph, usize)> {
    if let Ok(g) = load_with_siblings_text(path, text) {
        return Some(g);
    }
    let blanked: Vec<&str> = text
        .lines()
        .enumerate()
        .map(|(i, l)| if i == line { "" } else { l })
        .collect();
    load_with_siblings_text(path, &blanked.join("\n"))
        .or_else(|_| load_with_siblings(path))
        .ok()
}

/// The obligation whose block holds a 1-based line: the last signature at
/// or above it.
fn enclosing(graph: &Graph, idx: usize, line: usize) -> Option<&Function> {
    graph
        .functions_in(idx)
        .into_iter()
        .filter(|f| f.line <= line)
        .max_by_key(|f| f.line)
}

/// Lists and options are looked through: `orders.amount` is each order's.
fn element(t: Type) -> Type {
    match t {
        Type::List(t, _) | Type::Option(t) => element(*t),
        t => t,
    }
}

/// Where completion stands: the graph, the file, the obligation around
/// the cursor, and the text above the cursor in that obligation, where the
/// binders in scope were introduced.
struct At<'g> {
    graph: &'g Graph,
    idx: usize,
    f: Option<&'g Function>,
    above: Vec<String>,
}

/// Words and single punctuation marks: enough to read a quantifier head.
fn tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in line.chars() {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        if !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
        if !c.is_whitespace() {
            out.push(c.to_string());
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// The nearest quantifier head binding `name` above the cursor: its
/// pattern side (`Ok`, `Err`) if any, and the dotted chain it ranges over.
fn binder_head(above: &[String], name: &str) -> Option<(Option<String>, Vec<String>)> {
    for line in above.iter().rev() {
        let t = tokens(line);
        for k in (0..t.len()).rev() {
            if !matches!(t[k].as_str(), "all" | "any" | "none" | "exactly_one") {
                continue;
            }
            let (side, mut j) = match t.get(k + 1).map(String::as_str) {
                Some(s @ ("Ok" | "Err")) if t.get(k + 2).is_some_and(|p| p == "(") => {
                    if t.get(k + 3).map(String::as_str) != Some(name) {
                        continue;
                    }
                    (Some(s.to_string()), k + 5)
                }
                Some(n) if n == name => (None, k + 2),
                _ => continue,
            };
            if t.get(j).is_some_and(|w| w == "leftover") {
                j += 1;
            }
            if t.get(j).is_none_or(|w| w != "in") {
                continue;
            }
            let mut chain = vec![t.get(j + 1)?.clone()];
            j += 2;
            while t.get(j).is_some_and(|w| w == ".") {
                chain.push(t.get(j + 1)?.clone());
                j += 2;
            }
            return Some((side, chain));
        }
    }
    None
}

impl At<'_> {
    /// The type a dotted chain names: from an input, `out`, a named value or
    /// a binder, each step a field, looking through lists and options.
    fn chain_type(&self, chain: &[String], depth: usize) -> Option<Type> {
        let root = &chain[0];
        let line = self.f.map_or(0, |f| f.line);
        let declared = match self.f {
            Some(f) if root == "out" => Some(f.output.clone()),
            Some(f) => f
                .inputs
                .iter()
                .find(|(n, _)| n == root)
                .map(|(_, t)| t.clone()),
            None => None,
        };
        let mut ty = match declared {
            Some(t) => self.graph.expand(self.idx, &t, line).ok()?,
            None => match self.graph.named_value(self.idx, root) {
                Some(nv) => self.graph.named_value_type(self.idx, nv).ok()?,
                None if depth < 8 => self.binder_type(root, depth + 1)?,
                None => return None,
            },
        };
        for step in &chain[1..] {
            let fields = element(ty).record_fields()?.to_vec();
            ty = fields.into_iter().find(|(n, _)| n == step)?.1;
        }
        Some(ty)
    }

    /// A binder's type: an element of its domain, or the payload of the
    /// side its pattern selects.
    fn binder_type(&self, name: &str, depth: usize) -> Option<Type> {
        let (side, chain) = binder_head(&self.above, name)?;
        let domain = self.chain_type(&chain, depth)?;
        let each = match domain {
            Type::List(t, _) => *t,
            t => t,
        };
        match (side.as_deref(), each) {
            (None, t) => Some(t),
            (Some("Ok"), Type::Result(ok, _)) => Some(*ok),
            (Some("Err"), Type::Result(_, err)) => Some(*err),
            _ => None,
        }
    }

    /// The fields after a dotted chain, when its type is worked out here:
    /// none for a type that is not a record, since nothing can follow its
    /// dot; None when the type is not known.
    fn fields_after(&self, chain: &[String]) -> Option<Vec<(String, Type)>> {
        let t = element(self.chain_type(chain, 0)?);
        if t.is_unknown() {
            return None;
        }
        Some(t.record_fields().map(|fs| fs.to_vec()).unwrap_or_default())
    }
}

/// Every field of every record type in the graph: what a binder's `.` can
/// reach, when the binder's type is not worked out here.
fn all_fields(graph: &Graph) -> Vec<(String, Type)> {
    let mut out: Vec<(String, Type)> = Vec::new();
    for (i, gf) in graph.files.iter().enumerate() {
        for d in &gf.file.decls {
            if let Decl::Type(t) = d {
                if let Ok(Type::Record { fields, .. }) = graph.expand(i, &t.ty, t.line) {
                    for (n, ft) in fields {
                        if !out.iter().any(|(m, _)| *m == n) {
                            out.push((n, ft));
                        }
                    }
                }
            }
        }
    }
    out
}

/// Completion: after a `.`, the fields of what precedes it; anywhere else,
/// the enclosing obligation's inputs, `out` and abbreviations, every name
/// the directory declares, the keywords, built-ins and primitive types, and
/// the identifiers already in the file's code. The editor filters by what
/// has been typed.
pub fn completion(path: &Path, text: &str, pos: &Json) -> Json {
    let (line, character) = position(pos);
    let loaded = load_for_completion(path, text, line);
    let graph = loaded.as_ref().map(|(g, i)| (g, *i));
    let f = graph.and_then(|(g, i)| enclosing(g, i, line + 1));
    let typing = word_at(text, line, character).unwrap_or_default();
    let mut items: Vec<Json> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut add = |label: &str, kind: u32, detail: Option<String>, rank: u8| {
        if label != typing && !seen.iter().any(|s| s == label) {
            seen.push(label.to_string());
            items.push(item(label, kind, detail, rank));
        }
    };
    if let Some(chain) = chain_before(text, line, character) {
        if let Some((g, i)) = graph {
            // The obligation's lines down to the cursor: where its binders
            // were introduced.
            let start = f.map_or(0, |f| f.line.saturating_sub(1)).min(line);
            let mut above: Vec<String> = text
                .lines()
                .skip(start)
                .take(line + 1 - start)
                .map(String::from)
                .collect();
            if let Some(last) = above.last_mut() {
                *last = last.chars().take(character).collect();
            }
            let at = At {
                graph: g,
                idx: i,
                f,
                above,
            };
            let fields = at.fields_after(&chain).unwrap_or_else(|| all_fields(g));
            for (n, t) in fields {
                add(&n, FIELD, Some(t.to_string()), 0);
            }
        }
        return Json::Array(items);
    }
    if let Some(f) = f {
        for (n, t) in &f.inputs {
            add(n, VARIABLE, Some(format!("{n}: {t}")), 0);
        }
        add("out", VARIABLE, Some(format!("out: {}", f.output)), 0);
        for a in &f.abbrevs {
            add(&a.name, FUNCTION, Some(render::abbrev(a)), 0);
        }
    }
    if let Some((g, i)) = graph {
        let order = std::iter::once(i).chain((0..g.files.len()).filter(|&j| j != i));
        for j in order {
            for d in &g.files[j].file.decls {
                match d {
                    Decl::Type(t) => add(
                        &t.name,
                        STRUCT,
                        Some(format!("type {} = {}", t.name, t.ty)),
                        1,
                    ),
                    Decl::NamedValue(nv) => add(&nv.name, CONSTANT, Some(value_text(nv)), 1),
                    Decl::Abbrev(a) => add(&a.name, FUNCTION, Some(render::abbrev(a)), 1),
                    Decl::Function(f) => add(&f.name, FUNCTION, Some(signature(f)), 1),
                }
            }
        }
    }
    for w in RESERVED {
        add(w, KEYWORD, keyword_detail(w).map(String::from), 2);
    }
    for w in ["note", "out"] {
        add(w, KEYWORD, None, 2);
    }
    for b in BUILTINS {
        add(b, FUNCTION, builtin(b).map(|d| format!("built-in: {d}")), 2);
    }
    for p in PRIMITIVES {
        add(p, CLASS, None, 2);
    }
    for w in code_words(text) {
        add(&w, TEXT, None, 3);
    }
    Json::Array(items)
}

