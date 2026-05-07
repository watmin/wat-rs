//! Integration tests for arc 153 slice 1a — rename
//! `:wat::core::unit` -> `:wat::core::nil` (canonical FQDN for the
//! singleton type) plus value-position recognition.
//!
//! Two coordinated substrate changes ship together:
//!
//!   1. **Type-position rename.** `:wat::core::unit` is RETIRED;
//!      `:wat::core::nil` is the canonical FQDN. The walker
//!      (`walk_type_for_bare`) emits `BareLegacyUnitName` per
//!      offending site (substrate-as-teacher Pattern 3, mirroring
//!      arc 109 slice 1d's `BareLegacyUnitType`). The
//!      `:wat::core::unit` typealias stays registered through the
//!      deprecation window so unification keeps resolving the
//!      legacy spelling to the empty-tuple singleton; the walker
//!      is the only signal consumers see.
//!
//!   2. **Value-position recognition.** `:wat::core::nil` at
//!      value position parses as a Keyword; the substrate's
//!      `infer` arm types it as the singleton (internally
//!      `TypeExpr::Tuple(vec![])`); the runtime's `eval` arm
//!      returns `Value::Unit`. The empty-list literal `()` at
//!      value position continues to evaluate to `Value::Unit`
//!      too -- both spellings produce the same singleton
//!      (transitional; sweep 1b transforms `()` -> `:wat::core::nil`).
//!
//! ## Test design across slice 1a + 1b
//!
//! Slice 1a substrate shipped BEFORE sweep 1b touched stdlib +
//! consumer wat. During the slice-1a-only window, every
//! `:wat::core::unit` site in stdlib fired `BareLegacyUnitName` at
//! startup, so `startup_from_source` could not succeed even if the
//! user-source code was canonical. After sweep 1b scrubbed stdlib +
//! consumer wat, the noise disappeared; positive-shape tests now
//! assert full startup success.
//!
//! Tests come in two shapes:
//!
//!   - **Negative-case tests** (verify migration error fires on
//!     user-source `:wat::core::unit`): assert the error stream
//!     contains a `<entry>`-spanned `BareLegacyUnitName`. Use
//!     `startup_err`.
//!
//!   - **Positive-case tests** (verify the canonical
//!     `:wat::core::nil` flow works): assert that
//!     `startup_from_source` returns Ok. Use `startup_ok`.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Error string from a startup that MUST fail. Returns the
/// Debug-formatted CheckErrors bundle so tests can assert which
/// spans/variants appear.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Asserts the given source starts up cleanly (post-sweep-1b: stdlib
/// + consumer wat are migrated, so a canonical-nil user source has
/// nothing to choke on).
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

// --- 1. Type-position retired: :wat::core::unit fires migration --------

#[test]
fn type_position_unit_fires_bare_legacy_unit_name() {
    // `:wat::core::unit` at type position (function return) is
    // retired. The substrate walker emits `BareLegacyUnitName`
    // pointing the consumer at the canonical `:wat::core::nil`
    // form. Per arc 153 slice 1a, this is Pattern 3
    // substrate-as-teacher.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::unit)
          ())

        (:wat::core::define (:user::main -> :wat::core::i64)
          42)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyUnitName"),
        "expected BareLegacyUnitName naming the retired token; got: {}",
        err
    );
    // The error stream MUST include a `<entry>`-spanned
    // `BareLegacyUnitName` -- proves the walker fires on the
    // user-source `:wat::core::unit` (not just on unswept stdlib
    // usages bleeding through).
    assert!(
        err.contains(r#"BareLegacyUnitName { span: Span { file: "<entry>""#),
        "expected user-source-spanned BareLegacyUnitName; got: {}",
        err
    );
}

// --- 2. Type-position canonical: :wat::core::nil works -----------------

#[test]
fn type_position_nil_canonical_works() {
    // `:wat::core::nil` at type position is the canonical FQDN
    // form. Same internal representation as the legacy
    // `:wat::core::unit`; substrate canonicalizes to
    // `TypeExpr::Tuple(vec![])` so unification with existing
    // empty-tuple types succeeds. Post-sweep-1b: full startup
    // success.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::nil)
          ())

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do
            (:my::probe)
            42))
    "#;
    startup_ok(src);
}

// --- 3. Value-position works: :wat::core::nil at value position --------

#[test]
fn value_position_nil_keyword_type_checks_and_evaluates() {
    // `:wat::core::nil` at value position is the nil-value
    // literal. The infer hook types it as the nil singleton; the
    // eval hook returns `Value::Unit`. Recipient unification with
    // a `-> :wat::core::nil` declaration succeeds. Post-sweep-1b:
    // full startup success.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::nil)
          :wat::core::nil)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do
            (:my::probe)
            7))
    "#;
    startup_ok(src);
}

// --- 4. Type mismatch: declaring i64 but body is :wat::core::nil -------

