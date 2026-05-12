//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `do` splicing of `enum` forms.
//!
//! Two probes confirm that `(:wat::core::enum ...)` forms inside a top-level
//! `(:wat::core::do ...)` pre-register their tagged-variant constructor stubs
//! in `sym.functions` (via `preregister_fn_defs_in_do`) before
//! `resolve_references` runs.
//!
//! Gap C V2 extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extended it to handle the legacy `define` form.
//! Gap F-1 extends it to handle `enum` forms — pre-generating tagged-variant
//! constructors as stubs so the outer resolver can validate calls to those
//! names inside helper `define` bodies in the same `do` block.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: enum declaration + define calling tagged-variant constructor, both in top-level `do`.
//! Probe 2: `defmacro` that emits `do` wrapping enum + define — the Phase E use case directly.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — enum + define calling tagged-variant constructor, both in a top-level `do`.
///
/// `:my::Request::Push` must be registered in `sym.functions` after startup.
/// Before Gap F-1, `resolve_references` fails because
/// `preregister_fn_defs_in_do` does not process `enum` forms, so
/// `:my::Request::Push` never enters `sym.functions`.
#[test]
fn probe_do_enum_constructor_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::enum :my::Request
            (Push (value :wat::core::i64))
            :NoOp)
          (:wat::core::define (:my::make-push -> :my::Request)
            (:my::Request::Push 99)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::Request::Push").is_some(), ":my::Request::Push not registered");
    assert!(world.symbols().get(":my::make-push").is_some(), ":my::make-push not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping enum + define.
///
/// The Phase E use case: a `deftest`-style macro emits
/// `(:wat::core::do enum-form (:wat::core::define (name -> type) body))`
/// at top level. Tagged-variant constructors must be pre-registered for
/// the define body to resolve.
#[test]
fn probe_do_enum_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::enum :my::probe::Event
               (Created (id :wat::core::i64))
               (Deleted (id :wat::core::i64))
               :NoOp)
             ~body))

        (:my::probe
          (:wat::core::define (:my::probe::make-created -> :my::probe::Event)
            (:my::probe::Event::Created 1)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::probe::Event::Created").is_some(), ":my::probe::Event::Created not registered");
    assert!(world.symbols().get(":my::probe::Event::Deleted").is_some(), ":my::probe::Event::Deleted not registered");
    assert!(world.symbols().get(":my::probe::make-created").is_some(), ":my::probe::make-created not registered");
}
