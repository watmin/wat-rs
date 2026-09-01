//! `src/holon/` — the VSA algebra's home. Implementation of the `:wat::holon::*`
//! surface: HolonAST/Value/WatAST conversion, the `Hologram` coordinate-cell
//! store, the sigma-function capability, outcome-enum construction
//! (`CosineOutcome`, `DotOutcome`, `VectorDecodeOutcome`, `CombineOutcome`),
//! the presence?/coincident? measurement family, and the `require_*` argument
//! coercions the registered verbs lean on.
//!
//! ## The doctrine — corrected by the holon-into-parity stone
//!
//! Stone HOME-8 (2026-08-26) drew this home's boundary as a signature test:
//! a function taking `env: &Environment` and/or `sym: &SymbolTable` was
//! **binding** and had to stay in `runtime.rs`; only a function taking
//! **neither** could live here. The premise was that `Environment` and
//! `SymbolTable` were `runtime.rs`'s own evaluator types, so a function
//! naming either was, by definition, reaching outside this home.
//!
//! **That premise was false.** `Environment` lives in
//! `src/value/environment.rs`; `SymbolTable` lives in
//! `src/value/symbol_table.rs` — and both already lived there on `d43f75887`,
//! the very commit that wrote the rule. That commit's own
//! `runtime.rs:758-770` merely `pub use`-re-exports them for zero-churn call
//! sites. The rule was authored against that re-export as though it were
//! the types' home — the same facade artifact that elsewhere in this
//! campaign inflated every impl home's measured cycle count and made
//! `check.rs` import `SymbolTable` from `runtime`. Everywhere else it misled
//! an import; here it authored an architectural split with nothing under
//! it: every sibling impl home built during this campaign (`collection`,
//! `edn`, `numeric`, `record`, `reflect`) holds functions taking `env`/`sym`
//! freely, and eleven of the twelve functions the split had been excluding
//! from this home call no evaluator at all.
//!
//! **The corrected rule — the one every sibling impl home already
//! observes:** an impl home must not reference its own registration edge
//! (that would be a cycle); it may use `crate::value` types and call the
//! evaluator, exactly as `collection`, `edn`, `numeric`, `record` and
//! `reflect` do. `env`/`sym` in a signature never drew a real module
//! boundary here — the crate already enforces it as an ordinary parameter
//! type, the same way it does in every other home.
//!
//! A future reader should not re-derive the struck split from the same
//! mistake: see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-holon-into-parity.md` before
//! reintroducing an `env`/`sym` test as this home's boundary.
//!
//! ## Layout
//!
//! - [`hologram`] — the `Hologram` therm-routed coordinate-cell store
//!   (formerly top-level `src/hologram.rs`).
//! - [`sigma`] — the ambient `SigmaFn` capability used by `presence?` /
//!   `coincident?` (formerly top-level `src/sigma.rs`).
//! - `ast` — HolonAST ⟷ `Value`/`WatAST` conversion algebra, plus the
//!   shared `Bundle` capacity guard.
//! - `outcome` — outcome-enum constructors (`CosineOutcome`, `DotOutcome`,
//!   `VectorDecodeOutcome`, `CombineOutcome`, `DegenerateSide`), the
//!   value-in measurement functions that build them (`cosine_outcome_from_values`,
//!   `dot_outcome_from_values`), and the shared rete-Fallback
//!   projection/classification (`project_holon_rete_fallback`,
//!   `classify_fallback_outcome`) they feed.
//! - `coincident` — the `presence?`/`coincident?` predicate family
//!   (`presence_q_from_values`, `coincident_q_from_values`) and the
//!   `eval-*-coincident?` embedded-program family that reduces to the same
//!   measurement.
//! - `require` — `Value -> holon-domain-type` coercion helpers
//!   (`Hologram`, `Vector`, `OnlineSubspace`, `Reckoner`, `Engram`,
//!   `EngramLibrary`, plus primitive `String`/`f64`/`Function` args).
//! - `codec` — the vector wire format (`dim:u32-LE ++ packed-cells`):
//!   encode/decode, held to a stricter purity bar than its siblings above —
//!   no wat type (`WatAST`/`Value`/`RuntimeError`/`Span`/`Environment`/
//!   `SymbolTable`) anywhere in its signatures. Stone layer-2.
//!
//! `codec`'s bar is earned on its own evidence — a wire format has no
//! legitimate reason to name any wat type — and is independent of the
//! doctrine above: it does not loosen just because the general rule did,
//! and the general rule does not fold into it either.
//!
//! `src/intrinsic/holon/` is the registration edge this home implements
//! for — registration and delegation only. This home must never reach back
//! into it; that is the one cycle the corrected rule forbids. Every
//! function here may use `crate::value` types (`Environment`, `SymbolTable`,
//! `Value`, …) and call the evaluator (`eval_inner` and its neighbors)
//! freely, imported from their canonical module — never through
//! `runtime.rs`'s re-export of them (see the doctrine's history above).

mod ast;
mod codec;
mod coincident;
mod outcome;
mod require;

pub mod hologram;
pub mod sigma;

pub(crate) use ast::*;
pub(crate) use codec::*;
pub(crate) use coincident::*;
pub(crate) use outcome::*;
pub(crate) use require::*;
