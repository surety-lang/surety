//! The `surety` command: loads a corpus directory as one graph, parses,
//! hashes, and checks obligations against performances.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value as Json, json};
use surety_check::value::to_json;
use surety_check::{Budget, Checker, Counterexample, PropVerdict, Report, Value, Verdict};
use surety_syntax::{Error, Graph, parse};

/// Per-call timeout for a performance.
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// Every `.sur` file in a directory, parsed, as one graph. Parse errors are
/// returned per file so one bad file does not hide the others.
pub fn load_dir(dir: &Path) -> Result<Graph, Vec<(PathBuf, Error)>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| {
            vec![(
                dir.to_path_buf(),
                Error::new(surety_syntax::ErrorKind::Syntax, 0, e.to_string()),
            )]
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "sur"))
        .collect();
    paths.sort();
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for p in paths {
        let text = match fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => {
                errors.push((
                    p.clone(),
                    Error::new(surety_syntax::ErrorKind::Syntax, 0, e.to_string()),
                ));
                continue;
            }
        };
        match parse(&text) {
            Ok(f) => files.push((stem(&p), f)),
            Err(e) => errors.push((p.clone(), e)),
        }
    }
    if errors.is_empty() {
        Ok(Graph::new(files))
    } else {
        Err(errors)
    }
}

pub fn stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Load the directory holding `file` as a graph and return the index of
/// `file` in it. If `file` does not parse, the error is returned alone.
pub fn load_with_siblings(file: &Path) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let text = fs::read_to_string(file).map_err(|e| {
        vec![(
            file.to_path_buf(),
            Error::new(surety_syntax::ErrorKind::Syntax, 0, e.to_string()),
        )]
    })?;
    let own = parse(&text).map_err(|e| vec![(file.to_path_buf(), e)])?;
    let own_stem = stem(file);
    let mut files = vec![(own_stem.clone(), own)];
    if let Ok(rd) = fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sur") && stem(p) != own_stem)
            .collect();
        paths.sort();
        for p in paths {
            // Siblings that do not parse are skipped: they are not this file's problem.
            if let Ok(t) = fs::read_to_string(&p) {
                if let Ok(f) = parse(&t) {
                    files.push((stem(&p), f));
                }
            }
        }
    }
    Ok((Graph::new(files), 0))
}

