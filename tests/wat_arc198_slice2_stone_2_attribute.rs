//! Arc 198 slice 2 Stone 2 — `#[restricted_to(...)]` proc-macro attribute.
//!
//! This stone mints the attribute that emits `inventory::submit!` blocks
//! tying into Stone 1's `RestrictionEntry` + `inventory::collect!` wiring.
//!
//! Stone 1 proved the wiring (manual `inventory::submit!` lands in
//! `defined_value_restrictions` after startup). Stone 2 proves the
//! ergonomic surface: annotating a fn with `#[restricted_to(wat_name,
//! prefixes...)]` makes the same submission happen automatically without
//! the consumer typing a single `inventory::submit!`.
//!
//! ## Verification shape
//!
//! Three probe fns, each annotated with a different prefix-list shape:
//!
//! 1. **Single prefix:** one namespace-prefix entry (trailing `::`).
//! 2. **Multi-prefix:** two+ entries; verify all preserved in order.
//! 3. **Exact-FQDN:** prefix with NO trailing `::`; verify the exact-FQDN
//!    form is preserved unchanged (slice 1's matching rules treat
//!    trailing `::` vs none as namespace-prefix vs exact-FQDN match).
//!
//! Each test runs `startup_from_source` against a minimal valid wat
//! source and asserts the annotated fn's entry landed in
//! `frozen.symbols.defined_value_restrictions`.
//!
//! The probe fns themselves never execute — only their `#[restricted_to]`
//! annotations matter (the attribute emits the sibling `inventory::submit!`).
//! Bodies are `unreachable!()` to signal "never called by this test".

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat_macros::restricted_to;

// ── Single-prefix probe ────────────────────────────────────────────────

#[restricted_to(":arc198::s2::s2::probe::single", ":wat::kernel::")]
#[allow(dead_code)]
fn probe_single() -> i64 {
    unreachable!("probe_single is never invoked; only its annotation matters")
}

#[test]
fn single_prefix_attribute_lands_in_symbol_table() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;

    let frozen = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("minimal wat source should freeze cleanly");

    let entry = frozen
        .symbols
        .defined_value_restrictions
        .get(":arc198::s2::s2::probe::single");

    assert!(
        entry.is_some(),
        "single-prefix #[restricted_to] entry should land in \
         defined_value_restrictions after startup. Map has {} entries; \
         probe key missing.",
        frozen.symbols.defined_value_restrictions.len()
    );

    let prefixes = entry.expect("entry presence asserted above");
    assert_eq!(
        prefixes,
        &vec![":wat::kernel::".to_string()],
        "single-prefix entry should round-trip the one prefix verbatim"
    );
}

// ── Multi-prefix probe ─────────────────────────────────────────────────

#[restricted_to(
    ":arc198::s2::s2::probe::multi",
    ":wat::kernel::",
    ":wat::core::",
    ":my::specific::caller"
)]
#[allow(dead_code)]
fn probe_multi() -> i64 {
    unreachable!("probe_multi is never invoked; only its annotation matters")
}

#[test]
fn multi_prefix_attribute_preserves_all_prefixes_in_order() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;

    let frozen = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("minimal wat source should freeze cleanly");

    let entry = frozen
        .symbols
        .defined_value_restrictions
        .get(":arc198::s2::s2::probe::multi");

    assert!(
        entry.is_some(),
        "multi-prefix #[restricted_to] entry should land in \
         defined_value_restrictions after startup. Map has {} entries; \
         probe key missing.",
        frozen.symbols.defined_value_restrictions.len()
    );

    let prefixes = entry.expect("entry presence asserted above");
    assert_eq!(
        prefixes,
        &vec![
            ":wat::kernel::".to_string(),
            ":wat::core::".to_string(),
            ":my::specific::caller".to_string(),
        ],
        "multi-prefix entry should preserve all prefixes in declaration order"
    );
}

// ── Exact-FQDN probe (no trailing `::`) ────────────────────────────────

#[restricted_to(":arc198::s2::s2::probe::exact", ":wat::kernel::spawn-thread")]
#[allow(dead_code)]
fn probe_exact_fqdn() -> i64 {
    unreachable!("probe_exact_fqdn is never invoked; only its annotation matters")
}

#[test]
fn exact_fqdn_prefix_preserves_no_trailing_colons() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;

    let frozen = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("minimal wat source should freeze cleanly");

    let entry = frozen
        .symbols
        .defined_value_restrictions
        .get(":arc198::s2::s2::probe::exact");

    assert!(
        entry.is_some(),
        "exact-FQDN #[restricted_to] entry should land in \
         defined_value_restrictions after startup. Map has {} entries; \
         probe key missing.",
        frozen.symbols.defined_value_restrictions.len()
    );

    let prefixes = entry.expect("entry presence asserted above");
    assert_eq!(
        prefixes,
        &vec![":wat::kernel::spawn-thread".to_string()],
        "exact-FQDN form (no trailing `::`) must round-trip unchanged so \
         slice 1's matching rules can distinguish it from a namespace prefix"
    );
}
