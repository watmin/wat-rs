//! Arc 170 slice 3 Gap E — regression probes for top-level `do` splicing of `define` forms.
//!
//! Two probes confirm that `(:wat::core::define ...)` forms inside a top-level
//! `(:wat::core::do ...)` are pre-registered in `sym.functions` by
//! `preregister_fn_defs_in_do` before `resolve_references` runs.
//!
//! Gap C V2 extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extends it to also handle the legacy `define` form, which is what
//! `deftest` still emits (arc 170 slice 3 Phase E).
//!
//! Both probes FAIL before Gap E ships; both PASS after.
//!
//! Probe 1: two `define` forms inside a top-level `do`.
//! Probe 2: `defmacro` that emits `do` wrapping `define` — the Phase E use case directly.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — two `define` forms inside a top-level `do`.
///
/// Both `:my::helper` and `:my::main` must be registered in the symbol
/// table after startup. Before Gap E, `resolve_references` fails because
/// `preregister_fn_defs_in_do` does not call `is_define_form` / `parse_define_form`,
/// so `:my::helper` never enters `sym.functions`.
#[test]
fn probe_do_define_two_vars_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::define (:my::helper -> :wat::core::i64)
            42)
          (:wat::core::define (:my::main -> :wat::core::i64)
            (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping a `define`.
///
/// The Phase E use case: a `deftest`-style macro emits
/// `(:wat::core::do prelude-form (:wat::core::define (name -> type) body))`
/// at top level. Both the prelude define and the body define must register.
#[test]
fn probe_do_define_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::define (:my::probe::helper -> :wat::core::i64)
               42)
             ~body))

        (:my::probe
          (:wat::core::define (:my::probe::main -> :wat::core::i64)
            (:my::probe::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::probe::helper").is_some(), ":my::probe::helper not registered");
    assert!(world.symbols().get(":my::probe::main").is_some(), ":my::probe::main not registered");
}
