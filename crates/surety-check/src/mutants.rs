//! The underdetermination check (slice 2, acceptance tests 3 to 7).
//!
//! A fixed library of wrong performances is built from the obligation's
//! types and the input value, with no performance anywhere, and run against
//! the obligation over the same inputs `check` uses. A mutant that no
//! property or frame property catches on any input has slipped through,
//! and is reported with the clauses it slipped through; whether an example
//! caught it is stated beside that, because an example-only catch means
//! the property layer has slack on every input that is not that example.
//! A bounded search then looks, per input, for two different outputs that
//! both pass every clause: an input on which the obligation is
//! underdetermined.

use surety_syntax::parser::BUILTINS;
use surety_syntax::resolve::Scope;
use surety_syntax::{Expr, Type};

use crate::checker::Checker;
use crate::examples::{Step, compare};
use crate::frame::frame_pairs;
use crate::generate::{Bucket, Domain, candidates};
use crate::value::{Value, from_json, to_json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutant {
    Empty,
    OkEmpty,
    Identity,
    DropOne,
    DuplicateOne,
    AppendFabricated,
    SwapAdjacent,
}

pub const LIBRARY: [Mutant; 7] = [
    Mutant::Empty,
    Mutant::OkEmpty,
    Mutant::Identity,
    Mutant::DropOne,
    Mutant::DuplicateOne,
    Mutant::AppendFabricated,
    Mutant::SwapAdjacent,
];

impl Mutant {
    pub fn name(self) -> &'static str {
        match self {
            Mutant::Empty => "empty",
            Mutant::OkEmpty => "ok_empty",
            Mutant::Identity => "identity",
            Mutant::DropOne => "drop_one",
            Mutant::DuplicateOne => "duplicate_one",
            Mutant::AppendFabricated => "append_fabricated",
            Mutant::SwapAdjacent => "swap_adjacent",
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Mutant::Empty => "returns the empty value of the output type",
            Mutant::OkEmpty => "returns Ok of the empty value",
            Mutant::Identity => "returns the input unchanged",
            Mutant::DropOne => "returns the input without its first element",
            Mutant::AppendFabricated => "returns the input with a fabricated element appended",
            Mutant::DuplicateOne => "returns the input with its first element repeated",
            Mutant::SwapAdjacent => "returns the input with its first two elements swapped",
        }
    }

    pub fn from_name(name: &str) -> Option<Mutant> {
        LIBRARY.into_iter().find(|m| m.name() == name)
    }
}

const EPOCH: &str = "1970-01-01T00:00:00Z";
const ZERO_UUID: &str = "00000000-0000-0000-0000-000000000000";

fn scalar(ty: &Type, text: &str) -> Value {
    match ty {
        Type::Int => Value::Int(0),
        Type::Float => Value::Float(0.0),
        Type::Text | Type::Id => Value::Text(text.to_string()),
        Type::Uuid => Value::Text(ZERO_UUID.to_string()),
        Type::Time => Value::Time {
            instant: 0,
            text: EPOCH.to_string(),
        },
        Type::Bool => Value::Bool(false),
        Type::Option(_) => Value::None,
        Type::List(..) => Value::List(Vec::new()),
        Type::Record { fields, .. } => Value::record(
            fields
                .iter()
                .map(|(n, t)| (n.clone(), scalar(t, text)))
                .collect(),
        ),
        Type::Result(_, err) => Value::Err(Box::new(scalar(err, text))),
        Type::Named(_) | Type::Unknown => Value::None,
    }
}

/// The empty value of a type: `[]`, `""`, 0, the epoch, `None`, a record of
/// empty fields, `Err` of the empty error.
pub fn empty_value(ty: &Type) -> Value {
    scalar(ty, "")
}

/// An element invented from nothing: like the empty value, but Text and Id
/// are `"fab"`, so it is visibly not from the input.
pub fn fabricated(ty: &Type) -> Value {
    scalar(ty, "fab")
}

