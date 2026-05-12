//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `let` splicing of `enum` forms.
//!
//! Two probes confirm that `(:wat::core::enum ...)` forms in the body of a
//! top-level `(:wat::core::let ...)` pre-register their tagged-variant
//! constructor stubs in `sym.functions` (via `preregister_fn_defs_in_let`)
//! before `resolve_references` runs.
//!
//! Gap D extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extended it to handle the legacy `define` form.
//! Gap F-1 extends it to handle `enum` forms — pre-generating tagged-variant
//! constructors as stubs, consistent with the parallel `preregister_fn_defs_in_do`
//! extension.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: enum declaration + define calling tagged-variant constructor, both in top-level `let` body.
//! Probe 2: `defmacro` that emits `let` wrapping enum + define.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — enum + define calling tagged-variant constructor, both in the body of a top-level `let`.
///
/// `:my::Request::Push` must be registered in `sym.functions` after startup.
/// Before Gap F-1, `resolve_references` fails because
/// `preregister_fn_defs_in_let` does not process `enum` forms, so
/// `:my::Request::Push` never enters `sym.functions`.
#[test]
fn probe_let_enum_constructor_visible() {
    let src = r#"
        (:wat::core::let []
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

/// Probe 2 — `defmacro` that emits a top-level `let` wrapping enum + define.
///
/// The let-parallel of `probe_do_enum_via_macro_emission`: a macro emits
/// `(:wat::core::let [] enum-form body-form)` at top level. Tagged-variant
/// constructors must be pre-registered for the define body to resolve.
#[test]
fn probe_let_enum_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::let []
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
