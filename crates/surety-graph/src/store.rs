//! The layout on disk: a visible committed directory in which content is
//! named by its hash, so two writers never conflict on it, and the
//! namespace is the only thing that moves.
//!
//! ```text
//! graph/
//!   namespace.json                                  name → obligation hash, source path at pin time
//!   nodes/<obligation hash>/
//!     canonical.sur                                 the canonical form: what the hash is computed over
//!     presentations/<text hash>.sur                 the file as the author wrote it, pins included
//!     references.json                               callees by name
//!     performances/<target>/<performance hash>.py   the performance as pinned
//!     evidence/<target>/<performance hash>.json     the record
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value as Json, json};
use sha2::{Digest, Sha256};

pub fn bytes_hash(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}

pub fn text_hash(s: &str) -> String {
    bytes_hash(s.as_bytes())
}

/// What the namespace says about a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub obligation: String,
    /// The source path at pin time, relative to the graph's parent.
    pub source: String,
}

pub type Namespace = BTreeMap<String, Entry>;

/// A node's edges, by obligation name: the obligations its clauses call,
/// and the obligations whose files define the types it uses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct References {
    pub calls: Vec<String>,
    pub types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub root: PathBuf,
}

fn read(p: &Path) -> Result<String, String> {
    fs::read_to_string(p).map_err(|e| format!("cannot read {}: {e}", p.display()))
}

/// Write only when the bytes differ, so an unchanged pin leaves no trace.
pub fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool, String> {
    if let Ok(old) = fs::read(path) {
        if old == bytes {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(true)
}

fn pretty(j: &Json) -> String {
    serde_json::to_string_pretty(j).unwrap_or_default() + "\n"
}

/// A record's text: pretty-printed, except the run log, which goes one run
/// per line so the file stays readable and a diff of it stays line-shaped.
fn record_text(record: &Json) -> String {
    let mut r = record.clone();
    let runs = match r["evidence"].get_mut("runs") {
        Some(runs) => runs.take(),
        None => return pretty(&r),
    };
    let text = pretty(&r);
    let block = match runs.as_array() {
        Some(xs) if !xs.is_empty() => {
            let lines: Vec<String> = xs.iter().map(|x| format!("      {x}")).collect();
            format!("[\n{}\n    ]", lines.join(",\n"))
        }
        _ => "[]".to_string(),
    };
    text.replacen("\"runs\": null", &format!("\"runs\": {block}"), 1)
}

fn sorted_stems(dir: &Path, ext: &str) -> Result<Vec<String>, String> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for e in fs::read_dir(dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|x| x == ext) {
            out.push(surety_syntax::load::stem(&p));
        }
    }
    out.sort();
    Ok(out)
}

fn extension(target: &str) -> &'static str {
    match target {
        "python" => "py",
        _ => "txt",
    }
}

impl Store {
    pub fn open(root: &Path) -> Store {
        Store {
            root: root.to_path_buf(),
        }
    }

    pub fn exists(&self) -> bool {
        self.namespace_path().is_file()
    }

    pub fn namespace_path(&self) -> PathBuf {
        self.root.join("namespace.json")
    }

    pub fn namespace(&self) -> Result<Namespace, String> {
        let p = self.namespace_path();
        if !p.is_file() {
            return Ok(Namespace::new());
        }
        let j: Json = serde_json::from_str(&read(&p)?)
            .map_err(|e| format!("{}: not JSON: {e}", p.display()))?;
        let map = j
            .as_object()
            .ok_or_else(|| format!("{}: not an object", p.display()))?;
        let mut ns = Namespace::new();
        for (name, v) in map {
            ns.insert(
                name.clone(),
                Entry {
                    obligation: v["obligation"].as_str().unwrap_or_default().to_string(),
                    source: v["source"].as_str().unwrap_or_default().to_string(),
                },
            );
        }
        Ok(ns)
    }

    pub fn write_namespace(&self, ns: &Namespace) -> Result<bool, String> {
        let mut map = Map::new();
        for (name, e) in ns {
            map.insert(
                name.clone(),
                json!({ "obligation": e.obligation, "source": e.source }),
            );
        }
        write_if_changed(
            &self.namespace_path(),
            pretty(&Json::Object(map)).as_bytes(),
        )
    }

    pub fn node_dir(&self, hash: &str) -> PathBuf {
        self.root.join("nodes").join(hash)
    }

