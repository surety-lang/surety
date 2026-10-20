//! Surety checker: boundary validator, example checker, property evaluator,
//! frame rule.

pub mod checker;
pub mod eval;
pub mod examples;
pub mod frame;
pub mod value;

pub use checker::{
    Checker, ExampleReport, FrameReport, PropVerdict, PropertyReport, Report, Verdict,
};
pub use value::{BoundaryError, Value};
