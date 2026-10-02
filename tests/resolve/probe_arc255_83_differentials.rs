//! 255.83 — differential pairs for the rows whose door is not the quasiquote
//! router. Keyword text stays the keyword text. A reference symbol of the same
//! name takes that door. The programs live in co-located fixtures.

use wat::check::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, Value};

fn startup(path: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_file(path)
}

fn check_errs(path: &str) -> Vec<wat::check::CheckError> {
    match startup(path) {
        Err(StartupError::Check(errs)) => errs.0,
        other => panic!("{path}: expected a check error\n{other:?}"),
    }
}

fn has_kind(path: &str, pred: fn(&CheckErrorKind) -> bool) -> bool {
    match startup(path) {
        Err(StartupError::Check(errs)) => errs.0.iter().any(|e| pred(&e.kind)),
        _ => false,
    }
}

#[test]
fn spelling_25583_legacy_walker_agrees_on_both_spellings() {
    let let_kw = "tests/resolve/probe_arc255_83_diff_let_kw.wat.bad";
    let let_sym = "tests/resolve/probe_arc255_83_diff_let_sym.wat.bad";
    assert!(has_kind(let_kw, |k| matches!(k, CheckErrorKind::BareLegacyLetStar)));
    assert!(has_kind(let_sym, |k| matches!(k, CheckErrorKind::BareLegacyLetStar)));
    assert_eq!(check_errs(let_kw).len(), check_errs(let_sym).len());

    assert!(has_kind("tests/resolve/probe_arc255_83_diff_lam_kw.wat.bad", |k| {
        matches!(k, CheckErrorKind::BareLegacyLambda)
    }));
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_lam_sym.wat.bad", |k| {
        matches!(k, CheckErrorKind::BareLegacyLambda)
    }));

    assert!(has_kind("tests/resolve/probe_arc255_83_diff_unit_kw.wat.bad", |k| {
        matches!(k, CheckErrorKind::BareLegacyUnitName)
    }));
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_unit_sym.wat.bad", |k| {
        matches!(k, CheckErrorKind::BareLegacyUnitName)
    }));

    let is_char = |k: &CheckErrorKind| {
        matches!(k, CheckErrorKind::MalformedForm { reason, .. } if reason.contains("Stone 242.1"))
    };
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_char_kw.wat.bad", is_char));
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_char_sym.wat.bad", is_char));
    assert!(
        !has_kind("tests/resolve/probe_arc255_83_diff_char_dotted.wat.bad", is_char),
        "a dotted keyword is not rewritten"
    );

    let is_uuid = |k: &CheckErrorKind| {
        matches!(k, CheckErrorKind::MalformedForm { reason, .. } if reason.contains("arc 255.77"))
    };
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_uuid_kw.wat.bad", is_uuid));
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_uuid_sym.wat.bad", is_uuid));

    // `:fn(` is keyword text. The door does not manufacture it from a symbol.
    assert!(has_kind("tests/resolve/probe_arc255_83_diff_fn_kw.wat.bad", |k| {
        matches!(k, CheckErrorKind::BareLegacyLowercaseFn)
    }));
    match startup("tests/resolve/probe_arc255_83_diff_fn_sym.wat.bad") {
        Err(StartupError::Check(errs)) => {
            assert!(errs
                .0
                .iter()
                .all(|e| !matches!(e.kind, CheckErrorKind::BareLegacyLowercaseFn)));
        }
        Err(_) => {}
        Ok(_) => panic!("wat.core/fn is not a return type"),
    }
}

#[test]
fn spelling_25583_expect_call_returns_i64_7_in_either_spelling() {
    for path in [
        "tests/resolve/probe_arc255_83_diff_expect_kw.wat",
        "tests/resolve/probe_arc255_83_diff_expect_sym.wat",
    ] {
        let world = startup(path).unwrap_or_else(|e| panic!("{path}: expect defines: {e:?}"));
        let func = world.symbols().get(":user::p").expect("user/p").clone();
        let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
            .unwrap_or_else(|e| panic!("{path}: expect runs: {e:?}"));
        assert!(matches!(v, Value::i64(7)), "{path}: expected i64 7, got {v:?}");
    }
}
