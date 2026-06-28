//! Arc 170 slice 3 Gap E — regression probes for top-level `let` splicing of `defn` forms.
//!
//! Two probes confirm that `(:wat::core::defn ...)` forms in the body of a
//! top-level `(:wat::core::let ...)` are pre-registered in `sym.functions`.
//!
//! Stone 241.11 migrated the fixtures from `define` to `defn` (HARD CUT).
//! Stone 241.16 — header comments updated to reflect defn migration.
//!
//! Both probes PASS.
//!
//! Probe 1: two `defn` forms in the body of a top-level `let`.
//! Probe 2: `defmacro` that emits `let` wrapping `defn` forms.

use wat::freeze::startup_from_file;

/// Probe 1 — two `defn` forms in the body of a top-level `let` with empty bindings.
#[test]
fn probe_let_define_two_vars_visible() {
    let world = startup_from_file("tests/macros/probe_let_splice_define_two_vars.wat").expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `let` wrapping `define` forms.
#[test]
fn probe_let_define_via_macro_emission() {
    let world = startup_from_file("tests/macros/probe_let_splice_define_via_macro.wat").expect("freeze");
    assert!(world.symbols().get(":my::probe::helper").is_some(), ":my::probe::helper not registered");
    assert!(world.symbols().get(":my::probe::main").is_some(), ":my::probe::main not registered");
}
