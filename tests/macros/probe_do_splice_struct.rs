//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `do` splicing of `struct` forms.
//!
//! Two probes confirm that `(:wat::core::struct ...)` forms inside a top-level
//! `(:wat::core::do ...)` pre-register their accessor stubs in `sym.functions`.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: struct declaration + define using accessor, both in top-level `do`.
//! Probe 2: `defmacro` that emits `do` wrapping struct + define — the Phase E use case directly.

use wat::freeze::startup_from_file;

/// Probe 1 — struct + define using its constructor, both inside a top-level `do`.
#[test]
fn probe_do_struct_accessor_visible() {
    let world = startup_from_file("tests/macros/probe_do_splice_struct_accessor.wat").expect("freeze");
    assert!(world.symbols().get(":my::State/new").is_some(), ":my::State/new not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `do` wrapping struct + define.
#[test]
fn probe_do_struct_via_macro_emission() {
    let world = startup_from_file("tests/macros/probe_do_splice_struct_via_macro.wat").expect("freeze");
    assert!(world.symbols().get(":my::probe::Point/new").is_some(), ":my::probe::Point/new not registered");
    assert!(world.symbols().get(":my::probe::Point/x").is_some(), ":my::probe::Point/x not registered");
    assert!(world.symbols().get(":my::probe::Point/y").is_some(), ":my::probe::Point/y not registered");
    assert!(world.symbols().get(":my::probe::make-origin").is_some(), ":my::probe::make-origin not registered");
}
