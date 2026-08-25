//! Arc 296 Strike 3b probe — `#[derive(ToEdn)]` on `LoadErrorKind` is
//! byte-identical to the deleted hand-written serializer.
//!
//! Asserts that `wat_edn::write(&err.to_edn())` equals the pre-derive
//! golden EDN string for every `LoadErrorKind` variant (7 variants,
//! one deterministic-span assertion each). SET-diff ∅.
//!
//! Arc 298.2 note: the former per-variant `*_unknown_span` tests proved the
//! elide-when-span-unknown branch of the hand-written serializer. That branch
//! was annihilated with `Span::unknown()` — there is now exactly one code path
//! (always emit `:span`), so a second span state is a byte-for-byte duplicate
//! of the `*_known_span` golden. The redundant tests were deleted; the two
//! variants that only had an unknown-span test (`CycleDetected`, and `Parse`
//! whose inner `ParseError` location was previously path-dependent) were
//! converted to a single deterministic-span golden with a fixed inner span.
//!
//! ## How the golden strings were derived
//!
//! The old hand-written `impl ToEdn for LoadError` (deleted in Strike 3b)
//! produced `edn_tag(variant, Map(fields_in_declaration_order ++ span_if_known))`.
//! Each golden string was captured from the HEAD hand-written serializer
//! before the change, then committed here as the byte-identical contract.
//!
//! ## What this proves
//!
//! - The derive generates the same variant tag (`#wat.kernel/<Name>`).
//! - Field keys are snake→kebab converted in declaration order.
//! - `:span` is appended LAST by `splice_span` when the span is known.
//! - `:span` is ALWAYS emitted (arc 298.2 retired the elide-when-unknown branch).
//! - `Fetch(LoadFetchError)` — the new tuple-variant rule — produces
//!   `{:cause <inner.to_edn()>}` byte-identically.
//! - `Parse.err` uses `error_edn_of` (the recursive floor), NOT raw `to_edn`.
//! - `VerificationFailed.err` uses plain `to_edn` on `HashError`.

use std::sync::Arc;
use wat::hash::HashError;
use wat::load::{LoadError, LoadErrorKind, LoadFetchError};
use wat::span::Span;
use wat::edn::contract::ToEdn;
use wat_reader::parser::{ParseError, ParseErrorKind};

// ─── Shared span fixtures ─────────────────────────────────────────────────────

fn known_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn write(err: &LoadError) -> String {
    wat_edn::write(&err.to_edn())
}

fn make(span: Span, kind: LoadErrorKind) -> LoadError {
    LoadError::new(span, kind)
}

// ─── 1. MalformedLoadForm ─────────────────────────────────────────────────────

#[test]
fn probe_malformed_load_form_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::MalformedLoadForm { reason: "bad form".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__malformed_load_form.edn", "MalformedLoadForm with known span");
}

// ─── 2. SetterInLoadedFile ────────────────────────────────────────────────────

#[test]
fn probe_setter_in_loaded_file_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::SetterInLoadedFile {
            loaded_path: "foo.wat".to_string(),
            setter_head: "set-dims!".to_string(),
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__setter_in_loaded_file.edn", "SetterInLoadedFile with known span");
}

// ─── 3. DuplicateLoad ────────────────────────────────────────────────────────

#[test]
fn probe_duplicate_load_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::DuplicateLoad { path: "foo.wat".to_string() },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__duplicate_load.edn", "DuplicateLoad with known span");
}

// ─── 4. CycleDetected ────────────────────────────────────────────────────────

#[test]
fn probe_cycle_detected_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::CycleDetected {
            cycle: vec!["a.wat".to_string(), "b.wat".to_string()],
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__cycle_detected.edn", "CycleDetected with known span");
}

// ─── 5. Fetch — the new single-field tuple-variant rule ──────────────────────

#[test]
fn probe_fetch_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::Fetch(LoadFetchError::NotFound("missing.wat".to_string())),
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__fetch.edn", "Fetch with known span");
}

// ─── 6. Parse — recursive floor (error_edn_of, NOT raw to_edn) ───────────────

#[test]
fn probe_parse_known_span() {
    // Arc 298.2: the inner ParseError span uses a deterministic fixture too, so
    // the recursive `:location` is byte-stable — the whole EDN is a fixed golden.
    let err = make(
        known_span(),
        LoadErrorKind::Parse {
            path: "foo.wat".to_string(),
            err: ParseError {
                span: Span::new(Arc::new("inner.wat".to_string()), 7, 3),
                kind: ParseErrorKind::UnexpectedRParen,
            },
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__parse.edn", "Parse with known outer + inner span");
}

// ─── 7. VerificationFailed ───────────────────────────────────────────────────

#[test]
fn probe_verification_failed_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::VerificationFailed {
            path: "foo.wat".to_string(),
            err: HashError::UnsupportedAlgorithm { algo: "SHA1".to_string() },
        },
    );
    wat::assert_edn_matches_file!(write(&err), "probe_arc296_3b_loaderror_derive_identical__verification_failed.edn", "VerificationFailed with known span");
}
