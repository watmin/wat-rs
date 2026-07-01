//! Arc 030 slice 2 — make-deftest-with-nonempty-default-prelude
//! regression test. Ships with the fix that closes arc 029's bug:
//! `expand_form` preserves `(:wat::core::quote X)` bodies from
//! macro expansion, same discipline as quasiquote. Without that
//! fix, a `(quote (:my-macro ...))` form would expand :my-macro
//! eagerly, turning macroexpand-1's input into the FULLY expanded
//! form — defeating the whole point of macroexpand.
//!
//! This test builds a configured-deftest variant with a non-empty
//! default-prelude, then runs macroexpand-1 on a call to it, and
//! asserts the one-step expansion produces the expected
//! `(:wat::test::deftest <name> <prelude> <body>)` shape with the
//! prelude intact. Arc 031 slice 2 dropped the mode/dims args —
//! tests inherit Config from the outer test binary's preamble.
//!
//! Also contains the negative-space test for
//! `:wat::core::macroexpand` fixpoint-iteration failure: a
//! self-recursive macro that never reaches a fixpoint must produce
//! `RuntimeErrorKind::MacroExpansionFailed` (not an infinite loop).
//! The fixpoint loop at src/runtime.rs `eval_macroexpand` runs at
//! most `EXPANSION_DEPTH_LIMIT` iterations, then fails with a
//! human-readable "did not reach fixpoint" diagnostic. Item 3b of
//! the src/macros/ perimeter audit (2026-06-05).
//!
//! Wat source lives in the co-located fixture: make_deftest.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{startup_beside};
use wat::runtime::Value;
use wat::span::Span;

#[test]
fn diag_make_deftest_with_prelude_expansion() {
    let world = startup_beside(file!()).expect("startup");

    // The registered :my-deftest macro's body should be a
    // (quasiquote (:wat::test::deftest ...)) form — deftest NOT
    // pre-expanded. Arc 029's quasi-preserve fix keeps the quasi
    // body from being eagerly walked by expand_form.
    let macros = world.macros();
    let def = macros
        .get(":my-deftest")
        .expect(":my-deftest registered");
    if let wat::ast::WatAST::List(items, _) = &def.body {
        assert!(
            matches!(items.first(), Some(wat::ast::WatAST::Keyword(k, _))
                if k == ":wat::core::quasiquote"),
            "registered body should be a quasiquote form; got {:?}",
            items.first()
        );
        // items[1] is the quasi content. Should start with
        // :wat::test::deftest — not deftest's OWN expansion.
        if let wat::ast::WatAST::List(inner, _) = &items[1] {
            assert!(
                matches!(inner.first(), Some(wat::ast::WatAST::Keyword(k, _))
                    if k == ":wat::test::deftest"),
                "inner template should call :wat::test::deftest; got {:?}",
                inner.first()
            );
        } else {
            panic!("quasi body should be a list");
        }
    } else {
        panic!("macro body should be a list");
    }

    // Expand the user's call — one step should give deftest call.
    let func = world
        .symbols()
        .get(":probe::get-expansion")
        .expect("probe function registered")
        .clone();
    let expansion = wat::runtime::apply_function(
        func,
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("probe call ok");

    let ast = match expansion {
        Value::wat__WatAST(a) => a,
        other => panic!("expected wat::WatAST, got {:?}", other),
    };

    // Expect (:wat::test::deftest :my-test <prelude> <body>). Arc 031
    // slice 2: no more mode/dims args.
    let items = match &*ast {
        wat::ast::WatAST::List(items, _) => items,
        _ => panic!("expansion should be a list"),
    };
    assert!(
        matches!(items.first(), Some(wat::ast::WatAST::Keyword(k, _))
            if k == ":wat::test::deftest"),
        "expansion should be a deftest call; got {:?}",
        items.first()
    );
    assert_eq!(items.len(), 4, "expected 4 items (deftest + name + prelude + body)");
}

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
                    &e.kind,
                    wat::RuntimeErrorKind::MacroExpansionFailed { .. }
                ),
                "expected MacroExpansionFailed; got: {:?}",
                e.kind
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
