//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `do` splicing of `struct` forms.
//!
//! Two probes confirm that `(:wat::core::struct ...)` forms inside a top-level
//! `(:wat::core::do ...)` pre-register their accessor stubs in `sym.functions`
//! (via `preregister_fn_defs_in_do`) before `resolve_references` runs.
//!
//! Gap C V2 extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extended it to handle the legacy `define` form.
//! Gap F-1 extends it to handle `struct` forms — pre-generating the
//! `Type/new` constructor and per-field accessors as stubs so the outer
//! resolver can validate calls to those names inside helper `define` bodies
//! that appear in the same `do` block.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: struct declaration + define using accessor, both in top-level `do`.
//! Probe 2: `defmacro` that emits `do` wrapping struct + define — the Phase E use case directly.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — struct + define using its constructor, both inside a top-level `do`.
///
/// `:my::State/new` must be registered in the symbol table after startup.
/// Before Gap F-1, `resolve_references` fails because
/// `preregister_fn_defs_in_do` does not process `struct` forms, so
/// `:my::State/new` never enters `sym.functions`.
#[test]
fn probe_do_struct_accessor_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::struct :my::State
            (counter :wat::core::i64))
          (:wat::core::define (:my::main -> :my::State)
            (:my::State/new 42)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::State/new").is_some(), ":my::State/new not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping struct + define.
///
/// The Phase E use case: a `deftest`-style macro emits
/// `(:wat::core::do struct-form (:wat::core::define (name -> type) body))`
/// at top level. The struct accessors must be pre-registered for the
/// define body to resolve.
#[test]
fn probe_do_struct_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::struct :my::probe::Point
               (x :wat::core::i64)
               (y :wat::core::i64))
             ~body))

        (:my::probe
          (:wat::core::define (:my::probe::make-origin -> :my::probe::Point)
            (:my::probe::Point/new 0 0)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::probe::Point/new").is_some(), ":my::probe::Point/new not registered");
    assert!(world.symbols().get(":my::probe::Point/x").is_some(), ":my::probe::Point/x not registered");
    assert!(world.symbols().get(":my::probe::Point/y").is_some(), ":my::probe::Point/y not registered");
    assert!(world.symbols().get(":my::probe::make-origin").is_some(), ":my::probe::make-origin not registered");
}