/// Types equal up to refinements.
pub fn same_shape(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::List(x, _), Type::List(y, _)) => same_shape(x, y),
        (Type::Option(x), Type::Option(y)) => same_shape(x, y),
        (Type::Result(a1, b1), Type::Result(a2, b2)) => same_shape(a1, a2) && same_shape(b1, b2),
        (Type::Record { fields: f, .. }, Type::Record { fields: g, .. }) => {
            f.len() == g.len()
                && f.iter()
                    .all(|(n, t)| g.iter().any(|(m, u)| m == n && same_shape(t, u)))
        }
        (a, b) => a == b,
    }
}

/// The input the identity family copies, as `(index, wrap in Ok)`: the one
/// input with the output's shape, or, when the output is a Result, the one
/// input with its Ok shape.
pub fn identity_source(scope: &Scope<'_>) -> Option<(usize, bool)> {
    let out = scope.output.as_ref()?;
    let matching = |target: &Type| -> Vec<usize> {
        scope
            .inputs
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| same_shape(t, target))
            .map(|(i, _)| i)
            .collect()
    };
    let direct = matching(out);
    if direct.len() == 1 {
        return Some((direct[0], false));
    }
    if direct.is_empty() {
        if let Type::Result(ok, _) = out {
            let wrapped = matching(ok);
            if wrapped.len() == 1 {
                return Some((wrapped[0], true));
            }
        }
    }
    None
}

/// The list type the identity family edits, if the identity is a list.
fn identity_list(scope: &Scope<'_>) -> Option<Type> {
    let (_, wrap) = identity_source(scope)?;
    let out = scope.output.as_ref()?;
    let ty = if wrap {
        match out {
            Type::Result(ok, _) => ok.as_ref(),
            _ => return None,
        }
    } else {
        out
    };
    match ty {
        Type::List(..) => Some(ty.clone()),
        _ => None,
    }
}

/// Whether a mutant can be built for this signature at all.
pub fn applies(m: Mutant, scope: &Scope<'_>) -> bool {
    let Some(out) = scope.output.as_ref() else {
        return false;
    };
    match m {
        Mutant::Empty => true,
        Mutant::OkEmpty => matches!(out, Type::Result(..)),
        Mutant::Identity => identity_source(scope).is_some(),
        _ => identity_list(scope).is_some(),
    }
}

/// The mutant's output for given inputs, or `None` where it has none on
/// those inputs (nothing to drop from an empty list).
pub fn apply(m: Mutant, inputs: &[Value], scope: &Scope<'_>) -> Option<Value> {
    let out = scope.output.as_ref()?;
    match m {
        Mutant::Empty => Some(empty_value(out)),
        Mutant::OkEmpty => match out {
            Type::Result(ok, _) => Some(Value::Ok(Box::new(empty_value(ok)))),
            _ => None,
        },
        _ => {
            let (idx, wrap) = identity_source(scope)?;
            let base = inputs.get(idx)?.clone();
            let wrapped = |v: Value| if wrap { Value::Ok(Box::new(v)) } else { v };
            if m == Mutant::Identity {
                return Some(wrapped(base));
            }
            let list_ty = identity_list(scope)?;
            let Type::List(elem, _) = &list_ty else {
                return None;
            };
            let Value::List(xs) = base else {
                return None;
            };
            let edited = match m {
                Mutant::DropOne => {
                    if xs.is_empty() {
                        return None;
                    }
                    xs[1..].to_vec()
                }
                Mutant::DuplicateOne => {
                    if xs.is_empty() {
                        return None;
                    }
                    let mut v = xs.clone();
                    v.insert(0, xs[0].clone());
                    v
                }
                Mutant::AppendFabricated => {
                    let mut v = xs.clone();
                    v.push(fabricated(elem));
                    v
                }
                Mutant::SwapAdjacent => {
                    if xs.len() < 2 {
                        return None;
                    }
                    let mut v = xs.clone();
                    v.swap(0, 1);
                    v
                }
                _ => return None,
            };
            Some(wrapped(Value::List(edited)))
        }
    }
}

/// A clause of the obligation, as the report names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    Property {
        line: usize,
        text: String,
    },
    Frame {
        line: usize,
        description: String,
    },
    /// The output type itself.
    Boundary,
}

impl Clause {
    pub fn line(&self) -> Option<usize> {
        match self {
            Clause::Property { line, .. } | Clause::Frame { line, .. } => Some(*line),
            Clause::Boundary => None,
        }
    }

