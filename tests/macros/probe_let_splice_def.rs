//! Arc 170 slice 3 Gap D — regression probes for top-level `let` splicing of `def`/`defn`.
//!
//! Three probes confirm that `(:wat::core::let ...)` at top level splices
//! its body children uniformly across ALL substrate passes.
//!
//! All three probes FAIL before Gap D ships; all three PASS after.
//!
//! Probe 1: `let []` of two `def`-of-fn forms.
//! Probe 2: `let []` of two `defn` forms (via `defn` macro expansion to `def`).
//! Probe 3: `let [x ...]` with real bindings followed by `defn` forms.

use wat::freeze::startup_from_file;

/// Probe 1 — two `def`s wrapping `fn` inside a top-level `let` with empty bindings.
#[test]
fn probe_let_def_two_vars_visible() {
    let world = startup_from_file("tests/macros/probe_let_splice_def_two_defs.wat").expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — two `defn` forms inside a top-level `let` with empty bindings.
#[test]
fn probe_let_defn_via_expansion() {
    let world = startup_from_file("tests/macros/probe_let_splice_def_defn_expand.wat").expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 3 — non-empty bindings followed by `defn` forms in the body.
#[test]
fn probe_let_with_real_bindings_then_defn() {
    let world = startup_from_file("tests/macros/probe_let_splice_def_real_bindings.wat").expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}
