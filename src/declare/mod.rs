//! Arc 109 Stone 2 — `src/declare/`: the load-time declaration pass's home.
//!
//! **Builder, `DESIGN-STONE-the-declare-home.md`:** *"the syntax/semantics of top-level
//! declaration forms (`defn`, `defstruct`, `defenum`, `defalias`, `extend`,
//! `declare-acronyms`) — a load-time pre-pass that populates the `SymbolTable` BEFORE any
//! expression runs."* Three `eval_inner` calls in 3,600+ lines; zero `dispatch_keyword_head_value`
//! or `apply_function` calls — this pass barely knows the evaluator exists, which is what makes
//! it liftable.
//!
//! ## The one contract decision: split by PHASE, never by declaration FORM
//!
//! `defn`/`defstruct`/`defenum`/`defalias`/`extend-type`/`declare-acronyms` are forms this
//! substrate mints regularly (`defalias` and `declare-acronyms` are both recent) — a per-form
//! layout (`defn.rs`, `defstruct.rs`, …) would multiply one file per form and look prepared while
//! growing the surface, the same defect `src/numeric/`'s stone rejected for per-TYPE. PHASE is the
//! honest axis instead, because it is already this module's reason to change:
//!
//! ```text
//! src/declare/register.rs      populate the SymbolTable — the register_* fns
//! src/declare/parse.rs         read a declaration form's shape — is_*/parse_*/try_parse_*
//! src/declare/preregister.rs   the earlier pass — stubs before bodies
//! src/declare/typevar.rs       free/bound type-variable walking
//! ```
//!
//! `preregister_*` runs before `register_*`; `parse_*` serves both; `typevar` is a helper family
//! neither phase owns outright. Every placement not settled by the DESIGN doc's own reading was
//! verified against its callers, not inherited — see this stone's report for the call-site
//! evidence, including `build_delegate_body` (the DESIGN doc named this one unplaceable; its only
//! two callers are both `register_defalias`, so it ships in `register.rs`).
//!
//! ## EDGE vs IMPL — the architecture this module is one instance of
//!
//! `src/intrinsic/<domain>` is the EDGE — registration and delegation, the kernel's rim.
//! `src/<domain>/` is the IMPL — the actual work. Already built for `collection`, `edn`, `holon`,
//! `kernel`, `numeric`, `rete`, `stream`, `string`. This module has no `src/intrinsic/declare.rs`
//! edge of its own — declaration forms are consumed by the loader/freeze pipeline directly, not
//! dispatched through the intrinsic registry — but the same EDGE/IMPL discipline applies to its
//! one real boundary: **this module must never reference its own edge module.**
//!
//! ## What this stone shipped, and what it did not
//!
//! Stone 2 (this one): 44 functions (plus two small const tables and two private helper types,
//! each moved with its sole consumer) relocate out of `src/runtime.rs` into this module, split by
//! phase as above. Behaviour is unchanged — every declaration form registers identically; only
//! the location moved. `eval_tail` (the evaluator's own tail-call spine, one line past the
//! DESIGN doc's line range) stays in `runtime.rs`, as do `ClauseRegPhase` and
//! `synthesize_fn_body` — neither is on this stone's function list.
//!
//! Arc 109 update — the defclause-into-function-home stone (later than this one) moved
//! `parse_defclause_form`/`parse_extend_type_form` on from the `defclause_dispatch` region of
//! `runtime.rs` into `src/function/parse.rs`; this file's `use` block now points there instead.
//!
//! `register_defclause` / `preregister_stdlib_defclause_stub` are a named practitioner's-call in
//! the DESIGN doc (lifecycle vs. feature grouping); they ship here, split by lifecycle across
//! `register.rs` / `preregister.rs`, where they already sat.
//!
//! The facade re-point sweep (`crate::runtime::X` → `crate::value::X` for the 22 re-exported
//! names) is explicitly NOT this stone — see the DESIGN doc's "Out of scope" section.

pub(crate) mod parse;
pub(crate) mod preregister;
pub(crate) mod register;
pub(crate) mod typevar;