    pub fn text(&self) -> String {
        match self {
            Clause::Property { text, .. } => text.clone(),
            Clause::Frame { description, .. } => format!("frame {description}"),
            Clause::Boundary => "the output type".to_string(),
        }
    }
}

/// A clause that caught a mutant: on how many inputs, and the first.
#[derive(Debug, Clone)]
pub struct Kill {
    pub clause: Clause,
    /// The boundary message, or an evaluation error, when it was not a plain `false`.
    pub message: Option<String>,
    pub inputs: Vec<Value>,
    pub output: Value,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutantVerdict {
    /// No clause was false on any input and every output passed the boundary.
    Slipped,
    Caught,
    NotApplicable,
}

/// The input a report shows for a mutant: an example that caught it when
/// one did, else the first input it ran on.
#[derive(Debug, Clone)]
pub struct Shown {
    pub inputs: Vec<Value>,
    pub output: Value,
    pub expected: Option<Value>,
    /// 1-based example number, when the input is an example's.
    pub example: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MutantRun {
    pub mutant: Mutant,
    pub verdict: MutantVerdict,
    pub inputs_tried: usize,
    /// Inputs on which at least one clause caught it.
    pub caught_inputs: usize,
    pub kills: Vec<Kill>,
    /// Clauses never false for it.
    pub slipped: Vec<Clause>,
    /// 1-based examples whose expected output differs from the mutant's.
    pub example_catches: Vec<usize>,
    pub shown: Option<Shown>,
}

/// An input on which two different outputs pass every clause.
#[derive(Debug, Clone)]
pub struct Underdetermined {
    pub inputs: Vec<Value>,
    pub example: Option<usize>,
    pub a: Value,
    pub b: Value,
    /// `a` is the example's expected output.
    pub a_is_expected: bool,
    pub candidates_tried: usize,
}

#[derive(Debug, Clone, Default)]
pub struct MutantReport {
    pub obligation: String,
    pub obligation_hash: String,
    pub example_inputs: usize,
    pub generated_inputs: usize,
    pub buckets: Vec<Bucket>,
    pub mutants: Vec<MutantRun>,
    pub search: Vec<Underdetermined>,
    /// Inputs the search covered: every example plus the budget's share of generated ones.
    pub search_inputs: usize,
    pub search_cap: usize,
    /// Clauses the search could not evaluate on any candidate, with the first error.
    pub unevaluable: Vec<(Clause, String)>,
    /// Clauses that caught no applicable mutant.
    pub uncaught: Vec<Clause>,
}

impl MutantReport {
    pub fn applicable(&self) -> usize {
        self.mutants
            .iter()
            .filter(|m| m.verdict != MutantVerdict::NotApplicable)
            .count()
    }

    pub fn slipped(&self) -> Vec<&MutantRun> {
        self.mutants
            .iter()
            .filter(|m| m.verdict == MutantVerdict::Slipped)
            .collect()
    }

    pub fn caught(&self) -> usize {
        self.mutants
            .iter()
            .filter(|m| m.verdict == MutantVerdict::Caught)
            .count()
    }

    /// The underdetermination rate's numerator for this obligation: a
    /// slipping mutant or an input with two passing outputs.
    pub fn underdetermined(&self) -> bool {
        !self.slipped().is_empty() || !self.search.is_empty()
    }

