//! Arc 296 slice 4 probe — CheckError implements `ToEdn`, emitting
//! `#wat.kernel/<VariantName>` tagged EDN (NOT the old `#wat.diag/<kind>` flat shape).
//!
//! ## What 296.4 changes
//!
//! Before: `CheckError::diagnostic()` returns a `Diagnostic` (a half-typed
//! intermediate); `emit_check_failure` calls `render_edn(&diag)` which produces
//! `#wat.diag/TypeMismatch {:callee "..." :expected "..." ...}`.
//!
//! After: `CheckError` implements `ToEdn`; `emit_check_failure` calls
//! `to_edn()` which produces `#wat.kernel/TypeMismatch {:callee "..." :expected "..." ...}`.
//!
//! Key invariants this probe verifies:
//! 1. `CheckError.to_edn()` compiles (trait exists, impl present).
//! 2. Tag is in `wat.kernel` namespace (NOT `wat.diag`).
//! 3. The error kind's discriminator is the tag name (e.g. `TypeMismatch`).
//! 4. Structured fields are present as EDN keywords.
//! 5. The output is valid EDN (round-trips through parse+write).
//!
//! RED before 296.4: `wat::to_edn::ToEdn` not implemented for `CheckError`.
//! GREEN after 296.4: all impls present, `diagnostic.rs` retired.

use std::sync::Arc;
use wat::check::error::{CheckError, CheckErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;

// ─── Probe 1 — TypeMismatch uses wat.kernel namespace ────────────────────────

#[test]
fn probe_1_type_mismatch_to_edn_is_wat_kernel_tagged() {
    let span = Span::new(Arc::new("test.wat".to_string()), 10, 5);
    let err = CheckError {
        span,
        kind: CheckErrorKind::TypeMismatch {
            callee: ":user::greet".into(),
            param: "name".into(),
            expected: ":wat::core::String".into(),
            got: ":wat::core::i64".into(),
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);

    eprintln!("=== probe_1: {}", s);

    // Must be tagged EDN (starts with #).
    assert!(s.starts_with('#'), "must be tagged EDN; got: {}", s);

    // Must use wat.kernel namespace (NOT wat.diag).
    assert!(
        s.contains("wat.kernel"),
        "must use wat.kernel namespace; got: {}",
        s
    );
    assert!(
        !s.contains("wat.diag"),
        "must NOT use old wat.diag namespace; got: {}",
        s
    );

    // Must carry structured callee field.
    assert!(
        s.contains(":user::greet") || s.contains("user::greet"),
        "must contain callee; got: {}",
        s
    );

    // Must be valid EDN (parseable).
    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 2 — ArityMismatch carries expected/got integers ───────────────────

#[test]
fn probe_2_arity_mismatch_to_edn_carries_counts() {
    let span = Span::new(Arc::new("src/main.wat".to_string()), 5, 1);
    let err = CheckError {
        span,
        kind: CheckErrorKind::ArityMismatch {
            callee: ":user::add".into(),
            expected: 2,
            got: 3,
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);

    eprintln!("=== probe_2: {}", s);

    assert!(s.starts_with('#'), "must be tagged EDN; got: {}", s);
    assert!(s.contains("wat.kernel"), "must use wat.kernel namespace; got: {}", s);
    // Expected and got counts must appear.
    assert!(s.contains('2'), "must contain expected count 2; got: {}", s);
    assert!(s.contains('3'), "must contain got count 3; got: {}", s);
    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 3 — UnknownCallee carries callee field ────────────────────────────

#[test]
fn probe_3_unknown_callee_to_edn_carries_callee() {
    let span = Span::new(Arc::new("lib.wat".to_string()), 3, 7);
    let err = CheckError {
        span,
        kind: CheckErrorKind::UnknownCallee {
            callee: ":user::do-thing".into(),
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);

    eprintln!("=== probe_3: {}", s);

    assert!(s.starts_with('#'), "must be tagged EDN; got: {}", s);
    assert!(s.contains("wat.kernel"), "must use wat.kernel namespace; got: {}", s);
    assert!(
        s.contains("do-thing") || s.contains(":user::do-thing"),
        "must contain callee; got: {}",
        s
    );
    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 4 — CommCallOutOfPosition (the CLI test case) ─────────────────────

#[test]
fn probe_4_comm_call_out_of_position_to_edn() {
    let span = Span::new(Arc::new("user.wat".to_string()), 8, 3);
    let err = CheckError {
        span,
        kind: CheckErrorKind::CommCallOutOfPosition {
            callee: ":wat::kernel::send".into(),
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);

    eprintln!("=== probe_4: {}", s);

    // Must produce #wat.kernel/CommCallOutOfPosition (NOT #wat.diag/).
    assert!(
        s.starts_with("#wat.kernel/CommCallOutOfPosition"),
        "must start with #wat.kernel/CommCallOutOfPosition; got: {}",
        s
    );
    // Must carry :callee field.
    assert!(s.contains(":callee"), "must contain :callee keyword; got: {}", s);
    assert!(
        s.contains(":wat::kernel::send"),
        "must contain callee value; got: {}",
        s
    );
    wat_edn::parse_owned(&s).expect("must be valid EDN");
}
