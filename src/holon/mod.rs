//! `src/holon/` — the VSA algebra. Home to the pure, `env`/`sym`-free
//! implementation of the `:wat::holon::*` surface: HolonAST/Value/WatAST
//! conversion, the `Hologram` coordinate-cell store, the sigma-function
//! capability, outcome-enum construction (`CosineOutcome`, `DotOutcome`,
//! `VectorDecodeOutcome`, `CombineOutcome`), and the `require_*` argument
//! coercions the registered verbs lean on.
//!
//! ## The two-layer doctrine (Stone HOME-8)
//!
//! Every future home whose implementation is worth naming — not a single
//! stdlib call per verb — gets TWO layers, decided mechanically by one
//! signature test:
//!
//! - a function taking `env: &Environment` and/or `sym: &SymbolTable` is
//!   **binding**: it reaches into the running program (lookups, calling
//!   user functions, provenance). It lives in `runtime.rs` today (a future
//!   strike turns it into a `#[wat_intrinsic]` shim under
//!   `src/intrinsic/holon/`).
//! - a function taking **neither** is **algebra**: pure computation over
//!   already-evaluated values. It lives here, in `src/holon/`.
//!
//! `env`/`sym` in a signature is a line the compiler already enforces on
//! every function in the crate — this home just draws the module boundary
//! along it. `string` and `time` drew the same line before HOME-8 named
//! it (`src/string/` + `src/intrinsic/string.rs`, `src/time.rs` +
//! `src/intrinsic/time.rs`); `src/intrinsic/holon/` is this home's
//! registered-interface half, built by the sibling strike on top of what
//! lives here.
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
//!   `VectorDecodeOutcome`, `CombineOutcome`, `DegenerateSide`) and the
//!   shared rete-Fallback projection they feed.
//! - `require` — `Value -> holon-domain-type` coercion helpers
//!   (`Hologram`, `Vector`, `OnlineSubspace`, `Reckoner`, `Engram`,
//!   `EngramLibrary`, plus primitive `String`/`f64`/`Function` args).
//!
//! `runtime.rs` calls into every one of these for the binding half of the
//! same verbs; nothing here reaches back into `runtime.rs`'s evaluator
//! (`eval_inner`, `Environment`, `SymbolTable`) — that boundary is the
//! whole point.

mod ast;
mod outcome;
mod require;

pub mod hologram;
pub mod sigma;

pub(crate) use ast::*;
pub(crate) use outcome::*;
pub(crate) use require::*;
