//! The behavioural diff: two performances of one obligation run on the same
//! inputs, the examples and then the generated tier, and compared under the
//! obligation's order claims. The report is the inputs on which the outputs
//! differ, each classed by what the obligation says about the two outputs.
//! It is never a text diff.

use std::path::PathBuf;

use surety_py::RunError;
use surety_syntax::Expr;
use surety_syntax::normalize::canonical_file;

use crate::checker::{Checker, Outcome};
use crate::examples::{compare, unordered_paths};
use crate::generate::size;
use crate::mutants::Clause;
use crate::value::Value;

/// How a difference reads against the obligation. The order is the order
/// a reviewer wants them in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// The old output passed every clause; the new one fails a clause, or
    /// there is no new output.
    Regression,
    /// Neither output passes every clause.
    NeitherPasses,
    /// The old output failed a clause, or there was none; the new one passes.
    Fix,
    /// Both outputs pass every clause: the obligation admits both.
    Within,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Regression => "regression",
            Class::NeitherPasses => "neither_passes",
            Class::Fix => "fix",
            Class::Within => "within",
        }
    }
}

/// One input on which the two performances behave differently.
#[derive(Debug, Clone)]
pub struct Difference {
    pub inputs: Vec<Value>,
    /// The example number, from 1, when the input is an example's.
    pub example: Option<usize>,
    pub old: Outcome,
    pub new: Outcome,
    /// The clauses each output fails: empty when it passes every clause,
    /// and empty when there is no output to judge.
    pub old_fails: Vec<Clause>,
    pub new_fails: Vec<Clause>,
    pub class: Class,
    /// The line of the `accept` that fires on this input, when both outputs
    /// pass: the obligation decided not to decide between them.
    pub accept: Option<usize>,
}

impl Difference {
    pub fn size(&self) -> usize {
        self.inputs.iter().map(size).sum()
    }
}

#[derive(Debug, Clone)]
pub struct DiffReport {
    pub obligation: String,
    pub obligation_hash: String,
    pub old: PathBuf,
    pub new: PathBuf,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    /// Example inputs run.
    pub examples: usize,
    /// Generated inputs run.
    pub generated: usize,
    /// Inputs on which the two agree under the order claims.
    pub agree: usize,
    /// Sorted: regressions first, then examples before generated inputs,
    /// then smallest first; within the obligation, undecided before accepted.
    pub differences: Vec<Difference>,
}

impl DiffReport {
    pub fn same(&self) -> bool {
        self.differences.is_empty()
    }

    pub fn inputs(&self) -> usize {
        self.examples + self.generated
    }

    pub fn count(&self, class: Class) -> usize {
        self.differences.iter().filter(|d| d.class == class).count()
    }

    /// Within the obligation and covered by an accept line.
    pub fn accepted(&self) -> usize {
        self.differences
            .iter()
            .filter(|d| d.accept.is_some())
            .count()
    }

    /// Within the obligation and covered by no accept line: an input the
    /// obligation does not decide, found from two real performances.
    pub fn undecided(&self) -> usize {
        self.count(Class::Within) - self.accepted()
    }
}

fn outcome(r: Result<Result<Value, crate::value::BoundaryError>, RunError>) -> Outcome {
    match r {
        Ok(Ok(v)) => Outcome::Output(v),
        Ok(Err(b)) => Outcome::Boundary(b),
        Err(RunError::Timeout(_)) => Outcome::Timeout,
        Err(e) => Outcome::Raised(e.to_string()),
    }
}

/// The same behaviour as a caller sees it: equal outputs under the order
/// claims, or the same kind of non-answer (two raises are one behaviour,
/// whatever they say).
fn same(a: &Outcome, b: &Outcome, unordered: &[Vec<crate::examples::Step>]) -> bool {
    match (a, b) {
        (Outcome::Output(x), Outcome::Output(y)) => compare(x, y, unordered).is_ok(),
        (Outcome::Boundary(_), Outcome::Boundary(_))
        | (Outcome::Raised(_), Outcome::Raised(_))
        | (Outcome::Timeout, Outcome::Timeout) => true,
        _ => false,
    }
}

fn fails(
    c: &mut Checker,
    clauses: &[(Clause, Expr)],
    inputs: &[Value],
    outcome: &Outcome,
) -> Vec<Clause> {
    match outcome {
        Outcome::Output(v) => clauses
            .iter()
            .filter(|(_, e)| !matches!(c.holds(e, inputs, v), Ok(true)))
            .map(|(cl, _)| cl.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// Run both performances on the obligation's inputs and report where they
/// differ. `old` and `new` check the same obligation; each resolves graph
/// references in its own module directory.
pub fn diff(old: &mut Checker, new: &mut Checker) -> DiffReport {
    let examples = old.example_input_set();
    let generated = old.generated_inputs().inputs;
    let unordered = unordered_paths(old.fun);
    let clauses = old.clauses();
    let accepts: Vec<(usize, Expr)> = old
        .fun
        .accepts
        .iter()
        .map(|a| (a.line, a.expr.clone()))
        .collect();
    let runs = examples
        .iter()
        .enumerate()
        .map(|(i, x)| (Some(i + 1), x.clone()))
        .chain(generated.iter().map(|x| (None, x.clone())));
    let mut agree = 0;
    let mut differences = Vec::new();
    for (example, inputs) in runs {
        let a = outcome(old.run_performance(&inputs));
        let b = outcome(new.run_performance(&inputs));
        if same(&a, &b, &unordered) {
            agree += 1;
            continue;
        }
        let old_fails = fails(old, &clauses, &inputs, &a);
        let new_fails = fails(new, &clauses, &inputs, &b);
        let old_ok = matches!(a, Outcome::Output(_)) && old_fails.is_empty();
        let new_ok = matches!(b, Outcome::Output(_)) && new_fails.is_empty();
        let class = match (old_ok, new_ok) {
            (true, true) => Class::Within,
            (true, false) => Class::Regression,
            (false, true) => Class::Fix,
            (false, false) => Class::NeitherPasses,
        };
        let accept = if class == Class::Within {
            accepts
                .iter()
                .find(|(_, e)| old.accepts_input(e, &inputs))
                .map(|(line, _)| *line)
        } else {
            None
        };
        differences.push(Difference {
            inputs,
            example,
            old: a,
            new: b,
            old_fails,
            new_fails,
            class,
            accept,
        });
    }
    differences.sort_by_key(|d| {
        (
            d.class,
            d.accept.is_some(),
            d.example.is_none(),
            d.example,
            d.size(),
        )
    });
    DiffReport {
        obligation: old.fun.name.clone(),
        obligation_hash: canonical_file(old.graph, old.idx)
            .map(|c| c.hash)
            .unwrap_or_default(),
        old: old.performance.clone(),
        new: new.performance.clone(),
        old_hash: surety_py::inspect(&old.performance).ok().map(|i| i.hash),
        new_hash: surety_py::inspect(&new.performance).ok().map(|i| i.hash),
        examples: examples.len(),
        generated: generated.len(),
        agree,
        differences,
    }
}
