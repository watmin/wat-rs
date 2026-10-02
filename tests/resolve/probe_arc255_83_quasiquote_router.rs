//! 255.83 — the quasiquote escape and the defmacro router.
//!
//! A symbol spelling of `unquote` is the same escape as the keyword.
//! A whole-body quasiquote, in either spelling, checks those escapes at
//! definition. An unquoted `println` is refused. A pure unquote still expands.
//! The programs live in co-located fixtures.

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::MacroErrorKind;
use wat::resolve::ResolveError;
use wat::runtime::{apply_function, Value};

fn unresolved_foozle(path: &str) -> (String, &'static str) {
    match startup_from_file(path) {
        Err(StartupError::Resolve(ResolveError::UnresolvedReferences(refs))) => {
            assert_eq!(refs.len(), 1, "{refs:?}");
            (refs[0].path.clone(), refs[0].context)
        }
        other => panic!("{path}: expected one unresolved reference, got {other:?}"),
    }
}

fn defmacro_reason(path: &str) -> String {
    match startup_from_file(path) {
        Err(StartupError::Macro(err)) => match err.kind {
            MacroErrorKind::MalformedDefmacro { reason } => reason,
            other => panic!("{path}: expected MalformedDefmacro, got {other:?}"),
        },
        other => panic!("{path}: expected a macro error, got {other:?}"),
    }
}

const PRINTLN_REFUSED: &str = "program-body macro purity check failed at definition: \
keyword head `:wat::kernel::println` refused at macro expand time — not on the \
pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only \
pure-total heads are permitted";

const TEMPLATE_PRINTLN_REFUSED: &str = "quasiquote template purity check failed at definition of :user::m: \
keyword head `:wat::kernel::println` refused at macro expand time — not on the \
pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only \
pure-total heads are permitted";

#[test]
fn unquote_of_foozle_is_unresolved_in_either_spelling() {
    let kw = unresolved_foozle("tests/resolve/probe_arc255_83_qq_unquote_kw.wat.bad");
    let sym = unresolved_foozle("tests/resolve/probe_arc255_83_qq_unquote_sym.wat.bad");
    assert_eq!(kw.0, "foozle");
    assert_eq!(sym.0, "foozle");
    assert_eq!(kw.1, "call head — not a builtin, not a registered function");
    assert_eq!(kw, sym);
}

#[test]
fn nested_impure_unquote_is_refused_in_either_spelling() {
    assert_eq!(
        defmacro_reason("tests/resolve/probe_arc255_83_qq_nested_kw.wat.bad"),
        PRINTLN_REFUSED
    );
    assert_eq!(
        defmacro_reason("tests/resolve/probe_arc255_83_qq_nested_sym.wat.bad"),
        PRINTLN_REFUSED
    );
}

#[test]
fn whole_body_impure_quasiquote_is_refused_in_either_spelling() {
    assert_eq!(
        defmacro_reason("tests/resolve/probe_arc255_83_qq_whole_kw.wat.bad"),
        TEMPLATE_PRINTLN_REFUSED
    );
    assert_eq!(
        defmacro_reason("tests/resolve/probe_arc255_83_qq_whole_sym.wat.bad"),
        TEMPLATE_PRINTLN_REFUSED
    );
}

#[test]
fn whole_body_pure_quasiquote_expands_in_either_spelling() {
    for path in [
        "tests/resolve/probe_arc255_83_qq_pure_kw.wat",
        "tests/resolve/probe_arc255_83_qq_pure_sym.wat",
    ] {
        let world = startup_from_file(path).expect("a pure whole-body template defines");
        let func = world.symbols().get(":user::p").expect("user/p").clone();
        let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
            .expect("user/p runs");
        assert!(matches!(v, Value::i64(2)), "{path}: expected i64 2, got {v:?}");
    }
}

/// The five wat predicates name a head through `:wat::core::canonical-identity`,
/// so the keyword spelling and the symbol spelling of the same head agree.
#[test]
fn wat_head_predicates_agree_on_both_spellings() {
    let world = startup_from_file("tests/resolve/probe_arc255_83_qq_preds.wat")
        .expect("predicates define");
    let func = world.symbols().get(":user::probe").expect("probe").clone();
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("probe runs");
    match v {
        Value::String(s) => assert_eq!(s.as_str(), "I1i1D1d1C1c1F1f1M1m1"),
        other => panic!("expected the spelling string, got {other:?}"),
    }
}
