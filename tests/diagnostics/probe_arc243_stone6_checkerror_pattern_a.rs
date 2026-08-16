//! Probe — arc 243 Stone 243.6 — CheckError Pattern A structural verification
//!
//! FM 2-bis disconfirming probe: asserts the POST-stone shape.
//!
//! - PRE-stone state: this probe FAILS TO COMPILE. `CheckError` is currently
//!   a flat 34-variant enum (`pub enum CheckError { Variant { span, ... }, ...
//!   }`) where every variant carries its own `span` field; there is no
//!   `CheckErrorKind` type and `CheckError` has no `kind`/`span` struct fields.
//!   Constructing `CheckError { span, kind }` is a type error, and
//!   `CheckErrorKind` is an unresolved import.
//! - POST-stone state: this probe COMPILES + PASSES. `CheckError` is the
//!   Pattern A shape (`pub struct CheckError { pub span: Span, pub kind:
//!   CheckErrorKind }`); the constructor requires `span` at the outer field;
//!   the variants live on `CheckErrorKind` with no per-variant `span` field
//!   (multi-span variants keep only their SECONDARY spans as domain-named
//!   kind fields per CONFORMARE.md § Multi-span).
//!
//! The disconfirmation is STRUCTURAL not behavioral: pre-stone CheckError
//! requires a `span` on EVERY variant (and `diagnostic()` does N-arm span
//! extraction); post-stone span lives once at the outer struct and every
//! consumer reads `err.span` — one path, not a 34-arm match. The probe
//! demonstrates the structural enforcement Rust's type system now imposes.

use std::sync::Arc;
use wat::check::{CheckError, CheckErrorKind};
use wat::span::Span;

/// Contract 1: CheckError carries `span: Span` at the outer struct level —
/// every variant inherits the location discipline by construction.
#[test]
fn checkerror_outer_span_field_required() {
    let err = CheckError {
        span: wat::rust_caller_span!(),
        kind: CheckErrorKind::ArityMismatch {
            callee: "f".to_string(),
            expected: 2,
            got: 3,
        },
    };

    // The struct's span field is universally accessible — no exhaustive match
    // across 34 variants required. This is the load-bearing UX win.
    let _span: &Span = &err.span;

    // The kind enum holds variant-specific data only — no span field per
    // variant.
    assert!(matches!(err.kind, CheckErrorKind::ArityMismatch { .. }));
}

/// Contract 2: CheckErrorKind variants do NOT carry per-variant span fields.
/// This contract documents the type-level shape post-Pattern-A retrofit.
///
/// Pre-stone (current): the variants live directly on `enum CheckError` with
/// each carrying its own `span: Span` field (ArityMismatch, TypeMismatch,
/// UnknownCallee, ... — all 34).
///
/// Post-stone: variants live on `enum CheckErrorKind` with NO per-variant
/// span field; span lives at the outer struct level uniformly.
#[test]
fn checkerrorkind_variants_have_no_span_field() {
    // Construct a variant; verify no `span:` field is required at the kind
    // level (the kind holds only the variant's domain data).
    let kind = CheckErrorKind::UnknownCallee {
        callee: "g".to_string(),
    };

    let err = CheckError {
        span: wat::rust_caller_span!(),
        kind,
    };

    let _span: &Span = &err.span;
    assert!(matches!(err.kind, CheckErrorKind::UnknownCallee { .. }));
}

/// Contract 3: Span access is universal — no exhaustive match across 34
/// variants required. This contract demonstrates the consumer-side UX win:
/// any CheckError consumer needing span data accesses `err.span` directly.
///
/// Pre-stone (current): `diagnostic()` (src/check.rs:1361) does N-arm span
/// extraction across the variants — the consumer overhead conformare flagged.
///
/// Post-stone: `err.span` — single field access; the N-arm match collapses.
#[test]
fn checkerror_span_access_is_single_path() {
    let variants_under_test: Vec<(CheckError, Span)> = vec![
        (
            CheckError {
                span: wat::rust_caller_span!(),
                kind: CheckErrorKind::ArityMismatch {
                    callee: "a".into(),
                    expected: 1,
                    got: 0,
                },
            },
            wat::rust_caller_span!(),
        ),
        (
            CheckError {
                span: wat::rust_caller_span!(),
                kind: CheckErrorKind::UnknownCallee { callee: "b".into() },
            },
            wat::rust_caller_span!(),
        ),
    ];

    for (err, expected_span) in &variants_under_test {
        // Universal single-path access — works for EVERY CheckError regardless
        // of which kind variant. The whole point of Pattern A.
        let actual_span: &Span = &err.span;
        assert_eq!(actual_span, expected_span);
    }
}

