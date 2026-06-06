//! The runtime value model — the data the interpreter computes with; grows as
//! the migration lifts Value/Environment/SymbolTable/… here. This home is the
//! first destination in the great migration out of the flat runtime.rs monolith
//! (Stone 251.2a); each subsequent stone lifts more segments in.

pub mod encoding_ctx;
pub mod frame;
pub mod observe;
pub mod signal;

pub use encoding_ctx::EncodingCtx;
pub use frame::{FrameInfo, snapshot_call_stack};
pub(crate) use frame::{FrameGuard, replace_top_frame};
pub use observe::{Provenance, TrackedValue, ValueSnapshot};
pub use signal::{EvalBreak, EvalSignal, RuntimeError, RuntimeErrorKind};
