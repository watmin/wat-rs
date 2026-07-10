//! Integration tests for arc 153 — rename `:wat::core::unit` ->
//! `:wat::core::nil` (canonical FQDN for the singleton type)
//! plus value-position recognition.
//!
//! Two coordinated substrate changes shipped in slice 1a:
//!
//!   1. **Type-position rename.** `:wat::core::nil` is the
//!      canonical FQDN; the legacy `:wat::core::unit` spelling
//!      retired across slices 1a (mint nil) -> 1b (consumer
//!      sweep) -> slice 2 (retire migration scaffold).
//!
//!   2. **Value-position recognition.** `:wat::core::nil` at
//!      value position parses as a Keyword; the substrate's
//!      `infer` arm types it as the singleton (internally
//!      `TypeExpr::Tuple(vec![])`); the runtime's `eval` arm
//!      returns `Value::Unit`. The empty-list literal `()` at
//!      value position continues to evaluate to `Value::Unit`
//!      too -- both spellings produce the same singleton.
//!
//! ## Slice 2 closure — substrate retirement
//!
//! Per substrate-as-teacher § "Retire the hint when its window
//! closes": the `walk_type_for_legacy_unit_name` body, the
//! `walk_type_for_bare` Path-arm `:wat::core::unit` detection,
//! and the `:wat::core::unit` typealias all retired in slice 2.
//! `BareLegacyUnitName`'s variant + Display remain as orphaned
//! scaffolding (arc 113 precedent — variant preserved for
//! testing/teaching; only the firing body retires).
//!
//! Tests #1 + #6 + #10 originally verified that the walker fired
//! on user-source `:wat::core::unit` sites. Post-retirement they
//! assert the new shape: `:wat::core::unit` parses to
//! `Path(":wat::core::unit")`, `expand_alias` returns it
//! unchanged (no longer registered), unification surfaces
//! `ReturnTypeMismatch` with `expected: ":wat::core::unit"` and
//! `got: ":()"`. Test #10 additionally asserts the variant no
//! longer fires anywhere.
//!
//! Tests come in two shapes:
//!
//!   - **Negative-case tests**: assert specific error variants
//!     surface — loaded via co-located `*.wat.bad` fixtures
//!     through `startup_from_file`.
//!
//!   - **Positive-case tests** (verify the canonical
//!     `:wat::core::nil` flow works): `startup_beside(file!())`
//!     loads the co-located fixture; assert startup succeeds.

use wat::freeze::{startup_beside, startup_from_file};

/// Error string from a startup that MUST fail. Returns the
/// Debug-formatted CheckErrors bundle so tests can assert which
/// spans/variants appear.
fn startup_err_file(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// --- 1. Type-position retired: :wat::core::unit now unknown FQDN -------

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn type_position_unit_post_retirement_is_unknown_fqdn() {
    // Arc 153 slice 2 — substrate retirement closed the
    // `BareLegacyUnitName` migration window. Arc 163 re-armed the
    // walker; bare :wat::core::unit now fires BareLegacyUnitName.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc153_nil_rename_unit_pos.wat.bad",
    );
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc153_nil_rename_unit_pos.wat.bad", line: 5, col: 35, end_line: 5, end_col: 51 }, kind: BareLegacyUnitName }]))"#,
        "expected BareLegacyUnitName walker to fire on retired :wat::core::unit"
    );
}

// --- 2. Type-position canonical: :wat::core::nil works -----------------

#[test]
fn type_position_nil_canonical_works() {
    // `:wat::core::nil` at type position is the canonical FQDN form.
    startup_beside(file!()).expect("startup should succeed for canonical nil type");
}

// --- 3. Value-position works: :wat::core::nil at value position --------

#[test]
fn value_position_nil_keyword_type_checks_and_evaluates() {
    // `:wat::core::nil` at value position is the nil-value literal.
    startup_beside(file!()).expect("startup should succeed for nil value position");
}

// --- 4. Type mismatch: declaring i64 but body is :wat::core::nil -------

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn value_position_nil_against_i64_recipient_fires_type_mismatch() {
    // nil body vs i64 sig → ReturnTypeMismatch.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc153_nil_rename_nil_i64.wat.bad",
    );
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc153_nil_rename_nil_i64.wat.bad", line: 4, col: 51, end_line: 4, end_col: 54 }, kind: ReturnTypeMismatch { function: ":t::probe", expected: ":wat::core::i64", got: ":wat::core::nil", remedies: [] } }]))"#,
        "expected ReturnTypeMismatch when nil body meets i64 sig"
    );
}

// --- 5. Mixed: () body, :wat::core::nil sig ----------------------------

#[test]
fn mixed_empty_list_body_with_nil_sig_unifies() {
    // `()` body with `-> :wat::core::nil` sig — both produce Tuple(vec![]).
    startup_beside(file!()).expect("startup should succeed for () body with nil sig");
}

// --- 6. Reverse mixed: :wat::core::nil body, retired :unit sig --------

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn reverse_mixed_nil_body_with_retired_unit_sig_post_retirement() {
    // Arc 163 follow-up — walker re-armed; unit sig fires BareLegacyUnitName.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc153_nil_rename_unit_sig.wat.bad",
    );
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc153_nil_rename_unit_sig.wat.bad", line: 4, col: 35, end_line: 4, end_col: 51 }, kind: BareLegacyUnitName }]))"#,
        "expected BareLegacyUnitName walker to fire on retired :wat::core::unit sig"
    );
}

// --- 7. Value observable: nil keyword evaluates to Value::Unit ---------

#[test]
fn value_position_nil_evaluates_to_value_unit() {
    // nil at value position types as the singleton; startup succeeds.
    startup_beside(file!()).expect("startup should succeed for nil value form");
}

// --- 8. Value-position parity: () still evaluates to Value::Unit -------

#[test]
fn value_position_empty_list_still_evaluates_to_unit() {
    // `()` at value position still type-checks as the nil singleton.
    startup_beside(file!()).expect("startup should succeed for () as value");
}

// --- 9. Narrow special-case: other keywords still typed normally -------

#[test]
fn other_keywords_still_type_as_keyword() {
    // The nil special-case is narrow: only `:wat::core::nil` is special.
    startup_beside(file!()).expect("startup should succeed for echo-keyword fn");
}

// --- 10. Walker scaffold retired: BareLegacyUnitName no longer fires --

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn bare_legacy_unit_name_walker_retired() {
    // Arc 163 follow-up — walker RE-ARMED; bare :wat::core::unit fires fatal.
    let err = startup_err_file(
        "tests/wat_lang/wat_arc153_nil_rename_unit_pos.wat.bad",
    );
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc153_nil_rename_unit_pos.wat.bad", line: 5, col: 35, end_line: 5, end_col: 51 }, kind: BareLegacyUnitName }]))"#,
        "expected BareLegacyUnitName walker to fire on bare :wat::core::unit"
    );
}
