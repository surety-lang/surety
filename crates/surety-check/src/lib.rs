//! Surety checker: boundary validator, example checker, property evaluator,
//! frame rule.

pub mod checker;
pub mod examples;
pub mod value;

pub use checker::{Checker, ExampleReport, Report, Verdict};
pub use value::{BoundaryError, Value};
