//! Arc 170 slice 3 Gap E — regression probes for top-level `let` splicing of `defn` forms.
//!
//! Two probes confirm that `(:wat::core::defn ...)` forms in the body of a
//! top-level `(:wat::core::let ...)` are pre-registered in `sym.functions` by
//! `preregister_fn_defs_in_let` before `resolve_references` runs.
//!
//! Stone 241.11 migrated the fixtures from `define` to `defn` (HARD CUT).
//! Stone 241.16 — header comments updated to reflect defn migration; define references removed.
//!
//! Gap D extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extended it to handle let-nested defn forms (parallel to do).
//!
//! Both probes PASS.
//!
//! Probe 1: two `defn` forms in the body of a top-level `let`.
//! Probe 2: `defmacro` that emits `let` wrapping `defn` forms.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — two `defn` forms in the body of a top-level `let` with empty bindings.
///
/// Both `:my::helper` and `:my::main` must be registered in the symbol
/// table after startup. Stone 241.11 migrated fixtures from `define` to `defn`.
/// Stone 241.16 — doc comment updated; `is_define_form`/`parse_define_form` DELETED.
#[test]
fn probe_let_define_two_vars_visible() {
    let src = r#"
        (:wat::core::let []
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `let` wrapping `define` forms.
///
/// The let-parallel of `probe_do_define_via_macro_emission`: a macro emits
/// `(:wat::core::let [] prelude-form body-form)` at top level. Both the
/// prelude define and the body define must register.
#[test]
fn probe_let_define_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro :my::probe
          [body <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::let []
             (:wat::core::defn :my::probe::helper [] -> :wat::core::i64 42)
             ~body))

        (:my::probe
          (:wat::core::defn :my::probe::main [] -> :wat::core::i64 (:my::probe::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::probe::helper").is_some(), ":my::probe::helper not registered");
    assert!(world.symbols().get(":my::probe::main").is_some(), ":my::probe::main not registered");
}
