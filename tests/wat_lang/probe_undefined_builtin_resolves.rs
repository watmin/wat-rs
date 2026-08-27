//! FM-2-bis disconfirming probe: an undefined call-head under a RESERVED prefix
//! must be caught at CHECK/RESOLVE time — never reach runtime as "unknown function".
//!
//! THE GAP (resolve.rs `is_resolvable_call_head`): blanket-accepts ANY leaf under
//! `:wat::core::`/`:wat::kernel::` etc. — the namespace is validated, the leaf is
//! not. A wrong leaf (`+'2`, `Bogus`) falls through both gates and dies at RUNTIME.
//!
//! RED AT HEAD: `(:wat::core::i64::+'2 1 2)` freezes CLEAN — error deferred to runtime.
//! GREEN AFTER: resolve checks leaf membership against the dispatchable-builtin
//! source of truth and rejects it at check time.

use wat::freeze::{startup_beside, startup_from_file};

// A renamed-away operator (`+'2` → `+`): a wrong leaf under :wat::core::i64::.
// RED at HEAD: freezes clean today (deferred to runtime); GREEN after the fix.
#[test]
#[ignore = "RED-at-HEAD: checker rejection of undefined builtins (arc-255 builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn wrong_operator_leaf_is_a_check_error() {
    let result = startup_from_file(
        "tests/wat_lang/probe_undefined_builtin_resolves_wrong_leaf.wat.bad",
    );
    // ⛔ STOP-1-SHAPED FINDING (arc 296 Stone L) — NOT migrated, and no bare-is-err exemption
    // taken either (that door is for a REAL error with no stable discriminant; this fixture
    // raises NO error at all). Grounded via `./target/release/wat --check`: exits 0
    // today, confirming the `#[ignore]` above's own "RED-at-HEAD: checker rejection ... not yet
    // built" — the checker rejection this message describes does not exist yet. Lower-severity
    // than an active green false-proof (this test is `#[ignore]`d, so it proves nothing to
    // anyone today either way), but asserting a discriminant now would still be fabricating
    // grounding for behavior that isn't there. Un-ignoring this test (arc 255 circling back) is
    // the trigger to migrate it for real.
    assert!(
        result.is_err(),
        "(:wat::core::i64::+'2 ...) — a renamed-away operator leaf — must be caught \
         at check/resolve time, not deferred to a runtime 'unknown function'."
    );
}

// A bogus leaf under a real namespace. RED at HEAD.
#[test]
#[ignore = "RED-at-HEAD: checker rejection of undefined builtins (arc-255 builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn bogus_leaf_under_known_namespace_is_a_check_error() {
    let result = startup_from_file(
        "tests/wat_lang/probe_undefined_builtin_resolves_bogus.wat.bad",
    );
    // ⛔ STOP-1-SHAPED FINDING (arc 296 Stone L) — NOT migrated, same reasoning as the sibling
    // test above: `--check` exits 0 on this fixture too (verified), so there is no error yet,
    // let alone a discriminant, until arc 255's builtin-registry lands.
    assert!(
        result.is_err(),
        "(:wat::core::Bogus ...) — a wrong leaf under a real namespace — must be a \
         check/resolve error, not a runtime surprise."
    );
}

// Control — must NOT over-reject: the real operator keeps resolving.
#[test]
fn valid_operator_still_resolves() {
    let result = startup_beside(file!());
    assert!(
        result.is_ok(),
        "(:wat::core::i64::+ ...) is a real dispatchable builtin and must keep \
         type-checking; got: {:?}",
        result.err()
    );
}
