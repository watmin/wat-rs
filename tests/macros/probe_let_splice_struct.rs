//! Arc 170 slice 3 Gap F-1 — regression probes for top-level `let` splicing of `struct` forms.
//!
//! Two probes confirm that `(:wat::core::struct ...)` forms in the body of a
//! top-level `(:wat::core::let ...)` pre-register their accessor stubs.
//!
//! Both probes FAIL before Gap F-1 ships; both PASS after.
//!
//! Probe 1: struct declaration + define using accessor, both in top-level `let` body.
//! Probe 2: `defmacro` that emits `let` wrapping struct + define.

use wat::freeze::startup_from_file;

/// Probe 1 — struct + define using its constructor, both in the body of a top-level `let`.
#[test]
fn probe_let_struct_accessor_visible() {
    let world = startup_from_file("tests/macros/probe_let_splice_struct_accessor.wat").expect("freeze");
    assert!(world.symbols().get(":my::State").is_some(), ":my::State ctor not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — `defmacro` that emits a top-level `let` wrapping struct + define.
#[test]
fn probe_let_struct_via_macro_emission() {
    let world = startup_from_file("tests/macros/probe_let_splice_struct_via_macro.wat").expect("freeze");
    assert!(world.symbols().get(":my::probe::Point").is_some(), ":my::probe::Point ctor not registered");
    assert!(world.symbols().get(":my::probe::Point/x").is_some(), ":my::probe::Point/x not registered");
    assert!(world.symbols().get(":my::probe::Point/y").is_some(), ":my::probe::Point/y not registered");
    assert!(world.symbols().get(":my::probe::make-origin").is_some(), ":my::probe::make-origin not registered");
}