/// Contract 4a: Display elides SECONDARY unknown spans — secondary-span
/// variants must not emit "<runtime>" when their secondary span is unknown.
///
/// Variant under test: `ProcessJoinHoldsStdinSender` with a known outer span
/// and an UNKNOWN secondary `stdin_sender_span`.  The pre-fix code interpolated
/// `stdin_sender_span` unconditionally; the fix gates it so the phrase reads
/// naturally without a location.
#[test]
fn checkerror_display_elides_unknown_secondary_span() {
    let known_outer = Span::new(Arc::new("src/bar.wat".to_string()), 5, 1);

    // --- (a) UNKNOWN secondary span: must not emit "<runtime>" ---
    let err_unknown_secondary = CheckError {
        span: known_outer.clone(),
        kind: CheckErrorKind::ProcessJoinHoldsStdinSender {
            process_identifier: "worker".to_string(),
            stdin_sender_span: wat::rust_caller_span!(),
        },
    };
    let rendered = err_unknown_secondary.to_string();
    // rune:lint(loose-assert) — the stdin_sender_span is from rust_caller_span!() which
    // embeds the absolute host filesystem path to this test source file; the full rendered
    // string varies by host. The absence of "<runtime>" is the real contract.
    assert!(
        !rendered.contains("<runtime>"),
        "unknown secondary span must not appear in Display output; got: {rendered:?}"
    );

    // --- (b) KNOWN secondary span: location must appear ---
    let known_secondary = Span::new(Arc::new("src/bar.wat".to_string()), 2, 7);
    let err_known_secondary = CheckError {
        span: known_outer,
        kind: CheckErrorKind::ProcessJoinHoldsStdinSender {
            process_identifier: "worker".to_string(),
            stdin_sender_span: known_secondary,
        },
    };
    let rendered_known = err_known_secondary.to_string();
    // 296 recapture: staleness — EDN face (Stone B); message text, :location (src/bar.wat
    // 5:1), and :bind-location (src/bar.wat 2:7) are byte-identical to the pre-stone-B
    // prose face's embedded values; additive :message/:causes/:location/:process-identifier.
    wat::assert_edn_matches_file!(
        rendered_known,
        "probe_arc243_stone6_checkerror_pattern_a__checkerror_display_elides_unknown_secondary_span.edn",
        "known secondary span must appear in Display output"
    );
}