#[test]
fn value_position_nil_against_i64_recipient_fires_type_mismatch() {
    // The probe declares `-> :wat::core::i64` but the body is the
    // nil keyword. Substrate types the body as nil (singleton);
    // recipient unification against i64 fails; ReturnTypeMismatch
    // fires WITH a `<entry>` span. Verifies the value-position
    // special-case really ascribes the nil type (not
    // :wat::core::keyword).
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::i64)
          :wat::core::nil)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:my::probe))
    "#;
    let err = startup_err(src);
    // Expect a `<entry>`-spanned ReturnTypeMismatch (or
    // TypeMismatch) where expected is i64 and got is the nil
    // singleton (`:()` internally).
    assert!(
        err.contains(r#"ReturnTypeMismatch { function: ":my::probe""#)
            || err.contains(r#"file: "<entry>""#) && err.contains("TypeMismatch"),
        "expected user-source ReturnTypeMismatch when nil body meets i64 sig; got: {}",
        err
    );
}

// --- 5. Mixed: () body, :wat::core::nil sig ----------------------------

#[test]
fn mixed_empty_list_body_with_nil_sig_unifies() {
    // The body is `()` (the legacy empty-list literal at value
    // position; types as `:()` ~ singleton). The signature is
    // `-> :wat::core::nil` (canonical). Both produce the same
    // internal representation (`TypeExpr::Tuple(vec![])`);
    // unification succeeds; full startup success.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::nil)
          ())

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::do
            (:my::probe)
            123))
    "#;
    startup_ok(src);
}

// --- 6. Reverse mixed: :wat::core::nil body, retired :unit sig fires ---

#[test]
fn reverse_mixed_nil_body_with_retired_unit_sig_fires_migration() {
    // The body is `:wat::core::nil` (canonical, types as nil).
    // The signature is `-> :wat::core::unit` (RETIRED). The
    // walker fires `BareLegacyUnitName` against the SIG; the body
    // itself is fine. Verifies that the migration error attaches
    // to the type-position site, not the value-position site.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::unit)
          :wat::core::nil)

        (:wat::core::define (:user::main -> :wat::core::i64)
          42)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains(r#"BareLegacyUnitName { span: Span { file: "<entry>""#),
        "expected user-source-spanned BareLegacyUnitName against the retired sig; got: {}",
        err
    );
}

// --- 7. Value observable: nil keyword evaluates to Value::Unit ---------

#[test]
fn value_position_nil_evaluates_to_value_unit() {
    // The substrate's `eval` hook returns `Value::Unit` for
    // `:wat::core::nil` at value position. Post-sweep-1b the test
    // proves the infer hook types it as the singleton (recipient
    // unification with `-> :wat::core::nil` succeeds) and full
    // startup completes.
    let src = r#"
        (:wat::core::define (:my::nil-form -> :wat::core::nil)
          :wat::core::nil)

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:my::nil-form))
    "#;
    startup_ok(src);
}

// --- 8. Value-position parity: () still evaluates to Value::Unit -------

#[test]
fn value_position_empty_list_still_evaluates_to_unit() {
    // The empty-list literal `()` at value position continues to
    // type as the singleton (transitional spelling; sweep 1b
    // migrated value-position `()` to `:wat::core::nil` mechanically
    // but the legacy spelling still type-checks). Verifies arc 153
    // didn't accidentally retire the legacy spelling at the
    // type-check level.
    let src = r#"
        (:wat::core::define (:my::nil-form -> :wat::core::nil)
          ())

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:my::nil-form))
    "#;
    startup_ok(src);
}

// --- 9. Narrow special-case: other keywords still typed normally -------

#[test]
fn other_keywords_still_type_as_keyword() {
    // The value-position special-case is NARROW: only the exact
    // FQDN string `:wat::core::nil` participates. Every other
    // keyword (e.g. `:user::foo`, `:my::tag`) keeps its existing
    // typing path -- bare keywords pass the
    // `WatAST::Keyword(_, _) => Some(TypeExpr::Path(":wat::core::keyword"))`
    // arm and produce `:wat::core::keyword`-typed values.
    //
    // Verified via a function that takes a `:wat::core::keyword`
    // parameter and is called with `:user::foo`. If `:user::foo`
    // had been special-cased, type-check would fail at the call
    // site with a mismatch; if the special-case is correctly
    // narrow, type-check passes (no user-source errors).
    let src = r#"
        (:wat::core::define
          (:my::echo-keyword
            (k :wat::core::keyword)
            -> :wat::core::keyword)
          k)

        (:wat::core::define (:user::main -> :wat::core::keyword)
          (:my::echo-keyword :user::foo))
    "#;
    startup_ok(src);
}

// --- 10. Walker hits inside parametric: Option<wat::core::unit> -------

#[test]
fn walker_fires_inside_parametric_arg() {
    // The retired `:wat::core::unit` token nested inside a
    // parametric (here a `:wat::core::Option<wat::core::unit>`
    // annotation) still reaches the walker recursion through the
    // Parametric arm and fires `BareLegacyUnitName`. Validates
    // walker recursion mirrors arc 109 slice 1d's
    // `BareLegacyUnitType` recursion through Parametric/Fn/Tuple
    // arms.
    let src = r#"
        (:wat::core::define
          (:my::probe -> :wat::core::Option<wat::core::unit>)
          :wat::core::None)

        (:wat::core::define (:user::main -> :wat::core::i64)
          42)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains(r#"BareLegacyUnitName { span: Span { file: "<entry>""#),
        "expected user-source-spanned BareLegacyUnitName against the nested retired token; got: {}",
        err
    );
}