    pub fn run(&self, m: Mutant) -> Option<&MutantRun> {
        self.mutants.iter().find(|r| r.mutant == m)
    }
}

struct Case {
    inputs: Vec<Value>,
    expected: Option<Value>,
    example: Option<usize>,
}

/// How much pairs of distinct elements agree, over every list in the
/// inputs (an input, or a list field of a record input): the most fields
/// any one pair shares, then the total over pairs. Identical elements do
/// not count: nothing distinguishes them to begin with.
pub fn agreement(inputs: &[Value]) -> (usize, usize) {
    fn pairs(xs: &[Value], best: &mut usize, total: &mut usize) {
        for (i, a) in xs.iter().enumerate() {
            for b in &xs[..i] {
                if a == b {
                    continue;
                }
                if let (Value::Record(fa), Value::Record(fb)) = (a, b) {
                    let shared = fa
                        .iter()
                        .filter(|(n, v)| fb.iter().any(|(m, w)| m == n && w == v))
                        .count();
                    *best = (*best).max(shared);
                    *total += shared;
                }
            }
        }
    }
    let (mut best, mut total) = (0, 0);
    for v in inputs {
        match v {
            Value::List(xs) => pairs(xs, &mut best, &mut total),
            Value::Record(fields) => {
                for (_, x) in fields {
                    if let Value::List(xs) = x {
                        pairs(xs, &mut best, &mut total);
                    }
                }
            }
            _ => {}
        }
    }
    (best, total)
}

fn permutations(xs: &[Value]) -> Vec<Vec<Value>> {
    if xs.len() <= 1 {
        return vec![xs.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..xs.len() {
        let mut rest = xs.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, head.clone());
            out.push(tail);
        }
    }
    out
}

/// Output candidates drawn from the input: when the output is a list (or
/// `Ok` of one) whose element shape some input list has, every
/// sub-multiset of those elements, smallest first, in every order while
/// the pool is small enough for that to be cheap. A filter, a selection or
/// a reordering has its answer here.
pub fn drawn_candidates(out_ty: &Type, inputs: &[Value], scope: &Scope<'_>) -> Vec<Value> {
    let (list_ty, wrap) = match out_ty {
        Type::List(..) => (out_ty, false),
        Type::Result(ok, _) if matches!(**ok, Type::List(..)) => (ok.as_ref(), true),
        _ => return Vec::new(),
    };
    let Type::List(elem, _) = list_ty else {
        return Vec::new();
    };
    let mut pool: Vec<Value> = Vec::new();
    for (v, (_, t)) in inputs.iter().zip(&scope.inputs) {
        if let (Value::List(xs), Type::List(e, _)) = (v, t) {
            if same_shape(e, elem) {
                pool.extend(xs.iter().cloned());
            }
        }
    }
    if pool.is_empty() || pool.len() > 8 {
        return Vec::new();
    }
    let mut subsets: Vec<Vec<Value>> = (0..(1u32 << pool.len()))
        .map(|mask| {
            pool.iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, v)| v.clone())
                .collect()
        })
        .collect();
    subsets.sort_by_key(|s| s.len());
    let mut out = Vec::new();
    for subset in subsets {
        let orderings = if pool.len() <= 6 {
            permutations(&subset)
        } else {
            let mut rev = subset.clone();
            rev.reverse();
            vec![subset, rev]
        };
        for o in orderings {
            let v = Value::List(o);
            out.push(if wrap { Value::Ok(Box::new(v)) } else { v });
        }
    }
    out
}

fn kill(run: &mut MutantRun, clause: Clause, message: Option<String>, case: &Case, out: &Value) {
    if let Some(k) = run.kills.iter_mut().find(|k| k.clause == clause) {
        k.count += 1;
    } else {
        run.kills.push(Kill {
            clause,
            message,
            inputs: case.inputs.clone(),
            output: out.clone(),
            count: 1,
        });
    }
}

impl<'g> Checker<'g> {
    /// Whether evaluating an expression calls a performance (a graph
    /// reference, or the obligation itself), directly or through an
    /// abbreviation.
    pub fn has_graph_call(&self, e: &Expr) -> bool {
        fn walk(scope: &Scope<'_>, e: &Expr, depth: usize) -> bool {
            let mut found = false;
            e.walk(&mut |sub| {
                if found {
                    return;
                }
                if let Expr::Call(name, _, _) = sub {
                    if BUILTINS.contains(&name.as_str()) {
                        return;
                    }
                    match scope.abbrev(name) {
                        Some(a) if depth < 8 => found = walk(scope, &a.body, depth + 1),
                        Some(_) => {}
                        None => found = true,
                    }
                }
                if let Expr::Name(name, _) = sub {
                    if let Some(a) = scope.abbrev(name) {
                        if depth < 8 && walk(scope, &a.body, depth + 1) {
                            found = true;
                        }
                    }
                }
            });
            found
        }
        walk(&self.scope, e, 0)
    }

