//! FM 2-bis probe for Stone 241.6 — Phase 2 opens; optional `{...}` metadata-map on `def`.
//!
//! ## Why this probe
//!
//! Stone 241.6 ships the SUBSTRATE STORAGE for binding-level metadata per
//! `FORM-COLLAPSE-NOTES.md`. The `def` parser accepts an optional `{...}`
//! HashMap clause between the binding name and the value-expr:
//!
//! ```scheme
//! (:wat::core::def :my::ns::my-fn
//!   {:doc "what this does"
//!    :restricted-to [:my::ns::]}
//!   value-expr)
//! ```
//!
//! Discrimination: `{...}` parses to `(:wat::core::HashMap K V k v ...)` List;
//! if items[2] head is `:wat::core::HashMap`, it's metadata. Else it's value-expr.
//! Empty `{}` is ILLEGAL.
//!
//! `defn` inherits via macro expansion: `(defn :name {meta} [args] -> :ret body)`
//! expands to `(def :name {meta} (fn [args] -> :ret body))`.
//!
//! ## What this probe proves
//!
//! Pre-stone (HEAD `d1cd488a`+): the substrate's `def` parser accepts only
//! 3-item forms `(def :name value)`; 4-item forms with `{...}` between name
//! and value FAIL because the metadata-map isn't recognized.
//!
//! Post-stone: 4-item def with metadata-map parses cleanly; metadata persists
//! in SymbolTable.binding_metadata; reflection (Stone 241.7) can read it.
//!
//! Run: `cargo test --release --test probe_arc241_stone6_def_metadata_map`

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

// ─── Contracts 1–3: def-with-metadata storage success paths ──────────────────

#[test]
fn contract_01_def_with_doc_metadata_parses() {
    startup_from_file("tests/reflection/probe_arc241_stone6_def_metadata_map_c01.wat")
        .unwrap_or_else(|e| panic!("def-with-metadata should startup cleanly; got: {:?}", e));
}

#[test]
fn contract_02_def_with_multi_entry_metadata_parses() {
    startup_from_file("tests/reflection/probe_arc241_stone6_def_metadata_map_c02.wat")
        .unwrap_or_else(|e| panic!("def-with-multi-metadata should startup cleanly; got: {:?}", e));
}

#[test]
fn contract_03_defn_with_metadata_inherits_via_macro() {
    startup_from_file("tests/reflection/probe_arc241_stone6_def_metadata_map_c03.wat")
        .unwrap_or_else(|e| panic!("defn-with-metadata should startup cleanly; got: {:?}", e));
}

// ─── Contracts 4–5: regression — existing behavior preserved ────────────────

#[test]
fn contract_04_def_without_metadata_unchanged() {
    // Regression: 3-item def (no metadata) MUST still work.
    startup_from_file("tests/reflection/probe_arc241_stone6_def_metadata_map_c04.wat")
        .unwrap_or_else(|e| panic!("def-without-metadata regression should startup cleanly; got: {:?}", e));
}

#[test]
fn contract_05_defn_without_metadata_unchanged() {
    // Regression: defn without metadata MUST still work.
    startup_from_file("tests/reflection/probe_arc241_stone6_def_metadata_map_c05.wat")
        .unwrap_or_else(|e| panic!("defn-without-metadata regression should startup cleanly; got: {:?}", e));
}

// ─── Contracts 6: error path ─────────────────────────────────────────────────

#[test]
fn contract_06_empty_metadata_rejected() {
    // (def :name {} value) — empty metadata is ILLEGAL per FORM-COLLAPSE-NOTES
    // ("divide-by-zero"; presence/absence distinction MUST be honored).
    let result = startup_from_file("tests/reflection/probe_arc241_stone6_def_metadata_map_c06.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::def"
            && reason == "empty metadata-map `{}` is illegal; provide at least one key-value pair"
    );
}
