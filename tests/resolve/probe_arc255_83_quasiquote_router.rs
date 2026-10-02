//! 255.83 — the quasiquote escape and the defmacro router.
//!
//! A symbol spelling of `unquote` is the same escape as the keyword.
//! A symbol spelling of `quasiquote` as a whole `defmacro` body is not:
//! `is_quasiquote_form` is the router that skips `validate_macro_definition`,
//! and it stays keyword-only. Teaching it the symbol spelling would skip
//! that check.

use std::sync::Arc;

use wat::freeze::{startup_from_source, StartupError};
use wat::load::loader::InMemoryLoader;
use wat::macros::MacroErrorKind;
use wat::resolve::ResolveError;
use wat::runtime::{apply_function, Value};

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
}

fn unresolved_foozle(src: &str) -> (String, &'static str) {
    match startup(src) {
        Err(StartupError::Resolve(ResolveError::UnresolvedReferences(refs))) => {
            assert_eq!(refs.len(), 1, "{refs:?}");
            (refs[0].path.clone(), refs[0].context)
        }
        other => panic!("expected one unresolved reference, got {other:?}"),
    }
}

fn defmacro_reason(src: &str) -> String {
    match startup(src) {
        Err(StartupError::Macro(err)) => match err.kind {
            MacroErrorKind::MalformedDefmacro { reason } => reason,
            other => panic!("expected MalformedDefmacro, got {other:?}"),
        },
        other => panic!("expected a macro error, got {other:?}"),
    }
}

const PRINTLN_REFUSED: &str = "program-body macro purity check failed at definition: \
keyword head `:wat::kernel::println` refused at macro expand time — not on the \
pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only \
pure-total heads are permitted";

#[test]
fn unquote_of_foozle_is_unresolved_in_either_spelling() {
    let kw = r#"(wat.core/defn user/p [] -> wat.type/nil
        (wat.core/do (:wat::core::quasiquote (:wat::core::unquote (foozle 1))) nil))"#;
    let sym = r#"(wat.core/defn user/p [] -> wat.type/nil
        (wat.core/do (wat.core/quasiquote (wat.core/unquote (foozle 1))) nil))"#;
    let kw_ref = unresolved_foozle(kw);
    let sym_ref = unresolved_foozle(sym);
    assert_eq!(kw_ref.0, "foozle");
    assert_eq!(sym_ref.0, "foozle");
    assert_eq!(
        kw_ref.1,
        "call head — not a builtin, not a registered function"
    );
    assert_eq!(kw_ref, sym_ref);
}

#[test]
fn nested_impure_unquote_is_refused_in_either_spelling() {
    let kw = r#"(:wat::core::defmacro user/m [] -> wat.type/AST
        (wat.core/do
          (:wat::core::quasiquote (:wat::core::unquote (:wat::kernel::println "x")))
          (:wat::core::quote 1)))
       (wat.core/defn user/p [] -> wat.type/i64 1)"#;
    let sym = r#"(:wat::core::defmacro user/m [] -> wat.type/AST
        (wat.core/do
          (wat.core/quasiquote (wat.core/unquote (:wat::kernel::println "x")))
          (:wat::core::quote 1)))
       (wat.core/defn user/p [] -> wat.type/i64 1)"#;
    assert_eq!(defmacro_reason(kw), PRINTLN_REFUSED);
    assert_eq!(defmacro_reason(sym), PRINTLN_REFUSED);
}

#[test]
fn whole_body_quasiquote_keeps_the_keyword_router() {
    let kw = r#"(:wat::core::defmacro user/m [] -> wat.type/AST
        (:wat::core::quasiquote (:wat::core::unquote (:wat::kernel::println "x"))))
       (wat.core/defn user/p [] -> wat.type/i64 1)"#;
    let sym = r#"(:wat::core::defmacro user/m [] -> wat.type/AST
        (wat.core/quasiquote (wat.core/unquote (:wat::kernel::println "x"))))
       (wat.core/defn user/p [] -> wat.type/i64 1)"#;

    let world = startup(kw).expect("keyword quasiquote body skips validate_macro_definition");
    let func = world.symbols().get(":user::p").expect("user/p").clone();
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("user/p runs");
    assert!(matches!(v, Value::i64(1)), "expected i64 1, got {v:?}");

    assert_eq!(defmacro_reason(sym), PRINTLN_REFUSED);
}