    /// Every node, by hash, sorted.
    pub fn nodes(&self) -> Result<Vec<String>, String> {
        let dir = self.root.join("nodes");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for e in fs::read_dir(&dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))? {
            let p = e.map_err(|e| e.to_string())?.path();
            if p.join("canonical.sur").is_file() {
                out.push(surety_syntax::load::stem(&p));
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn has_node(&self, hash: &str) -> bool {
        self.node_dir(hash).join("canonical.sur").is_file()
    }

    /// A node by a prefix of its hash; an error when the prefix is ambiguous.
    pub fn find_node(&self, prefix: &str) -> Result<Option<String>, String> {
        let hits: Vec<String> = self
            .nodes()?
            .into_iter()
            .filter(|h| h.starts_with(prefix))
            .collect();
        match hits.len() {
            0 => Ok(None),
            1 => Ok(hits.into_iter().next()),
            n => Err(format!("`{prefix}` names {n} nodes; give more of the hash")),
        }
    }

    pub fn canonical(&self, hash: &str) -> Result<String, String> {
        read(&self.node_dir(hash).join("canonical.sur"))
    }

    pub fn write_node(
        &self,
        hash: &str,
        canonical: &str,
        presentation: &str,
        references: &References,
    ) -> Result<(), String> {
        let dir = self.node_dir(hash);
        write_if_changed(&dir.join("canonical.sur"), canonical.as_bytes())?;
        write_if_changed(
            &dir.join("presentations")
                .join(format!("{}.sur", text_hash(presentation))),
            presentation.as_bytes(),
        )?;
        write_if_changed(
            &dir.join("references.json"),
            pretty(&json!({ "calls": references.calls, "types": references.types })).as_bytes(),
        )?;
        Ok(())
    }

    /// The presentations of a node, by text hash, sorted.
    pub fn presentations(&self, hash: &str) -> Result<Vec<String>, String> {
        sorted_stems(&self.node_dir(hash).join("presentations"), "sur")
    }

    pub fn presentation(&self, hash: &str, text: &str) -> Result<String, String> {
        read(
            &self
                .node_dir(hash)
                .join("presentations")
                .join(format!("{text}.sur")),
        )
    }

    pub fn references(&self, hash: &str) -> Result<References, String> {
        let p = self.node_dir(hash).join("references.json");
        if !p.is_file() {
            return Ok(References::default());
        }
        let j: Json = serde_json::from_str(&read(&p)?)
            .map_err(|e| format!("{}: not JSON: {e}", p.display()))?;
        let names = |v: &Json| -> Vec<String> {
            v.as_array()
                .map(|xs| {
                    xs.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        Ok(References {
            calls: names(&j["calls"]),
            types: names(&j["types"]),
        })
    }

    pub fn performance_path(&self, hash: &str, target: &str, perf: &str) -> PathBuf {
        self.node_dir(hash)
            .join("performances")
            .join(target)
            .join(format!("{perf}.{}", extension(target)))
    }

    pub fn performances(&self, hash: &str, target: &str) -> Result<Vec<String>, String> {
        sorted_stems(
            &self.node_dir(hash).join("performances").join(target),
            extension(target),
        )
    }

    pub fn write_performance(
        &self,
        hash: &str,
        target: &str,
        perf: &str,
        bytes: &[u8],
    ) -> Result<bool, String> {
        write_if_changed(&self.performance_path(hash, target, perf), bytes)
    }

    pub fn record_path(&self, hash: &str, target: &str, perf: &str) -> PathBuf {
        self.node_dir(hash)
            .join("evidence")
            .join(target)
            .join(format!("{perf}.json"))
    }

    /// Every record of a node for a target, by performance hash, sorted.
    pub fn records(&self, hash: &str, target: &str) -> Result<Vec<(String, Json)>, String> {
        let mut out = Vec::new();
        for perf in sorted_stems(&self.node_dir(hash).join("evidence").join(target), "json")? {
            if let Some(r) = self.record(hash, target, &perf)? {
                out.push((perf, r));
            }
        }
        Ok(out)
    }

    pub fn record(&self, hash: &str, target: &str, perf: &str) -> Result<Option<Json>, String> {
        let p = self.record_path(hash, target, perf);
        if !p.is_file() {
            return Ok(None);
        }
        serde_json::from_str(&read(&p)?)
            .map(Some)
            .map_err(|e| format!("{}: not JSON: {e}", p.display()))
    }

    pub fn write_record(
        &self,
        hash: &str,
        target: &str,
        perf: &str,
        record: &Json,
    ) -> Result<bool, String> {
        write_if_changed(
            &self.record_path(hash, target, perf),
            record_text(record).as_bytes(),
        )
    }
}
