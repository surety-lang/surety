//! The evidence record and the kernel.
//!
//! A record has two parts. Provenance (the checker's version, the Python
//! version, the source paths, the presentation, the sibling performances
//! the check ran against) is not compared on verify. Evidence (both hashes,
//! the budget, the verdict line, the report `check --json` prints, and the
//! run log) is compared byte for byte, so it carries no timestamp, no
//! absolute path and no machine name.
//!
//! The invariant: the evidence part is a function of the obligation hash,
//! the performance hash and the budget, and of nothing else. A line number
//! or a clause's source text belongs to a presentation, and two
//! presentations of one node must carry one record, so at pin every clause
//! is named by its ordinal in canonical order with its canonical print as
//! the label, every example by its canonical ordinal, and the run log's
//! example runs stand in that order. `check --json` keeps lines and text:
//! the editor is presentation-bound by nature; the record is not. The
//! invariant cannot be checked field by field, so the store's acceptance
//! test enforces it: two presentations of one node pin to byte-identical
//! evidence, and a record verifies when materialized under the other one.
//!
//! The kernel checks the bookkeeping, never the logic: that every claim in
//! the evidence is a count over recorded runs, that the runs are the inputs
//! the obligation and budget produce, and that the hashes are the content's.
//! Whether a property holds is the checker's job, and verify tests it by
//! running the check again.

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value as Json, json};
use surety_check::value::to_json;
use surety_check::{Budget, Checker, Outcome, Report, Value, Verdict};
use surety_syntax::normalize::{Identities, canonical_file, identities};

use crate::store::text_hash;

pub const TARGET: &str = "python";

/// The record format, in provenance: bumped when the evidence part's shape
/// changes, so `verify` can say "format moved" rather than "evidence
/// differs". 1: clauses by presentation line. 2: clauses and examples by
/// canonical ordinal, the domain sorted.
pub const FORMAT: u64 = 2;

/// What the caller renders from a report for the record: the verdict line
/// and the `--json` lines. The graph does not render reports itself.
pub struct Rendered {
    pub verdict: String,
    pub lines: Vec<Json>,
}

pub type Render = fn(&Report) -> Rendered;

pub fn input_json(inputs: &[Value]) -> Json {
    Json::Array(inputs.iter().map(to_json).collect())
}

pub fn output_hash(v: &Value) -> String {
    text_hash(&to_json(v).to_string())
}

/// The evidence part of a record for one check, in presentation-free
/// terms (see the module's invariant).
pub fn evidence(
    checker: &Checker,
    report: &Report,
    budget: &Budget,
    timeout: Duration,
    perf_hash: &str,
    rendered: &Rendered,
) -> Result<Json, String> {
    let ids = identities(checker.graph, checker.idx, checker.fun)
        .map_err(|e| format!("cannot identify the clauses: {e}"))?;
    let differed: HashSet<String> = report
        .determinism
        .failures
        .iter()
        .map(|f| input_json(&f.inputs).to_string())
        .collect();
    let mut runs = Vec::new();
    for ex in &report.examples {
        let inputs = input_json(&ex.inputs);
        let mut e = json!({
            "example": ex.index,
            "in": inputs,
            "out": ex.output.as_ref().map(output_hash),
            "pass": ex.verdict == Verdict::Pass,
        });
        if differed.contains(&inputs.to_string()) {
            e["again"] = json!(false);
        }
        runs.push(e);
    }
    for g in &checker.generated {
        let inputs = input_json(&g.inputs);
        let mut e = json!({ "in": inputs });
        match &g.outcome {
            Outcome::Output(v) => e["out"] = json!(output_hash(v)),
            Outcome::Raised(_) => e["raised"] = json!(true),
            Outcome::Boundary(_) => e["boundary"] = json!(true),
            Outcome::Timeout => e["timeout"] = json!(true),
        }
        if differed.contains(&inputs.to_string()) {
            e["again"] = json!(false);
        }
        runs.push(e);
    }
    let (lines, runs) = canonical_terms(rendered.lines.clone(), runs, &ids);
    Ok(json!({
        "obligation": report.obligation_hash,
        "target": TARGET,
        "performance": perf_hash,
        "budget": {
            "generated": budget.generated,
            "search_inputs": budget.search_inputs,
            "search_candidates": budget.search_candidates,
            "mutants": budget.mutants,
        },
        "timeout_secs": timeout.as_secs(),
        "verdict": rendered.verdict,
        "report": lines,
        "runs": runs,
    }))
}

