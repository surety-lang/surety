//! The queries: a node with its state, its prose, its performances and
//! one hop; and the namespace with every name's state.

use std::path::Path;

use surety_syntax::load::load_with_siblings;
use surety_syntax::normalize::canonical_file;
use surety_syntax::{Decl, PinKind};

use crate::record::{TARGET, strict_passed};
use crate::store::{Namespace, Store};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// A node with no performance: what the model is handed.
    Open,
    /// A record for the current hash whose strict check passed.
    Verified,
    /// Records for the current hash, none strict-passing.
    Findings,
    /// The source now hashes to something the namespace does not point at.
    Stale { now: String },
    /// The source path is gone.
    Missing,
    /// A node no name points at.
    Unnamed,
}

impl State {
    pub fn word(&self) -> String {
        match self {
            State::Open => "open".into(),
            State::Verified => "verified".into(),
            State::Findings => "findings".into(),
            State::Stale { now } => format!("stale (now {})", now.get(..12).unwrap_or(now)),
            State::Missing => "missing".into(),
            State::Unnamed => "unnamed".into(),
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            State::Open => "open",
            State::Verified => "verified",
            State::Findings => "findings",
            State::Stale { .. } => "stale",
            State::Missing => "missing",
            State::Unnamed => "unnamed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceView {
    pub target: String,
    pub hash: String,
    pub verified: bool,
    pub verdict: String,
}

#[derive(Debug, Clone)]
pub struct NodeView {
    pub hash: String,
    pub names: Vec<String>,
    pub state: State,
    pub source: Option<String>,
    pub note: Option<String>,
    pub sentences: Vec<String>,
    pub comments: Vec<String>,
    pub presentations: Vec<String>,
    pub performances: Vec<PerformanceView>,
    pub callees: Vec<(String, State)>,
    pub callers: Vec<(String, State)>,
    /// The obligations whose files define the types this one uses.
    pub uses: Vec<(String, State)>,
}

#[derive(Debug, Clone)]
pub struct StatusRow {
    pub name: String,
    pub hash: String,
    pub source: String,
    pub state: State,
}

fn source_path(store: &Store, source: &str) -> std::path::PathBuf {
    let base = store
        .root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    base.join(source)
}

/// What the records under a node say: verified, findings, or open.
fn record_state(store: &Store, hash: &str) -> Result<State, String> {
    let records = store.records(hash, TARGET)?;
    if records.is_empty() {
        return Ok(State::Open);
    }
    if records.iter().any(|(_, r)| strict_passed(&r["evidence"])) {
        Ok(State::Verified)
    } else {
        Ok(State::Findings)
    }
}

/// A name's state: stale or missing when its source moved on, else what
/// its node's records say.
pub fn state_of_name(store: &Store, ns: &Namespace, name: &str) -> Result<State, String> {
    let entry = ns
        .get(name)
        .ok_or_else(|| format!("`{name}` is not in the namespace"))?;
    let src = source_path(store, &entry.source);
    if !src.is_file() {
        return Ok(State::Missing);
    }
    if let Ok((graph, idx)) = load_with_siblings(&src) {
        if let Ok(c) = canonical_file(&graph, idx) {
            if c.hash != entry.obligation {
                return Ok(State::Stale { now: c.hash });
            }
        }
    }
    record_state(store, &entry.obligation)
}

pub fn status(store: &Store) -> Result<Vec<StatusRow>, String> {
    let ns = store.namespace()?;
    let mut rows = Vec::new();
    for (name, entry) in &ns {
        rows.push(StatusRow {
            name: name.clone(),
            hash: entry.obligation.clone(),
            source: entry.source.clone(),
            state: state_of_name(store, &ns, name)?,
        });
    }
    Ok(rows)
}

/// A node by name or by a prefix of its hash.
pub fn show(store: &Store, key: &str) -> Result<NodeView, String> {
    let ns = store.namespace()?;
    let hash = match ns.get(key) {
        Some(e) => e.obligation.clone(),
        None => store
            .find_node(key)?
            .ok_or_else(|| format!("no name or node `{key}` in the graph"))?,
    };
    let names: Vec<String> = ns
        .iter()
        .filter(|(_, e)| e.obligation == hash)
        .map(|(n, _)| n.clone())
        .collect();
    let state = match names.first() {
        Some(n) => state_of_name(store, &ns, n)?,
        None => State::Unnamed,
    };
    let source = names
        .first()
        .and_then(|n| ns.get(n))
        .map(|e| e.source.clone());
    let presentations = store.presentations(&hash)?;
    let (mut note, mut sentences, mut comments) = (None, Vec::new(), Vec::new());
    if let Some(first) = presentations.first() {
        if let Ok(file) = surety_syntax::parse(&store.presentation(&hash, first)?) {
            for d in &file.decls {
                if let Decl::Function(f) = d {
                    note = f.note.clone();
                }
            }
            for p in &file.pins {
                match p.kind {
                    PinKind::Sentence => sentences.push(p.text.clone()),
                    PinKind::Comment => comments.push(p.text.clone()),
                    PinKind::Note => {}
                }
            }
        }
    }
    let performances = store
        .records(&hash, TARGET)?
        .into_iter()
        .map(|(perf, r)| PerformanceView {
            target: TARGET.to_string(),
            hash: perf,
            verified: strict_passed(&r["evidence"]),
            verdict: r["evidence"]["verdict"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
        .collect();
    let refs = store.references(&hash)?;
    let hop = |list: &[String]| -> Result<Vec<(String, State)>, String> {
        let mut out = Vec::new();
        for r in list {
            let st = if ns.contains_key(r) {
                state_of_name(store, &ns, r)?
            } else {
                State::Missing
            };
            out.push((r.clone(), st));
        }
        Ok(out)
    };
    let callees = hop(&refs.calls)?;
    let uses = hop(&refs.types)?;
    let mut callers = Vec::new();
    for (n, e) in &ns {
        let theirs = store.references(&e.obligation).unwrap_or_default();
        if names.iter().any(|own| theirs.calls.contains(own)) {
            callers.push((n.clone(), state_of_name(store, &ns, n)?));
        }
    }
    Ok(NodeView {
        hash,
        names,
        state,
        source,
        note,
        sentences,
        comments,
        presentations,
        performances,
        callees,
        callers,
        uses,
    })
}
