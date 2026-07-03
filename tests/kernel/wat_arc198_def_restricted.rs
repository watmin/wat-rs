//! Arc 198 (post-Stone 241.14) — caller-restriction via binding metadata-map.
//!
//! Stone 241.14 retires `:wat::core::def-restricted` and
//! `:wat::core::defn-restricted`. Restrictions now live as `:restricted-to`
//! in a metadata-map on `def`/`defn`:
//!
//!   (:wat::core::defn :my::kernel::restricted-fn
//!     {:restricted-to [:my::kernel::]}
//!     [x <- :wat::core::i64] -> :wat::core::i64 x)
//!
//! The restriction walker (`walk_for_restricted_call` in check.rs) reads
//! `binding_metadata` (sole restriction store post-stone) and fires
//! `DefRestrictedCallerNotAllowed` for callers outside the whitelist.
//!
//! Prefix matching:
//! - Whitelist entry ending in `::` (e.g. `:wat::kernel::`) → caller FQDN
//!   must start with this prefix (namespace prefix match).
//! - Whitelist entry NOT ending in `::` (e.g. `:wat::kernel::specific-fn`)
//!   → caller FQDN must equal this entry exactly (exact FQDN match).

use wat::freeze::{startup_beside, startup_from_file};

// ─── Test 1 — Positive prefix match ───────────────────────────────────────

#[test]
fn def_restricted_caller_inside_allowed_namespace_passes() {
    // A restricted fn declared with {:restricted-to [:my::kernel::]}. A caller
    // FQDN `:my::kernel::caller` starts with that prefix, so the walker allows.
    startup_beside(file!()).expect("expected startup success; got errors");
}

// ─── Test 2 — Negative prefix mismatch ────────────────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn def_restricted_caller_outside_allowed_namespace_fails() {
    // Same restricted fn whitelist `[:my::kernel::]` but the caller FQDN
    // `:user::app::caller` does NOT start with that prefix. Walker fires.
    let err = format!("{:?}", startup_from_file(
        "tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat",
    ).expect_err("expected startup failure; got Ok"));
    assert_eq!(
        err,
        "Check(CheckErrors([CheckError { span: Span { file: \"tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat\", line: 9, col: 4, end_line: 9, end_col: 30 }, kind: DefRestrictedCallerNotAllowed { callee: \":my::kernel::restricted-fn\", enclosing_fn: \":user::app::caller\", prefixes: [\":my::kernel::\"] } }]))",
        "error must match golden"
    );
}

// ─── Test 3 — Exact FQDN match (no trailing ::) ───────────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn def_restricted_exact_fqdn_match_only_allows_named_caller() {
    // Whitelist entry `:my::kernel::specific-caller` (no trailing `::`) is an
    // exact FQDN. Only that one caller can reach the restricted fn; a sibling
    // in the same namespace (`:my::kernel::other-caller`) fails.
    startup_from_file("tests/kernel/wat_arc198_def_restricted_ok_exact_fqdn_allowed.wat")
        .expect("expected startup success for the exactly-named caller");

    let err = format!("{:?}", startup_from_file(
        "tests/kernel/wat_arc198_def_restricted_bad_exact_fqdn_denied.wat",
    ).expect_err("expected startup failure; got Ok"));
    assert_eq!(
        err,
        "Check(CheckErrors([CheckError { span: Span { file: \"tests/kernel/wat_arc198_def_restricted_bad_exact_fqdn_denied.wat\", line: 9, col: 4, end_line: 9, end_col: 30 }, kind: DefRestrictedCallerNotAllowed { callee: \":my::kernel::restricted-fn\", enclosing_fn: \":my::kernel::other-caller\", prefixes: [\":my::kernel::specific-caller\"] } }]))",
        "error must match golden"
    );
}

// ─── Test 4 — Multi-prefix whitelist ──────────────────────────────────────

#[test]
fn def_restricted_multi_prefix_whitelist_admits_either_namespace() {
    // Whitelist `[:my::kernel:: :my::test::]` admits any caller whose FQDN
    // starts with either prefix. Two callers — one in each namespace —
    // both pass.
    startup_from_file("tests/kernel/wat_arc198_def_restricted_ok_multi_prefix.wat")
        .expect("expected startup success for multi-prefix whitelist");
}

// ─── Test 5 — defn metadata-map enforces restriction ──────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn defn_metadata_restricted_enforces_for_caller_outside_whitelist() {
    // Stone 241.14: defn-restricted is retired. Restrictions on defn live
    // as {:restricted-to [...]} metadata-map. This test proves the metadata-
    // map restriction on defn is enforced: an allowed caller passes; a
    // non-allowed caller fails with DefRestrictedCallerNotAllowed.
    //
    // Positive: caller in allowed namespace → startup succeeds.
    // Reuses test 1's fixture (same logical shape: :my::kernel:: prefix, caller inside).
    startup_from_file("tests/kernel/wat_arc198_def_restricted.wat")
        .expect("expected startup success for caller in allowed namespace");

    // Negative: caller outside allowed namespace → walker fires.
    let err = format!("{:?}", startup_from_file(
        "tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat",
    ).expect_err("expected startup failure; got Ok"));
    assert_eq!(
        err,
        "Check(CheckErrors([CheckError { span: Span { file: \"tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat\", line: 9, col: 4, end_line: 9, end_col: 30 }, kind: DefRestrictedCallerNotAllowed { callee: \":my::kernel::restricted-fn\", enclosing_fn: \":user::app::caller\", prefixes: [\":my::kernel::\"] } }]))",
        "error must match golden"
    );
}
