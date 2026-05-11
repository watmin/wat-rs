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
//!     surface from `startup_err`.
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

// --- 1. Type-position retired: :wat::core::unit now unknown FQDN -------

#[test]
fn type_position_unit_post_retirement_is_unknown_fqdn() {
    // Arc 153 slice 2 — substrate retirement closed the
    // `BareLegacyUnitName` migration window. The walker body
    // retired (substrate-as-teacher § "Retire the hint when its
    // window closes"); the typealias `:wat::core::unit -> :()`
    // also retired. Post-retirement behavior: `:wat::core::unit`
    // parses as `Path(":wat::core::unit")`, expand_alias returns
    // it unchanged (no longer registered), and unification
    // against the body's inferred `:()` (Tuple(vec![])) fails
    // with `ReturnTypeMismatch` carrying `expected:
    // ":wat::core::unit"` and `got: ":()"`. The variant +
    // Display for `BareLegacyUnitName` are retained as orphaned
    // scaffolding (arc 113 precedent — variant stays for
    // testing/teaching; only the firing body retires).
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::unit)
          ())

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    // Arc 163 follow-up — walker re-armed; bare :wat::core::unit
    // now fires BareLegacyUnitName fatal at check time (replaces the
    // post-arc-153 ReturnTypeMismatch fall-through path).
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyUnitName"),
        "expected BareLegacyUnitName walker to fire on retired :wat::core::unit; got: {}",
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

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:my::probe))
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

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:my::probe))
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

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
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

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:my::probe))
    "#;
    startup_ok(src);
}

// --- 6. Reverse mixed: :wat::core::nil body, retired :unit sig --------

#[test]
fn reverse_mixed_nil_body_with_retired_unit_sig_post_retirement() {
    // Arc 153 slice 2 — post-retirement shape (paired with test 1).
    // The body is `:wat::core::nil` (canonical, types as the nil
    // singleton). The signature is `-> :wat::core::unit` (retired
    // FQDN, no longer registered). The walker hint is gone;
    // unification surfaces `ReturnTypeMismatch` with
    // `expected: ":wat::core::unit"` against `got: ":()"`. The
    // body-side spelling is fine; the error attaches to the
    // signature mismatch as expected.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::unit)
          :wat::core::nil)

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    // Arc 163 follow-up — walker re-armed; the retired sig token
    // fires BareLegacyUnitName fatal before unification reaches
    // ReturnTypeMismatch.
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyUnitName"),
        "expected BareLegacyUnitName walker to fire on retired :wat::core::unit sig; got: {}",
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

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_ok(src);
}

// --- 10. Walker scaffold retired: BareLegacyUnitName no longer fires --

#[test]
fn bare_legacy_unit_name_walker_retired() {
    // Arc 153 slice 2 — substrate-as-teacher § "Retire the hint
    // when its window closes." The walker body retired; Path-arm
    // detection in `walk_type_for_bare` retired; signature-pass
    // call site retired. Sanity: even a top-level
    // `:wat::core::unit` annotation no longer produces
    // `BareLegacyUnitName` anywhere in the error stream. The
    // variant + Display remain as orphaned scaffolding for
    // testing/teaching (arc 113 precedent); future
    // symbol-migration arcs reintroduce the firing path with new
    // variants.
    let src = r#"
        (:wat::core::define (:my::probe -> :wat::core::unit)
          ())

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    // Arc 163 follow-up — walker RE-ARMED after arc 163 audit found
    // the silent-acceptance gap inconsistent with the FQDN-everywhere
    // discipline. The variant + Display preserved per arc 113
    // precedent stand; the firing path is back online.
    let err = startup_err(src);
    assert!(
        err.contains("BareLegacyUnitName"),
        "expected BareLegacyUnitName walker to fire on bare :wat::core::unit; got: {}",
        err
    );
}
