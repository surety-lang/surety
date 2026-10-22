//! The `surety` command: loads a corpus directory as one graph, parses,
//! hashes, and checks obligations against performances.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value as Json, json};
use surety_check::mutants::{Clause, MutantReport, MutantRun, MutantVerdict};
use surety_check::value::to_json;
use surety_check::{
    Bucket, Budget, Cause, Checker, Class, DiffReport, Difference, LookKind, Outcome, Party,
    PropVerdict, Report, Value, Verdict,
};
use surety_check::{Gap, gaps};
use surety_graph::{NodeView, PinState, Pinned, Rendered, State, StatusRow, Verification};
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

pub use surety_syntax::load::{load_with_siblings, stem};

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

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}

fn bucket_text(buckets: &[Bucket]) -> String {
    buckets
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

/// The `generated:` line: one line, whatever the number of buckets.
fn generated_text(r: &Report) -> String {
    if r.buckets.is_empty() {
        return format!("{} inputs", r.generated_inputs);
    }
    let list_inputs = r.buckets[0].lengths.len();
    let enumerated = r.buckets.iter().filter(|b| b.exhaustive).count();
    let max_len = r
        .buckets
        .iter()
        .flat_map(|b| b.lengths.iter().copied())
        .max()
        .unwrap_or(0);
    if list_inputs == 0 {
        return format!(
            "{} inputs over the record's fields{}",
            r.generated_inputs,
            if enumerated > 0 {
                ", enumerated whole"
            } else {
                ", sampled"
            }
        );
    }
    format!(
        "{} inputs: lists of length 0 to {max_len} over {}, {}, {enumerated} enumerated whole, {} sampled",
        r.generated_inputs,
        plural(list_inputs, "list input", "list inputs"),
        plural(r.buckets.len(), "length combination", "length combinations"),
        r.buckets.len() - enumerated
    )
}

/// What the report says about each party, as short phrases.
fn party_summaries(r: &Report) -> (Vec<String>, Vec<String>) {
    let mut obligation = Vec::new();
    let mut performance = Vec::new();
    let failing: Vec<String> = r
        .examples
        .iter()
        .filter(|e| e.verdict != Verdict::Pass)
        .map(|e| (e.index + 1).to_string())
        .collect();
    if !failing.is_empty() {
        performance.push(format!(
            "fails {} {}",
            if failing.len() == 1 {
                "example"
            } else {
                "examples"
            },
            failing.join(", ")
        ));
    }
    for p in &r.properties {
        match &p.verdict {
            PropVerdict::Fail { example, .. } => performance.push(format!(
                "fails property (line {}) on example {}",
                p.line,
                example + 1
            )),
            PropVerdict::Error(_) => {
                performance.push(format!("cannot evaluate property (line {})", p.line))
            }
            _ => {}
        }
    }
    for f in &r.frame {
        if let PropVerdict::Fail { example, .. } = &f.verdict {
            performance.push(format!(
                "fails frame {} on example {}",
                f.description,
                example + 1
            ));
        }
    }
    for c in &r.causes {
        let where_ = format!("{} of {} generated inputs", c.inputs, c.generated);
        match c.party {
            Party::Obligation => obligation.push(format!(
                "unsatisfiable on {where_}{}",
                match (&c.suggest, &c.note) {
                    (Some(sg), _) => format!(", closed by {sg}"),
                    (None, Some(_)) => ", no refinement closes it".to_string(),
                    (None, None) => String::new(),
                }
            )),
            Party::Performance => performance.push(format!("fails {} on {where_}", c.label())),
            Party::Undetermined => obligation.push(format!(
                "{} false on {where_} (party not determined: search budget 0)",
                c.label()
            )),
        }
    }
    if !r.generated_boundary.is_empty() {
        let n = r.generated_boundary.len();
        match &r.boundary_suggest {
            Some(sg) => obligation.push(format!(
                "forces an output outside the type on {n} generated inputs, closed by {sg}"
            )),
            None => performance.push(format!(
                "returns a value outside the type on {n} generated inputs"
            )),
        }
    }
    if !r.generated_raised.is_empty() {
        performance.push(format!(
            "raises on {} generated inputs",
            r.generated_raised.len()
        ));
    }
    if !r.generated_timeouts.is_empty() {
        performance.push(format!(
            "times out on {} generated inputs",
            r.generated_timeouts.len()
        ));
    }
    if !r.determinism.failures.is_empty() {
        performance.push(format!(
            "nondeterministic on {} inputs",
            r.determinism.failures.len()
        ));
    }
    if let Some(m) = &r.mutants {
        let slipped = m.slipped();
        if !slipped.is_empty() {
            obligation.push(format!(
                "{} slipped ({})",
                plural(slipped.len(), "mutant", "mutants"),
                slipped
                    .iter()
                    .map(|s| s.mutant.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let open = m.open();
        if !open.is_empty() {
            let gaps = gaps(m).len();
            obligation.push(format!(
                "{}{}",
                plural(
                    open.len(),
                    "underdetermined input",
                    "underdetermined inputs"
                ),
                if open.len() > 1 {
                    format!(", {}", plural(gaps, "gap", "gaps"))
                } else {
                    String::new()
                }
            ));
        }
    }
    let dead = r.dead_accepts();
    if dead > 0 {
        obligation.push(plural(dead, "dead accept", "dead accepts"));
    }
    (obligation, performance)
}

/// The first line of a report: the verdict, and whose problem anything is.
pub fn verdict_line(r: &Report) -> String {
    let (obligation, performance) = party_summaries(r);
    if obligation.is_empty() && performance.is_empty() {
        let claims = r.properties.len() + r.frame.len();
        let mutants = match &r.mutants {
            Some(m) => format!(
                "{} of {} mutants apply, none slipped",
                m.applicable(),
                m.mutants.len()
            ),
            None => "mutants not run".to_string(),
        };
        format!(
            "{}  ACCEPTED  examples {}/{}; {} clean over {} runs; {mutants}",
            r.obligation,
            r.examples
                .iter()
                .filter(|e| e.verdict == Verdict::Pass)
                .count(),
            r.examples.len(),
            plural(claims, "clause", "clauses"),
            r.runs
        )
    } else {
        format!(
            "{}  NOT ACCEPTED  obligation: {}  performance: {}",
            r.obligation,
            if obligation.is_empty() {
                "nothing found".to_string()
            } else {
                obligation.join("; ")
            },
            if performance.is_empty() {
                "passes".to_string()
            } else {
                performance.join("; ")
            }
        )
    }
}

/// One claim line: pass with its counts, a finding with its cause number,
/// or the binding verdicts of slice 1.
fn claim_line(
    s: &mut String,
    label: &str,
    verdict: &PropVerdict,
    runs: usize,
    findings: usize,
    cause: Option<usize>,
) {
    // No example to run a claim on is not a finding: with generated runs
    // the claim is evidenced like any other, and "skip" means no runs at all.
    let verdict = match verdict {
        PropVerdict::Skipped if runs > 0 => &PropVerdict::Pass,
        v => v,
    };
    match verdict {
        PropVerdict::Pass if findings == 0 => {
            s.push_str(&format!("  pass  {label}  ({runs} runs, 0 findings)\n"))
        }
        PropVerdict::Pass => s.push_str(&format!(
            "  finding  {label}  ({runs} runs, {findings} findings; cause {})\n",
            cause.map(|c| (c + 1).to_string()).unwrap_or_default()
        )),
        PropVerdict::Fail { detail, .. } => {
            s.push_str(&format!("  FAIL  {label}\n        {detail}\n"))
        }
        PropVerdict::Error(m) => s.push_str(&format!("  ERROR  {label}\n        {m}\n")),
        PropVerdict::Skipped => s.push_str(&format!("  skip  {label}\n")),
    }
}

fn cause_of(r: &Report, line: usize, label_prefix: &str) -> Option<usize> {
    r.causes.iter().position(|c| {
        c.clauses
            .iter()
            .any(|(l, lab, _)| *l == line && lab.starts_with(label_prefix))
    })
}

fn cause_block(s: &mut String, index: usize, c: &Cause) {
    let clauses = c
        .clauses
        .iter()
        .map(|(_, l, _)| l.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let where_ = format!("{} of {} generated inputs", c.inputs, c.generated);
    match c.party {
        Party::Obligation => {
            let closed = match (&c.suggest, &c.note) {
                (Some(sg), _) => format!("closed by {sg}"),
                (None, Some(n)) => format!("no refinement closes it: {n}"),
                (None, None) => "no refinement found".to_string(),
            };
            let searched = if c.candidates_tried > 0 {
                format!(
                    "; no output among {} candidates passes every clause",
                    c.candidates_tried
                )
            } else {
                String::new()
            };
            s.push_str(&format!(
                "  UNSATISFIABLE ON  cause {}: {where_}; {closed}{searched}; {}: {clauses}\n",
                index + 1,
                plural(c.clauses.len(), "clause", "clauses")
            ));
        }
        Party::Performance => {
            s.push_str(&format!(
                "  PERFORMANCE FAILS  cause {}: {clauses} on {where_}; an output that passes every clause exists\n",
                index + 1
            ));
        }
        Party::Undetermined => {
            s.push_str(&format!(
                "  FALSE ON GENERATED INPUT  cause {}: {clauses} on {where_}; party not determined (search budget 0)\n",
                index + 1
            ));
        }
    }
    s.push_str(&format!("        in: {}\n", sur_inputs(&c.witness.inputs)));
    s.push_str(&format!(
        "        out: {}    # the performance's\n",
        c.witness.output
    ));
    if let Some(e) = &c.witness.error {
        s.push_str(&format!("        {e}\n"));
    }
    if let Some(p) = &c.passing {
        s.push_str(&format!("        out: {p}    # passes every clause\n"));
    }
}

/// A gap's tail: how many inputs show it, and the slipped mutants that
/// return one of its outputs.
fn gap_tail(g: &Gap) -> String {
    let mut s = format!("{} show it", plural(g.inputs.len(), "input", "inputs"));
    if !g.mutants.is_empty() {
        let names: Vec<String> = g
            .mutants
            .iter()
            .map(|m| format!("`{}`", m.name()))
            .collect();
        s.push_str(&format!(
            "; {} return{} one of the two",
            names.join(" and "),
            if names.len() == 1 { "s" } else { "" }
        ));
    }
    s
}

/// A gap in one line: where the two outputs differ, the inputs and
/// mutants that show it, and the smallest input.
fn gap_line(r: &MutantReport, g: &Gap) -> String {
    format!(
        "{}; {}; the smallest in: {}",
        g.says(),
        gap_tail(g),
        short(&sur_inputs(&g.witness(r).inputs), 80)
    )
}

fn two_outputs(s: &mut String, u: &surety_check::Underdetermined) {
    s.push_str(&format!("        in: {}\n", sur_inputs(&u.inputs)));
    s.push_str(&format!(
        "        out: {}    # {}\n",
        u.a,
        if u.a_is_expected {
            format!("example {}", u.example.unwrap_or(0))
        } else {
            "passes every clause".to_string()
        }
    ));
    s.push_str(&format!(
        "        out: {}    # also passes every clause\n",
        u.b
    ));
}

/// A plain-text rendering of a report. `full` adds the bucket detail.
/// The reviewer's reading list, at the end of the report: the clauses no
/// mutant can violate and the accepts that fired, each with one input and
/// the output the performance gave.
fn look_here_text(r: &Report) -> String {
    let clauses = r
        .look_here
        .iter()
        .filter(|l| matches!(l.kind, LookKind::Uncaught(_)))
        .count();
    let accepts = r.look_here.len() - clauses;
    let mut parts = Vec::new();
    if clauses > 0 {
        parts.push(format!(
            "{} no mutant can violate",
            plural(clauses, "clause", "clauses")
        ));
    }
    if accepts > 0 {
        parts.push(format!(
            "{} that fired",
            plural(accepts, "accept", "accepts")
        ));
    }
    let mut s = format!(
        "  look here  {}: {}\n",
        plural(r.look_here.len(), "place", "places"),
        parts.join(", ")
    );
    for l in &r.look_here {
        let (label, why) = match &l.kind {
            LookKind::Uncaught(c) => (
                match c.line() {
                    Some(line) => format!("clause (line {line}): {}", c.text()),
                    None => format!("clause: {}", c.text()),
                },
                "no mutant can violate it".to_string(),
            ),
            LookKind::Accept {
                line,
                text,
                satisfied,
                ..
            } => (
                format!("accept (line {line}): {text}"),
                format!("fires on {satisfied} inputs"),
            ),
        };
        s.push_str(&format!(
            "    {label}    # {why}\n        in: {}\n",
            sur_inputs(&l.inputs)
        ));
        if let Some(o) = &l.output {
            s.push_str(&format!("        out: {o}    # the performance's\n"));
        }
    }
    s
}

pub fn render_with(report: &Report, full: bool) -> String {
    let mut s = String::new();
    s.push_str(&verdict_line(report));
    s.push('\n');
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
        claim_line(
            &mut s,
            &format!("property (line {}): {}", p.line, p.text),
            &p.verdict,
            p.runs,
            p.findings(),
            cause_of(report, p.line, "property"),
        );
    }
    for f in &report.frame {
        claim_line(
            &mut s,
            &format!(
                "frame {}: preserves {} by default",
                f.description,
                f.preserved.join(", ")
            ),
            &f.verdict,
            f.runs,
            f.findings(),
            cause_of(report, f.line, "frame"),
        );
    }
    for g in &report.graph_refs {
        s.push_str(&format!("  graph reference: {g}\n"));
    }
    for (i, c) in report.causes.iter().enumerate() {
        cause_block(&mut s, i, c);
    }
    if let Some(f) = report.generated_boundary.first() {
        let n = report.generated_boundary.len();
        match &report.boundary_suggest {
            Some(sg) => s.push_str(&format!(
                "  UNSATISFIABLE ON  the output type: {n} of {} generated inputs force an output outside it; closed by {sg}\n",
                report.generated_inputs
            )),
            None => s.push_str(&format!(
                "  PERFORMANCE FAILS  returns a value outside the type on {n} of {} generated inputs\n",
                report.generated_inputs
            )),
        }
        s.push_str(&format!(
            "        in: {}\n        {}\n",
            sur_inputs(&f.inputs),
            f.message
        ));
    }
    for f in &report.generated_raised {
        s.push_str(&format!(
            "  PERFORMANCE FAILS  raised on generated input {}: {}\n",
            sur_inputs(&f.inputs),
            last_line(&f.message)
        ));
    }
    for f in &report.generated_timeouts {
        s.push_str(&format!(
            "  PERFORMANCE FAILS  generated input {}: {}\n",
            sur_inputs(&f.inputs),
            f.message
        ));
    }
    for f in &report.determinism.failures {
        s.push_str(&format!(
            "  PERFORMANCE FAILS  nondeterministic on {}: {}\n",
            sur_inputs(&f.inputs),
            f.message
        ));
    }
    if let Some(m) = &report.mutants {
        for run in m.slipped() {
            s.push_str(&format!("  SLIPPED  {}\n", slipped_line(run)));
            if let Some(sh) = &run.shown {
                s.push_str(&format!("        in: {}\n", sur_inputs(&sh.inputs)));
                if let Some(e) = &sh.expected {
                    s.push_str(&format!(
                        "        out: {e}    # example {}\n",
                        sh.example.unwrap_or(0)
                    ));
                }
                s.push_str(&format!(
                    "        out: {}    # {}\n",
                    sh.output,
                    run.mutant.name()
                ));
            }
        }
        for g in gaps(m) {
            s.push_str(&format!(
                "  UNDERDETERMINED  {}; {}; the smallest:\n",
                g.says(),
                gap_tail(&g)
            ));
            two_outputs(&mut s, g.witness(m));
        }
        for u in m.accepted() {
            let line = u
                .accepted_by
                .and_then(|i| report.accepts.get(i))
                .map(|a| a.line)
                .unwrap_or(0);
            s.push_str(&format!(
                "  ACCEPTED  two outputs pass every clause; accept (line {line})\n"
            ));
            two_outputs(&mut s, u);
        }
    }
    for a in report.accepts.iter().filter(|a| a.dead()) {
        s.push_str(&format!(
            "  DEAD ACCEPT  accept (line {}): {} holds on no example or generated input\n",
            a.line, a.text
        ));
    }
    s.push_str(&render_evidence_with(report, full));
    if !report.look_here.is_empty() {
        s.push_str(&look_here_text(report));
    }
    s
}

pub fn render(report: &Report) -> String {
    render_with(report, false)
}

/// The evidence record, one line per kind; every count agrees with the body.
pub fn render_evidence(r: &Report) -> String {
    render_evidence_with(r, false)
}

pub fn render_evidence_with(r: &Report, full: bool) -> String {
    let mut s = String::new();
    let ex_pass = r
        .examples
        .iter()
        .filter(|e| e.verdict == Verdict::Pass)
        .count();
    let claims = r.properties.len() + r.frame.len();
    let with_findings = r
        .properties
        .iter()
        .filter(|p| {
            p.findings() > 0 || !matches!(p.verdict, PropVerdict::Pass | PropVerdict::Skipped)
        })
        .count()
        + r.frame
            .iter()
            .filter(|f| {
                f.findings() > 0 || !matches!(f.verdict, PropVerdict::Pass | PropVerdict::Skipped)
            })
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
    let mut parties: Vec<&str> = Vec::new();
    if r.causes.iter().any(|c| c.party == Party::Obligation) {
        parties.push("the obligation's");
    }
    if r.causes.iter().any(|c| c.party == Party::Performance) {
        parties.push("the performance's");
    }
    if r.causes.iter().any(|c| c.party == Party::Undetermined) {
        parties.push("party not determined");
    }
    s.push_str(&format!(
        "    properties:       {} of {claims} clean over {} runs{}\n",
        claims - with_findings,
        r.runs,
        if with_findings == 0 {
            String::new()
        } else {
            format!(
                "; {with_findings} with findings, {}{}",
                plural(r.causes.len(), "cause", "causes"),
                if parties.is_empty() {
                    String::new()
                } else {
                    format!(", {}", parties.join(" and "))
                }
            )
        }
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
    match &r.mutants {
        Some(m) => {
            let slipped = m.slipped();
            let open = m.open();
            let accepted = m.accepted().len();
            let groups = gaps(m).len();
            s.push_str(&format!(
                "    underdetermination: {} of {} mutants apply; {} slipped{}; {} underdetermined inputs{}{}\n",
                m.applicable(),
                m.mutants.len(),
                slipped.len(),
                if slipped.is_empty() {
                    String::new()
                } else {
                    format!(
                        " ({})",
                        slipped.iter().map(|s| s.mutant.name()).collect::<Vec<_>>().join(", ")
                    )
                },
                open.len(),
                if open.len() > 1 {
                    format!(", {}", plural(groups, "gap", "gaps"))
                } else {
                    String::new()
                },
                if accepted > 0 {
                    format!("; {accepted} accepted")
                } else {
                    String::new()
                }
            ));
        }
        None => s.push_str("    underdetermination: not run (surety mutants)\n"),
    }
    if !r.accepts.is_empty() {
        let parts: Vec<String> = r
            .accepts
            .iter()
            .map(|a| format!("line {} fires on {} inputs", a.line, a.satisfied))
            .collect();
        s.push_str(&format!(
            "    accepts:          {}; {}\n",
            r.accepts.len(),
            parts.join(", ")
        ));
    }
    s.push_str(&format!(
        "    obligation:       {}\n    performance:      {}\n",
        r.obligation_hash,
        r.inspection
            .as_ref()
            .map(|i| i.hash.as_str())
            .unwrap_or("-")
    ));
    if r.generated_inputs > 0 || !r.buckets.is_empty() {
        s.push_str(&format!("    generated:        {}\n", generated_text(r)));
        if full {
            s.push_str(&format!(
                "    buckets:          {}\n",
                bucket_text(&r.buckets)
            ));
        }
    }
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

fn party_text(p: Party) -> &'static str {
    match p {
        Party::Obligation => "obligation",
        Party::Performance => "performance",
        Party::Undetermined => "undetermined",
    }
}

/// The machine form: one object per finding, then one `evidence` object.
pub fn json_lines(r: &Report) -> Vec<Json> {
    let mut out = Vec::new();
    for e in &r.examples {
        let (kind, message) = match &e.verdict {
            Verdict::Pass => continue,
            Verdict::Boundary(m) => ("boundary", m),
            Verdict::Mismatch(m) => ("example_mismatch", m),
            Verdict::Error(m) => ("error", m),
        };
        let mut f = finding(kind, r, None, &e.inputs);
        f["party"] = json!("performance");
        f["example"] = json!(e.index + 1);
        f["message"] = json!(message);
        out.push(with_outputs(f, e.expected.as_ref(), e.output.as_ref()));
    }
    for p in &r.properties {
        match &p.verdict {
            PropVerdict::Fail {
                example, detail, ..
            } => {
                let ex = &r.examples[*example];
                let mut f = finding("property_failed", r, Some((p.line, &p.text)), &ex.inputs);
                f["party"] = json!("performance");
                f["example"] = json!(example + 1);
                f["message"] = json!(detail);
                out.push(with_outputs(f, ex.expected.as_ref(), ex.output.as_ref()));
            }
            PropVerdict::Error(m) => {
                let mut f = finding("error", r, Some((p.line, &p.text)), &[]);
                f["party"] = json!("performance");
                f["message"] = json!(m);
                out.push(f);
            }
            _ => {}
        }
    }
    for fr in &r.frame {
        if let PropVerdict::Fail {
            example, detail, ..
        } = &fr.verdict
        {
            let ex = &r.examples[*example];
            let text = format!(
                "frame {}: preserves {} by default",
                fr.description,
                fr.preserved.join(", ")
            );
            let mut f = finding("property_failed", r, Some((fr.line, &text)), &ex.inputs);
            f["party"] = json!("performance");
            f["example"] = json!(example + 1);
            f["message"] = json!(detail);
            out.push(with_outputs(f, ex.expected.as_ref(), ex.output.as_ref()));
        }
    }
    for (i, c) in r.causes.iter().enumerate() {
        let kind = match c.party {
            Party::Obligation => "unsatisfiable_on",
            Party::Performance | Party::Undetermined => "property_failed",
        };
        let (line, _, text) = &c.clauses[0];
        let mut f = finding(kind, r, Some((*line, text)), &c.witness.inputs);
        f["party"] = json!(party_text(c.party));
        f["cause"] = json!(i + 1);
        f["generated"] = json!(true);
        f["clauses"] = json!(
            c.clauses
                .iter()
                .map(|(l, _, t)| json!({ "line": l, "text": t }))
                .collect::<Vec<_>>()
        );
        f["inputs_failing"] = json!(c.inputs);
        f["generated_inputs"] = json!(c.generated);
        f["candidates_tried"] = json!(c.candidates_tried);
        if let Some(e) = &c.witness.error {
            f["message"] = json!(e);
        }
        if let Some(sg) = &c.suggest {
            f["suggest"] = json!(sg);
        }
        if let Some(n) = &c.note {
            f["note"] = json!(n);
        }
        if let Some(p) = &c.passing {
            f["passing"] = to_json(p);
        }
        out.push(with_outputs(f, None, Some(&c.witness.output)));
    }
    for (kind, list, party) in [
        (
            "boundary",
            &r.generated_boundary,
            if r.boundary_suggest.is_some() {
                "obligation"
            } else {
                "performance"
            },
        ),
        ("raised", &r.generated_raised, "performance"),
        ("timeout", &r.generated_timeouts, "performance"),
        ("nondeterministic", &r.determinism.failures, "performance"),
    ] {
        for fl in list {
            let mut f = finding(kind, r, None, &fl.inputs);
            f["party"] = json!(party);
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
        .filter(|p| matches!(p.verdict, PropVerdict::Pass | PropVerdict::Skipped))
        .count()
        + r.frame
            .iter()
            .filter(|f| matches!(f.verdict, PropVerdict::Pass | PropVerdict::Skipped))
            .count();
    let (obligation, performance) = party_summaries(r);
    let mut ev = json!({
        "kind": "evidence",
        "obligation": r.obligation,
        "obligation_hash": r.obligation_hash,
        "verdict": if obligation.is_empty() && performance.is_empty() { "ACCEPTED" } else { "NOT ACCEPTED" },
        "obligation_findings": obligation,
        "performance_findings": performance,
        "passed": r.passed(),
        "strict_passed": r.strict_passed(),
        "types": { "example_inputs": r.example_inputs, "generated_inputs": r.generated_inputs, "outputs": r.runs, "boundary_failures": r.generated_boundary.len() },
        "examples": { "passed": r.examples.iter().filter(|e| e.verdict == Verdict::Pass).count(), "total": r.examples.len() },
        "properties": { "passed": claims_pass, "total": claims_total, "runs": r.runs, "findings": r.claim_findings(), "causes": r.causes.len(),
            "each": r.properties.iter().map(|p| json!({ "line": p.line, "runs": p.runs, "findings": p.findings() })).collect::<Vec<_>>() },
        "determinism": { "matched": r.determinism.matched, "total": r.determinism.total },
        "totality": { "answered": (r.example_inputs + r.generated_inputs).saturating_sub(r.generated_timeouts.len()), "total": r.example_inputs + r.generated_inputs, "raised": r.generated_raised.len(), "timeout_seconds": r.timeout.as_secs_f64() },
        "purity": "unverified",
        "findings": r.findings(),
        "underdetermination": match &r.mutants {
            Some(m) => json!({ "applicable": m.applicable(), "total": m.mutants.len(), "slipped": m.slipped().len(), "caught": m.caught(), "underdetermined_inputs": m.search.len(), "accepted": m.accepted().len(), "open": m.open().len(), "causes": gaps(m).len() }),
            None => json!("not run"),
        },
        "accepts": r.accepts.iter().map(|a| json!({ "line": a.line, "text": a.text, "satisfied": a.satisfied, "dead": a.dead() })).collect::<Vec<_>>(),
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
    if let Some(m) = &r.mutants {
        out.extend(mutants_json_lines(m).into_iter().filter(|j| {
            j["kind"] == "mutant_slipped"
                || j["kind"] == "underdetermined"
                || j["kind"] == "gap"
                || j["kind"] == "accepted"
        }));
    }
    for a in r.accepts.iter().filter(|a| a.dead()) {
        let mut f = finding("dead_accept", r, Some((a.line, &a.text)), &[]);
        f["party"] = json!("obligation");
        f["message"] = json!("holds on no example or generated input");
        out.push(f);
    }
    for l in &r.look_here {
        let mut f = json!({
            "kind": "look_here",
            "obligation": r.obligation,
            "obligation_hash": r.obligation_hash,
            "inputs": input_json(&l.inputs),
        });
        if let Some(i) = &r.inspection {
            f["performance_hash"] = json!(i.hash);
        }
        match &l.kind {
            LookKind::Uncaught(c) => {
                f["clause"] = clause_json(c);
                f["why"] = json!("no mutant can violate it");
                if r.mutants.as_ref().is_some_and(|m| m.input_only.contains(c)) {
                    f["input_only"] = json!(true);
                }
            }
            LookKind::Accept {
                index, satisfied, ..
            } => {
                f["accept"] = json!(index);
                f["why"] = json!(format!("fires on {satisfied} inputs"));
            }
        }
        if let Some(o) = &l.output {
            f["output"] = to_json(o);
        }
        out.push(f);
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

/// The underdetermination check on one obligation file. The performance is
/// optional: it is only consulted where a property calls the obligation's
/// own name and the active mutant has no value for those inputs, and for
/// the search.
pub fn mutants_file(
    file: &Path,
    performance: Option<&Path>,
    budget: Budget,
) -> Result<MutantReport, String> {
    let (graph, idx) = load_with_siblings(file).map_err(errors_text)?;
    let perf = performance
        .map(Path::to_path_buf)
        .unwrap_or_else(|| file.with_extension("py"));
    let mut checker = checker_for(&graph, idx, file, &perf)?;
    checker.budget = budget;
    Ok(checker.mutants())
}

fn clause_label(c: &Clause) -> String {
    match c {
        Clause::Property { line, text } => format!("line {line}: {text}"),
        Clause::Frame { line, description } => format!("frame {description} (line {line})"),
        Clause::Boundary => "the output type".to_string(),
    }
}

fn clause_lines(cs: &[Clause]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let lines: Vec<String> = cs
        .iter()
        .filter_map(|c| c.line().map(|l| l.to_string()))
        .collect();
    if !lines.is_empty() {
        parts.push(format!(
            "line{} {}",
            if lines.len() == 1 { "" } else { "s" },
            lines.join(", ")
        ));
    }
    if cs.contains(&Clause::Boundary) {
        parts.push("the output type".to_string());
    }
    parts.join(" and ")
}

fn short(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max).collect();
        format!("{cut}…")
    }
}

fn example_catch_text(m: &MutantRun) -> String {
    match m.example_catches.as_slice() {
        [] => "caught by no example".to_string(),
        [one] => format!("caught by example {one} only"),
        many => format!(
            "caught by examples {} only",
            many.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn not_applicable_reason(m: surety_check::Mutant) -> &'static str {
    match m {
        surety_check::Mutant::Empty => "always applies",
        surety_check::Mutant::OkEmpty => "the output is not a Result",
        surety_check::Mutant::Identity => "no single input has the output's shape",
        surety_check::Mutant::Skeleton => "no frame pair gives the output a source",
        surety_check::Mutant::SkeletonPerKey => "no frame pair matches on a field",
        m if m.is_skeleton() => "the skeleton has no list to edit, or no input gave it an element",
        _ => "the identity is not a list, or no input gave it an element to edit",
    }
}

/// One line for a slipping mutant: what it does, what it slipped through,
/// whether an example caught it, and the input that shows it.
fn slipped_line(m: &MutantRun) -> String {
    let shown = m
        .shown
        .as_ref()
        .map(|sh| format!("; in: {}", short(&sur_inputs(&sh.inputs), 80)))
        .unwrap_or_default();
    format!(
        "{}: {}; through {} clause{} ({}); {}{shown}",
        m.mutant.name(),
        m.mutant.describe(),
        m.slipped.len(),
        if m.slipped.len() == 1 { "" } else { "s" },
        clause_lines(&m.slipped),
        example_catch_text(m)
    )
}

/// One line for an input on which two outputs pass every clause.
fn underdetermined_line(u: &surety_check::Underdetermined) -> String {
    format!(
        "in: {}: two outputs pass every clause{}",
        short(&sur_inputs(&u.inputs), 80),
        if u.a_is_expected {
            " (one is the example's)"
        } else {
            ""
        }
    )
}

/// The default view: one line per obligation, one per gap, one per
/// slipping mutant, one per accepted input. `--full` adds the
/// behaviour-first detail.
pub fn render_mutants(r: &MutantReport, full: bool) -> String {
    let mut s = String::new();
    let slipped = r.slipped();
    let gaps = gaps(r);
    s.push_str(&format!(
        "{}  inputs {}+{}  mutants {} of {} apply  slipped {}  caught {}  underdetermined {}{}{}\n",
        r.obligation,
        r.example_inputs,
        r.generated_inputs,
        r.applicable(),
        r.mutants.len(),
        slipped.len(),
        r.caught(),
        r.open().len(),
        if gaps.is_empty() {
            String::new()
        } else {
            format!(" in {}", plural(gaps.len(), "gap", "gaps"))
        },
        if r.accepted().is_empty() {
            String::new()
        } else {
            format!("  accepted {}", r.accepted().len())
        }
    ));
    for g in &gaps {
        s.push_str(&format!("  gap  {}\n", gap_line(r, g)));
    }
    for m in &slipped {
        s.push_str(&format!("  slipped  {}\n", slipped_line(m)));
    }
    for u in r.accepted() {
        s.push_str(&format!("  accepted  {}\n", underdetermined_line(u)));
    }
    if !full {
        return s;
    }
    for m in &slipped {
        s.push_str(&format!(
            "  mutant {} slipped through the properties: {}\n",
            m.mutant.name(),
            m.mutant.describe()
        ));
        if let Some(sh) = &m.shown {
            s.push_str("    e:\n");
            s.push_str(&format!("      in: {}\n", sur_inputs(&sh.inputs)));
            if let Some(e) = &sh.expected {
                s.push_str(&format!(
                    "      out: {e}    # example {}\n",
                    sh.example.unwrap_or(0)
                ));
            }
            s.push_str(&format!(
                "      out: {}    # {}\n",
                sh.output,
                m.mutant.name()
            ));
        }
        s.push_str(&format!(
            "    {}; on {} inputs\n",
            example_catch_text(m),
            m.inputs_tried
        ));
        for c in &m.slipped {
            s.push_str(&format!("    slipped through {}\n", clause_label(c)));
        }
    }
    let mut detail: Vec<(String, &surety_check::Underdetermined)> = gaps
        .iter()
        .map(|g| {
            (
                format!(
                    "  gap: {}\n    {}; the smallest:\n    e:\n",
                    g.says(),
                    gap_tail(g)
                ),
                g.witness(r),
            )
        })
        .collect();
    detail.extend(r.accepted().into_iter().map(|u| {
        (
            "  accepted: two outputs pass every clause, and an `accept` line says so\n    e:\n"
                .to_string(),
            u,
        )
    }));
    for (head, u) in detail {
        s.push_str(&head);
        s.push_str(&format!("      in: {}\n", sur_inputs(&u.inputs)));
        s.push_str(&format!(
            "      out: {}    # {}\n",
            u.a,
            if u.a_is_expected {
                format!("example {}", u.example.unwrap_or(0))
            } else {
                "passes every clause".to_string()
            }
        ));
        s.push_str(&format!(
            "      out: {}    # also passes every clause; {} candidates tried\n",
            u.b, u.candidates_tried
        ));
    }
    for m in r
        .mutants
        .iter()
        .filter(|m| m.verdict == MutantVerdict::Caught)
    {
        let kills: Vec<String> = m
            .kills
            .iter()
            .map(|k| {
                format!(
                    "{} ({} input{})",
                    clause_label(&k.clause),
                    k.count,
                    if k.count == 1 { "" } else { "s" }
                )
            })
            .collect();
        s.push_str(&format!(
            "  mutant {} caught on {} of {} inputs by {}\n",
            m.mutant.name(),
            m.caught_inputs,
            m.inputs_tried,
            kills.join("; ")
        ));
        if !m.slipped.is_empty() {
            s.push_str(&format!(
                "    slipped through anyway: {}\n",
                clause_lines(&m.slipped)
            ));
        }
    }
    for m in r
        .mutants
        .iter()
        .filter(|m| m.verdict == MutantVerdict::NotApplicable)
    {
        s.push_str(&format!(
            "  mutant {} does not apply: {}\n",
            m.mutant.name(),
            not_applicable_reason(m.mutant)
        ));
    }
    if !r.uncaught.is_empty() {
        s.push_str("  clauses that caught no mutant:\n");
        for c in &r.uncaught {
            s.push_str(&format!("    {}\n", clause_label(c)));
        }
    }
    for (c, e) in &r.unevaluable {
        s.push_str(&format!(
            "  the search could not evaluate {}: {}\n",
            clause_label(c),
            last_line(e)
        ));
    }
    s.push_str(&format!(
        "  search: {} inputs, up to {} candidates each\n",
        r.search_inputs, r.search_cap
    ));
    s.push_str(&format!(
        "  generated: {} inputs by list length: {}\n",
        r.generated_inputs,
        bucket_text(&r.buckets)
    ));
    s.push_str(&format!("  obligation: {}\n", r.obligation_hash));
    s
}

fn clause_json(c: &Clause) -> Json {
    match c {
        Clause::Property { line, text } => json!({ "line": line, "text": text }),
        Clause::Frame { line, description } => {
            json!({ "line": line, "text": format!("frame {description}") })
        }
        Clause::Boundary => json!({ "text": "the output type" }),
    }
}

/// A gap as one object: the line that says it, the place and what each
/// output holds there, the smallest input with both outputs in full (flat
/// and laid out to fit a hover), how many inputs show it, and the slipped
/// mutants that return one of them.
fn gap_json(r: &MutantReport, g: &Gap, index: usize) -> Json {
    let w = g.witness(r);
    let d = &g.divergence;
    let mut f = json!({
        "kind": "gap",
        "obligation": r.obligation,
        "obligation_hash": r.obligation_hash,
        "gap": index,
        "says": g.says(),
        "place": d.place(),
        "change": d.kind(),
        "inputs": input_json(&w.inputs),
        "outputs": { "a": to_json(&w.a), "b": to_json(&w.b) },
        "sur": { "a": w.a.to_string(), "b": w.b.to_string() },
        "pretty": { "in": surety_check::pretty_inputs(&w.inputs), "a": surety_check::pretty(&w.a), "b": surety_check::pretty(&w.b) },
        "a_is_expected": w.a_is_expected,
        "shown_by": g.inputs.len(),
        "mutants": g.mutants.iter().map(|m| {
            let mut j = json!({ "mutant": m.name(), "describes": m.describe() });
            if let Some(sh) = r.run(*m).and_then(|run| run.shown.as_ref()) {
                j["returns"] = json!(sh.output.to_string());
            }
            j
        }).collect::<Vec<_>>(),
    });
    if let Some((a, b)) = d.parts() {
        f["at"] = json!({ "a": a, "b": b });
    }
    if let Some(n) = w.example {
        f["example"] = json!(n);
    }
    f
}

/// The machine form of the underdetermination check: one object per
/// mutant, one per underdetermined input, one per gap, then a summary.
pub fn mutants_json_lines(r: &MutantReport) -> Vec<Json> {
    let mut out = Vec::new();
    let gaps = gaps(r);
    for m in &r.mutants {
        let kind = match m.verdict {
            MutantVerdict::Slipped => "mutant_slipped",
            MutantVerdict::Caught => "mutant_caught",
            MutantVerdict::NotApplicable => "mutant_not_applicable",
        };
        let mut f = json!({
            "kind": kind,
            "obligation": r.obligation,
            "obligation_hash": r.obligation_hash,
            "candidate": { "mutant": m.mutant.name(), "describes": m.mutant.describe() },
            "inputs_tried": m.inputs_tried,
            "caught_inputs": m.caught_inputs,
            "slipped": m.slipped.iter().map(clause_json).collect::<Vec<_>>(),
        });
        let mut caught_by: Vec<Json> = m
            .example_catches
            .iter()
            .map(|n| json!(format!("example {n}")))
            .collect();
        for k in &m.kills {
            let mut c = clause_json(&k.clause);
            c["inputs"] = json!(k.count);
            if let Some(msg) = &k.message {
                c["message"] = json!(msg);
            }
            caught_by.push(c);
        }
        f["caught_by"] = Json::Array(caught_by);
        if let Some(g) = gaps.iter().position(|g| g.mutants.contains(&m.mutant)) {
            f["gap"] = json!(g);
        }
        if let Some(sh) = &m.shown {
            f["inputs"] = input_json(&sh.inputs);
            f = with_outputs(f, sh.expected.as_ref(), Some(&sh.output));
            if let Some(n) = sh.example {
                f["example"] = json!(n);
            }
        }
        out.push(f);
    }
    let gap_of = |i: usize| gaps.iter().position(|g| g.inputs.contains(&i));
    for (i, u) in r.search.iter().enumerate() {
        let mut f = json!({
            "kind": if u.accepted_by.is_some() { "accepted" } else { "underdetermined" },
            "obligation": r.obligation,
            "obligation_hash": r.obligation_hash,
            "inputs": input_json(&u.inputs),
            "outputs": { "a": to_json(&u.a), "b": to_json(&u.b) },
            "a_is_expected": u.a_is_expected,
            "candidates_tried": u.candidates_tried,
        });
        if let Some(n) = u.example {
            f["example"] = json!(n);
        }
        if let Some(i) = u.accepted_by {
            f["accept"] = json!(i);
        }
        if let Some(m) = u.same_as {
            f["same_as"] = json!(m.name());
        }
        if let Some(g) = gap_of(i) {
            f["gap"] = json!(g);
        }
        out.push(f);
    }
    for (i, g) in gaps.iter().enumerate() {
        out.push(gap_json(r, g, i));
    }
    out.push(json!({
        "kind": "mutant_report",
        "obligation": r.obligation,
        "obligation_hash": r.obligation_hash,
        "example_inputs": r.example_inputs,
        "generated_inputs": r.generated_inputs,
        "mutants": { "total": r.mutants.len(), "applicable": r.applicable(), "slipped": r.slipped().len(), "caught": r.caught() },
        "underdetermined_inputs": r.search.len(),
        "gaps": gaps.len(),
        "accepted_inputs": r.accepted().len(),
        "underdetermined": r.underdetermined(),
        "uncaught": r.uncaught.iter().map(|c| {
            let mut j = clause_json(c);
            if r.input_only.contains(c) {
                j["input_only"] = json!(true);
            }
            j
        }).collect::<Vec<_>>(),
        "unevaluable": r.unevaluable.iter().map(|(c, e)| { let mut j = clause_json(c); j["message"] = json!(last_line(e)); j }).collect::<Vec<_>>(),
        "search": { "inputs": r.search_inputs, "candidates_cap": r.search_cap },
        "generated": r.buckets.iter().map(|b| json!({ "lengths": b.lengths, "taken": b.taken, "space": b.space.to_string(), "exhaustive": b.exhaustive })).collect::<Vec<_>>(),
    }));
    out
}

pub fn render_mutants_json(r: &MutantReport) -> String {
    mutants_json_lines(r)
        .iter()
        .map(|j| j.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

// ---- the behavioural diff ----

fn side<'g>(
    graph: &'g Graph,
    idx: usize,
    perf: &Path,
    budget: Budget,
) -> Result<Checker<'g>, String> {
    // Graph references resolve beside the performance, not beside the
    // obligation: each side of a diff brings its own siblings.
    let dir = perf.parent().unwrap_or(Path::new("."));
    let mut c = Checker::new(graph, idx, perf, dir, TIMEOUT)?;
    c.budget = budget;
    Ok(c)
}

/// Two performances of one obligation file, diffed on the file's examples
/// and generated inputs under `budget`.
pub fn diff_files(
    file: &Path,
    old: &Path,
    new: &Path,
    budget: Budget,
) -> Result<DiffReport, String> {
    let (graph, idx) = load_with_siblings(file).map_err(errors_text)?;
    graph
        .check_file(idx)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    let mut a = side(&graph, idx, old, budget.clone())?;
    let mut b = side(&graph, idx, new, budget)?;
    Ok(surety_check::diff::diff(&mut a, &mut b))
}

fn outcome_text(o: &Outcome) -> String {
    match o {
        Outcome::Output(v) => v.to_string(),
        Outcome::Raised(m) => format!("raised: {}", last_line(m)),
        Outcome::Boundary(b) => format!("rejected at the boundary: {b}"),
        Outcome::Timeout => "timed out".to_string(),
    }
}

fn fails_text(cs: &[Clause]) -> String {
    match cs {
        [Clause::Property { line, text }] => format!("line {line} ({})", short(text, 70)),
        [Clause::Frame { line, description }] => format!("frame {description} (line {line})"),
        [Clause::Boundary] => "the output type".to_string(),
        _ => clause_lines(cs),
    }
}

fn side_text(label: &str, o: &Outcome, fails: &[Clause]) -> String {
    match o {
        Outcome::Output(_) if fails.is_empty() => format!("{label} passes"),
        Outcome::Output(_) => format!("{label} fails {}", fails_text(fails)),
        Outcome::Raised(_) => format!("{label} raised"),
        Outcome::Boundary(_) => format!("{label} was rejected at the boundary"),
        Outcome::Timeout => format!("{label} timed out"),
    }
}

fn difference_label(d: &Difference) -> String {
    let old = || side_text("old", &d.old, &d.old_fails);
    let new = || side_text("new", &d.new, &d.new_fails);
    match d.class {
        Class::Regression => format!("REGRESSION  {}; {}", new(), old()),
        Class::Fix => format!("FIX  {}; {}", old(), new()),
        Class::NeitherPasses => format!("NEITHER PASSES  {}; {}", old(), new()),
        Class::Within => match d.accept {
            Some(line) => format!(
                "WITHIN THE OBLIGATION  both outputs pass every clause; accept (line {line}) covers this input"
            ),
            None => "WITHIN THE OBLIGATION  both outputs pass every clause and no accept line covers this input: the obligation does not decide between them".to_string(),
        },
    }
}

fn count_text(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The first line of a diff report: the verdict and the counts.
pub fn diff_verdict_line(r: &DiffReport) -> String {
    let inputs = format!(
        "{} inputs ({} examples, {} generated)",
        r.inputs(),
        r.examples,
        r.generated
    );
    if r.same() {
        return format!("{}  SAME  old and new agree on all {inputs}", r.obligation);
    }
    let mut parts = Vec::new();
    let regressions = r.count(Class::Regression);
    if regressions > 0 {
        parts.push(count_text(regressions, "regression", "regressions"));
    }
    let neither = r.count(Class::NeitherPasses);
    if neither > 0 {
        parts.push(format!("{neither} where neither passes"));
    }
    let fixes = r.count(Class::Fix);
    if fixes > 0 {
        parts.push(count_text(fixes, "fix", "fixes"));
    }
    let within = r.count(Class::Within);
    if within > 0 {
        parts.push(format!(
            "{within} within the obligation ({} accepted, {} undecided)",
            r.accepted(),
            r.undecided()
        ));
    }
    format!(
        "{}  DIFFERS  on {} of {inputs}: {}",
        r.obligation,
        r.differences.len(),
        parts.join(", ")
    )
}

/// The text report: the verdict line, both performances by path and hash,
/// the first three differences (all with `full`), the obligation's hash.
pub fn render_diff(r: &DiffReport, full: bool) -> String {
    let mut s = String::new();
    s.push_str(&diff_verdict_line(r));
    s.push('\n');
    let hash = |h: &Option<String>| h.clone().unwrap_or_else(|| "(not readable)".to_string());
    s.push_str(&format!(
        "  old: {}  {}\n",
        r.old.display(),
        hash(&r.old_hash)
    ));
    s.push_str(&format!(
        "  new: {}  {}\n",
        r.new.display(),
        hash(&r.new_hash)
    ));
    let shown = if full {
        r.differences.len()
    } else {
        r.differences.len().min(3)
    };
    for d in &r.differences[..shown] {
        s.push_str(&format!("  {}\n", difference_label(d)));
        s.push_str(&format!("        in: {}", sur_inputs(&d.inputs)));
        if let Some(n) = d.example {
            s.push_str(&format!("    # example {n}"));
        }
        s.push('\n');
        s.push_str(&format!("        old: {}\n", outcome_text(&d.old)));
        s.push_str(&format!("        new: {}\n", outcome_text(&d.new)));
    }
    if shown < r.differences.len() {
        s.push_str(&format!(
            "  {shown} of {} shown, regressions and smallest first; --full for all\n",
            r.differences.len()
        ));
    }
    s.push_str(&format!("  obligation: {}\n", r.obligation_hash));
    s
}

fn outcome_json(o: &Outcome) -> Json {
    match o {
        Outcome::Output(v) => json!({ "output": to_json(v) }),
        Outcome::Raised(m) => json!({ "raised": last_line(m) }),
        Outcome::Boundary(b) => json!({ "boundary": b.to_string() }),
        Outcome::Timeout => json!({ "timeout": true }),
    }
}

/// One object per difference, then one summary object.
pub fn diff_json_lines(r: &DiffReport) -> Vec<Json> {
    let mut out: Vec<Json> = r
        .differences
        .iter()
        .map(|d| {
            json!({
                "kind": "difference",
                "class": d.class.name(),
                "example": d.example,
                "inputs": input_json(&d.inputs),
                "old": outcome_json(&d.old),
                "new": outcome_json(&d.new),
                "old_fails": d.old_fails.iter().map(clause_json).collect::<Vec<_>>(),
                "new_fails": d.new_fails.iter().map(clause_json).collect::<Vec<_>>(),
                "accept": d.accept,
            })
        })
        .collect();
    out.push(json!({
        "kind": "summary",
        "obligation": r.obligation,
        "obligation_hash": r.obligation_hash,
        "verdict": if r.same() { "same" } else { "differs" },
        "inputs": r.inputs(),
        "examples": r.examples,
        "generated": r.generated,
        "agree": r.agree,
        "differences": r.differences.len(),
        "regressions": r.count(Class::Regression),
        "neither_passes": r.count(Class::NeitherPasses),
        "fixes": r.count(Class::Fix),
        "within": r.count(Class::Within),
        "accepted": r.accepted(),
        "undecided": r.undecided(),
        "old": { "path": r.old.display().to_string(), "hash": r.old_hash },
        "new": { "path": r.new.display().to_string(), "hash": r.new_hash },
    }));
    out
}

pub fn render_diff_json(r: &DiffReport) -> String {
    let mut s = String::new();
    for j in diff_json_lines(r) {
        s.push_str(&j.to_string());
        s.push('\n');
    }
    s
}

// ---- the graph: pin, show, status, verify ----

/// What a record keeps of a report: its verdict line and its `--json` lines.
pub fn rendered(report: &Report) -> Rendered {
    Rendered {
        verdict: verdict_line(report),
        lines: json_lines(report),
    }
}

pub fn pin_line(p: &Pinned) -> String {
    let unchanged = if p.wrote { "" } else { " (unchanged)" };
    match &p.performance {
        Some(h) => format!(
            "pinned {} {} python {}: {}{unchanged}",
            p.name,
            p.obligation,
            h,
            p.state.word()
        ),
        None => format!(
            "pinned {} {}: {}{unchanged}",
            p.name,
            p.obligation,
            PinState::Open.word()
        ),
    }
}

pub fn pin_json(p: &Pinned) -> Json {
    json!({
        "kind": "pinned",
        "name": p.name,
        "obligation": p.obligation,
        "target": "python",
        "performance": p.performance,
        "state": p.state.word(),
        "wrote": p.wrote,
    })
}

fn hop_text(hop: &[(String, State)]) -> String {
    if hop.is_empty() {
        return "none".to_string();
    }
    hop.iter()
        .map(|(n, s)| format!("{n} ({})", s.word()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A node as `show` prints it: the model's query, for a reader.
pub fn render_show(v: &NodeView) -> String {
    let mut s = String::new();
    let name = v
        .names
        .first()
        .cloned()
        .unwrap_or_else(|| "(unnamed)".to_string());
    s.push_str(&format!(
        "{name}  {}  {}\n",
        v.state.word().to_uppercase(),
        v.hash
    ));
    for other in v.names.iter().skip(1) {
        s.push_str(&format!("  also named: {other}\n"));
    }
    if let Some(src) = &v.source {
        s.push_str(&format!("  source: {src}\n"));
    }
    if let Some(n) = &v.note {
        s.push_str(&format!("  note: {n}\n"));
    }
    for sent in &v.sentences {
        s.push_str(&format!("  # {sent}\n"));
    }
    s.push_str(&format!(
        "  presentations: {}\n",
        if v.presentations.is_empty() {
            "none".to_string()
        } else {
            v.presentations.join(", ")
        }
    ));
    if v.performances.is_empty() {
        s.push_str("  performances: none (open)\n");
    }
    for p in &v.performances {
        s.push_str(&format!(
            "  {} {}  {}\n        {}\n",
            p.target,
            p.hash,
            if p.verified { "verified" } else { "findings" },
            p.verdict
        ));
    }
    s.push_str(&format!("  calls: {}\n", hop_text(&v.callees)));
    s.push_str(&format!("  called by: {}\n", hop_text(&v.callers)));
    if !v.uses.is_empty() {
        s.push_str(&format!("  uses types from: {}\n", hop_text(&v.uses)));
    }
    s
}

fn hop_json(hop: &[(String, State)]) -> Json {
    Json::Array(
        hop.iter()
            .map(|(n, s)| json!({ "name": n, "state": s.key() }))
            .collect(),
    )
}

pub fn show_json(v: &NodeView) -> Json {
    json!({
        "kind": "node",
        "name": v.names.first(),
        "names": v.names,
        "hash": v.hash,
        "state": v.state.key(),
        "source": v.source,
        "note": v.note,
        "sentences": v.sentences,
        "comments": v.comments,
        "presentations": v.presentations,
        "performances": v.performances.iter().map(|p| json!({
            "target": p.target, "hash": p.hash,
            "state": if p.verified { "verified" } else { "findings" },
            "verdict": p.verdict,
        })).collect::<Vec<_>>(),
        "calls": hop_json(&v.callees),
        "called_by": hop_json(&v.callers),
        "uses_types_from": hop_json(&v.uses),
    })
}

pub fn render_status(rows: &[StatusRow]) -> String {
    let mut s = String::new();
    let mut counts = std::collections::BTreeMap::new();
    for r in rows {
        *counts.entry(r.state.key()).or_insert(0usize) += 1;
        s.push_str(&format!(
            "{:<9} {}  {}  {}\n",
            r.state.key(),
            r.name,
            r.hash.get(..12).unwrap_or(&r.hash),
            r.source
        ));
        if let State::Stale { now } = &r.state {
            s.push_str(&format!("          now {now}\n"));
        }
    }
    let summary: Vec<String> = ["verified", "findings", "open", "stale", "missing"]
        .iter()
        .filter_map(|k| counts.get(k).map(|n| format!("{n} {k}")))
        .collect();
    s.push_str(&format!(
        "{} name{}: {}\n",
        rows.len(),
        if rows.len() == 1 { "" } else { "s" },
        if summary.is_empty() {
            "none".to_string()
        } else {
            summary.join(", ")
        }
    ));
    s
}

pub fn status_json(rows: &[StatusRow]) -> Vec<Json> {
    rows.iter()
        .map(|r| {
            let mut j = json!({
                "kind": "name", "name": r.name, "obligation": r.hash,
                "source": r.source, "state": r.state.key(),
            });
            if let State::Stale { now } = &r.state {
                j["now"] = json!(now);
            }
            j
        })
        .collect()
}

pub fn render_verify(vs: &[Verification], rerun: bool) -> String {
    let mut s = String::new();
    let mut failed = 0;
    for v in vs {
        let node = v.node.get(..12).unwrap_or(&v.node);
        let perf = v.performance.get(..12).unwrap_or(&v.performance);
        match &v.result {
            Ok(()) => s.push_str(&format!("ok    {}  {node}  python {perf}\n", v.name)),
            Err(e) => {
                failed += 1;
                s.push_str(&format!("FAIL  {}  {node}  python {perf}: {e}\n", v.name));
            }
        }
    }
    s.push_str(&format!(
        "{} of {} record{} {}{}\n",
        vs.len() - failed,
        vs.len(),
        if vs.len() == 1 { "" } else { "s" },
        if rerun { "reproduced" } else { "admitted" },
        if failed > 0 {
            format!("; {failed} failed")
        } else {
            String::new()
        }
    ));
    s
}

pub fn verify_json(vs: &[Verification]) -> Vec<Json> {
    vs.iter()
        .map(|v| {
            json!({
                "kind": "record", "name": v.name, "obligation": v.node, "target": "python",
                "performance": v.performance,
                "ok": v.result.is_ok(),
                "error": v.result.as_ref().err(),
            })
        })
        .collect()
}
