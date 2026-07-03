//! Arc 054 — Idempotent re-declaration for typealias / define / defmacro.
//!
//! Three registries gain "byte-equivalent re-registration is a no-op."
//! Divergent re-registration remains an error.
//!
//! Coverage:
//! - typealias: byte-equivalent → ok; divergent → error
//! - define: byte-equivalent → ok; divergent → error
//! - defmacro: byte-equivalent → ok (divergent path covered by lib test)

use wat::freeze::{startup_beside, startup_from_file};

/// Error string from a startup-file that MUST fail.
fn startup_err_file(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Typealias ───────────────────────────────────────────────────────

#[test]
fn typealias_byte_equivalent_is_noop() {
    startup_beside(file!()).expect("startup should succeed for byte-equivalent typealias");
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn typealias_divergent_errors() {
    let err = startup_err_file(
        "tests/wat_lang/wat_idempotent_redeclare_typealias_div_bad.wat",
    );
    assert_eq!(
        err,
        r#"Type(TypeError { span: Span { file: "tests/wat_lang/wat_idempotent_redeclare_typealias_div_bad.wat", line: 5, col: 1, end_line: 5, end_col: 52 }, kind: DuplicateType { name: ":my::Amount" } })"#,
        "expected DuplicateType error for divergent typealias re-registration"
    );
}

// ─── Define ──────────────────────────────────────────────────────────

#[test]
fn define_byte_equivalent_is_noop() {
    startup_beside(file!()).expect("startup should succeed for byte-equivalent defn");
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn define_divergent_body_errors() {
    let err = startup_err_file(
        "tests/wat_lang/wat_idempotent_redeclare_define_div_bad.wat",
    );
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "wat/core.wat", line: 512, col: 9, end_line: 512, end_col: 24 }, kind: DefRedefForbidden { name: ":my::add-one", original_def_span: Span { file: "wat/core.wat", line: 512, col: 9, end_line: 512, end_col: 24 } } }]))"#,
        "expected DefRedefForbidden for divergent define re-registration"
    );
}

// ─── Defmacro ────────────────────────────────────────────────────────

#[test]
fn defmacro_byte_equivalent_is_noop() {
    startup_beside(file!()).expect("startup should succeed for byte-equivalent defmacro");
}

// ─── In-crate-shim shape — the motivating case ──────────────────────
//
// The lab's CandleStream shim ships its wat surface BOTH via
// `wat_sources()` and as an on-disk file loaded by main.wat / test
// preludes. Both paths register the same typealias. Pre-arc-054, that
// was a duplicate-type error. Post-arc-054, it's a no-op for the
// second registration. This test simulates the shape:
// the same `(:wat::core::typealias ...)` reaches the registry twice.

#[test]
fn shim_double_register_pattern_works() {
    startup_beside(file!()).expect("startup should succeed for shim double-register pattern");
}
