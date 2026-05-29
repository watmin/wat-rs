//! FM 2-bis probe for Stone 241.9 — `:wat::core::defenum` HARD CUT.
//!
//! Replaces `:wat::core::enum` (positional unit variants + pair-form tagged variants)
//! with `:wat::core::defenum` (positional + one-token look-ahead grammar per
//! FORM-COLLAPSE-NOTES verdict D). Tagged-variant argspec Vectors use the canonical
//! `parse_argspec_triples` from Stone 241.1.
//!
//! HEAD-disconfirmation map (FM 2-bis discipline):
//! - C01-C04: success-path startup; weakly pass at HEAD (substrate silently no-ops
//!   unrecognized type-decl forms). Strong post-stone (defenum recognized).
//! - C05: empty {} rejection; CLEANLY DISCONFIRMS at HEAD (defenum no-ops →
//!   startup OK → rejection assertion fails). Strong post-stone.
//! - C06, C07: HARD CUT rejection of legacy enum; CLEANLY DISCONFIRMS at HEAD
//!   (legacy works → rejection assertion fails).
//! - C08: variant constructor :app::Status::Ok used as defn body; CLEANLY
//!   DISCONFIRMS at HEAD (type unregistered → constructor is bare :wat::core::keyword
//!   → ReturnTypeMismatch). Verified failure trace at HEAD.
//!
//! 4 of 8 contracts disconfirm cleanly at HEAD; the other 4 are post-stone
//! semantic contracts (consistent with Stone 241.8 precedent).
//!
//! Run: `cargo test --release --test probe_arc241_stone9_defenum`

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

// ─── Contracts 1-4: defenum success paths ─────────────────────────────────────

#[test]
fn contract_01_defenum_unit_only() {
    // Plain defenum — positional unit variants only; no metadata, no tagged.
    let src = r#"
        (:wat::core::defenum :app::Status
          :Ok
          :Pending
          :Error)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "plain defenum with unit variants should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defenum_mixed_unit_and_tagged() {
    // Mixed shape: one unit + one tagged variant via look-ahead grammar.
    // :Ok has no following Vector (next is end-of-args) → UNIT.
    // :Err is followed by Vector [...] → TAGGED; argspec triples per parse_argspec_triples.
    let src = r#"
        (:wat::core::defenum :app::Result
          :Ok
          :Err [code    <- :wat::core::i64
                message <- :wat::core::String])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defenum with mixed unit + tagged should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_03_defenum_interleaved_variants() {
    // Interleaved unit and tagged variants — verdict D's look-ahead grammar
    // handles arbitrary positional ordering.
    let src = r#"
        (:wat::core::defenum :app::Event
          :Tick
          :Move [x <- :wat::core::i64
                 y <- :wat::core::i64]
          :Reset
          :Resize [width <- :wat::core::i64])
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defenum with interleaved unit + tagged variants should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_04_defenum_with_variant_metadata() {
    // Form-level :variant-metadata mapping variant-keyword → metadata-map.
    // Inner keys are KEYWORDS (per Stone 241.8 T-fd trap-door — parser routes
    // {bareSymbol {submap}} to struct-destructure; keyword keys parse correctly).
    let src = r#"
        (:wat::core::defenum :app::Status
          {:variant-metadata {:Error {:doc "raised when the operation fails"}}}
          :Ok
          :Pending
          :Error)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defenum with :variant-metadata should startup; got: {:?}",
        result
    );
}

// ─── Contract 5: rejection (empty {} metadata) ────────────────────────────────

#[test]
fn contract_05_defenum_empty_metadata_rejected() {
    // Empty {} metadata is illegal per Stone 241.6 doctrine (divide-by-zero;
    // absence-of-metadata distinct from empty-metadata). defenum inherits.
    let src = r#"
        (:wat::core::defenum :app::Status
          {}
          :Ok
          :Err)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "defenum with empty {{}} metadata should be REJECTED; got OK"
    );
}

// ─── Contracts 6-7: HARD CUT rejection of legacy enum form ────────────────────

#[test]
fn contract_06_legacy_enum_unit_form_rejected() {
    // Legacy positional-unit enum was ACCEPTED pre-241.9; HARD CUT REJECTS it.
    let src = r#"
        (:wat::core::enum :app::Status
          :Ok
          :Pending
          :Error)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "legacy :wat::core::enum should be HARD-CUT-rejected post-241.9; got OK"
    );
}

#[test]
fn contract_07_legacy_enum_tagged_pair_form_rejected() {
    // Legacy pair-form tagged variant `(VariantName (field :Type) ...)` was
    // ACCEPTED pre-241.9; HARD CUT REJECTS it.
    let src = r#"
        (:wat::core::enum :app::Result
          :Ok
          (Err (code :wat::core::i64) (message :wat::core::String)))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "legacy enum tagged pair-form should be HARD-CUT-rejected post-241.9; got OK"
    );
}

// ─── Contract 8: defenum REGISTERS the type (semantic gap check) ──────────────

#[test]
fn contract_08_defenum_registers_usable_variants() {
    // Proves defenum actually REGISTERS the type + variant constructors —
    // not just no-ops the form. Variant constructors take the keyword shape
    // `:EnumName::VariantName` (per legacy enum tests).
    //
    // At HEAD: defenum unrecognized; :app::Status never registered;
    // `:app::Status::Ok` is an unknown keyword; startup ERRs.
    // Post-stone: defenum registers :app::Status + the three unit variants;
    // `:app::Status::Ok` is a usable constructor; startup OKs.
    let src = r#"
        (:wat::core::defenum :app::Status
          :Ok
          :Pending
          :Error)
        (:wat::core::defn :test::pick [] -> :app::Status :app::Status::Ok)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defenum should register :app::Status and its variants; got: {:?}",
        result
    );
}
