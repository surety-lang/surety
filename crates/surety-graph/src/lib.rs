//! The graph: definitions by canonical hash, performances by target and
//! structural hash, evidence pinned to both, a kernel that admits a record
//! only when its claims are backed by recorded runs, and the queries a
//! reader or a model uses instead of reading files.

pub mod pin;
pub mod query;
pub mod record;
pub mod store;

pub use pin::{PinState, Pinned, Verification, pin, references_of, type_references_of, verify};
pub use query::{NodeView, PerformanceView, State, StatusRow, show, state, state_of_name, status};
pub use record::{FORMAT, Render, Rendered, TARGET, admit, budget_of, evidence, strict_passed};
pub use store::{Entry, Namespace, References, Store, text_hash};
