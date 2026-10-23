//! An example runs against the lines (slice 6, piece 1): every clause over
//! each example's own input and output, before any performance is looked
//! for, under `check` and `mutants` alike. An `e:` block is a run the author
//! wrote, so a clause false on it is the obligation's finding whatever the
//! performance does: both the example and the line are the author's, and
//! one of them is wrong. These evaluations are not runs of a performance.

use surety_syntax::Expr;

use crate::checker::Checker;
use crate::eval::{Env, is_obligation_fault};
use crate::mutants::Clause;
use crate::value::Value;

/// One clause on one example's own pair.
#[derive(Debug, Clone, PartialEq)]
pub enum OnExample {
    /// The clause holds; `reached` when the example decides it (`reach.rs`).
    Holds { reached: bool },
    /// The clause is false on the example: a contradiction.
    False,
    /// The clause calls a performance that is not there (a graph reference,
    /// or the obligation itself): unchecked on this example.
    Unchecked(String),
    /// The clause cannot be evaluated on the author's own values: an
    /// overflow of its arithmetic, or a value of the wrong kind. The
    /// obligation's, like a contradiction.
    Fault(String),
}

/// An example whose input and output both decoded.
#[derive(Debug, Clone)]
pub struct ExampleCase {
    /// Its index among the obligation's examples, from 0.
    pub index: usize,
    pub line: usize,
    pub note: Option<String>,
    pub inputs: Vec<Value>,
    pub expected: Value,
}

/// One clause against every example.
#[derive(Debug, Clone)]
pub struct ClauseAgainst {
    pub clause: Clause,
    pub sentence: Option<String>,
    /// One outcome per example in `Against::examples`.
    pub on: Vec<OnExample>,
}

/// The examples against the lines.
#[derive(Debug, Clone, Default)]
pub struct Against {
    pub examples: Vec<ExampleCase>,
    /// In the order of `Checker::clauses`: the properties, then the frame.
    pub clauses: Vec<ClauseAgainst>,
}

/// A clause false on an example, or not evaluable on it.
#[derive(Debug, Clone)]
pub struct Contradiction<'a> {
    pub clause: &'a ClauseAgainst,
    pub example: &'a ExampleCase,
    /// Set for a fault: why the clause could not be evaluated.
    pub fault: Option<&'a str>,
}

impl Against {
    /// Examples evaluated against the lines.
    pub fn evaluated(&self) -> usize {
        self.examples.len()
    }

    /// Every clause false on an example, and every fault, by clause then
    /// example.
    pub fn contradictions(&self) -> Vec<Contradiction<'_>> {
        let mut out = Vec::new();
        for c in &self.clauses {
            for (on, example) in c.on.iter().zip(&self.examples) {
                let fault = match on {
                    OnExample::False => None,
                    OnExample::Fault(m) => Some(m.as_str()),
                    _ => continue,
                };
                out.push(Contradiction {
                    clause: c,
                    example,
                    fault,
                });
            }
        }
        out
    }

    /// No clause is false on an example, and none faults.
    pub fn clean(&self) -> bool {
        self.contradictions().is_empty()
    }

    /// The examples, by index, on which the clause at `k` is the
    /// obligation's finding.
    pub fn contradicted_on(&self, k: usize) -> Vec<usize> {
        let Some(c) = self.clauses.get(k) else {
            return Vec::new();
        };
        c.on.iter()
            .zip(&self.examples)
            .filter(|(on, _)| matches!(on, OnExample::False | OnExample::Fault(_)))
            .map(|(_, e)| e.index)
            .collect()
    }

    /// The 1-based numbers of the examples that reach the clause at `k`.
    pub fn reached_by(&self, k: usize) -> Vec<usize> {
        self.clauses[k]
            .on
            .iter()
            .zip(&self.examples)
            .filter(|(on, _)| matches!(on, OnExample::Holds { reached: true }))
            .map(|(_, e)| e.index + 1)
            .collect()
    }

    /// Why a pin refuses: each contradiction in one line.
    pub fn refusal(&self) -> String {
        self.contradictions()
            .iter()
            .map(|c| {
                let example = format!("example {} (line {})", c.example.index + 1, c.example.line);
                let line = c
                    .clause
                    .clause
                    .line()
                    .map(|l| format!("line {l}"))
                    .unwrap_or_default();
                match c.fault {
                    None => format!("{example} contradicts {line}: {}", c.clause.clause.text()),
                    Some(m) => format!("{line} cannot be evaluated on {example}: {m}"),
                }
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl<'g> Checker<'g> {
    /// Every clause over each example's own pair, `...` filled from the
    /// inputs. Computed once per checker: `check` and its `mutants` share it.
    pub fn against(&mut self) -> Against {
        if let Some(a) = &self.against {
            return a.clone();
        }
        // The obligation's own name means its performance here, never a mutant.
        let mutant = self.mutant.take();
        let fun = self.fun;
        let mut examples = Vec::new();
        for (index, ex) in fun.examples.iter().enumerate() {
            let Ok(inputs) = self.example_inputs(ex) else {
                continue;
            };
            let Ok(expected) = self.expected_output(ex, &inputs) else {
                continue;
            };
            examples.push(ExampleCase {
                index,
                line: ex.line,
                note: ex.note.clone(),
                inputs,
                expected,
            });
        }
        let sentences: Vec<Option<String>> =
            fun.properties.iter().map(|p| p.sentence.clone()).collect();
        let mut clauses = Vec::new();
        for (k, (clause, expr)) in self.clauses().into_iter().enumerate() {
            let on = examples
                .iter()
                .map(|ex| self.on_example(&expr, ex))
                .collect();
            clauses.push(ClauseAgainst {
                clause,
                sentence: sentences.get(k).cloned().flatten(),
                on,
            });
        }
        let a = Against { examples, clauses };
        self.mutant = mutant;
        self.against = Some(a.clone());
        a
    }

    fn on_example(&mut self, expr: &Expr, ex: &ExampleCase) -> OnExample {
        let named: Vec<(String, Value)> = self
            .scope
            .inputs
            .iter()
            .map(|(n, _)| n.clone())
            .zip(ex.inputs.iter().cloned())
            .collect();
        let env = Env::new(&named, &ex.expected);
        match self.eval_bool(expr, &env) {
            Ok(true) => OnExample::Holds {
                reached: self.reaches(expr, &env),
            },
            Ok(false) => OnExample::False,
            Err(m) if is_obligation_fault(&m) => OnExample::Fault(m),
            // The performance a clause calls is not the author's: with it
            // missing, or failing, the clause is unchecked on the example.
            Err(m) if self.has_graph_call(expr) => OnExample::Unchecked(m),
            Err(m) => OnExample::Fault(m),
        }
    }
}