/// The report's lines and the run log with every presentation reference
/// replaced by a canonical one: a clause object `{line, text}` becomes
/// `{clause, canonical}` (`frame: true` for a frame clause, other keys
/// kept), an `example` number becomes its canonical ordinal from 1 and an
/// `accept` index its ordinal, the evidence object's per-clause lists stand
/// in canonical order, and so do the example runs in the log.
fn canonical_terms(lines: Vec<Json>, runs: Vec<Json>, ids: &Identities) -> (Vec<Json>, Vec<Json>) {
    let lines = lines
        .into_iter()
        .map(|mut line| {
            if line.get("clause").is_some_and(Json::is_object) {
                line["clause"] = clause_ref(&line["clause"], ids);
            }
            for key in ["clauses", "slipped", "uncaught", "unevaluable", "caught_by"] {
                if let Some(items) = line.get(key).and_then(Json::as_array) {
                    let mapped: Vec<Json> = items.iter().map(|c| clause_ref(c, ids)).collect();
                    line[key] = Json::Array(mapped);
                }
            }
            // The report's lines number examples from 1.
            if let Some(i) = line.get("example").and_then(Json::as_u64) {
                if let Some(id) = i.checked_sub(1).and_then(|i| ids.example_at(i as usize)) {
                    line["example"] = json!(id.ordinal + 1);
                }
            }
            if let Some(i) = line.get("accept").and_then(Json::as_u64) {
                if let Some(id) = ids.accept_at(i as usize) {
                    line["accept"] = json!(id.ordinal);
                }
            }
            if line["kind"] == "evidence" {
                if let Some(each) = line["properties"]["each"].as_array() {
                    let mut rows: Vec<Json> = each
                        .iter()
                        .filter_map(|p| {
                            let id = ids.property_at(p["line"].as_u64()? as usize)?;
                            Some(json!({
                                "clause": id.ordinal,
                                "canonical": id.canonical,
                                "runs": p["runs"],
                                "findings": p["findings"],
                            }))
                        })
                        .collect();
                    rows.sort_by_key(|r| r["clause"].as_u64());
                    rows.dedup_by_key(|r| r["clause"].as_u64());
                    line["properties"]["each"] = Json::Array(rows);
                }
                if let Some(accepts) = line["accepts"].as_array() {
                    let mut rows: Vec<Json> = accepts
                        .iter()
                        .enumerate()
                        .filter_map(|(i, a)| {
                            let id = ids.accept_at(i)?;
                            Some(json!({
                                "accept": id.ordinal,
                                "canonical": id.canonical,
                                "satisfied": a["satisfied"],
                                "dead": a["dead"],
                            }))
                        })
                        .collect();
                    rows.sort_by_key(|r| r["accept"].as_u64());
                    rows.dedup_by_key(|r| r["accept"].as_u64());
                    line["accepts"] = Json::Array(rows);
                }
            }
            line
        })
        .collect();
    let mut examples: Vec<Json> = Vec::new();
    let mut generated: Vec<Json> = Vec::new();
    for mut run in runs {
        match run.get("example").and_then(Json::as_u64) {
            Some(i) => {
                // The checker numbers example runs from 0; the record from 1,
                // as the report's lines do.
                if let Some(id) = ids.example_at(i as usize) {
                    run["example"] = json!(id.ordinal + 1);
                }
                examples.push(run);
            }
            None => generated.push(run),
        }
    }
    examples.sort_by_key(|r| r["example"].as_u64());
    examples.extend(generated);
    (lines, examples)
}

