//! FM 2-bis probe for Stone 241.8 — `:wat::core::defstruct` HARD CUT.
//!
//! Replaces `:wat::core::struct` + `:wat::core::struct-restricted` with `:wat::core::defstruct`
//! using the metadata-map mechanism from Stone 241.6/7. Field-vector uses the canonical
//! `parse_argspec_triples` from Stone 241.1.
//!
//! Pre-stone: contracts 01-06 FAIL (defstruct verb doesn't exist).
//!            Contracts 07-08 FAIL (legacy struct + struct-restricted still work).
//! Post-stone: all 8 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone8_defstruct`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

fn try_startup(src: &str) -> Result<(), String> {
    let full = with_nil_main(src);
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1–5: defstruct success paths ──────────────────────────────────

#[test]
fn contract_01_defstruct_plain_struct() {
    // Plain defstruct — no metadata; argspec triples only.
    let src = r#"
        (:wat::core::defstruct :my::Point
          [x <- :wat::core::i64
           y <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "plain defstruct should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defstruct_with_restricted_to_metadata() {
    // defstruct with form-level :restricted-to (replaces struct-restricted's ctor whitelist).
    let src = r#"
        (:wat::core::defstruct :my::Token
          {:restricted-to [:my::]}
          [value <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defstruct with :restricted-to metadata should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_03_defstruct_with_field_metadata() {
    // defstruct with :field-metadata (per-field restrictions; replaces struct-restricted's
    // restricted-section).
    // Note: field keys in :field-metadata use keyword syntax (:witness) because the
    // parser routes {sym {map}} to struct-destructure (parse error); keyword keys parse correctly.
    let src = r#"
        (:wat::core::defstruct :my::Capability
          {:field-metadata {:witness {:restricted-to [:my::]}}}
          [witness <- :wat::core::i64
           data <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defstruct with :field-metadata should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_04_defstruct_with_both_form_and_field_metadata() {
    // Both form-level AND per-field metadata (full Counter::Client capability shape).
    // Note: field keys in :field-metadata use keyword syntax (:server-id, :client-id)
    // because the parser routes {sym {map}} to struct-destructure; keyword keys parse correctly.
    let src = r#"
        (:wat::core::defstruct :my::Client
          {:restricted-to  [:my::]
           :field-metadata {:server-id {:restricted-to [:my::]}
                            :client-id {:restricted-to [:my::]}}}
          [server-id <- :wat::core::Uuid
           client-id <- :wat::core::Uuid
           public-data <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defstruct with both form + field metadata should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_05_defstruct_multi_field_triples() {
    // Multiple fields; argspec stays RIGID 3-slot triples.
    let src = r#"
        (:wat::core::defstruct :my::Candle
          [open <- :wat::core::f64
           high <- :wat::core::f64
           low <- :wat::core::f64
           close <- :wat::core::f64
           volume <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "multi-field defstruct should startup; got: {:?}",
        result
    );
}

// ─── Contracts 6: error paths ────────────────────────────────────────────────

#[test]
fn contract_06_defstruct_empty_metadata_rejected() {
    // Empty {} ILLEGAL per FORM-COLLAPSE-NOTES.
    let src = r#"
        (:wat::core::defstruct :my::Bad
          {}
          [x <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "empty {{}} metadata-map must error; got Ok"
    );
}

// ─── Contracts 7–8: HARD CUT — legacy verbs REJECTED ─────────────────────────

#[test]
fn contract_07_legacy_struct_hard_cut() {
    // `:wat::core::struct` MUST be REJECTED post-stone (HARD CUT; no shim).
    let src = r#"
        (:wat::core::struct :my::Legacy (x :wat::core::i64) (y :wat::core::i64))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "legacy :wat::core::struct must be HARD CUT REJECTED; got Ok"
    );
}

#[test]
fn contract_08_legacy_struct_restricted_hard_cut() {
    // `:wat::core::struct-restricted` MUST be REJECTED post-stone (HARD CUT).
    let src = r#"
        (:wat::core::struct-restricted :my::LegacyR
          [:my::]
          ()
          (x <- :wat::core::i64))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "legacy :wat::core::struct-restricted must be HARD CUT REJECTED; got Ok"
    );
}
