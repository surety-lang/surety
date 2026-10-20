//! Runs one obligation's examples against a performance.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use surety_py::{RunError, Runner};
use surety_syntax::resolve::Scope;
use surety_syntax::{Decl, Example, Expr, Function, Graph};

use crate::examples::{Step, compare, fill_rest, unordered_paths};
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
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub obligation: String,
    pub examples: Vec<ExampleReport>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.examples.iter().all(|e| e.verdict == Verdict::Pass)
    }
}

/// A run of one example: its inputs and the decoded output, kept for the
/// property phase.
#[derive(Debug, Clone)]
pub struct Run {
    pub index: usize,
    pub inputs: Vec<Value>,
    pub output: Value,
}

pub struct Checker<'g> {
    pub graph: &'g Graph,
    pub idx: usize,
    pub fun: &'g Function,
    pub scope: Scope<'g>,
    pub performance: PathBuf,
    pub timeout: Duration,
    runners: HashMap<PathBuf, Runner>,
    named: HashMap<String, Value>,
    unordered: Vec<Vec<Step>>,
    pub runs: Vec<Run>,
}

impl<'g> Checker<'g> {
    /// Prepare to check the one obligation in file `idx` against the
    /// performance module at `performance`.
    pub fn new(
        graph: &'g Graph,
        idx: usize,
        performance: &Path,
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
            timeout,
            runners: HashMap::new(),
            named: HashMap::new(),
            unordered: unordered_paths(fun),
            runs: Vec::new(),
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

    pub fn unordered(&self) -> &[Vec<Step>] {
        &self.unordered
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
        from_literal(
            &ex.output,
            self.scope.output.as_ref().unwrap(),
            &lookup,
            Some(&filler),
        )
    }

    fn runner(&mut self, path: &Path) -> Result<&mut Runner, RunError> {
        if !self.runners.contains_key(path) {
            let r = Runner::new(path, self.timeout)?;
            self.runners.insert(path.to_path_buf(), r);
        }
        Ok(self.runners.get_mut(path).unwrap())
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
        Ok(from_json(&json, self.scope.output.as_ref().unwrap()))
    }

    /// Everything slice 1 checks: examples, then properties over the
    /// example runs.
    pub fn check(&mut self) -> Report {
        let examples = self.check_examples();
        Report {
            obligation: self.fun.name.clone(),
            examples,
        }
    }

    /// Acceptance test 3: every example against the performance.
    pub fn check_examples(&mut self) -> Vec<ExampleReport> {
        let mut reports = Vec::new();
        self.runs.clear();
        for (index, ex) in self.fun.examples.clone().iter().enumerate() {
            let verdict = self.check_example(index, ex);
            reports.push(ExampleReport {
                index,
                line: ex.line,
                note: ex.note.clone(),
                verdict,
            });
        }
        reports
    }

    fn check_example(&mut self, index: usize, ex: &Example) -> Verdict {
        let inputs = match self.example_inputs(ex) {
            Ok(i) => i,
            Err(m) => return Verdict::Error(format!("example input: {m}")),
        };
        let expected = match self.expected_output(ex, &inputs) {
            Ok(v) => v,
            Err(m) => return Verdict::Error(format!("example output: {m}")),
        };
        let output = match self.run_performance(&inputs) {
            Ok(Ok(v)) => v,
            Ok(Err(b)) => return Verdict::Boundary(b.to_string()),
            Err(e) => return Verdict::Error(e.to_string()),
        };
        let verdict = match compare(&expected, &output, &self.unordered) {
            Ok(()) => Verdict::Pass,
            Err(m) => Verdict::Mismatch(m),
        };
        self.runs.push(Run {
            index,
            inputs,
            output,
        });
        verdict
    }
}