/// A clause object in canonical terms. One with no line (the output type)
/// is left as it is.
fn clause_ref(c: &Json, ids: &Identities) -> Json {
    let Some(line) = c.get("line").and_then(Json::as_u64) else {
        return c.clone();
    };
    let mut out = serde_json::Map::new();
    match ids.property_at(line as usize) {
        Some(id) => {
            out.insert("clause".into(), json!(id.ordinal));
            out.insert("canonical".into(), json!(id.canonical));
        }
        None => {
            out.insert("clause".into(), Json::Null);
        }
    }
    if c["text"].as_str().is_some_and(|t| t.starts_with("frame ")) {
        out.insert("frame".into(), json!(true));
    }
    if let Some(obj) = c.as_object() {
        for (k, v) in obj {
            if k != "line" && k != "text" {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    Json::Object(out)
}

pub fn budget_of(evidence: &Json) -> Budget {
    let b = &evidence["budget"];
    let d = Budget::default();
    Budget {
        generated: b["generated"]
            .as_u64()
            .map(|n| n as usize)
            .unwrap_or(d.generated),
        search_inputs: b["search_inputs"]
            .as_u64()
            .map(|n| n as usize)
            .unwrap_or(d.search_inputs),
        search_candidates: b["search_candidates"]
            .as_u64()
            .map(|n| n as usize)
            .unwrap_or(d.search_candidates),
        mutants: b["mutants"].as_bool().unwrap_or(d.mutants),
    }
}

pub fn timeout_of(evidence: &Json) -> Duration {
    Duration::from_secs(evidence["timeout_secs"].as_u64().unwrap_or(10))
}

/// The `kind: evidence` line of the report.
pub fn evidence_object(evidence: &Json) -> Option<&Json> {
    evidence["report"]
        .as_array()?
        .iter()
        .find(|l| l["kind"] == "evidence")
}

pub fn strict_passed(evidence: &Json) -> bool {
    evidence_object(evidence).is_some_and(|ev| ev["strict_passed"] == true)
}

pub fn passed(evidence: &Json) -> bool {
    evidence_object(evidence).is_some_and(|ev| ev["passed"] == true)
}

fn short(h: &str) -> &str {
    h.get(..12).unwrap_or(h)
}

/// The kernel: admit a record or say which check it failed. `checker` is
/// built on the obligation the node holds, with the performance under
/// judgment; its budget is set from the record here.
pub fn admit(
    evidence: &Json,
    node_hash: &str,
    canonical: &str,
    perf_path: &Path,
    checker: &mut Checker,
) -> Result<(), String> {
    // 1. The hashes are the content's.
    let stored = text_hash(canonical);
    if stored != node_hash {
        return Err(format!(
            "kernel: the node's canonical form hashes to {}, not the directory's {}",
            short(&stored),
            short(node_hash)
        ));
    }
    let claimed = evidence["obligation"].as_str().unwrap_or_default();
    if claimed != node_hash {
        return Err(format!(
            "kernel: the record's obligation hash {} is not the node's {}",
            short(claimed),
            short(node_hash)
        ));
    }
    let parsed = canonical_file(checker.graph, checker.idx).map_err(|e| e.to_string())?;
    if parsed.hash != node_hash {
        return Err(format!(
            "kernel: the presentation parses to {}, not the node's {}",
            short(&parsed.hash),
            short(node_hash)
        ));
    }
    let perf_claimed = evidence["performance"].as_str().unwrap_or_default();
    let perf_actual = surety_py::inspect(perf_path)
        .map_err(|e| format!("kernel: cannot hash the performance: {e}"))?
        .hash;
    if perf_claimed != perf_actual {
        return Err(format!(
            "kernel: the record's performance hash {} is not the pinned copy's {}",
            short(perf_claimed),
            short(&perf_actual)
        ));
    }

    // 2. The runs are the inputs the obligation and the budget produce: the
    // examples in canonical order, then the generated inputs in the
    // generator's order.
    checker.budget = budget_of(evidence);
    let ids = identities(checker.graph, checker.idx, checker.fun)
        .map_err(|e| format!("kernel: cannot identify the examples: {e}"))?;
    let mut examples: Vec<(usize, Json)> = checker
        .fun
        .examples
        .iter()
        .enumerate()
        .filter_map(|(i, ex)| {
            let inputs = checker.example_inputs(ex).ok()?;
            let ordinal = ids.example_at(i).map(|d| d.ordinal).unwrap_or(i);
            Some((ordinal, input_json(&inputs)))
        })
        .collect();
    examples.sort_by_key(|(o, _)| *o);
    let mut expected: Vec<Json> = examples.into_iter().map(|(_, j)| j).collect();
    expected.extend(
        checker
            .generated_inputs()
            .inputs
            .iter()
            .map(|i| input_json(i)),
    );
    let runs = evidence["runs"]
        .as_array()
        .ok_or("kernel: the record has no run log")?;
    let has_inputs = !checker.fun.inputs.is_empty();
    let logged: Vec<&Json> = runs
        .iter()
        .filter(|r| {
            // An example whose `in:` did not decode has no inputs to log.
            !(r.get("example").is_some()
                && has_inputs
                && r["in"].as_array().is_some_and(|a| a.is_empty()))
        })
        .map(|r| &r["in"])
        .collect();
    if logged.len() != expected.len() {
        return Err(format!(
            "kernel: the run log holds {} inputs; the obligation and budget produce {}",
            logged.len(),
            expected.len()
        ));
    }
    for (i, (l, e)) in logged.iter().zip(&expected).enumerate() {
        if *l != e {
            return Err(format!(
                "kernel: run {} is not the input the generator produces in that place",
                i + 1
            ));
        }
    }

    // 3. Every count is a count over the log.
    let ev = evidence_object(evidence).ok_or("kernel: the record has no evidence object")?;
    let examples: Vec<&Json> = runs.iter().filter(|r| r.get("example").is_some()).collect();
    let examples_passed = examples.iter().filter(|r| r["pass"] == true).count();
    if ev["examples"]["passed"] != examples_passed || ev["examples"]["total"] != examples.len() {
        return Err(format!(
            "kernel: examples {}/{} claimed; the log holds {} passing of {}",
            ev["examples"]["passed"],
            ev["examples"]["total"],
            examples_passed,
            examples.len()
        ));
    }
    let with_output = runs.iter().filter(|r| r["out"].is_string()).count();
    if let Some(each) = ev["properties"]["each"].as_array() {
        for p in each {
            if p["runs"] != with_output {
                return Err(format!(
                    "kernel: clause {} claims {} runs; the log holds {} outputs",
                    p["clause"], p["runs"], with_output
                ));
            }
        }
    }
    let differed = runs
        .iter()
        .filter(|r| r["out"].is_string() && r["again"] == false)
        .count();
    let d = &ev["determinism"];
    if d["total"] != with_output || d["matched"] != with_output - differed {
        return Err(format!(
            "kernel: determinism {}/{} claimed; the log holds {} of {} agreeing",
            d["matched"],
            d["total"],
            with_output - differed,
            with_output
        ));
    }
    if let (Some(answered), Some(total)) = (
        ev["totality"]["answered"].as_u64(),
        ev["totality"]["total"].as_u64(),
    ) {
        let timeouts = runs.iter().filter(|r| r["timeout"] == true).count() as u64;
        if total != runs.len() as u64 || answered != total - timeouts {
            return Err(format!(
                "kernel: totality {answered}/{total} claimed; the log holds {} runs, {timeouts} timed out",
                runs.len()
            ));
        }
    }

    // 4. Every finding names an input the log holds.
    if let Some(lines) = evidence["report"].as_array() {
        for line in lines.iter().filter(|l| l["kind"] != "evidence") {
            if let Some(ins) = line["inputs"].as_array() {
                for i in ins {
                    if !runs.iter().any(|r| r["in"] == i["json"]) {
                        return Err(format!(
                            "kernel: a finding of kind {} names an input the log does not hold",
                            line["kind"]
                        ));
                    }
                }
            }
        }
    }

    // 5. The derived fields agree with the parts.
    let strict = ev["strict_passed"] == true;
    let passed = ev["passed"] == true;
    if strict && !(passed && ev["findings"] == 0) {
        return Err("kernel: strict_passed claimed with a binding failure or a finding".into());
    }
    if passed && examples_passed != examples.len() {
        return Err("kernel: passed claimed with a failing example".into());
    }
    Ok(())
}
