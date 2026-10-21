//! Surety checker: boundary validator, example checker, property evaluator,
//! frame rule, generated inputs, evidence record.

pub mod checker;
pub mod eval;
pub mod examples;
pub mod frame;
pub mod generate;
pub mod value;

pub use checker::{
    Checker, Counterexample, Determinism, ExampleReport, Failure, FrameReport, GenRun, Outcome,
    PropVerdict, PropertyReport, Report, Verdict,
};
pub use generate::{Bucket, Budget, Domain, Generated};
pub use value::{BoundaryError, Value};