    /// The underdetermination check: the library against the obligation
    /// over the examples and the generated inputs, then the search.
    pub fn mutants(&mut self) -> MutantReport {
        let mut cases: Vec<Case> = Vec::new();
        for (i, ex) in self.fun.examples.iter().enumerate() {
            if let Ok(inputs) = self.example_inputs(ex) {
                let expected = self.expected_output(ex, &inputs).ok();
                cases.push(Case {
                    inputs,
                    expected,
                    example: Some(i + 1),
                });
            }
        }
        let example_inputs = cases.len();
        let generated = self.generated_inputs();
        for inputs in &generated.inputs {
            cases.push(Case {
                inputs: inputs.clone(),
                expected: None,
                example: None,
            });
        }
        let out_ty = self.output_type().clone();
        let fun = self.fun;
        let mut clauses: Vec<(Clause, Expr)> = fun
            .properties
            .iter()
            .map(|p| {
                (
                    Clause::Property {
                        line: p.line,
                        text: p.text.clone(),
                    },
                    p.expr.clone(),
                )
            })
            .collect();
        for pair in frame_pairs(&self.scope, fun) {
            clauses.push((
                Clause::Frame {
                    line: pair.line,
                    description: pair.description,
                },
                pair.implicit,
            ));
        }
        let unordered: Vec<Vec<Step>> = self.unordered().to_vec();
        let mut mutants = Vec::new();
        for m in LIBRARY {
            let mut run = MutantRun {
                mutant: m,
                verdict: MutantVerdict::NotApplicable,
                inputs_tried: 0,
                caught_inputs: 0,
                kills: Vec::new(),
                slipped: Vec::new(),
                example_catches: Vec::new(),
                shown: None,
            };
            if !applies(m, &self.scope) {
                mutants.push(run);
                continue;
            }
            self.mutant = Some(m);
            for case in &cases {
                let Some(out) = apply(m, &case.inputs, &self.scope) else {
                    continue;
                };
                run.inputs_tried += 1;
                let before = run.kills.iter().map(|k| k.count).sum::<usize>();
                if let Err(b) = from_json(&to_json(&out), &out_ty) {
                    kill(&mut run, Clause::Boundary, Some(b.to_string()), case, &out);
                } else {
                    for (clause, expr) in &clauses {
                        match self.holds(expr, &case.inputs, &out) {
                            Ok(true) => {}
                            Ok(false) => kill(&mut run, clause.clone(), None, case, &out),
                            Err(e) => kill(&mut run, clause.clone(), Some(e), case, &out),
                        }
                    }
                }
                if run.kills.iter().map(|k| k.count).sum::<usize>() > before {
                    run.caught_inputs += 1;
                }
                let example_caught = case
                    .expected
                    .as_ref()
                    .is_some_and(|exp| compare(exp, &out, &unordered).is_err());
                if example_caught {
                    run.example_catches.push(case.example.unwrap_or(0));
                }
                let replace = match &run.shown {
                    None => true,
                    Some(s) => example_caught && s.example.is_none(),
                };
                if replace {
                    run.shown = Some(Shown {
                        inputs: case.inputs.clone(),
                        output: out.clone(),
                        expected: case.expected.clone(),
                        example: case.example,
                    });
                }
            }
            self.mutant = None;
            run.slipped = clauses
                .iter()
                .map(|(c, _)| c.clone())
                .filter(|c| !run.kills.iter().any(|k| k.clause == *c))
                .collect();
            run.verdict = if run.inputs_tried == 0 {
                MutantVerdict::NotApplicable
            } else if run.kills.is_empty() {
                MutantVerdict::Slipped
            } else {
                MutantVerdict::Caught
            };
            mutants.push(run);
        }
        let uncaught: Vec<Clause> = clauses
            .iter()
            .map(|(c, _)| c.clone())
            .filter(|c| {
                !mutants
                    .iter()
                    .any(|r| r.kills.iter().any(|k| k.clause == *c))
            })
            .collect();
        let (search, search_inputs, unevaluable) = self.search(&cases, &clauses, &unordered);
        MutantReport {
            obligation: self.fun.name.clone(),
            obligation_hash: surety_syntax::normalize::canonical_file(self.graph, self.idx)
                .map(|c| c.hash)
                .unwrap_or_default(),
            example_inputs,
            generated_inputs: generated.inputs.len(),
            buckets: generated.buckets,
            mutants,
            search,
            search_inputs,
            search_cap: self.budget.search_candidates,
            unevaluable,
            uncaught,
        }
    }

