//! Arc 170 Stone D2 — substrate-gap probe.
//!
//! D2 requires variadic iteration over a user-source Vector AST
//! (`[[:I :O f] [:I :O f] ...]`) at macro expand time. This probe
//! captures the two concrete substrate gaps surfaced during D2
//! investigation 2026-05-16:
//!
//! Gap 1: `~@bound-symbol` where the symbol is bound to a `WatAST::Vector`
//!        errors at `splice_argument` (`src/macros.rs:1081`) with
//!        `MacroError::SpliceNotList`. Splice accepts only List-shaped
//!        bound values, not Vectors.
//!
//! Gap 2: `~@(...)` INSIDE a Vector template (e.g. let bindings `[...]`)
//!        does NOT fire splice — the Vector branch in `walk_template`
//!        (`src/macros.rs:926-941`) walks children but lacks the
//!        unquote-splicing dispatch that the List branch has at
//!        `src/macros.rs:860-898`.
//!
//! Either gap blocks the BRIEF-mandated D2 call form
//! `(:wat::kernel::run-threads [[:I :O f] ...] client-fn)` from
//! reaching expansion without new substrate work. Both fixes are
//! tiny (relax existing primitives to also accept Vector inputs;
//! ~5 lines each) but live OUTSIDE D2's scope.
//!
//! These tests don't assert PASS — they assert the EXPECTED FAILURE
//! mode. When substrate ships the relaxation, they migrate to PASS
//! assertions and D2 unblocks.

use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn freeze_result(src: &str) -> Result<wat::freeze::FrozenWorld, String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("{}", e))
}

/// Gap 1 — `~@vec-symbol` is rejected by `splice_argument` because
/// the bound value is `WatAST::Vector`, not `WatAST::List`.
#[test]
fn splice_of_vector_bound_symbol_errors_with_splice_not_list() {
    let src = r#"
        (:wat::core::defmacro
          (:my::splice-vec (xs :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::Vector :wat::core::i64 ~@xs))

        (:my::splice-vec [1 2 3])
    "#;
    let err = freeze_result(src).expect_err(
        "Stone D2 substrate-gap probe Gap 1: \
         expected `~@bound-vector` to error at expand-time with \
         SpliceNotList — splice_argument only accepts List bindings",
    );
    eprintln!("Gap 1 actual error: {}", err);
    let lower = err.to_lowercase();
    assert!(
        lower.contains("splice")
            || lower.contains("unquote-splicing")
            || lower.contains("vector")
            || lower.contains("list"),
        "expected SpliceNotList-ish error message; got: {err}"
    );
}

/// Gap 2 — `~@(...)` INSIDE a Vector template does not fire splice;
/// the unquote-splicing call survives literally into the substituted
/// Vector. Demonstrates by using a computed `(:wat::core::forms ...)`
/// call inside a `[...]` — even though `forms` returns a Vec at
/// expand time, the Vector branch of `walk_template` doesn't dispatch
/// to `splice_argument`, so the AST that lands at the call site is
/// invalid (contains `:wat::core::unquote-splicing` as a literal head).
#[test]
fn splice_inside_vector_template_does_not_fire() {
    let src = r#"
        (:wat::core::defmacro
          (:my::vec-splice-probe -> :AST<wat::core::nil>)
          `[~@(:wat::core::forms 10 20 30)])

        (:wat::core::define (:my::expand-probe -> :wat::core::nil)
          (:my::vec-splice-probe))
    "#;
    let err = freeze_result(src).expect_err(
        "Stone D2 substrate-gap probe Gap 2: \
         expected `~@(...)` inside a `[...]` template to fail \
         downstream — Vector branch of walk_template (src/macros.rs:926-941) \
         does not dispatch unquote-splicing the way the List branch does",
    );
    eprintln!("Gap 2 actual error: {}", err);
    // Acceptable error surfaces: parse-time (unknown form),
    // check-time (type mismatch), or runtime (eval failure on
    // `:wat::core::unquote-splicing` as a literal head). Any of
    // those proves the splice didn't fire.
    assert!(
        !err.is_empty(),
        "expected some non-empty error proving the splice was preserved literally; got empty error"
    );
}
