//! Arc 296 — STOP probe: does `edn_to_value` cleanly round-trip the synthetic
//! `#wat.kernel/<diagnostic>` tagged values that `to_edn()` produces?
//!
//! The coordinator asked to build the `ProcessDiedError` payload as the NESTED
//! structured `Value` via `edn_to_value(e.to_edn())` instead of a
//! `Value::String`. This probe tests whether that conversion is even possible.
//!
//! FINDING (this probe documents the STOP): `edn_to_value` reconstructs a
//! tagged Map body ONLY as a registered wat type (struct/record/enum). The
//! diagnostic tags (`#wat.kernel/CheckErrors`, `#wat.kernel/UnknownCallee`,
//! `#wat.kernel/MainSignatureError`, …) are NOT registered wat types and the
//! `Value` enum has NO generic tagged variant to hold them — so the decode
//! fails with `NoTypeRegistry` (no registry) / `UnknownTag` (registry, no such
//! type). The structured EDN cannot become a `Value` without registering a
//! family of synthetic diagnostic types.

use wat::check::error::{CheckError, CheckErrorKind, CheckErrors};
use wat::edn::render::EdnReadErrorKind;
use wat::freeze::StartupError;
use wat::span::Span;
use wat::edn::contract::ToEdn;

/// `edn_to_value` on a `#wat.kernel/CheckErrors {…}` value FAILS — there is no
/// registered `:wat::kernel::CheckErrors` type and no generic tagged `Value`.
#[test]
fn check_errors_edn_does_not_round_trip_to_value() {
    let span = Span::new(std::sync::Arc::new("user.wat".to_string()), 8, 3);
    let startup = StartupError::Check(CheckErrors(vec![CheckError {
        span,
        kind: CheckErrorKind::UnknownCallee { callee: ":user::x".into() },
    }]));

    let edn = startup.to_edn(); // #wat.kernel/CheckErrors {:errors [#wat.kernel/UnknownCallee {…}]}
    eprintln!("structured EDN: {}", wat_edn::write(&edn));

    // No type registry available here (the common case for a freshly-parsed
    // envelope) → NoTypeRegistry.
    let decoded = wat::edn::render::edn_to_value(&edn, None, None);
    eprintln!("edn_to_value(None) → {:?}", decoded.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));
    // Not a `StartupError` result (this is `edn_to_value`'s own `EdnReadError`), so
    // `assert_startup_error!` doesn't apply here — matched directly against the inner
    // `EdnReadErrorKind` discriminant instead (arc 296 Stone L: the outer bool-shaped
    // `is_err()` asserted nothing about WHICH failure — `UnknownTag` and `UnsupportedTag`
    // are both live siblings a retirement/regression could produce instead).
    assert!(
        matches!(&decoded, Err(e) if matches!(e.kind, EdnReadErrorKind::NoTypeRegistry)),
        "EXPECTED edn_to_value to FAIL with EdnReadErrorKind::NoTypeRegistry on the synthetic \
         #wat.kernel/CheckErrors tag (no registered wat type, no generic tagged Value); if this \
         ever passes, the STOP no longer holds and the nested-Value payload becomes possible. \
         got: {:?}",
        decoded
    );
}
