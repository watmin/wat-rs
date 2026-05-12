//! Arc 170 slice 3 Gap C V2 — regression probes for top-level `do` splicing of `def`/`defn`.
//!
//! Three probes confirm that `(:wat::core::do ...)` at top level splices
//! its children uniformly across ALL substrate passes, not just the
//! `def`-legality check (arc 157) and runtime eval (arc 136). The gap:
//! `register_defines` did not recurse into top-level `do`, so functions
//! defined via `def`/`defn` inside a top-level `do` were invisible to
//! `resolve_references` at startup time.
//!
//! All three probes FAIL before Gap C V2 ships; all three PASS after.
//!
//! Probe 1: `do` of two `def` forms.
//! Probe 2: `do` of two `defn` forms (via `defn` macro expansion to `def`).
//! Probe 3: `defmacro` that emits `do` wrapping `defn` — the Phase E use case.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — two `def`s wrapping `fn` inside a top-level `do`.
///
/// Both `:my::helper` and `:my::main` must be registered in the symbol
/// table after startup. Before Gap C, `resolve_references` fails because
/// `:my::helper` is called inside the `do` before it enters `sym.functions`.
#[test]
fn probe_do_def_two_vars_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::def :my::helper (:wat::core::fn [] -> :wat::core::i64 42))
          (:wat::core::def :my::main (:wat::core::fn [] -> :wat::core::i64 (:my::helper))))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — two `defn` forms inside a top-level `do`.
///
/// `defn` is a macro that expands to `(:wat::core::def name (:wat::core::fn ...))`.
/// After macro expansion the forms inside the `do` are `def`-of-fn, same
/// as Probe 1. Both must register.
#[test]
fn probe_do_defn_via_expansion() {
    let src = r#"
        (:wat::core::do
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 3 — `defmacro` that emits a top-level `do` wrapping a `defn`.
///
/// The Phase E use case: a `deftest`-style macro emits
/// `(:wat::core::do prelude-form body-form)` at top level. The prelude
/// defines a helper; the body references it. Both must register.
#[test]
fn probe_do_def_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
             ~body))

        (:my::probe (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}
