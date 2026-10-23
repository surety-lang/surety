//! Surety checker: boundary validator, example checker, property evaluator,
//! frame rule, generated inputs, evidence record.

pub mod against;
pub mod checker;
pub mod diff;
pub mod eval;
pub mod examples;
pub mod frame;
pub mod gap;
pub mod generate;
pub mod mutants;
pub mod reach;
pub mod value;

pub use against::{Against, ClauseAgainst, Contradiction, ExampleCase, OnExample};
pub use checker::{
    AcceptReport, Cause, Checker, Counterexample, Determinism, ExampleReport, Failure, FrameReport,
    GenRun, LookHere, LookKind, Outcome, Party, PropVerdict, PropertyReport, Ran, Report, Verdict,
};
pub use diff::{Class, DiffReport, Difference};
pub use eval::{OBLIGATION_FAULT, is_obligation_fault};
pub use gap::{Change, Divergence, Gap, Step, diverge, gaps, pretty, pretty_inputs};
pub use generate::{Bucket, Budget, Domain, Generated};
pub use mutants::{Clause, Mutant, MutantReport, MutantRun, MutantVerdict, Underdetermined};
pub use value::{BoundaryError, Value};