/// Contract 4: Display elides unknown spans — the doc claim is true in code.
///
/// Two sub-cases:
/// (a) UNKNOWN span (`wat::rust_caller_span!()`) — mid-prose " at <runtime>:0:0" must
///     NOT appear in the rendered string.  Only the `span_prefix` and
///     `diagnostic()` paths gated this; `fmt_with_span`'s mid-prose branches
///     previously checked `Some`-ness only and would emit the synthetic noise.
///     After the fix they check `!is_unknown()` via `shown` and stay silent.
///
/// (b) KNOWN span (`Span::new(...)`) — the " at <file>:<line>:<col>:" text
///     MUST appear (unchanged behavior; the fix must not suppress known spans).
///
/// `BareLegacyContainerHead` is chosen because it is a simple mid-prose
/// variant (no secondary-span fields, emits "bare container type '{head}'
/// at {s}" inline). The probe's original specimen was a scope/channel-pair
/// deadlock-detection variant whose walker was retired (the mistake it
/// detected is now structurally unrepresentable); that variant no longer
/// exists on `CheckErrorKind`, so this probe changes specimen rather than
/// dying with the walker.
#[test]
fn checkerror_display_elides_unknown_span() {
    // --- (a) UNKNOWN span: mid-prose location must be suppressed ---
    let err_unknown = CheckError {
        span: wat::rust_caller_span!(),
        kind: CheckErrorKind::BareLegacyContainerHead {
            head: "HashMap".to_string(),
            fqdn: "wat::core::HashMap".to_string(),
        },
    };
    let rendered_unknown = err_unknown.to_string();
    // rune:lint(loose-assert) — the outer span is from rust_caller_span!() which embeds
    // the absolute host filesystem path to this test source file (file:line:col varies
    // by host and source edits); asserting the full string would be host-specific.
    // The absence of "<runtime>" is the real contract.
    assert!(
        !rendered_unknown.contains("<runtime>"),
        "unknown span must not appear in Display output; got: {rendered_unknown:?}"
    );
    // rune:lint(loose-assert) — same as above: rendered_unknown is from err_unknown.to_string() where the outer span is from rust_caller_span!() which embeds the absolute host filesystem path; full string varies by host. The absence of the "<runtime>" sentinel is the real contract.
    assert!(
        !rendered_unknown.contains(" at <runtime>:0:0"),
        "unknown span sentinel must not appear mid-prose; got: {rendered_unknown:?}"
    );

    // --- (b) KNOWN span: mid-prose location must appear ---
    let known_span = Span::new(Arc::new("src/foo.wat".to_string()), 10, 3);
    let err_known = CheckError {
        span: known_span,
        kind: CheckErrorKind::BareLegacyContainerHead {
            head: "HashMap".to_string(),
            fqdn: "wat::core::HashMap".to_string(),
        },
    };
    let rendered_known = err_known.to_string();
    // 296 recapture: staleness — EDN face (Stone B); message text and :location
    // (src/foo.wat 10:3) are byte-identical to the pre-stone-B prose face; additive
    // :message/:causes/:location/:head/:fqdn.
    wat::assert_edn_matches_file!(
        rendered_known,
        "probe_arc243_stone6_checkerror_pattern_a__checkerror_display_elides_unknown_span.edn",
        "known span must appear in Display output"
    );
}

/// Contract 5: `to_edn()` elides unknown spans — the `push_span` helper
/// makes it structurally impossible to emit "<runtime>:0:0" as an EDN
/// field key or value.
///
/// Variant under test: `BareLegacyContainerHead` — the `:location` key is
/// only added to the EDN body when `!span.is_unknown()` (via `push_span`).
///
/// (a) UNKNOWN outer span — the serialized EDN must not contain `"<runtime>"`.
/// (b) KNOWN outer span — the serialized EDN must contain `:location` and
///     the file:line:col data.
#[test]
fn edn_elides_unknown_span() {
    use wat::to_edn::ToEdn;

    // --- (a) UNKNOWN span: serialized EDN must not mention "<runtime>" ---
    let err_unknown = CheckError {
        span: wat::rust_caller_span!(),
        kind: CheckErrorKind::BareLegacyContainerHead {
            head: "HashMap".to_string(),
            fqdn: "wat::core::HashMap".to_string(),
        },
    };
    let edn_str = wat_edn::write(&err_unknown.to_edn());
    // rune:lint(loose-assert) — edn_str contains the absolute host filesystem path in :span from rust_caller_span!(); full string varies by host. Targeted absence of "<runtime>" sentinel is the real contract.
    assert!(
        !edn_str.contains("<runtime>"),
        "unknown span must not appear in EDN output; got: {edn_str:?}"
    );

    // --- (b) KNOWN span: :location key + file:line:col must appear ---
    let known_span = Span::new(Arc::new("src/baz.wat".to_string()), 7, 4);
    let err_known = CheckError {
        span: known_span,
        kind: CheckErrorKind::BareLegacyContainerHead {
            head: "HashMap".to_string(),
            fqdn: "wat::core::HashMap".to_string(),
        },
    };
    let edn_str_known = wat_edn::write(&err_known.to_edn());
    // D1 (arc 296 Strike 2b): primary span key is now uniformly `:span`
    // across ALL CheckErrorKind variants. The outer CheckError::to_edn()
    // calls splice_span(kind.to_edn(), &self.span) which always appends `:span`.
    wat::assert_edn_matches_file!(edn_str_known, "probe_arc243_stone6_checkerror_pattern_a__bare_legacy_container_head_known_span.edn", "known span must produce :span key + file data in EDN output (D1: primary span normalized)");
}
