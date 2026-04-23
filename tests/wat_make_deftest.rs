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
//! `(:wat::test::deftest <name> <dims> <mode> <prelude> <body>)`
//! shape with the prelude intact.

use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::Value;
use wat::span::Span;
use std::sync::Arc;

#[test]
fn diag_make_deftest_with_prelude_expansion() {
    let src = r##"
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::make-deftest :my-deftest 1024 :error
  ((:wat::load-file! "foo.wat")))

;; Expose the expansion result as the main function's return value.
;; :user::main returns :() per contract, so we stash in a side
;; effect via stderr? No — we need to inspect structurally from Rust.
;; Approach: register a :define that returns the expansion, then
;; invoke it manually from Rust-level symbol lookup.
(:wat::core::define (:probe::get-expansion -> :wat::WatAST)
  (:wat::core::macroexpand-1
    (:wat::core::quote (:my-deftest :my-test (:wat::test::assert-eq 1 1)))))

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  ())
"##;

    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");

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
        Span::unknown(),
    )
    .expect("probe call ok");

    let ast = match expansion {
        Value::wat__WatAST(a) => a,
        other => panic!("expected wat::WatAST, got {:?}", other),
    };

    // Expect (:wat::test::deftest :my-test 1024 :error <prelude> <body>)
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
    assert_eq!(items.len(), 6, "expected 6 items (deftest + 5 args)");
}
