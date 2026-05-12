//! Arc 170 slice 3 Gap D — regression probes for top-level `let` splicing of `def`/`defn`.
//!
//! Three probes confirm that `(:wat::core::let ...)` at top level splices
//! its body children uniformly across ALL substrate passes. The gap:
//! `register_defines` did not recurse into the body of a top-level `let`
//! (items[2..] per arc 168 multi-form body), so functions defined via
//! `def`/`defn` inside a top-level `let` body were invisible to
//! `resolve_references` at startup time — the same gap as Gap C for `do`.
//!
//! Arc 157 doctrine (`src/check.rs:715`): def is legal at (1) direct file
//! top-level, (2) inside top-level `do`, (3) inside top-level `let` body.
//! The `collect_splice_defs_ctx` pass already handles position (3); Gap D
//! extends `register_defines` to match.
//!
//! All three probes FAIL before Gap D ships; all three PASS after.
//!
//! Probe 1: `let []` of two `def`-of-fn forms.
//! Probe 2: `let []` of two `defn` forms (via `defn` macro expansion to `def`).
//! Probe 3: `let [x ...]` with real bindings followed by `defn` forms.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Probe 1 — two `def`s wrapping `fn` inside a top-level `let` with empty bindings.
///
/// Both `:my::helper` and `:my::main` must be registered in the symbol
/// table after startup. Before Gap D, `resolve_references` fails because
/// `:my::helper` is called inside the `let` body before it enters `sym.functions`.
#[test]
fn probe_let_def_two_vars_visible() {
    let src = r#"
        (:wat::core::let []
          (:wat::core::def :my::helper (:wat::core::fn [] -> :wat::core::i64 42))
          (:wat::core::def :my::main (:wat::core::fn [] -> :wat::core::i64 (:my::helper))))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 2 — two `defn` forms inside a top-level `let` with empty bindings.
///
/// `defn` is a macro that expands to `(:wat::core::def name (:wat::core::fn ...))`.
/// After macro expansion the forms inside the `let` body are `def`-of-fn, same
/// as Probe 1. Both must register.
#[test]
fn probe_let_defn_via_expansion() {
    let src = r#"
        (:wat::core::let []
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}

/// Probe 3 — non-empty bindings followed by `defn` forms in the body.
///
/// Verifies that real bindings in `items[1]` do not interfere with the body
/// scan at `items[2..]`. Both `:my::helper` and `:my::main` must register.
#[test]
fn probe_let_with_real_bindings_then_defn() {
    let src = r#"
        (:wat::core::let [x (:wat::core::i64::+'2 1 1)]
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some(), ":my::helper not registered");
    assert!(world.symbols().get(":my::main").is_some(), ":my::main not registered");
}
