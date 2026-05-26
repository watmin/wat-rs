//! FM 2-bis GATE PROBE — arc 237 records-first-class thread, Stone S0.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`
//! § 8 (rooms / traps / stepping-stones). This is the GATE: nothing in the
//! records thread is briefed until T1 is green. Unlike a stone's load-bearing
//! probe (which fails pre-implementation), a gate probe CONFIRMS a feasibility
//! assumption — passing = green light to proceed; failing = fall to the
//! substrate-registration-hook fallback.
//!
//! ── T1 — macro-emitted type declaration is picked up by the registration pass ──
//!
//! The `:wat::Record::def` / `:wat::holon::Record::def` macros must emit a
//! type-declaration form (today: struct/typeunion; soon: a `typesub` edge) and
//! have it (a) register a TypeDef in the TypeEnv and (b) flow through
//! `register_type_predicates` so `is-<Name>?` auto-synthesizes — the ∀T predicate
//! that kills the is-X? asymmetry.
//!
//! Pipeline fact (freeze.rs): `expand_all` (793) runs BEFORE
//! `register_types(expanded_user, …)` (837) before `register_type_predicates`
//! (871). So a macro-emitted decl is seen on the expanded AST. This probe proves
//! the full chain end-to-end through the actual `defmacro` surface, not by
//! reading the pipeline.
//!
//! ── T2 — `subtype?` insertion point (RECON, captured here; not a runtime probe) ──
//!
//! `is_subtype` cannot be probed before it exists. Recon finding for the S-A
//! sub-DESIGN: there are ~18 `unify(&arg_ty, &expected, …)` call sites in
//! check.rs (consistent order: arg/actual first, expected/param second), and
//! `unify` is used in BOTH directional (arg-vs-param) and symmetric contexts.
//! So directional subtyping is a LAYER decision, not a one-line insert:
//!   (a) a shared `accepts(arg, param) = is_subtype(arg,param) || unify(...).is_ok()`
//!       applied at the arg-boundary sites, OR
//!   (b) a guarded directional arm in unify's (Path,Path) case — cheaper but
//!       riskier (fires everywhere unify sees two concrete paths, including
//!       symmetric/return-position uses).
//! S-A picks the layer; this probe does not gate on it.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// T1a — a `defmacro` that emits a `struct` decl. After startup the macro-emitted
/// type must be first-class: its `is-<Name>?` predicate auto-synthesized (proves
/// the decl flowed through register_types → register_type_predicates).
#[test]
fn s0_t1a_macro_emitted_struct_synthesizes_is_predicate() {
    let src = r#"
        (:wat::core::defmacro
          (:my::defthing (name :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::struct ~name (n :wat::core::i64)))

        (:my::defthing :my::g::Widget)
    "#;
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(
        world.symbols().get(":my::g::is-Widget?").is_some(),
        "T1 FAIL: macro-emitted struct decl did NOT flow through \
         register_types + register_type_predicates (no :my::g::is-Widget?). \
         Records macro cannot self-register a TypeDef → fall to substrate hook."
    );
}

/// T1b — same for a `typeunion` decl (the hierarchy roots / relate-types forms
/// are union-shaped). Proves union decls also register + go first-class from a
/// macro emission.
#[test]
fn s0_t1b_macro_emitted_typeunion_synthesizes_is_predicate() {
    let src = r#"
        (:wat::core::defmacro
          (:my::defnum (name :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::typeunion ~name [:wat::core::i64 :wat::core::f64]))

        (:my::defnum :my::g::Num)
    "#;
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(
        world.symbols().get(":my::g::is-Num?").is_some(),
        "T1 FAIL: macro-emitted typeunion decl did NOT register + synthesize \
         :my::g::is-Num?."
    );
}
