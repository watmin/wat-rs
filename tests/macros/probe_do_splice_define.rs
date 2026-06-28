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

use wat::freeze::startup_from_file;

/// Probe 1 — two `defn` forms inside a top-level `do`.
#[test]
fn probe_do_define_two_vars_visible() {
    let world = startup_from_file("tests/macros/probe_do_splice_define_two_vars.wat").expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping a `defn`.
#[test]
fn probe_do_define_via_macro_emission() {
    let world = startup_from_file("tests/macros/probe_do_splice_define_via_macro.wat").expect("freeze");
    assert!(world.symbols().get(":my::probe::helper").is_some(), ":my::probe::helper not registered");
    assert!(world.symbols().get(":my::probe::main").is_some(), ":my::probe::main not registered");
}
