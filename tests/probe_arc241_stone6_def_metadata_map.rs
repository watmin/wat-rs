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

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn try_startup(src: &str) -> Result<(), String> {
    let full = with_nil_main(src);
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1–3: def-with-metadata storage success paths ──────────────────

#[test]
fn contract_01_def_with_doc_metadata_parses() {
    // (def :name {:doc "..."} value) — single-entry metadata
    let src = r#"
        (:wat::core::def :my::x
          {:doc "the x value"}
          42)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "def-with-metadata should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_02_def_with_multi_entry_metadata_parses() {
    // (def :name {:k1 :v1 :k2 :v2} value) — multi-entry
    let src = r#"
        (:wat::core::def :my::y
          {:doc "documented"
           :deprecated true}
          100)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "def-with-multi-metadata should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_03_defn_with_metadata_inherits_via_macro() {
    // defn-with-metadata expands to (def :name {meta} (fn ...))
    let src = r#"
        (:wat::core::defn :my::f
          {:doc "doubles its input"}
          [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+'2 x x))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defn-with-metadata should startup cleanly; got: {:?}",
        result
    );
}

// ─── Contracts 4–5: regression — existing behavior preserved ────────────────

#[test]
fn contract_04_def_without_metadata_unchanged() {
    // Regression: 3-item def (no metadata) MUST still work.
    let src = r#"
        (:wat::core::def :my::z 42)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "def-without-metadata regression should startup cleanly; got: {:?}",
        result
    );
}

#[test]
fn contract_05_defn_without_metadata_unchanged() {
    // Regression: defn without metadata MUST still work.
    let src = r#"
        (:wat::core::defn :my::g
          [x <- :wat::core::i64] -> :wat::core::i64
          x)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defn-without-metadata regression should startup cleanly; got: {:?}",
        result
    );
}

// ─── Contracts 6: error path ─────────────────────────────────────────────────

#[test]
fn contract_06_empty_metadata_rejected() {
    // (def :name {} value) — empty metadata is ILLEGAL per FORM-COLLAPSE-NOTES
    // ("divide-by-zero"; presence/absence distinction MUST be honored).
    let src = r#"
        (:wat::core::def :my::illegal
          {}
          42)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "empty {{}} metadata-map must error; got Ok"
    );
}
