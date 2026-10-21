//! Pinning and verifying: a check whose record the kernel admits becomes a
//! node, a performance copy and a record in the graph; verify runs the
//! kernel over every record and, on request, the check again.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value as Json, json};
use surety_check::{Budget, Checker, Report};
use surety_syntax::load::load_with_siblings;
use surety_syntax::normalize::canonical_file;
use surety_syntax::{Decl, Expr, Function, Graph, Type};

use crate::record::{self, Render, TARGET, admit, budget_of, strict_passed, timeout_of};
use crate::store::{Entry, References, Store, text_hash};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinState {
    Open,
    Verified,
    Findings,
}

impl PinState {
    pub fn word(self) -> &'static str {
        match self {
            PinState::Open => "open",
            PinState::Verified => "verified",
            PinState::Findings => "findings",
        }
    }
}

#[derive(Debug)]
pub struct Pinned {
    pub name: String,
    pub obligation: String,
    pub performance: Option<String>,
    pub state: PinState,
    pub report: Option<Report>,
    /// Whether the record was new or changed; false when the pin left the
    /// graph as it was.
    pub wrote: bool,
}

fn errors_text(errs: Vec<(PathBuf, surety_syntax::Error)>) -> String {
    errs.iter()
        .map(|(p, e)| format!("{}: {e}", p.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A path relative to the graph's parent, which is what the namespace and
/// the provenance record; absolute paths never enter the graph.
pub fn relative(store: &Store, path: &Path) -> String {
    let base = store
        .root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    match (fs::canonicalize(base), fs::canonicalize(path)) {
        (Ok(b), Ok(p)) => relative_to(&b, &p),
        _ => path.display().to_string(),
    }
}

/// `path` relative to `base`, with `..` where `path` is not under it.
fn relative_to(base: &Path, path: &Path) -> String {
    let b: Vec<_> = base.components().collect();
    let p: Vec<_> = path.components().collect();
    let common = b.iter().zip(&p).take_while(|(x, y)| x == y).count();
    let mut out = PathBuf::new();
    for _ in common..b.len() {
        out.push("..");
    }
    for c in &p[common..] {
        out.push(c);
    }
    out.display().to_string()
}

/// The obligations this function's clauses call, in other files of the
/// graph: the node's edges. Abbreviations and module-local references
/// (a name no obligation defines) are not edges.
pub fn references_of(graph: &Graph, idx: usize, fun: &Function) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut visit = |e: &Expr| {
        e.walk(&mut |x| {
            if let Expr::Call(n, _, _) = x {
                if n != &fun.name && !names.contains(n) {
                    if let Some((fidx, _)) = graph.find_function(n) {
                        if fidx != idx {
                            names.push(n.clone());
                        }
                    }
                }
            }
        });
    };
    for p in fun.properties.iter().chain(&fun.accepts) {
        visit(&p.expr);
    }
    for a in &fun.abbrevs {
        visit(&a.body);
    }
    for d in &graph.file(idx).decls {
        if let Decl::Abbrev(a) = d {
            visit(&a.body);
        }
    }
    names.sort();
    names
}

fn named_types(t: &Type, out: &mut Vec<String>) {
    match t {
        Type::Named(n) => out.push(n.clone()),
        Type::Record { spreads, fields } => {
            out.extend(spreads.iter().cloned());
            for (_, t) in fields {
                named_types(t, out);
            }
        }
        Type::List(t, _) | Type::Option(t) => named_types(t, out),
        Type::Result(a, b) => {
            named_types(a, out);
            named_types(b, out);
        }
        _ => {}
    }
}

/// The obligations whose files define the types this file uses, directly
/// or through those files' own types, in name order. A file that defines
/// a type and no obligation cannot be a node, so it is an error here.
pub fn type_references_of(graph: &Graph, idx: usize) -> Result<Vec<String>, String> {
    let mut seen = vec![idx];
    let mut queue = vec![idx];
    let mut names = Vec::new();
    while let Some(fidx) = queue.pop() {
        let file = graph.file(fidx);
        let mut used = Vec::new();
        for d in &file.decls {
            match d {
                Decl::Type(t) => named_types(&t.ty, &mut used),
                Decl::NamedValue(nv) => {
                    if let Some(t) = &nv.ty {
                        named_types(t, &mut used);
                    }
                }
                Decl::Abbrev(a) => {
                    for (_, t) in &a.params {
                        named_types(t, &mut used);
                    }
                }
                Decl::Function(f) => {
                    for (_, t) in &f.inputs {
                        named_types(t, &mut used);
                    }
                    named_types(&f.output, &mut used);
                    for a in &f.abbrevs {
                        for (_, t) in &a.params {
                            named_types(t, &mut used);
                        }
                    }
                }
            }
        }
        for n in used {
            if let Ok((tidx, _)) = graph.find_type(fidx, &n, 0) {
                if !seen.contains(&tidx) {
                    seen.push(tidx);
                    queue.push(tidx);
                    let provider = graph
                        .functions_in(tidx)
                        .first()
                        .map(|f| f.name.clone())
                        .ok_or_else(|| {
                            format!(
                                "type `{n}` is defined in {}, which holds no obligation and so cannot be a node",
                                graph.files[tidx].name
                            )
                        })?;
                    names.push(provider);
                }
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Check and pin. With no performance given and no `<file>.py` beside the
/// file, the obligation is pinned alone and the node is open.
pub fn pin(
    store: &Store,
    file: &Path,
    perf: Option<&Path>,
    budget: Budget,
    timeout: Duration,
    render: Render,
) -> Result<Pinned, String> {
    let (graph, idx) = load_with_siblings(file).map_err(errors_text)?;
    graph
        .check_file(idx)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    let funs = graph.functions_in(idx);
    if funs.len() != 1 {
        return Err(format!(
            "{}: one definition per file; this one has {}",
            file.display(),
            funs.len()
        ));
    }
    let fun = funs[0];
    let name = fun.name.clone();
    let canonical = canonical_file(&graph, idx).map_err(|e| format!("{}: {e}", file.display()))?;
    let hash = canonical.hash.clone();
    let presentation =
        fs::read_to_string(file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    // A drafted obligation carries `// draft` on its first line until its
    // author removes it; the mark is outside the hash and keeps the gate human.
    if presentation
        .lines()
        .next()
        .is_some_and(|l| l.trim() == "// draft")
    {
        return Err(format!(
            "{}: carries the draft mark `// draft` on its first line; review it and remove the mark to pin",
            file.display()
        ));
    }
    let references = References {
        calls: references_of(&graph, idx, fun),
        types: type_references_of(&graph, idx)?,
    };
    let module_dir = file.parent().unwrap_or(Path::new("."));
    let perf_path = perf.map(Path::to_path_buf).or_else(|| {
        let p = file.with_extension("py");
        p.is_file().then_some(p)
    });
    let mut ns = store.namespace()?;
    let source = relative(store, file);
    store.write_node(&hash, &canonical.text, &presentation, &references)?;

    let Some(perf_path) = perf_path else {
        ns.insert(
            name.clone(),
            Entry {
                obligation: hash.clone(),
                source,
            },
        );
        let wrote = store.write_namespace(&ns)?;
        return Ok(Pinned {
            name,
            obligation: hash,
            performance: None,
            state: PinState::Open,
            report: None,
            wrote,
        });
    };

    // The siblings the check sees must be pinned under their own nodes, so
    // verify can materialize exactly them: the files whose types this one
    // uses, and the performances its clauses call.
    let mut siblings = Map::new();
    for t in &references.types {
        let (tidx, _) = graph
            .find_function(t)
            .ok_or_else(|| format!("`{name}` uses types from `{t}`, which is not in the graph"))?;
        let t_hash = canonical_file(&graph, tidx)
            .map_err(|e| e.to_string())?
            .hash;
        match ns.get(t) {
            Some(e) if e.obligation == t_hash => {}
            Some(_) => {
                return Err(format!(
                    "pin `{t}` first: `{name}` uses its types and its file changed since it was pinned"
                ));
            }
            None => {
                return Err(format!(
                    "pin `{t}` first: `{name}` uses its types and it is not in the namespace"
                ));
            }
        }
        siblings.insert(t.clone(), json!({ "obligation": t_hash }));
    }
    for r in &references.calls {
        let (sidx, _) = graph
            .find_function(r)
            .ok_or_else(|| format!("`{name}` refers to `{r}`, which is not in the graph"))?;
        let s_hash = canonical_file(&graph, sidx)
            .map_err(|e| e.to_string())?
            .hash;
        let entry = ns.get(r).ok_or_else(|| {
            format!("pin `{r}` first: `{name}` refers to it and it is not in the namespace")
        })?;
        if entry.obligation != s_hash {
            return Err(format!(
                "pin `{r}` first: its file changed since it was pinned"
            ));
        }
        let sp = module_dir.join(format!("{}.py", graph.files[sidx].name));
        if !sp.is_file() {
            // A module-local reference: the performance supplies it itself.
            continue;
        }
        let s_perf = surety_py::inspect(&sp)
            .map_err(|e| format!("cannot hash {}: {e}", sp.display()))?
            .hash;
        if !store.performance_path(&s_hash, TARGET, &s_perf).is_file() {
            return Err(format!(
                "pin `{r}` with {} first: `{name}` runs against it",
                sp.display()
            ));
        }
        siblings.insert(
            r.clone(),
            json!({ "obligation": s_hash, "performance": s_perf }),
        );
        // A caller's type providers are the callee's too when they coincide;
        // the callee's own record carries the rest.
    }

    let mut checker = Checker::new(&graph, idx, &perf_path, module_dir, timeout)?;
    checker.budget = budget.clone();
    let report = checker.check();
    let perf_hash = surety_py::inspect(&perf_path)
        .map_err(|e| format!("cannot hash {}: {e}", perf_path.display()))?
        .hash;
    let rendered = render(&report);
    let evidence = record::evidence(&checker, &report, &budget, timeout, &perf_hash, &rendered);
    admit(&evidence, &hash, &canonical.text, &perf_path, &mut checker)?;

    let bytes =
        fs::read(&perf_path).map_err(|e| format!("cannot read {}: {e}", perf_path.display()))?;
    store.write_performance(&hash, TARGET, &perf_hash, &bytes)?;
    let wrote = match store.record(&hash, TARGET, &perf_hash)? {
        Some(old) if old["evidence"] == evidence => false,
        _ => {
            let provenance = json!({
                "checker": env!("CARGO_PKG_VERSION"),
                "python": surety_py::python_version().unwrap_or_else(|_| "unknown".to_string()),
                "source": source,
                "performance_source": relative(store, &perf_path),
                "presentation": text_hash(&presentation),
                "siblings": Json::Object(siblings),
            });
            store.write_record(
                &hash,
                TARGET,
                &perf_hash,
                &json!({ "provenance": provenance, "evidence": evidence }),
            )?
        }
    };
    ns.insert(
        name.clone(),
        Entry {
            obligation: hash.clone(),
            source,
        },
    );
    let ns_wrote = store.write_namespace(&ns)?;
    let state = if strict_passed(&evidence) {
        PinState::Verified
    } else {
        PinState::Findings
    };
    Ok(Pinned {
        name,
        obligation: hash,
        performance: Some(perf_hash),
        state,
        report: Some(report),
        wrote: wrote || ns_wrote,
    })
}

/// The name of the one obligation in a presentation.
fn function_name(text: &str) -> Result<String, String> {
    let file = surety_syntax::parse(text).map_err(|e| e.to_string())?;
    file.decls
        .iter()
        .find_map(|d| match d {
            Decl::Function(f) => Some(f.name.clone()),
            _ => None,
        })
        .ok_or_else(|| "the presentation holds no obligation".to_string())
}

/// Write a node's presentation and performance into `dir` as `<name>.sur`
/// and `<name>.py`, and the siblings its record names, recursively, so the
/// check sees exactly what the pin saw.
fn materialize(
    store: &Store,
    node: &str,
    perf: &str,
    provenance: &Json,
    dir: &Path,
    seen: &mut HashSet<String>,
) -> Result<(String, PathBuf), String> {
    let presentations = store.presentations(node)?;
    let wanted = provenance["presentation"].as_str().unwrap_or_default();
    let text_hash = if presentations.iter().any(|p| p == wanted) {
        wanted.to_string()
    } else {
        presentations
            .first()
            .cloned()
            .ok_or_else(|| format!("node {node} has no presentation"))?
    };
    let text = store.presentation(node, &text_hash)?;
    let name = function_name(&text)?;
    fs::write(dir.join(format!("{name}.sur")), &text).map_err(|e| e.to_string())?;
    let perf_src = store.performance_path(node, TARGET, perf);
    let perf_dst = dir.join(format!("{name}.py"));
    fs::copy(&perf_src, &perf_dst)
        .map_err(|e| format!("cannot copy {}: {e}", perf_src.display()))?;
    if let Some(sibs) = provenance["siblings"].as_object() {
        for (sib, info) in sibs {
            if !seen.insert(sib.clone()) {
                continue;
            }
            let s_node = info["obligation"].as_str().unwrap_or_default();
            let Some(s_perf) = info["performance"].as_str() else {
                // A type provider: its presentation is all the check needs.
                let pres = store
                    .presentations(s_node)?
                    .first()
                    .cloned()
                    .ok_or_else(|| format!("node {s_node} has no presentation"))?;
                let t = store.presentation(s_node, &pres)?;
                fs::write(dir.join(format!("{sib}.sur")), &t).map_err(|e| e.to_string())?;
                continue;
            };
            match store.record(s_node, TARGET, s_perf)? {
                Some(r) => {
                    materialize(store, s_node, s_perf, &r["provenance"], dir, seen)?;
                }
                None => {
                    let pres = store
                        .presentations(s_node)?
                        .first()
                        .cloned()
                        .ok_or_else(|| format!("node {s_node} has no presentation"))?;
                    let t = store.presentation(s_node, &pres)?;
                    fs::write(dir.join(format!("{sib}.sur")), &t).map_err(|e| e.to_string())?;
                    let src = store.performance_path(s_node, TARGET, s_perf);
                    fs::copy(&src, dir.join(format!("{sib}.py")))
                        .map_err(|e| format!("cannot copy {}: {e}", src.display()))?;
                }
            }
        }
    }
    Ok((name, perf_dst))
}

pub struct Verification {
    pub node: String,
    pub name: String,
    pub performance: String,
    pub result: Result<(), String>,
}

fn differing_keys(a: &Json, b: &Json) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    if let (Some(x), Some(y)) = (a.as_object(), b.as_object()) {
        for k in x.keys().chain(y.keys()) {
            if x.get(k) != y.get(k) && !keys.contains(k) {
                keys.push(k.clone());
            }
        }
    }
    keys
}

fn verify_one(
    store: &Store,
    node: &str,
    perf: &str,
    record: &Json,
    rerun: bool,
    render: Render,
) -> Result<(), String> {
    let evidence = &record["evidence"];
    let dir = std::env::temp_dir().join(format!(
        "surety-verify-{}-{}-{}",
        std::process::id(),
        node.get(..8).unwrap_or(node),
        perf.get(..8).unwrap_or(perf)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let result = (|| {
        let (name, perf_path) = materialize(
            store,
            node,
            perf,
            &record["provenance"],
            &dir,
            &mut HashSet::new(),
        )?;
        let file = dir.join(format!("{name}.sur"));
        let (graph, idx) = load_with_siblings(&file).map_err(errors_text)?;
        graph.check_file(idx).map_err(|e| e.to_string())?;
        let canonical = store.canonical(node)?;
        let budget = budget_of(evidence);
        let timeout = timeout_of(evidence);
        let mut checker = Checker::new(&graph, idx, &perf_path, &dir, timeout)?;
        checker.budget = budget.clone();
        admit(evidence, node, &canonical, &perf_path, &mut checker)?;
        if rerun {
            let report = checker.check();
            let rendered = render(&report);
            let again = record::evidence(&checker, &report, &budget, timeout, perf, &rendered);
            if again != *evidence {
                return Err(format!(
                    "the re-run differs in: {}",
                    differing_keys(evidence, &again).join(", ")
                ));
            }
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&dir);
    result
}

/// The kernel over every record in the graph; with `rerun`, the check too.
pub fn verify(store: &Store, rerun: bool, render: Render) -> Result<Vec<Verification>, String> {
    let mut out = Vec::new();
    for node in store.nodes()? {
        for (perf, record) in store.records(&node, TARGET)? {
            let name = record["provenance"]["presentation"]
                .as_str()
                .and_then(|p| store.presentation(&node, p).ok())
                .and_then(|t| function_name(&t).ok())
                .unwrap_or_default();
            let result = verify_one(store, &node, &perf, &record, rerun, render);
            out.push(Verification {
                node: node.clone(),
                name,
                performance: perf,
                result,
            });
        }
    }
    Ok(out)
}
