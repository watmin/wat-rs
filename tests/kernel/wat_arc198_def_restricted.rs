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

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_beside, startup_from_file, StartupError};

/// Assert that `path` fails startup with exactly ONE
/// `DefRestrictedCallerNotAllowed`, carrying the given callee / enclosing fn /
/// whitelist.
///
/// This asserts the STRUCTURE, not a rendering. These three tests previously
/// compared `format!("{:?}", err)` against a hand-written Debug-face golden
/// string; arc 296 made that rendering EDN and every golden rotted at once —
/// while the mechanism underneath kept working perfectly. The tests were then
/// suppressed pending "the .edn data-equality flip", which is exactly this: a
/// test of the restriction walker must depend on the WALKER, not on how a
/// diagnostic happens to print today. Matching the typed error is stronger than
/// re-parsing EDN, because rustc checks it and no future rendering change can
/// break it (R59 — name what would have to break for this to go red: the
/// walker, and nothing else).
fn assert_restricted_call_rejected(
    path: &str,
    expected_callee: &str,
    expected_enclosing_fn: &str,
    expected_prefixes: &[&str],
) {
    let err = startup_from_file(path).expect_err("expected startup failure; got Ok");
    let errors: &CheckErrors = match &err {
        StartupError::Check(errs) => errs,
        other => panic!("expected StartupError::Check for {path}; got {other:?}"),
    };
    assert_eq!(errors.0.len(), 1, "expected exactly one check error for {path}; got {errors:?}");
    match &errors.0[0].kind {
        CheckErrorKind::DefRestrictedCallerNotAllowed { callee, enclosing_fn, prefixes } => {
            assert_eq!(callee, expected_callee, "callee mismatch for {path}");
            assert_eq!(enclosing_fn, expected_enclosing_fn, "enclosing fn mismatch for {path}");
            assert_eq!(
                prefixes.as_slice(),
                expected_prefixes,
                "whitelist mismatch for {path}"
            );
        }
        other => panic!("expected DefRestrictedCallerNotAllowed for {path}; got {other:?}"),
    }
}

// ─── Test 1 — Positive prefix match ───────────────────────────────────────

#[test]
fn def_restricted_caller_inside_allowed_namespace_passes() {
    // A restricted fn declared with {:restricted-to [:my::kernel::]}. A caller
    // FQDN `:my::kernel::caller` starts with that prefix, so the walker allows.
    startup_beside(file!()).expect("expected startup success; got errors");
}

// ─── Test 2 — Negative prefix mismatch ────────────────────────────────────

#[test]
fn def_restricted_caller_outside_allowed_namespace_fails() {
    // Same restricted fn whitelist `[:my::kernel::]` but the caller FQDN
    // `:user::app::caller` does NOT start with that prefix. Walker fires.
    assert_restricted_call_rejected(
        "tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat",
        ":my::kernel::restricted-fn",
        ":user::app::caller",
        &[":my::kernel::"],
    );
}

// ─── Test 3 — Exact FQDN match (no trailing ::) ───────────────────────────

#[test]
fn def_restricted_exact_fqdn_match_only_allows_named_caller() {
    // Whitelist entry `:my::kernel::specific-caller` (no trailing `::`) is an
    // exact FQDN. Only that one caller can reach the restricted fn; a sibling
    // in the same namespace (`:my::kernel::other-caller`) fails.
    startup_from_file("tests/kernel/wat_arc198_def_restricted_ok_exact_fqdn_allowed.wat")
        .expect("expected startup success for the exactly-named caller");

    assert_restricted_call_rejected(
        "tests/kernel/wat_arc198_def_restricted_bad_exact_fqdn_denied.wat",
        ":my::kernel::restricted-fn",
        ":my::kernel::other-caller",
        &[":my::kernel::specific-caller"],
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
    assert_restricted_call_rejected(
        "tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat",
        ":my::kernel::restricted-fn",
        ":user::app::caller",
        &[":my::kernel::"],
    );
}

// ─── Test 6 — a restriction governs MENTION, not head position ────────────

#[test]
fn def_restricted_value_position_alias_denied() {
    // DESIGN-STONE-a-restriction-governs-mention-not-head-position (arc 198,
    // filed 2026-08-15). Before this stone, `walk_for_restricted_call` only
    // checked the List-head position of a call site — a restricted FQDN
    // bound via `let` in VALUE position (never a call head) was a bare
    // `WatAST::Keyword` the walker recursed past in silence, so this exact
    // shape type-checked and RAN. The walker now fires on every
    // `WatAST::Keyword` mention, so the alias route is refused identically
    // to a direct call.
    assert_restricted_call_rejected(
        "tests/kernel/wat_arc198_def_restricted_bad_value_position_alias.wat",
        ":my::kernel::restricted-fn",
        ":user::sneaky",
        &[":my::kernel::"],
    );
}
