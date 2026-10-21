//! Runs one obligation against a performance: the examples (binding), the
//! properties over the example runs (binding), the frame rule, and then
//! the generated tier: inputs built from the types, every claim counted
//! over them, determinism by running each input twice, totality by the time
//! cap, and the performance's own shape. The record of all that is the
//! evidence.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use surety_py::{Inspection, RunError, Runner};
use surety_syntax::normalize::canonical_file;
use surety_syntax::resolve::Scope;
use surety_syntax::{BinOp, Decl, Example, Expr, Function, Graph, Refinement, Type};

use crate::eval::Env;
use crate::examples::{Step, compare, fill_rest, unordered_paths};
use crate::generate::{Bucket, Budget, Domain, Generated, generate};
use crate::value::{BoundaryError, Value, from_json, from_literal, to_json};

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Pass,
    /// The output did not fit the declared type. Its own verdict, distinct
    /// from a mismatch or a property failure.
    Boundary(String),
    /// The output fit the type but was not the expected `out`.
    Mismatch(String),
    /// The performance could not be run, or the example itself is broken.
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ExampleReport {
    pub index: usize,
    pub line: usize,
    pub note: Option<String>,
    pub verdict: Verdict,
    /// The decoded inputs, when the example's `in:` decoded.
    pub inputs: Vec<Value>,
    pub expected: Option<Value>,
    /// The performance's output, when it passed the boundary.
    pub output: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropVerdict {
    Pass,
    /// False for at least one example; the first is named.
    Fail {
        example: usize,
        line: usize,
        detail: String,
    },
    /// Could not be evaluated: a missing performance, a type mismatch at
    /// run time, or an error raised by a graph reference.
    Error(String),
    /// No example run to evaluate it over.
    Skipped,
}

/// A generated run on which a claim did not hold.
#[derive(Debug, Clone)]
pub struct Counterexample {
    pub inputs: Vec<Value>,
    pub output: Value,
    /// Set when the claim could not be evaluated rather than being false.
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PropertyReport {
    pub line: usize,
    pub text: String,
    pub sentence: Option<String>,
    /// Over the example runs: binding.
    pub verdict: PropVerdict,
    /// Runs the claim was evaluated over: example runs plus generated runs.
    pub runs: usize,
    /// Generated runs on which it was false or could not be evaluated.
    pub counterexamples: Vec<Counterexample>,
    /// A refinement on an input that would exclude every counterexample.
    pub suggest: Option<String>,
    /// What the counterexamples have in common when no refinement fixes it.
    pub note: Option<String>,
}

impl PropertyReport {
    pub fn findings(&self) -> usize {
        self.counterexamples.len()
    }
}

/// One frame-rule pair and the verdict of its implicit property.
#[derive(Debug, Clone)]
pub struct FrameReport {
    pub description: String,
    pub line: usize,
    pub preserved: Vec<String>,
    pub verdict: PropVerdict,
    pub runs: usize,
    pub counterexamples: Vec<Counterexample>,
    pub suggest: Option<String>,
    pub note: Option<String>,
}

impl FrameReport {
    pub fn findings(&self) -> usize {
        self.counterexamples.len()
    }
}

/// A generated input the performance did not handle; the message says how.
#[derive(Debug, Clone)]
pub struct Failure {
    pub inputs: Vec<Value>,
    pub message: String,
}

/// Every boundary-valid input run twice, the outputs compared under the
/// obligation's order claims.
#[derive(Debug, Clone, Default)]
pub struct Determinism {
    pub matched: usize,
    pub total: usize,
    pub failures: Vec<Failure>,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub obligation: String,
    pub obligation_hash: String,
    pub examples: Vec<ExampleReport>,
    pub properties: Vec<PropertyReport>,
    pub frame: Vec<FrameReport>,
    /// Graph references the properties called, as `name -> module`.
    pub graph_refs: Vec<String>,
    /// Example inputs that decoded.
    pub example_inputs: usize,
    /// Example runs whose output passed the boundary.
    pub example_runs: usize,
    /// Boundary-valid runs in all: what every claim was evaluated over.
    pub runs: usize,
    pub buckets: Vec<Bucket>,
    pub generated_inputs: usize,
    /// Generated outputs that failed the boundary: a finding, since the
    /// obligation itself can force one (apply_diff, when `diff.added`
    /// repeats a sku).
    pub generated_boundary: Vec<Failure>,
    /// A refinement on an input that would exclude every boundary failure.
    pub boundary_suggest: Option<String>,
    /// Generated inputs on which the performance raised: a finding.
    pub generated_raised: Vec<Failure>,
    /// Generated inputs on which the performance did not answer in time.
    pub generated_timeouts: Vec<Failure>,
    pub determinism: Determinism,
    pub timeout: Duration,
    pub inspection: Option<Inspection>,
    pub inspection_error: Option<String>,
}

impl Report {
    /// The binding tier: examples, the claims over them, and the boundary
    /// on every output.
    pub fn passed(&self) -> bool {
        self.examples.iter().all(|e| e.verdict == Verdict::Pass)
            && self
                .properties
                .iter()
                .all(|p| p.verdict == PropVerdict::Pass)
            && self.frame.iter().all(|f| f.verdict == PropVerdict::Pass)
    }

    /// Findings: claims false on generated inputs, generated outputs that
    /// failed the boundary, raises and timeouts, and nondeterminism.
    /// Reported, not binding, unless `--strict`.
    pub fn findings(&self) -> usize {
        self.claim_findings()
            + self.generated_boundary.len()
            + self.generated_raised.len()
            + self.generated_timeouts.len()
            + self.determinism.failures.len()
    }

    pub fn claim_findings(&self) -> usize {
        self.properties.iter().map(|p| p.findings()).sum::<usize>()
            + self.frame.iter().map(|f| f.findings()).sum::<usize>()
    }

    pub fn strict_passed(&self) -> bool {
        self.passed() && self.findings() == 0
    }

    pub fn failed_properties(&self) -> Vec<&PropertyReport> {
        self.properties
            .iter()
            .filter(|p| !matches!(p.verdict, PropVerdict::Pass | PropVerdict::Skipped))
            .collect()
    }

    /// Every field the frame rule preserved by default, sorted.
    pub fn frame_preserved(&self) -> Vec<String> {
        let mut all: Vec<String> = Vec::new();
        for f in &self.frame {
            for n in &f.preserved {
                if !all.contains(n) {
                    all.push(n.clone());
                }
            }
        }
        all.sort();
        all
    }
}

/// The field of `binder` that an expression compares for equality with
/// some other field (`x.f == y.g`, either way round), if any.
fn match_key(e: &Expr, binder: &str) -> Option<String> {
    let mut key = None;
    e.walk(&mut |sub| {
        if key.is_some() {
            return;
        }
        let Expr::Binary(BinOp::Eq, a, b) = sub else {
            return;
        };
        let (Expr::Field(ba, fa, _), Expr::Field(bb, fb, _)) = (&**a, &**b) else {
            return;
        };
        let is_binder = |base: &Expr| matches!(base, Expr::Name(n, _) if n == binder);
        if is_binder(ba) && !is_binder(bb) {
            key = Some(fa.clone());
        } else if is_binder(bb) && !is_binder(ba) {
            key = Some(fb.clone());
        }
    });
    key
}

/// A run of one example: its inputs and the decoded output, kept for the
/// property phase.
#[derive(Debug, Clone)]
pub struct Run {
    pub index: usize,
    pub inputs: Vec<Value>,
    pub output: Value,
}

/// What a generated input produced.
#[derive(Debug, Clone)]
pub enum Outcome {
    Output(Value),
    Boundary(BoundaryError),
    Raised(String),
    Timeout,
}

#[derive(Debug, Clone)]
pub struct GenRun {
    pub inputs: Vec<Value>,
    pub outcome: Outcome,
}

pub struct Checker<'g> {
    pub graph: &'g Graph,
    pub idx: usize,
    pub fun: &'g Function,
    pub scope: Scope<'g>,
    pub performance: PathBuf,
    /// Where other obligations' performances live: `<module_dir>/<fixture>.py`.
    pub module_dir: PathBuf,
    pub timeout: Duration,
    pub budget: Budget,
    /// While the underdetermination check runs a mutant, a property's call
    /// of the obligation's own name means that mutant.
    pub mutant: Option<crate::mutants::Mutant>,
    runners: HashMap<PathBuf, Runner>,
    named: HashMap<String, Value>,
    unordered: Vec<Vec<Step>>,
    pub runs: Vec<Run>,
    pub generated: Vec<GenRun>,
    pub graph_refs: Vec<String>,
    pub(crate) call_cache: HashMap<String, serde_json::Value>,
}

impl<'g> Checker<'g> {
    /// Prepare to check the one obligation in file `idx` against the
    /// performance module at `performance`.
    pub fn new(
        graph: &'g Graph,
        idx: usize,
        performance: &Path,
        module_dir: &Path,
        timeout: Duration,
    ) -> Result<Checker<'g>, String> {
        let funs = graph.functions_in(idx);
        let fun = *funs.first().ok_or("file has no obligation")?;
        let scope = Scope::for_function(graph, idx, fun).map_err(|e| e.to_string())?;
        let mut checker = Checker {
            graph,
            idx,
            fun,
            scope,
            performance: performance.to_path_buf(),
            module_dir: module_dir.to_path_buf(),
            timeout,
            budget: Budget::default(),
            mutant: None,
            runners: HashMap::new(),
            named: HashMap::new(),
            unordered: unordered_paths(fun),
            runs: Vec::new(),
            generated: Vec::new(),
            graph_refs: Vec::new(),
            call_cache: HashMap::new(),
        };
        checker.decode_named_values()?;
        Ok(checker)
    }

    fn decode_named_values(&mut self) -> Result<(), String> {
        let decls: Vec<_> = self
            .graph
            .file(self.idx)
            .decls
            .iter()
            .filter_map(|d| match d {
                Decl::NamedValue(v) => Some(v),
                _ => None,
            })
            .collect();
        // Values may name earlier values; a few passes resolve any order.
        for _ in 0..=decls.len() {
            let mut progressed = false;
            for v in &decls {
                if self.named.contains_key(&v.name) {
                    continue;
                }
                let ty = self
                    .graph
                    .named_value_type(self.idx, v)
                    .map_err(|e| e.to_string())?;
                let named = &self.named;
                if let Ok(val) = from_literal(&v.value, &ty, &|n| named.get(n).cloned(), None) {
                    self.named.insert(v.name.clone(), val);
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
        for v in &decls {
            if !self.named.contains_key(&v.name) {
                let ty = self
                    .graph
                    .named_value_type(self.idx, v)
                    .map_err(|e| e.to_string())?;
                let named = &self.named;
                from_literal(&v.value, &ty, &|n| named.get(n).cloned(), None)
                    .map_err(|e| format!("named value `{}` (line {}): {e}", v.name, v.line))?;
            }
        }
        Ok(())
    }

    pub fn named_value(&self, name: &str) -> Option<Value> {
        self.named.get(name).cloned()
    }

    /// Every named value with its declared type.
    pub fn named_values_typed(&self) -> Vec<(Value, Type)> {
        let mut out = Vec::new();
        for d in &self.graph.file(self.idx).decls {
            if let Decl::NamedValue(v) = d {
                if let (Some(val), Ok(ty)) = (
                    self.named.get(&v.name),
                    self.graph.named_value_type(self.idx, v),
                ) {
                    out.push((val.clone(), ty));
                }
            }
        }
        out
    }

    pub fn unordered(&self) -> &[Vec<Step>] {
        &self.unordered
    }

    pub fn output_type(&self) -> &Type {
        self.scope
            .output
            .as_ref()
            .expect("a function has an output type")
    }

    /// The inputs of an example, as values.
    pub fn example_inputs(&self, ex: &Example) -> Result<Vec<Value>, String> {
        let lookup = |n: &str| self.named.get(n).cloned();
        let parts: Vec<&Expr> = match (&ex.input, self.scope.inputs.len()) {
            (Expr::Tuple(xs), n) if xs.len() == n => xs.iter().collect(),
            (e, 1) => vec![e],
            (_, n) => return Err(format!("`in:` must be a tuple of {n}")),
        };
        let mut out = Vec::new();
        for (e, (name, ty)) in parts.iter().zip(&self.scope.inputs) {
            out.push(
                from_literal(e, ty, &lookup, None).map_err(|m| format!("input `{name}`: {m}"))?,
            );
        }
        Ok(out)
    }

    /// The expected output of an example, with `...` filled from the inputs.
    pub fn expected_output(&self, ex: &Example, inputs: &[Value]) -> Result<Value, String> {
        let lookup = |n: &str| self.named.get(n).cloned();
        let filler = |stated: &[(String, Value)], missing: &[(String, surety_syntax::Type)]| {
            fill_rest(stated, missing, inputs)
        };
        from_literal(&ex.output, self.output_type(), &lookup, Some(&filler))
    }

    /// Every example's inputs that decode.
    pub fn example_input_set(&self) -> Vec<Vec<Value>> {
        self.fun
            .examples
            .iter()
            .filter_map(|ex| self.example_inputs(ex).ok())
            .collect()
    }

    fn runner(&mut self, path: &Path) -> Result<&mut Runner, RunError> {
        if self.runners.get(path).is_some_and(|r| r.is_dead()) {
            self.runners.remove(path);
        }
        if !self.runners.contains_key(path) {
            let r = Runner::new(path, self.timeout)?;
            self.runners.insert(path.to_path_buf(), r);
        }
        Ok(self.runners.get_mut(path).unwrap())
    }

    /// Whether a module defines a callable of that name.
    pub fn module_has(&mut self, module: &Path, name: &str) -> Result<bool, String> {
        let module = module.to_path_buf();
        self.runner(&module)
            .and_then(|r| r.has(name))
            .map_err(|e| e.to_string())
    }

    /// Call a function in a module with values; the reply is raw JSON.
    pub fn call_raw(
        &mut self,
        module: &Path,
        name: &str,
        args: &[Value],
    ) -> Result<serde_json::Value, RunError> {
        let json: Vec<serde_json::Value> = args.iter().map(to_json).collect();
        let module = module.to_path_buf();
        self.runner(&module)?.call(name, &json)
    }

    /// Run the performance under check on inputs and decode the output at
    /// the boundary.
    pub fn run_performance(
        &mut self,
        inputs: &[Value],
    ) -> Result<Result<Value, BoundaryError>, RunError> {
        let module = self.performance.clone();
        let json = self.call_raw(&module, &self.fun.name.clone(), inputs)?;
        Ok(from_json(&json, self.output_type()))
    }

    /// The domain generation draws from: the base values plus every literal
    /// the obligation uses, typed where an example or a named value types it.
    pub fn domain(&self) -> Domain {
        let mut d = Domain::base();
        for (val, ty) in self.named_values_typed() {
            d.add_typed(&val, &ty);
        }
        for decl in &self.graph.file(self.idx).decls {
            if let Decl::Abbrev(a) = decl {
                d.add_expr(&a.body);
            }
        }
        for ex in &self.fun.examples {
            if let Ok(inputs) = self.example_inputs(ex) {
                for (v, (_, t)) in inputs.iter().zip(&self.scope.inputs) {
                    d.add_typed(v, t);
                }
                if let Ok(out) = self.expected_output(ex, &inputs) {
                    d.add_typed(&out, self.output_type());
                }
            }
        }
        for a in &self.fun.abbrevs {
            d.add_expr(&a.body);
        }
        for p in &self.fun.properties {
            d.add_expr(&p.expr);
        }
        d.finish();
        d
    }

    /// The generated inputs for this obligation under the budget: the same
    /// every time.
    pub fn generated_inputs(&self) -> Generated {
        generate(
            &self.scope.inputs,
            &self.domain(),
            self.budget.generated,
            &self.example_input_set(),
        )
    }

    /// Whether a claim holds for given inputs and output.
    pub fn holds(&mut self, expr: &Expr, inputs: &[Value], out: &Value) -> Result<bool, String> {
        let named: Vec<(String, Value)> = self
            .scope
            .inputs
            .iter()
            .map(|(n, _)| n.clone())
            .zip(inputs.iter().cloned())
            .collect();
        let env = Env::new(&named, out);
        self.eval_bool(expr, &env)
    }

    /// Everything `check` checks: the binding tier over the examples, then
    /// the generated tier and the evidence record.
    pub fn check(&mut self) -> Report {
        let examples = self.check_examples();
        let mut properties = self.check_properties();
        let mut frame = self.check_frame();
        let example_runs = self.runs.len();
        let example_inputs = examples
            .iter()
            .filter(|e| !e.inputs.is_empty() || self.scope.inputs.is_empty())
            .count();
        let obligation_hash = canonical_file(self.graph, self.idx)
            .map(|c| c.hash)
            .unwrap_or_default();
        let generated = self.run_generated();
        let examples_pass = examples.iter().all(|e| e.verdict == Verdict::Pass);
        let fun = self.fun;
        for (report, p) in properties.iter_mut().zip(&fun.properties) {
            let (runs, cex) = self.over_generated(&p.expr);
            report.runs = example_runs + runs;
            if examples_pass && report.verdict == PropVerdict::Pass {
                (report.suggest, report.note) = self.diagnose(&p.expr, &cex);
            }
            report.counterexamples = cex;
        }
        let pairs = crate::frame::frame_pairs(&self.scope, fun);
        for (report, pair) in frame.iter_mut().zip(&pairs) {
            let (runs, cex) = self.over_generated(&pair.implicit);
            report.runs = example_runs + runs;
            if examples_pass && report.verdict == PropVerdict::Pass {
                (report.suggest, report.note) = self.diagnose(&pair.implicit, &cex);
            }
            report.counterexamples = cex;
        }
        let determinism = self.check_determinism();
        let (inspection, inspection_error) = match surety_py::inspect(&self.performance) {
            Ok(i) => (Some(i), None),
            Err(e) => (None, Some(e.to_string())),
        };
        let mut generated_ok = 0;
        let mut generated_boundary = Vec::new();
        let mut boundary_keys: Vec<(Vec<Value>, String)> = Vec::new();
        let mut generated_raised = Vec::new();
        let mut generated_timeouts = Vec::new();
        for g in &self.generated {
            match &g.outcome {
                Outcome::Output(_) => generated_ok += 1,
                Outcome::Boundary(b) => {
                    if let Some(k) = &b.unique_key {
                        boundary_keys.push((g.inputs.clone(), k.clone()));
                    }
                    generated_boundary.push(Failure {
                        inputs: g.inputs.clone(),
                        message: b.to_string(),
                    });
                }
                Outcome::Raised(m) => generated_raised.push(Failure {
                    inputs: g.inputs.clone(),
                    message: m.clone(),
                }),
                Outcome::Timeout => generated_timeouts.push(Failure {
                    inputs: g.inputs.clone(),
                    message: format!("no answer within {:?}", self.timeout),
                }),
            }
        }
        let boundary_suggest = if boundary_keys.len() == generated_boundary.len() {
            self.diagnose_boundary(&boundary_keys)
        } else {
            None
        };
        Report {
            obligation: self.fun.name.clone(),
            obligation_hash,
            examples,
            properties,
            frame,
            graph_refs: self.graph_refs.clone(),
            example_inputs,
            example_runs,
            runs: example_runs + generated_ok,
            buckets: generated.buckets,
            generated_inputs: generated.inputs.len(),
            generated_boundary,
            boundary_suggest,
            generated_raised,
            generated_timeouts,
            determinism,
            timeout: self.timeout,
            inspection,
            inspection_error,
        }
    }

    /// Run the performance on every generated input.
    fn run_generated(&mut self) -> Generated {
        let generated = self.generated_inputs();
        self.generated.clear();
        for inputs in &generated.inputs {
            let outcome = match self.run_performance(inputs) {
                Ok(Ok(v)) => Outcome::Output(v),
                Ok(Err(b)) => Outcome::Boundary(b),
                Err(RunError::Timeout(_)) => Outcome::Timeout,
                Err(e) => Outcome::Raised(e.to_string()),
            };
            self.generated.push(GenRun {
                inputs: inputs.clone(),
                outcome,
            });
        }
        generated
    }

    /// A claim over every boundary-valid generated run: how many, and the
    /// runs it did not hold on.
    fn over_generated(&mut self, expr: &Expr) -> (usize, Vec<Counterexample>) {
        let mut runs = 0;
        let mut cex = Vec::new();
        for i in 0..self.generated.len() {
            let (inputs, output) = match &self.generated[i].outcome {
                Outcome::Output(o) => (self.generated[i].inputs.clone(), o.clone()),
                _ => continue,
            };
            runs += 1;
            match self.holds(expr, &inputs, &output) {
                Ok(true) => {}
                Ok(false) => cex.push(Counterexample {
                    inputs,
                    output,
                    error: None,
                }),
                Err(m) => cex.push(Counterexample {
                    inputs,
                    output,
                    error: Some(m),
                }),
            }
        }
        (runs, cex)
    }

    /// Every boundary-valid input again, outputs compared under the order
    /// claims.
    fn check_determinism(&mut self) -> Determinism {
        let mut cases: Vec<(Vec<Value>, Value)> = self
            .runs
            .iter()
            .map(|r| (r.inputs.clone(), r.output.clone()))
            .collect();
        for g in &self.generated {
            if let Outcome::Output(o) = &g.outcome {
                cases.push((g.inputs.clone(), o.clone()));
            }
        }
        let mut d = Determinism::default();
        for (inputs, first) in cases {
            d.total += 1;
            let message = match self.run_performance(&inputs) {
                Ok(Ok(second)) => {
                    if compare(&first, &second, &self.unordered).is_ok() {
                        d.matched += 1;
                        continue;
                    }
                    format!("first run {first}, second run {second}")
                }
                Ok(Err(b)) => format!("second run failed the boundary: {b}"),
                Err(e) => format!("second run: {e}"),
            };
            d.failures.push(Failure { inputs, message });
        }
        d
    }

    /// What a claim's counterexamples have in common. The claim matches
    /// elements of an input list on a field (a `where`, or the body of an
    /// `exactly_one`). When every counterexample repeats a whole element of
    /// that list, no refinement helps and the note says so; when every one
    /// repeats the matched field, the refinement that would exclude them
    /// is the suggestion.
    fn diagnose(
        &self,
        expr: &Expr,
        failing: &[Counterexample],
    ) -> (Option<String>, Option<String>) {
        if failing.is_empty() {
            return (None, None);
        }
        // A quantifier over an input list (or a list field of an input)
        // whose binder is matched on a field against another element's
        // field, anywhere inside it: a `where`, or an `exactly_one` body.
        let mut found: Option<(usize, Option<String>, String)> = None;
        expr.walk(&mut |e| {
            if found.is_some() {
                return;
            }
            let Expr::Quantifier(q) = e else { return };
            let Some(binder) = &q.binder.name else { return };
            let input_pos = |n: &str| self.scope.inputs.iter().position(|(m, _)| m == n);
            let (pos, field) = match &q.domain {
                Expr::Name(n, _) => match input_pos(n) {
                    Some(p) => (p, None),
                    None => return,
                },
                Expr::Field(base, f, _) => match &**base {
                    Expr::Name(n, _) => match input_pos(n) {
                        Some(p) => (p, Some(f.clone())),
                        None => return,
                    },
                    _ => return,
                },
                _ => return,
            };
            let key = [&q.where_, &q.body]
                .into_iter()
                .flatten()
                .find_map(|part| match_key(part, binder));
            if let Some(key) = key {
                found = Some((pos, field, key));
            }
        });
        let Some((pos, field, key)) = found else {
            return (None, None);
        };
        let list_of = |c: &Counterexample| -> Option<Vec<Value>> {
            let v = c.inputs.get(pos)?;
            let v = match &field {
                Some(f) => v.field(f)?,
                None => v,
            };
            match v {
                Value::List(xs) => Some(xs.clone()),
                _ => None,
            }
        };
        let lists: Vec<Vec<Value>> = failing.iter().filter_map(list_of).collect();
        if lists.len() != failing.len() {
            return (None, None);
        }
        let (input_name, _) = &self.scope.inputs[pos];
        let list_name = match &field {
            Some(f) => format!("{input_name}.{f}"),
            None => input_name.clone(),
        };
        let repeats_element =
            |xs: &[Value]| xs.iter().enumerate().any(|(i, x)| xs[..i].contains(x));
        if lists.iter().all(|xs| repeats_element(xs)) {
            return (
                None,
                Some(format!(
                    "every counterexample repeats an element of `{list_name}`; the obligation cannot tell identical elements apart"
                )),
            );
        }
        let repeats_key = |xs: &[Value]| {
            let keys: Vec<Option<&Value>> = xs.iter().map(|x| x.field(&key)).collect();
            keys.iter().enumerate().any(|(i, k)| keys[..i].contains(k))
        };
        if !lists.iter().all(|xs| repeats_key(xs)) {
            return (None, None);
        }
        // The declared type of that list, to print the refinement on it.
        let (_, declared) = &self.fun.inputs[pos];
        let (list_ty, where_) = match &field {
            None => (declared.clone(), String::new()),
            Some(f) => {
                let record = match declared {
                    Type::Named(n) => {
                        self.graph
                            .file(self.idx)
                            .decls
                            .iter()
                            .find_map(|d| match d {
                                Decl::Type(t) if t.name == *n => {
                                    Some((t.ty.clone(), format!(" in type {n}")))
                                }
                                _ => None,
                            })
                    }
                    Type::Record { .. } => Some((declared.clone(), String::new())),
                    _ => None,
                };
                let Some((record, where_)) = record else {
                    return (None, None);
                };
                let Some(fields) = record.record_fields() else {
                    return (None, None);
                };
                let Some((_, t)) = fields.iter().find(|(n, _)| n == f) else {
                    return (None, None);
                };
                (t.clone(), where_)
            }
        };
        let Type::List(elem, refs) = list_ty else {
            return (None, None);
        };
        if refs
            .iter()
            .any(|r| matches!(r, Refinement::UniqueBy(k) if *k == key))
        {
            return (None, None);
        }
        let mut refs = refs.clone();
        refs.push(Refinement::UniqueBy(key));
        let label = match &field {
            Some(f) => f.clone(),
            None => input_name.clone(),
        };
        (
            Some(format!("{label}: {}{where_}", Type::List(elem, refs))),
            None,
        )
    }

    /// When every generated output that failed a `unique` refinement came
    /// from an input whose list (an input, or a list field of one) repeats
    /// that key, the refinement on the input that would exclude them.
    fn diagnose_boundary(&self, failing: &[(Vec<Value>, String)]) -> Option<String> {
        let (_, key) = failing.first()?;
        if failing.iter().any(|(_, k)| k != key) {
            return None;
        }
        let repeats_key = |xs: &[Value]| {
            let keys: Vec<Option<&Value>> = xs.iter().map(|x| x.field(key)).collect();
            keys.iter().enumerate().any(|(i, k)| keys[..i].contains(k))
        };
        for (pos, (name, declared)) in self.fun.inputs.iter().enumerate() {
            // Candidate lists: the input itself, or each list field of it.
            let mut candidates: Vec<(Option<String>, Type)> = Vec::new();
            match declared {
                Type::List(..) => candidates.push((None, declared.clone())),
                Type::Named(n) => {
                    if let Some(Type::Record { fields, .. }) = self
                        .graph
                        .file(self.idx)
                        .decls
                        .iter()
                        .find_map(|d| match d {
                            Decl::Type(t) if t.name == *n => Some(t.ty.clone()),
                            _ => None,
                        })
                    {
                        for (f, t) in fields {
                            if matches!(t, Type::List(..)) {
                                candidates.push((Some(f.clone()), t.clone()));
                            }
                        }
                    }
                }
                Type::Record { fields, .. } => {
                    for (f, t) in fields {
                        if matches!(t, Type::List(..)) {
                            candidates.push((Some(f.clone()), t.clone()));
                        }
                    }
                }
                _ => {}
            }
            for (field, list_ty) in candidates {
                let Type::List(elem, refs) = &list_ty else {
                    continue;
                };
                if refs
                    .iter()
                    .any(|r| matches!(r, Refinement::UniqueBy(k) if k == key))
                {
                    continue;
                }
                let all_repeat = failing.iter().all(|(inputs, _)| {
                    let v = match (&inputs.get(pos), &field) {
                        (Some(v), Some(f)) => v.field(f),
                        (Some(v), None) => Some(*v),
                        _ => None,
                    };
                    matches!(v, Some(Value::List(xs)) if repeats_key(xs))
                });
                if all_repeat {
                    let mut refs = refs.clone();
                    refs.push(Refinement::UniqueBy(key.clone()));
                    let where_ = match (&field, declared) {
                        (Some(_), Type::Named(n)) => format!(" in type {n}"),
                        _ => String::new(),
                    };
                    let label = field.clone().unwrap_or_else(|| name.clone());
                    return Some(format!(
                        "{label}: {}{where_}",
                        Type::List(elem.clone(), refs)
                    ));
                }
            }
        }
        None
    }

    /// Acceptance test 6: the frame rule's implicit properties.
    pub fn check_frame(&mut self) -> Vec<FrameReport> {
        let pairs = crate::frame::frame_pairs(&self.scope, self.fun);
        let mut reports = Vec::new();
        for p in pairs {
            let verdict = self.check_property(&p.implicit);
            reports.push(FrameReport {
                description: p.description,
                line: p.line,
                preserved: p.preserved,
                verdict,
                runs: self.runs.len(),
                counterexamples: Vec::new(),
                suggest: None,
                note: None,
            });
        }
        reports
    }

    /// Acceptance test 4: every property, evaluated over the example runs
    /// (the performance's actual output for each example input). Runs
    /// whose output failed the boundary are not here: that verdict stands
    /// on its own.
    pub fn check_properties(&mut self) -> Vec<PropertyReport> {
        let mut reports = Vec::new();
        for p in self.fun.properties.clone() {
            let verdict = self.check_property(&p.expr);
            reports.push(PropertyReport {
                line: p.line,
                text: p.text.clone(),
                sentence: p.sentence.clone(),
                verdict,
                runs: self.runs.len(),
                counterexamples: Vec::new(),
                suggest: None,
                note: None,
            });
        }
        reports
    }

    /// One claim over every example run.
    pub fn check_property(&mut self, expr: &Expr) -> PropVerdict {
        if self.runs.is_empty() {
            return PropVerdict::Skipped;
        }
        let runs = self.runs.clone();
        for run in &runs {
            match self.holds(expr, &run.inputs, &run.output) {
                Ok(true) => {}
                Ok(false) => {
                    let ex = &self.fun.examples[run.index];
                    return PropVerdict::Fail {
                        example: run.index,
                        line: ex.line,
                        detail: format!(
                            "false for example {} (line {}): out = {}",
                            run.index + 1,
                            ex.line,
                            run.output
                        ),
                    };
                }
                Err(m) => return PropVerdict::Error(m),
            }
        }
        PropVerdict::Pass
    }

    /// Acceptance test 3: every example against the performance.
    pub fn check_examples(&mut self) -> Vec<ExampleReport> {
        let mut reports = Vec::new();
        self.runs.clear();
        for (index, ex) in self.fun.examples.clone().iter().enumerate() {
            reports.push(self.check_example(index, ex));
        }
        reports
    }

    fn check_example(&mut self, index: usize, ex: &Example) -> ExampleReport {
        let mut report = ExampleReport {
            index,
            line: ex.line,
            note: ex.note.clone(),
            verdict: Verdict::Pass,
            inputs: Vec::new(),
            expected: None,
            output: None,
        };
        let inputs = match self.example_inputs(ex) {
            Ok(i) => i,
            Err(m) => {
                report.verdict = Verdict::Error(format!("example input: {m}"));
                return report;
            }
        };
        report.inputs = inputs.clone();
        let expected = match self.expected_output(ex, &inputs) {
            Ok(v) => v,
            Err(m) => {
                report.verdict = Verdict::Error(format!("example output: {m}"));
                return report;
            }
        };
        report.expected = Some(expected.clone());
        let output = match self.run_performance(&inputs) {
            Ok(Ok(v)) => v,
            Ok(Err(b)) => {
                report.verdict = Verdict::Boundary(b.to_string());
                return report;
            }
            Err(e) => {
                report.verdict = Verdict::Error(e.to_string());
                return report;
            }
        };
        report.output = Some(output.clone());
        report.verdict = match compare(&expected, &output, &self.unordered) {
            Ok(()) => Verdict::Pass,
            Err(m) => Verdict::Mismatch(m),
        };
        self.runs.push(Run {
            index,
            inputs,
            output,
        });
        report
    }
}
