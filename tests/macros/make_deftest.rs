//! Negative-space test for `:wat::core::macroexpand` fixpoint-iteration
//! failure: a self-recursive macro that never reaches a fixpoint must
//! produce `RuntimeErrorKind::MacroExpansionFailed` (not an infinite loop).
//! The fixpoint loop at src/runtime.rs `eval_macroexpand` runs at most
//! `EXPANSION_DEPTH_LIMIT` iterations, then fails with a human-readable
//! "did not reach fixpoint" diagnostic. Item 3b of the src/macros/
//! perimeter audit (2026-06-05).
//!
//! Arc 278: the make-deftest factory macro was annihilated (a pure alias
//! shell after the prelude slot was removed); its former
//! `diag_make_deftest_with_prelude_expansion` regression test was retired
//! with it. This file keeps only the independent macroexpand-fixpoint probe.
//!
//! Wat source lives in the co-located fixture: make_deftest.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::startup_beside;

/// NEGATIVE-SPACE — `:wat::core::macroexpand` fixpoint-iteration cap.
///
/// A self-recursive macro (`:my::loop` expands to a call to itself) never
/// reaches a fixpoint. `macroexpand` runs the fixpoint loop in
/// `eval_macroexpand` (src/runtime.rs) for at most `EXPANSION_DEPTH_LIMIT`
/// iterations, then returns `MacroExpansionFailed` with a
/// "did not reach fixpoint" diagnostic. This test is the living witness that
/// the failure diagnostic is driven by the shared `EXPANSION_DEPTH_LIMIT`
/// constant and produces the correct `RuntimeErrorKind`.
///
/// Mechanistic difference from `ExpansionDepthExceeded`: the depth-limit check
/// in `expand_form` fires when the RECURSIVE AST walker goes too deep (a macro
/// whose template re-invokes itself); the fixpoint-iteration cap here fires when
/// `macroexpand`'s step loop (`expand_once` repeated) fails to converge. They
/// are distinct mechanisms and must NOT be merged — the orchestrator's ruling
/// (perimeter audit 2026-06-05, item 3b).
#[test]
fn macroexpand_self_recursive_macro_fails_with_macro_expansion_failed() {
    let world = startup_beside(file!())
        .expect("startup succeeded (macro is registered, not called at freeze time)");

    let func = world
        .symbols()
        .get(":probe::run-macroexpand")
        .expect(":probe::run-macroexpand defined")
        .clone();

    let result = wat::runtime::apply_function(
        func,
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    );

    // The call must FAIL with MacroExpansionFailed — the fixpoint loop
    // exhausted EXPANSION_DEPTH_LIMIT iterations without convergence.
    match result {
        Err(e) => {
            assert!(
                matches!(
                    e.kind(),
                    wat::RuntimeErrorKind::MacroExpansionFailed { .. }
                ),
                "expected MacroExpansionFailed; got: {:?}",
                e.kind()
            );
            // The message must include the dynamic limit so it scales with
            // EXPANSION_DEPTH_LIMIT (not a hardcoded "512").
            let msg = format!("{}", e);
            let limit_str = format!("{}", wat::macros::EXPANSION_DEPTH_LIMIT);
            assert!(
                msg.contains(&limit_str),
                "error message should contain the limit ({}) from EXPANSION_DEPTH_LIMIT; \
                 got: {}",
                limit_str,
                msg
            );
        }
        Ok(v) => panic!(
            "expected MacroExpansionFailed; macroexpand returned Ok({:?})",
            v
        ),
    }
}
