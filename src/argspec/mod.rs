//! # Argspec — canonical parser for the `[name <- :T name <- :T ... [& rest <- :T]]` triple form.
//!
//! ## Why this module exists — the failure class being eliminated
//!
//! The substrate carried FOUR copies of canonical argspec-parsing logic before
//! arc 241 (A1 `parse_fn_signature` runtime.rs:6750 / A2 `parse_fn_signature_for_check`
//! check.rs:15205 / A3 `parse_fn_signature_for_check_diag` check.rs:15258 / A4
//! `parse_defclause_args` runtime.rs:6880). Duplication ran all the way to the
//! error-enum class — the same structural failure ("name slot is not a Symbol")
//! produced three different error variants across sites: `RuntimeError::MalformedForm`
//! (A1+A4), `()` silenced (A2), `CheckError::MalformedForm` (A3). Two binding sites
//! could accept different forms; the substrate accepted what the next site silently
//! rejected; LLM co-authors generated code that worked in one site and broke in
//! another.
//!
//! Per failure-engineering doctrine: **eliminate the class**. State to make
//! unrepresentable: *two binding sites accepting different arg-vector forms*. This
//! module mints the ONE canonical parser; subsequent stones (241.2/241.3) migrate
//! callers; the class is closed when the four old parsers retire.
//!
//! ## Scope
//!
//! Argspec parses ONLY the canonical `[name <- :T name <- :T ... [& rest <- :T]]`
//! triple form. The ret-clause (`-> :Ret`) is NOT argspec's concern — fn-form parsers
//! (defn, fn, fn type-signature) compose argspec + ret-clause at the form level.
//! Per `FORM-COLLAPSE-NOTES.md` line 184:
//!
//! > Arc 241's `parse_argspec_triples` parses the canonical 3-slot triple uniformly
//! > across all binding sites. Form-level parsers decode the per-binding metadata map
//! > separately and associate by symbol.
//!
//! ## What this module owns
//!
//! The canonical parsing of the flat `name <- :T name <- :T` triple form when it
//! appears inside a `WatAST::Vector` at any binding site. Per-site invariants
//! (`allow_rest_binder`) live in `ParseOptions`.
//!
//! ## What this module does NOT own
//!
//! Form-shape parsing (def / defn / defstruct / defenum each parse their own
//! form-level shape including arity checks, name keyword, body expression, etc.).
//! Only the **argspec-triples region** — the Vector whose items are the flat
//! `name <- :T` triples — routes through this parser. Ret-clause (`-> :Ret`) is
//! fn-form-parser concern; those callers split at `->` before calling this parser.
//!
//! ## Migration plan
//!
//! - **Stone 241.1** — minted the canonical parser ALONGSIDE the old ones.
//!   A1/A2/A3/A4 remained untouched; probe `tests/probe_arc241_stone1_argspec_canonical.rs`
//!   verified the new parser independently. DONE.
//! - **Stone 241.2** — migrated A1+A2+A3 (the three fn-form parsers: runtime.rs
//!   `parse_fn_signature` + check.rs `parse_fn_signature_for_check` +
//!   `parse_fn_signature_for_check_diag`) to route through here. DONE.
//! - **Stone 241.3** — migrated A4 (runtime.rs `parse_defclause_args`) to route
//!   through here; Phase 1 closed (all 4 parsers route through canonical). DONE.
//! - **Stone 241.4** — added `&` rest-binder parsing (`allow_rest_binder = true` path);
//!   A4 inlined at `parse_defclause_clause` (wrapper deleted as thin braid). DONE.
//! - **Stone 241.5** — runtime dispatch wiring in `eval_clause_set` (unblocks probe
//!   237.8b Gate 1). PENDING.

mod error;
mod parse;

pub use error::ArgSpecError;
pub use parse::{parse_argspec_triples, ArgSpec, ParseOptions};
