//! Arc 198 slice 2 Stone 3 — apply `#[restricted_to(...)]` to substrate
//! `eval_kernel_*_join_result` fns.
//!
//! Stones 1 + 2 built the channel: Stone 1 wired `inventory` +
//! `RestrictionEntry` + setup-time drain into `binding_metadata`
//! (post-Stone 241.14; was `defined_value_restrictions` until migration);
//! Stone 2 added the `#[restricted_to(...)]` proc-macro attribute that
//! auto-emits the `inventory::submit!` block.
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
//! `frozen.symbols.binding_metadata` must contain entries for both
//! `Thread/join-result` and `Process/join-result`, each mapping to a
//! `:restricted-to` value encoding `[":wat::"]`.
//!
//! That assertion proves the substrate's `#[restricted_to(...)]`
//! annotations on the real eval fns reached the unified registry by
//! way of the same channel Stone 1 + Stone 2's probe tests already
//! validated independently.
//!
//! Stone 4 deletes Stone B's redundant ad-hoc walker once arc 198's
//! generic `walk_for_restricted_call` is observably providing
//! the same coverage — Stone 3 leaves Stone B's rule in place
//! (BOTH walkers fire on user-namespace calls until Stone 4).

use wat::ast::WatAST;
use wat::freeze::startup_bare;

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
fn thread_join_result_restriction_lands_in_symbol_table() {
    let frozen = startup_bare().expect("startup_bare should freeze cleanly");

    let meta = frozen
        .symbols
        .binding_metadata
        .get(":wat::kernel::Thread/join-result");

    assert!(
        meta.is_some(),
        "Stone 3: #[restricted_to(...)] on eval_kernel_thread_join_result \
         should land in binding_metadata after startup. Map has \
         {} entries; :wat::kernel::Thread/join-result key missing.",
        frozen.symbols.binding_metadata.len()
    );

    let meta_map = meta.expect("meta presence asserted above");
    let restricted_to_ast = meta_map.get(":restricted-to");
    assert!(
        restricted_to_ast.is_some(),
        "binding_metadata for Thread/join-result should have :restricted-to key"
    );

    let prefixes = extract_prefixes_from_binding_metadata_entry(
        restricted_to_ast.expect("restricted-to presence asserted above"),
    );
    assert_eq!(
        prefixes,
        vec![":wat::".to_string()],
        "Thread/join-result restriction should whitelist exactly the \
         :wat:: namespace prefix (any caller under :wat::* permitted)"
    );
}

#[test]
fn process_join_result_restriction_lands_in_symbol_table() {
    let frozen = startup_bare().expect("startup_bare should freeze cleanly");

    let meta = frozen
        .symbols
        .binding_metadata
        .get(":wat::kernel::Process/join-result");

    assert!(
        meta.is_some(),
        "Stone 3: #[restricted_to(...)] on eval_kernel_process_join_result \
         should land in binding_metadata after startup. Map has \
         {} entries; :wat::kernel::Process/join-result key missing.",
        frozen.symbols.binding_metadata.len()
    );

    let meta_map = meta.expect("meta presence asserted above");
    let restricted_to_ast = meta_map.get(":restricted-to");
    assert!(
        restricted_to_ast.is_some(),
        "binding_metadata for Process/join-result should have :restricted-to key"
    );

    let prefixes = extract_prefixes_from_binding_metadata_entry(
        restricted_to_ast.expect("restricted-to presence asserted above"),
    );
    assert_eq!(
        prefixes,
        vec![":wat::".to_string()],
        "Process/join-result restriction should whitelist exactly the \
         :wat:: namespace prefix (any caller under :wat::* permitted)"
    );
}