fn errors_text(errs: Vec<(PathBuf, Error)>) -> String {
    errs.iter()
        .map(|(p, e)| format!("{}: {e}", p.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A checker for one obligation file against a performance module, with
/// the file's siblings as the graph.
pub fn checker_for<'g>(
    graph: &'g Graph,
    idx: usize,
    file: &Path,
    performance: &Path,
) -> Result<Checker<'g>, String> {
    graph
        .check_file(idx)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    let module_dir = file.parent().unwrap_or(Path::new("."));
    Checker::new(graph, idx, performance, module_dir, TIMEOUT)
}

/// Check one obligation file against a performance module under the
/// default budget.
pub fn check_file(file: &Path, performance: &Path) -> Result<Report, String> {
    check_file_with(file, performance, Budget::default())
}

pub fn check_file_with(file: &Path, performance: &Path, budget: Budget) -> Result<Report, String> {
    let (graph, idx) = load_with_siblings(file).map_err(errors_text)?;
    let mut checker = checker_for(&graph, idx, file, performance)?;
    checker.budget = budget;
    Ok(checker.check())
}

/// An input tuple in `.sur` syntax: one value, or a tuple of them.
pub fn sur_inputs(inputs: &[Value]) -> String {
    if inputs.len() == 1 {
        format!("{}", inputs[0])
    } else {
        format!(
            "({})",
            inputs
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// The last line of a message: a traceback's exception line.
pub fn last_line(m: &str) -> &str {
    m.trim_end().lines().last().unwrap_or("").trim()
}

/// One claim as the text report shows it.
struct ClaimView<'a> {
    label: String,
    verdict: &'a PropVerdict,
    runs: usize,
    cex: &'a [Counterexample],
    suggest: Option<&'a str>,
    note: Option<&'a str>,
}

fn render_claim(s: &mut String, example_runs: usize, c: &ClaimView<'_>) {
    let label = &c.label;
    match c.verdict {
        PropVerdict::Pass if c.cex.is_empty() => {
            s.push_str(&format!("  pass  {label}  ({} runs)\n", c.runs))
        }
        PropVerdict::Pass => {
            let first = &c.cex[0];
            s.push_str(&format!(
                "  FINDING  {label}\n        false on {} of {} generated runs; first: in: {}, out = {}\n",
                c.cex.len(),
                c.runs - example_runs,
                sur_inputs(&first.inputs),
                first.output
            ));
            if let Some(e) = &first.error {
                s.push_str(&format!("        {e}\n"));
            }
            if let Some(sg) = c.suggest {
                s.push_str(&format!("        suggest {sg}\n"));
            }
            if let Some(n) = c.note {
                s.push_str(&format!("        note: {n}\n"));
            }
        }
        PropVerdict::Fail { detail, .. } => {
            s.push_str(&format!("  FAIL  {label}\n        {detail}\n"))
        }
        PropVerdict::Error(m) => s.push_str(&format!("  ERROR  {label}\n        {m}\n")),
        PropVerdict::Skipped => s.push_str(&format!("  skip  {label}\n")),
    }
}

fn bucket_text(report: &Report) -> String {
    report
        .buckets
        .iter()
        .map(|b| {
            let lengths = if b.lengths.is_empty() {
                "any".to_string()
            } else {
                b.lengths
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join("x")
            };
            if b.exhaustive {
                format!("{lengths}: all {}", b.taken)
            } else {
                format!("{lengths}: {} of {}", b.taken, b.space)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The evidence record, one line per kind.
pub fn render_evidence(r: &Report) -> String {
    let mut s = String::new();
    let ex_pass = r
        .examples
        .iter()
        .filter(|e| e.verdict == Verdict::Pass)
        .count();
    let claims = r.properties.len() + r.frame.len();
    let claims_pass = r
        .properties
        .iter()
        .filter(|p| p.verdict == PropVerdict::Pass)
        .count()
        + r.frame
            .iter()
            .filter(|f| f.verdict == PropVerdict::Pass)
            .count();
    let calls = r.example_inputs + r.generated_inputs;
    let timeouts = r.generated_timeouts.len();
    let raised = r.generated_raised.len();
    s.push_str("  evidence\n");
    s.push_str(&format!(
        "    types:            validated at the boundary; {} example inputs, {} generated inputs, {} outputs{}\n",
        r.example_inputs,
        r.generated_inputs,
        r.runs,
        if r.generated_boundary.is_empty() {
            String::new()
        } else {
            format!("; {} generated outputs failed the boundary", r.generated_boundary.len())
        }
    ));
    s.push_str(&format!(
        "    examples:         {ex_pass}/{}\n",
        r.examples.len()
    ));
    s.push_str(&format!(
        "    properties:       {claims_pass}/{claims} over {} runs; findings: {}\n",
        r.runs,
        r.claim_findings()
    ));
    s.push_str(&format!(
        "    determinism:      {}/{}, each input run twice and compared under the order claims\n",
        r.determinism.matched, r.determinism.total
    ));
    s.push_str(&format!(
        "    totality:         {}/{} answered within {:?}{}\n",
        calls.saturating_sub(timeouts),
        calls,
        r.timeout,
        if raised > 0 {
            format!("; {raised} raised")
        } else {
            String::new()
        }
    ));
    match &r.inspection {
        Some(i) => {
            let imports = if i.imports.is_empty() {
                "none".to_string()
            } else {
                i.imports.join(", ")
            };
            let deps = if i.dependencies.is_empty() {
                "none".to_string()
            } else {
                i.dependencies.join(", ")
            };
            s.push_str(&format!(
                "    purity:           unverified (no sandbox in this slice); imports: {imports}\n"
            ));
            s.push_str(&format!(
                "    shape:            {} lines, {} functions; dependencies: {deps}\n",
                i.lines, i.functions
            ));
        }
        None => {
            s.push_str("    purity:           unverified (no sandbox in this slice)\n");
            s.push_str(&format!(
                "    shape:            not inspected: {}\n",
                r.inspection_error
                    .as_deref()
                    .map(last_line)
                    .unwrap_or("no performance")
            ));
        }
    }
    s.push_str("    underdetermination: not run (surety mutants)\n");
    s.push_str(&format!(
        "    obligation:       {}\n    performance:      {}\n",
        r.obligation_hash,
        r.inspection
            .as_ref()
            .map(|i| i.hash.as_str())
            .unwrap_or("-")
    ));
    if r.generated_inputs > 0 || !r.buckets.is_empty() {
        s.push_str(&format!(
            "    generated:        {} inputs by list length: {}\n",
            r.generated_inputs,
            bucket_text(r)
        ));
    }
    s
}

/// A plain-text rendering of a report.
pub fn render(report: &Report) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", report.obligation));
    for e in &report.examples {
        let label = match &e.note {
            Some(n) => format!("example {} (line {}, {n:?})", e.index + 1, e.line),
            None => format!("example {} (line {})", e.index + 1, e.line),
        };
        match &e.verdict {
            Verdict::Pass => s.push_str(&format!("  pass  {label}\n")),
            Verdict::Boundary(m) => s.push_str(&format!("  BOUNDARY  {label}: {m}\n")),
            Verdict::Mismatch(m) => s.push_str(&format!("  FAIL  {label}: {m}\n")),
            Verdict::Error(m) => s.push_str(&format!("  ERROR  {label}: {m}\n")),
        }
    }
    for p in &report.properties {
        render_claim(
            &mut s,
            report.example_runs,
            &ClaimView {
                label: format!("property (line {}): {}", p.line, p.text),
                verdict: &p.verdict,
                runs: p.runs,
                cex: &p.counterexamples,
                suggest: p.suggest.as_deref(),
                note: p.note.as_deref(),
            },
        );
    }
    for f in &report.frame {
        render_claim(
            &mut s,
            report.example_runs,
            &ClaimView {
                label: format!(
                    "frame {}: preserves {} by default",
                    f.description,
                    f.preserved.join(", ")
                ),
                verdict: &f.verdict,
                runs: f.runs,
                cex: &f.counterexamples,
                suggest: f.suggest.as_deref(),
                note: f.note.as_deref(),
            },
        );
    }
    for g in &report.graph_refs {
        s.push_str(&format!("  graph reference: {g}\n"));
    }
    if let Some(f) = report.generated_boundary.first() {
        s.push_str(&format!(
            "  FINDING  {} generated outputs failed the boundary; first: in: {}: {}\n",
            report.generated_boundary.len(),
            sur_inputs(&f.inputs),
            f.message
        ));
        if let Some(sg) = &report.boundary_suggest {
            s.push_str(&format!("        suggest {sg}\n"));
        }
    }
    for f in &report.generated_raised {
        s.push_str(&format!(
            "  FINDING  raised on generated input {}: {}\n",
            sur_inputs(&f.inputs),
            last_line(&f.message)
        ));
    }
    for f in &report.generated_timeouts {
        s.push_str(&format!(
            "  FINDING  generated input {}: {}\n",
            sur_inputs(&f.inputs),
            f.message
        ));
    }
    for f in &report.determinism.failures {
        s.push_str(&format!(
            "  FINDING  nondeterministic on {}: {}\n",
            sur_inputs(&f.inputs),
            f.message
        ));
    }
    s.push_str(&render_evidence(report));
    s
}

fn input_json(inputs: &[Value]) -> Json {
    json!([{ "sur": format!("in: {}", sur_inputs(inputs)), "json": inputs.iter().map(to_json).collect::<Vec<_>>() }])
}

fn finding(kind: &str, r: &Report, clause: Option<(usize, &str)>, inputs: &[Value]) -> Json {
    let mut f = json!({
        "kind": kind,
        "obligation": r.obligation,
        "obligation_hash": r.obligation_hash,
        "inputs": input_json(inputs),
    });
    if let Some(i) = &r.inspection {
        f["performance_hash"] = json!(i.hash);
    }
    if let Some((line, text)) = clause {
        f["clause"] = json!({ "line": line, "text": text });
    }
    f
}

fn with_outputs(mut f: Json, expected: Option<&Value>, candidate: Option<&Value>) -> Json {
    let mut outputs = serde_json::Map::new();
    if let Some(e) = expected {
        outputs.insert("expected".into(), to_json(e));
    }
    if let Some(c) = candidate {
        outputs.insert("candidate".into(), to_json(c));
    }
    if !outputs.is_empty() {
        f["outputs"] = Json::Object(outputs);
    }
    f
}

/// The machine form: one object per finding, then one `evidence` object.
pub fn json_lines(r: &Report) -> Vec<Json> {
    let mut out = Vec::new();
    let examples_pass = r.examples.iter().all(|e| e.verdict == Verdict::Pass);
    for e in &r.examples {
        let (kind, message) = match &e.verdict {
            Verdict::Pass => continue,
            Verdict::Boundary(m) => ("boundary", m),
            Verdict::Mismatch(m) => ("example_mismatch", m),
            Verdict::Error(m) => ("error", m),
        };
        let mut f = finding(kind, r, None, &e.inputs);
        f["example"] = json!(e.index + 1);
        f["message"] = json!(message);
        out.push(with_outputs(f, e.expected.as_ref(), e.output.as_ref()));
    }
    type Claim<'a> = (
        usize,
        String,
        &'a PropVerdict,
        &'a [Counterexample],
        Option<&'a str>,
        Option<&'a str>,
    );
    let claims: Vec<Claim<'_>> = r
        .properties
        .iter()
        .map(|p| {
            (
                p.line,
                p.text.clone(),
                &p.verdict,
                p.counterexamples.as_slice(),
                p.suggest.as_deref(),
                p.note.as_deref(),
            )
        })
        .chain(r.frame.iter().map(|f| {
            (
                f.line,
                format!(
                    "frame {}: preserves {} by default",
                    f.description,
                    f.preserved.join(", ")
                ),
                &f.verdict,
                f.counterexamples.as_slice(),
                f.suggest.as_deref(),
                f.note.as_deref(),
            )
        }))
        .collect();
    for (line, text, verdict, cex, suggest, note) in claims {
        match verdict {
            PropVerdict::Fail {
                example, detail, ..
            } => {
                let ex = &r.examples[*example];
                let mut f = finding("property_failed", r, Some((line, &text)), &ex.inputs);
                f["example"] = json!(example + 1);
                f["message"] = json!(detail);
                out.push(with_outputs(f, ex.expected.as_ref(), ex.output.as_ref()));
            }
            PropVerdict::Error(m) => {
                let mut f = finding("error", r, Some((line, &text)), &[]);
                f["message"] = json!(m);
                out.push(f);
            }
            _ => {}
        }
        for c in cex {
            let kind = if examples_pass && *verdict == PropVerdict::Pass {
                "unsatisfiable_on"
            } else {
                "property_failed"
            };
            let mut f = finding(kind, r, Some((line, &text)), &c.inputs);
            f["generated"] = json!(true);
            if let Some(e) = &c.error {
                f["message"] = json!(e);
            }
            if let Some(sg) = suggest {
                f["suggest"] = json!(sg);
            }
            if let Some(n) = note {
                f["note"] = json!(n);
            }
            out.push(with_outputs(f, None, Some(&c.output)));
        }
    }
    for (kind, list) in [
        ("boundary", &r.generated_boundary),
        ("raised", &r.generated_raised),
        ("timeout", &r.generated_timeouts),
        ("nondeterministic", &r.determinism.failures),
    ] {
        for fl in list {
            let mut f = finding(kind, r, None, &fl.inputs);
            f["generated"] = json!(true);
            f["message"] = json!(fl.message);
            if kind == "boundary" {
                if let Some(sg) = &r.boundary_suggest {
                    f["suggest"] = json!(sg);
                }
            }
            out.push(f);
        }
    }
    let claims_total = r.properties.len() + r.frame.len();
    let claims_pass = r
        .properties
        .iter()
        .filter(|p| p.verdict == PropVerdict::Pass)
        .count()
        + r.frame
            .iter()
            .filter(|f| f.verdict == PropVerdict::Pass)
            .count();
    let mut ev = json!({
        "kind": "evidence",
        "obligation": r.obligation,
        "obligation_hash": r.obligation_hash,
        "passed": r.passed(),
        "strict_passed": r.strict_passed(),
        "types": { "example_inputs": r.example_inputs, "generated_inputs": r.generated_inputs, "outputs": r.runs, "boundary_failures": r.generated_boundary.len() },
        "examples": { "passed": r.examples.iter().filter(|e| e.verdict == Verdict::Pass).count(), "total": r.examples.len() },
        "properties": { "passed": claims_pass, "total": claims_total, "runs": r.runs, "findings": r.claim_findings(),
            "each": r.properties.iter().map(|p| json!({ "line": p.line, "runs": p.runs, "findings": p.findings() })).collect::<Vec<_>>() },
        "determinism": { "matched": r.determinism.matched, "total": r.determinism.total },
        "totality": { "answered": (r.example_inputs + r.generated_inputs).saturating_sub(r.generated_timeouts.len()), "total": r.example_inputs + r.generated_inputs, "raised": r.generated_raised.len(), "timeout_seconds": r.timeout.as_secs_f64() },
        "purity": "unverified",
        "underdetermination": "not run",
        "generated": r.buckets.iter().map(|b| json!({ "lengths": b.lengths, "taken": b.taken, "space": b.space.to_string(), "exhaustive": b.exhaustive })).collect::<Vec<_>>(),
    });
    if let Some(i) = &r.inspection {
        ev["performance_hash"] = json!(i.hash);
        ev["imports"] = json!(i.imports);
        ev["dependencies"] = json!(i.dependencies);
        ev["shape"] = json!({ "lines": i.lines, "functions": i.functions });
    } else if let Some(e) = &r.inspection_error {
        ev["inspection_error"] = json!(last_line(e));
    }
    out.push(ev);
    out
}

/// The machine form as text: one JSON object per line.
pub fn render_json(r: &Report) -> String {
    json_lines(r)
        .iter()
        .map(|j| j.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