    /// For each example input and the budget's share of generated ones:
    /// candidate outputs of the output type, built from the input's own
    /// scalars and the named values, smallest first; the first two that
    /// pass every clause and differ under the order claims are a finding.
    /// An example's expected output counts as the first.
    fn search(
        &mut self,
        cases: &[Case],
        clauses: &[(Clause, Expr)],
        unordered: &[Vec<Step>],
    ) -> (Vec<Underdetermined>, usize, Vec<(Clause, String)>) {
        let cap = self.budget.search_candidates;
        if cap == 0 {
            return (Vec::new(), 0, Vec::new());
        }
        // Every example, then the generated inputs whose list elements agree
        // on the most fields while differing somewhere: ties and near
        // duplicates are where an obligation's matches become ambiguous.
        let mut chosen: Vec<&Case> = cases.iter().filter(|c| c.example.is_some()).collect();
        let mut generated: Vec<((usize, usize), &Case)> = cases
            .iter()
            .filter(|c| c.example.is_none())
            .map(|c| (agreement(&c.inputs), c))
            .collect();
        generated.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        chosen.extend(
            generated
                .iter()
                .take(self.budget.search_inputs)
                .map(|(_, c)| *c),
        );
        let out_ty = self.output_type().clone();
        // Clauses that call no performance are evaluated first, so most
        // candidates die before a Python round trip.
        let expensive: Vec<bool> = clauses
            .iter()
            .map(|(_, e)| self.has_graph_call(e))
            .collect();
        let mut order: Vec<usize> = (0..clauses.len()).collect();
        order.sort_by_key(|i| expensive[*i]);
        let named = self.named_values_typed();
        let mut evaluated = vec![0usize; clauses.len()];
        let mut errors: Vec<(usize, Option<String>)> =
            (0..clauses.len()).map(|_| (0, None)).collect();
        let mut findings = Vec::new();
        let mut covered = 0;
        for case in chosen {
            covered += 1;
            let mut typed: Vec<(&Value, &Type)> = case
                .inputs
                .iter()
                .zip(self.scope.inputs.iter().map(|(_, t)| t))
                .collect();
            typed.extend(named.iter().map(|(v, t)| (v, t)));
            let domain = Domain::seeded(&typed);
            let drawn = drawn_candidates(&out_ty, &case.inputs, &self.scope);
            let mut passing: Vec<Value> = case.expected.iter().cloned().collect();
            for (tried, cand) in candidates(&out_ty, &domain, cap, drawn)
                .into_iter()
                .enumerate()
            {
                let tried = tried + 1;
                if passing.iter().any(|p| compare(p, &cand, unordered).is_ok()) {
                    continue;
                }
                if from_json(&to_json(&cand), &out_ty).is_err() {
                    continue;
                }
                let mut ok = true;
                for &i in &order {
                    evaluated[i] += 1;
                    match self.holds(&clauses[i].1, &case.inputs, &cand) {
                        Ok(true) => {}
                        Ok(false) => {
                            ok = false;
                            break;
                        }
                        Err(e) => {
                            errors[i].0 += 1;
                            if errors[i].1.is_none() {
                                errors[i].1 = Some(e);
                            }
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    continue;
                }
                passing.push(cand);
                if passing.len() >= 2 {
                    findings.push(Underdetermined {
                        inputs: case.inputs.clone(),
                        example: case.example,
                        a: passing[0].clone(),
                        b: passing[1].clone(),
                        a_is_expected: case.expected.is_some(),
                        candidates_tried: tried,
                    });
                    break;
                }
            }
        }
        let unevaluable = clauses
            .iter()
            .enumerate()
            .filter(|(i, _)| evaluated[*i] > 0 && errors[*i].0 == evaluated[*i])
            .map(|(i, (c, _))| (c.clone(), errors[i].1.clone().unwrap_or_default()))
            .collect();
        (findings, covered, unevaluable)
    }
}
