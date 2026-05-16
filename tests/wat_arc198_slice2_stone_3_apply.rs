//! Arc 198 slice 2 Stone 3 — apply `#[restricted_to(...)]` to substrate
//! `eval_kernel_*_join_result` fns.
//!
//! Stones 1 + 2 built the channel: Stone 1 wired `inventory` +
//! `RestrictionEntry` + setup-time drain into
//! `symbols.defined_value_restrictions`; Stone 2 added the
//! `#[restricted_to(...)]` proc-macro attribute that auto-emits the
//! `inventory::submit!` block.
//!
//! Stone 3 applies the attribute to the two real substrate fns that
//! arc 170 Stone B currently protects via an ad-hoc walker rule:
//!
//! - `eval_kernel_thread_join_result`  (wat name `:wat::kernel::Thread/join-result`)
//! - `eval_kernel_process_join_result` (wat name `:wat::kernel::Process/join-result`)
//!
//! Both restrictions whitelist exactly one caller-namespace prefix:
//! `:wat::` — meaning any caller whose FQDN lives anywhere under the
//! `:wat::` namespace tree is permitted; everything else is blocked.
//!
//! ## What this test verifies
//!
//! After `startup_from_source` against a minimal valid wat program,
//! `frozen.symbols.defined_value_restrictions` must contain entries
//! for both `Thread/join-result` and `Process/join-result`, each
//! mapping to a one-element vec containing `":wat::"`.
//!
//! That assertion proves the substrate's `#[restricted_to(...)]`
//! annotations on the real eval fns reached the unified registry by
//! way of the same channel Stone 1 + Stone 2's probe tests already
//! validated independently.
//!
//! Stone 4 deletes Stone B's redundant ad-hoc walker once arc 198's
//! generic `walk_for_def_restricted_call` is observably providing
//! the same coverage — Stone 3 leaves Stone B's rule in place
//! (BOTH walkers fire on user-namespace calls until Stone 4).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn thread_join_result_restriction_lands_in_symbol_table() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;

    let frozen = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("minimal wat source should freeze cleanly");

    let entry = frozen
        .symbols
        .defined_value_restrictions
        .get(":wat::kernel::Thread/join-result");

    assert!(
        entry.is_some(),
        "Stone 3: #[restricted_to(...)] on eval_kernel_thread_join_result \
         should land in defined_value_restrictions after startup. Map has \
         {} entries; :wat::kernel::Thread/join-result key missing.",
        frozen.symbols.defined_value_restrictions.len()
    );

    let prefixes = entry.expect("entry presence asserted above");
    assert_eq!(
        prefixes,
        &vec![":wat::".to_string()],
        "Thread/join-result restriction should whitelist exactly the \
         :wat:: namespace prefix (any caller under :wat::* permitted)"
    );
}

#[test]
fn process_join_result_restriction_lands_in_symbol_table() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;

    let frozen = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("minimal wat source should freeze cleanly");

    let entry = frozen
        .symbols
        .defined_value_restrictions
        .get(":wat::kernel::Process/join-result");

    assert!(
        entry.is_some(),
        "Stone 3: #[restricted_to(...)] on eval_kernel_process_join_result \
         should land in defined_value_restrictions after startup. Map has \
         {} entries; :wat::kernel::Process/join-result key missing.",
        frozen.symbols.defined_value_restrictions.len()
    );

    let prefixes = entry.expect("entry presence asserted above");
    assert_eq!(
        prefixes,
        &vec![":wat::".to_string()],
        "Process/join-result restriction should whitelist exactly the \
         :wat:: namespace prefix (any caller under :wat::* permitted)"
    );
}
