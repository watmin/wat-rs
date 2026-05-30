//! Arc 170 slice 3 Gap E — regression probes for top-level `do` splicing of `defn` forms.
//!
//! Two probes confirm that `(:wat::core::defn ...)` forms inside a top-level
//! `(:wat::core::do ...)` are pre-registered in `sym.functions` by
//! `preregister_fn_defs_in_do` before `resolve_references` runs.
//!
//! Stone 241.11 migrated the fixtures from `define` to `defn` (HARD CUT).
//! Stone 241.16 — header comments updated to reflect defn migration; define references removed.
//!
//! Gap C V2 extended the helper to handle `def`/`defn` (fn-shape) forms.
//! Gap E extended it to also handle defn forms inside do (consistent with let variant).
//!
//! Both probes PASS.
//!
//! Probe 1: two `defn` forms inside a top-level `do`.
//! Probe 2: `defmacro` that emits `do` wrapping `defn` forms.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — two `defn` forms inside a top-level `do`.
///
/// Both `:my::helper` and `:my::main` must be registered in the symbol
/// table after startup. Stone 241.11 migrated fixtures from `define` to `defn`.
/// Stone 241.16 — doc comment updated; `is_define_form`/`parse_define_form` DELETED.
#[test]
fn probe_do_define_two_vars_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping a `defn`.
///
/// A `deftest`-style macro emits
/// `(:wat::core::do prelude-form (:wat::core::defn :name [] -> :type body))`
/// at top level. Both the prelude defn and the body defn must register.
/// Stone 241.16 — doc comment updated to reflect defn migration.
#[test]
fn probe_do_define_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::defn :my::probe::helper [] -> :wat::core::i64 42)
             ~body))

        (:my::probe
          (:wat::core::defn :my::probe::main [] -> :wat::core::i64 (:my::probe::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::probe::helper").is_some(), ":my::probe::helper not registered");
    assert!(world.symbols().get(":my::probe::main").is_some(), ":my::probe::main not registered");
}
