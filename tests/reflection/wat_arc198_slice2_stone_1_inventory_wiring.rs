//! Arc 198 slice 2 Stone 1 — inventory crate wiring + `RestrictionEntry`
//! struct + setup-time iteration that populates
//! `SymbolTable.binding_metadata` from `inventory::iter::<RestrictionEntry>`.
//!
//! Stone 241.14 migrated the restriction store from the deleted
//! `defined_value_restrictions` HashMap to `binding_metadata` — the sole
//! restriction store post-stone. Each `RestrictionEntry` now lands in
//! `binding_metadata[wat_name][":restricted-to"]` as a `WatAST::List`
//! with `:wat::core::Vector` head followed by prefix keywords.
//!
//! This stone is SUBSTRATE-ONLY — no proc-macro yet, no annotation on any
//! existing substrate fn, no migration. The proof of wiring is a single
//! test that:
//!
//! 1. Declares a probe `RestrictionEntry` at the test crate's module-scope
//!    via `inventory::submit!`. The submit must happen at module-scope so
//!    the entry is collected by `inventory` at link time.
//! 2. Runs `startup_from_source` against a minimal valid wat program.
//! 3. Asserts that `frozen.symbols.binding_metadata` contains the probe
//!    name with the expected `:restricted-to` prefixes.
//!
//! If the wiring works, the probe entry lands in binding_metadata during the
//! setup-time iteration step. If not, the assertion fires.
//!
//! The probe binding name uses an `arc198::s2::s1::probe::` namespace so
//! it cannot collide with anything in the substrate or stdlib. The
//! corresponding wat program does NOT need to reference this name — the
//! iteration populates the map regardless of whether any wat code uses
//! the binding. (Stone 2's proc-macro + Stone 3's annotation will hook
//! real substrate fns into this same channel.)

use wat::ast::WatAST;
use wat::freeze::startup_bare;
use wat::restriction_entry::RestrictionEntry;

// Probe submission at module scope. `inventory::submit!` is a macro that
// emits a static item — these items are gathered at link time and exposed
// via `inventory::iter::<RestrictionEntry>`.
inventory::submit! {
    RestrictionEntry {
        wat_name: ":arc198::s2::s1::probe::test-fn",
        prefixes: &[":wat::kernel::"],
    }
}

/// Extract prefix strings from the internal-path `binding_metadata` encoding.
/// Post-Stone 241.14: value is `WatAST::List([Keyword(":wat::core::Vector"), Keyword(p1), ...])`.
fn extract_prefixes_from_binding_metadata_entry(entry: &WatAST) -> Vec<String> {
    match entry {
        WatAST::List(items, _) => items[1..]
            .iter()
            .filter_map(|n| {
                if let WatAST::Keyword(k, _) = n {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect(),
        _ => vec![],
    }
}

#[test]
fn inventory_submitted_restriction_entry_lands_in_symbol_table_after_startup() {
    // The iteration step runs unconditionally during startup, so the probe
    // entry should be present in the frozen world's symbol table even
    // when the user source declares no restrictions of its own.
    // startup_bare() = frozen default world, no user source — correct here
    // since this test's subject is the Rust substrate (inventory wiring),
    // not any wat program.
    let frozen = startup_bare().expect("startup_bare should freeze cleanly");

    let meta = frozen
        .symbols
        .binding_metadata
        .get(":arc198::s2::s1::probe::test-fn");

    assert!(
        meta.is_some(),
        "probe RestrictionEntry submitted via inventory::submit! should land \
         in frozen.symbols.binding_metadata after startup. \
         Map currently has {} entries; probe key missing.",
        frozen.symbols.binding_metadata.len()
    );

    let meta_map = meta.expect("meta presence asserted above");
    let restricted_to = meta_map.get(":restricted-to");
    assert!(
        restricted_to.is_some(),
        "binding_metadata for probe should have :restricted-to key"
    );

    let prefixes = extract_prefixes_from_binding_metadata_entry(
        restricted_to.expect("restricted-to presence asserted above"),
    );
    assert_eq!(
        prefixes,
        vec![":wat::kernel::".to_string()],
        "probe prefixes should round-trip through inventory iteration unchanged"
    );
}
