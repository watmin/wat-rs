//! Arc 198 slice 2 Stone 1 — inventory crate wiring + `RestrictionEntry`
//! struct + setup-time iteration that populates
//! `SymbolTable.defined_value_restrictions` from `inventory::iter::<RestrictionEntry>`.
//!
//! This stone is SUBSTRATE-ONLY — no proc-macro yet, no annotation on any
//! existing substrate fn, no migration. The proof of wiring is a single
//! test that:
//!
//! 1. Declares a probe `RestrictionEntry` at the test crate's module-scope
//!    via `inventory::submit!`. The submit must happen at module-scope so
//!    the entry is collected by `inventory` at link time.
//! 2. Runs `startup_from_source` against a minimal valid wat program.
//! 3. Asserts that `frozen.symbols.defined_value_restrictions.get(<probe wat_name>)`
//!    returns the prefixes vec from the probe submission.
//!
//! If the wiring works, the probe entry lands in the HashMap during the
//! setup-time iteration step. If not, the assertion fires.
//!
//! The probe binding name uses an `arc198::s2::s1::probe::` namespace so
//! it cannot collide with anything in the substrate or stdlib. The
//! corresponding wat program does NOT need to reference this name — the
//! iteration populates the map regardless of whether any wat code uses
//! the binding. (Stone 2's proc-macro + Stone 3's annotation will hook
//! real substrate fns into this same channel.)

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
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

#[test]
fn inventory_submitted_restriction_entry_lands_in_symbol_table_after_startup() {
    // Minimal valid wat source: just `:user::main`, nothing else. The
    // iteration step runs unconditionally during startup, so the probe
    // entry should be present in the frozen world's symbol table even
    // when the user source declares no restrictions of its own.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;

    let frozen = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("minimal wat source should freeze cleanly");

    let entry = frozen
        .symbols
        .defined_value_restrictions
        .get(":arc198::s2::s1::probe::test-fn");

    assert!(
        entry.is_some(),
        "probe RestrictionEntry submitted via inventory::submit! should land \
         in frozen.symbols.defined_value_restrictions after startup. \
         Map currently has {} entries; probe key missing.",
        frozen.symbols.defined_value_restrictions.len()
    );

    let prefixes = entry.expect("entry presence asserted above");
    assert_eq!(
        prefixes,
        &vec![":wat::kernel::".to_string()],
        "probe prefixes should round-trip through inventory iteration unchanged"
    );
}
