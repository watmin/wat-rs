//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `let` splicing of `struct` forms.
//!
//! Two probes confirm that `(:wat::core::struct ...)` forms in the body of a
//! top-level `(:wat::core::let ...)` pre-register their accessor stubs in
//! `sym.functions` (via `preregister_fn_defs_in_let`) before `resolve_references` runs.
//!
//! Gap D extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extended it to handle the legacy `define` form.
//! Gap F-1 extends it to handle `struct` forms — pre-generating the
//! `Type/new` constructor and per-field accessors as stubs, consistent
//! with the parallel `preregister_fn_defs_in_do` extension.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: struct declaration + define using accessor, both in top-level `let` body.
//! Probe 2: `defmacro` that emits `let` wrapping struct + define.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — struct + define using its constructor, both in the body of a top-level `let`.
///
/// `:my::State/new` must be registered in the symbol table after startup.
/// Before Gap F-1, `resolve_references` fails because
/// `preregister_fn_defs_in_let` does not process `struct` forms, so
/// `:my::State/new` never enters `sym.functions`.
#[test]
fn probe_let_struct_accessor_visible() {
    let src = r#"
        (:wat::core::let []
          (:wat::core::struct :my::State
            (counter :wat::core::i64))
          (:wat::core::define (:my::main -> :my::State)
            (:my::State/new 42)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::State/new").is_some(), ":my::State/new not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `let` wrapping struct + define.
///
/// The let-parallel of `probe_do_struct_via_macro_emission`: a macro emits
/// `(:wat::core::let [] struct-form body-form)` at top level. Both the
/// struct accessors and the define must be pre-registered.
#[test]
fn probe_let_struct_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::let []
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
