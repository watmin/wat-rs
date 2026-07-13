//! vigilatum: 2026-06-06T04:56:04Z — UPDATED-vigilia 13-spell guard L1+L2=0 (universal-7:
//! intueri/solvere/conformare/purgare/struere/sequi/temperare + exigere +
//! excusare + test-kind complectens/vocare + conditional perspicere [fired:
//! nested generics present] + circumspicere LAST; secare not mustered — no
//! parallel primitives in-home; mora not fired — no duration waits). Two full
//! inward rounds (12-cast + 9-cast fresh-eyes convergence) + the perimeter:
//! 47 findings fought to zero across three sweeps (R1 A-W 25, R2 17,
//! perimeter 5); 4 L1 killed (variadic arity-lie, untracked deferral,
//! message-less panic, unwitnessed hash-IS-identity claim); 11 runes all
//! verified against the live tree; clippy-clean in-home. Canonical record:
//! docs/arc/2026/06/249-total-pure-macros/WARD-MACROS-UPDATED-GUARD-AGGREGATE.md.
//! RE-EARNED 2026-06-06T04:56:04Z (diff-scoped, the 245 clear: +3 pure reflection
//! verbs on the allow-list [signature-of-fn / extract-arg-names / extract-arg-types,
//! handlers verified pure-total]; the ONE contextual fn-opacity [signature-of-fn's
//! literal-fn arg = signature-only; a BLANKET opacity was caught REOPENING F5 at
//! scoring and reverted]; the exception survived a 7-attempt adversarial breach
//! ledger [body-inert/if-smuggle/let-smuggle/user-defn/nondeterminism/totality/
//! empty-list — ALL HELD]; +3 fence witnesses incl. the inert-impure-body proof;
//! gates: lib 923/0/1, hygiene+caller+reader probes green, clippy-in-home empty).
//! Declared invariants, each enforced by a living gate:
//! (1) variable capture structurally prevented — sets-of-scopes tagging
//!     (tests/probe_macro_hygiene_capture.rs, end-to-end incl. 2-scope nesting);
//! (2) hash-IS-identity for macro aliases — alias-vs-direct canonical-hash
//!     equality (tests/probe_hash_scope_renumber.rs);
//! (3) default-deny purity fence — heads off the allow-list refuse
//!     (RefusedInMacro witnesses in tests.rs, incl. the macroexpand-1 deny);
//! (4) definition-time validation hoist — the pre-validated eval path admits
//!     exactly one sanctioned caller
//!     (tests/probe_macro_eval_prevalidated_caller_gate.rs);
//! (5) EXPANSION_DEPTH_LIMIT is the limit — depth refusal witnessed in-module;
//!     macroexpand fixpoint failure witnessed (tests/wat_make_deftest.rs);
//! (6) expand-before-register order — freeze-pipeline canary
//!     (src/freeze.rs expand_runs_before_register_defines_phase_order) +
//!     LOAD-BEARING ORDER markers at every out-of-freeze call site.
//!
//! `defmacro` — parse-time macro expansion with Racket sets-of-scopes
//! hygiene (Flatt 2016).
//!
//! Per 058-031: macros transform source forms BEFORE hashing, signing,
//! type-checking, or evaluation. Two source files that differ only in
//! macro aliases (e.g. `Subtract` vs `Blend _ _ 1 -1`) expand to the
//! same canonical AST and the same hash — the substrate commit of
//! hash-IS-identity holds.
//!
//! # File map
//!
//! - `registry` — storage (`MacroDef`, `MacroRegistry`)
//! - `parse`    — form → `MacroDef` + registration helpers
//! - `expand`   — call-site → AST (template walk, hygiene, fixpoint)
//! - `eval`     — the fenced expand-time evaluator (`macro_eval`; default-deny pure-total gate)
//! - `error`    — `MacroError` / `MacroErrorKind`
//!
//! # Hygiene by construction
//!
//! A macro that introduces a name (`(let ((tmp ,x)) ...)`) cannot
//! collide with a user's `tmp` in the caller's scope. The mechanism,
//! per FOUNDATION's specified algorithm:
//!
//! 1. At each macro invocation, allocate a fresh [`ScopeId`].
//! 2. Walk the macro's template. Every identifier whose origin is the
//!    template source has the fresh scope added to its scope set.
//! 3. Identifiers that came in via macro arguments (substituted at
//!    `,x` unquote sites) KEEP their original scope sets.
//! 4. Lexical-scope lookup compares `(name, scope_set)` pairs — so
//!    `tmp[{macro-scope}]` and `tmp[{user-scope}]` resolve to distinct
//!    bindings.
//!
//! Variable capture is structurally impossible (not discipline-enforced)
//! for quasiquote-template bodies via the scope-set mechanism above.
//! Program-body macros (arc 249.2b-ii) enforce hygiene by a default-deny
//! refusal: `check_program_body_hygiene` (expand.rs, gate E) rejects any
//! program body whose quasiquote introduces a literal binder name.
//!
//! # What this slice supports
//!
//! - `defmacro` forms with quasiquote-template bodies: `` ` `` for the
//!   template, `,expr` for parameter substitution, `,@expr` for list
//!   splicing.
//! - Fixpoint expansion (macros expand to more macros until no more
//!   remain). Depth limit prevents pathological infinite expansion.
//! - Full hygiene for the classic capture pattern.
//! - Threading macros `->` (thread-first) and `->>` (thread-last): ordinary
//!   registered wat macros in `wat/core.wat`; rehomed from Rust desugars in
//!   arc 249.3.
//! - `keyword/of` special form: constructs parametric keywords
//!   (e.g. `(:wat::core::keyword/of :Head :Arg)` → `:Head<Arg>`);
//!   ordinary registered wat macro in `wat/core.wat`; rehomed from Rust
//!   desugar in arc 249.4.
//! - Computed-unquote `,(expr)`: a List whose head is a Keyword is
//!   evaluated at expand-time via `macro_eval` (the default-deny
//!   pure-total fenced evaluator; arc 249.2b-i, building on the
//!   arc 143 slice 2 computed-unquote path) with macro params substituted.
//!
//! # Scope
//!
//! This slice handles quasiquote-template + program-body macro expansion.
//! Typed-macro `:AST<T>` checking lives in the type checker's phase
//! (058-032).

use crate::ast::WatAST;

pub(crate) mod error;
pub(crate) mod registry;
pub(crate) mod parse;
pub(crate) mod expand;
pub(crate) mod eval;
pub mod error_edn;

pub use error::{MacroError, MacroErrorKind};
pub use registry::{MacroDef, MacroRegistry};
pub use expand::{expand_all, expand_fully, expand_once};
pub use parse::{register_defmacros, register_stdlib_defmacros};
pub use expand::EXPANSION_DEPTH_LIMIT;

/// A batch of expanded forms, or the macro error that stopped expansion.
pub(super) type ExpandBatch = Result<Vec<WatAST>, MacroError>;

#[cfg(test)]
mod tests;
