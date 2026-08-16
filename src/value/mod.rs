//! vigilatum: 2026-06-07T01:16:28Z — value/ ward (arc 251.2), L1+L2=0, clippy-clean in-home.
//!
//! Spells cast: intueri · solvere · conformare · purgare · struere · perspicere · excusare ·
//! exigere · circumspicere (perimeter, last). sequi + temperare declared converged-by-inspection
//! (data-model home: no multi-step state-threading chains; a verbatim lift adds no new hot loops).
//! Convergence by per-finding fix-verification + excusare rune-confirmation + orchestrator drift
//! greps (caught 15 stale check.rs:NNNN citations + an incomplete de-deferral the casts walked past).
//!
//! Runes (all excusare-warranted): solvere(historical-shape) — the extract_classifier back-arc
//! (SpawnOutcome/ProgramHandleInner, the other historical-shape subject this rune once covered,
//! was purged in arc 278's vacate-spawn-outcome strike: a locus has no return value, so the
//! arc-060 join-result chain had no job left); solvere(load-bearing-coupling) — Config on EncodingCtx (sole
//! config-inheritance carrier into spawned sub-programs); purgare(future-fixture) — eval_redef_allowed
//! write-only scaffolding.
//!
//! Findings closed: a real latent bug (render_value's List arm rendered UNBOUNDED — lifted from the
//! monolith; now SHOW_MAX_LEN-guarded like every sibling arm); FrameGuard #[must_use]; deferral-prose
//! driven out of runes + docs; stale line-citations made name-based; BindingMetadata alias minted.
//! perspicere's 14 idiomatic 2-level wrappers weighed L3 (Arc<Vec<Value>>/Option<Arc<T>> ARE their
//! definitions — an alias would obscure). Out-of-home conformare findings (services/freeze) banked
//! for the stdio/ + freeze/ wards; Duration(u64) type-enforcement banked (#188). Full record:
//! docs/arc/2026/06/251-types-as-forms/SCORE-STONE-251.2-ward.md.
//!
//! The runtime value model — the data the interpreter computes with; grows as
//! the migration lifts Value/Environment/SymbolTable/… here. This home is the
//! first destination in the great migration out of the flat runtime.rs monolith
//! (Stone 251.2a); each subsequent stone lifts more segments in.

pub mod encoding_ctx;
pub mod environment;
pub mod frame;
pub(crate) mod numeric_order;
pub mod observe;
pub mod pmap;
pub mod signal;
pub mod symbol_table;
// The home's namesake module is value.rs (the Value enum); crate::value::Value collapses the path via the re-export below. The lint fires on the domain-required internal organization.
#[allow(clippy::module_inception)]
pub mod value;

pub use encoding_ctx::EncodingCtx;
pub use environment::{Function, FunctionBody, Environment, EnvBuilder, BoundEntry, ReteContract};
pub use frame::{FrameInfo, snapshot_call_stack};
pub(crate) use frame::{FrameGuard, replace_top_frame, MacroCallSiteGuard, current_macro_call_site, ANON_FN_SYMBOL};
pub use observe::{Provenance, TrackedValue, ValueSnapshot};
pub use signal::{EvalBreak, EvalSignal, RuntimeError, RuntimeErrorKind};
pub use symbol_table::SymbolTable;
pub use value::{Value, AggregateValue, HolonForm, EnumValue,
    Clause, ClauseSet, ClauseAttempt, ClauseFailureReason,
    ExtendDef, KeyEligibility, NotAKeyReason};
