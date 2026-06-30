//! Arc 254.1 — FM-2-bis disconfirming probe: a channel payload TYPE must be
//! portable (wire-serializable). Channels carry messages, not resources; an
//! opaque resource type (here `:wat::kernel::Sender<...>`) must be rejected at
//! CHECK TIME under the uniform serializable contract.
//!
//! RED AT HEAD: the checker has no portability constraint on channel payloads,
//! so a channel-of-Senders type-checks clean → the `is_err()` assertion fails.
//! GREEN AFTER 254.1: the checker gates payloads with `is_portable_type`
//! (factored from `closure_extract.rs`'s value-level portability classifier)
//! and rejects the non-portable payload.
//!
//! Uses `make-channel :T` (the one canonical depth-1 constructor — arc 254.0).

use wat::freeze::{startup_from_file, StartupError};

fn check_result(path: &str) -> Result<(), String> {
    match startup_from_file(path) {
        Ok(_) => Ok(()),
        Err(StartupError::Check(errs)) => Err(format!("{}", errs)),
        Err(other) => Err(format!("{}", other)),
    }
}

// FINDING (2026-06-06, de-risk): a BARE `Sender<T>` payload is already rejected
// — but by the `parse_type_keyword` "valid type keyword" gate (check.rs:12779),
// NOT a portability check. So 254.1 is not greenfield; a payload-type gate
// exists. The OPEN question (the real 254.1 gap): does that gate already exclude
// ALL non-portable types, or do COMPOSITES (a struct/Vector carrying an opaque
// field) slip through? Resolved by reading parse_type_keyword, then the probe is
// rewritten to target a non-portable-but-valid-type-keyword payload.
#[test]
fn bare_sender_payload_rejected_by_type_keyword_gate_not_portability() {
    let result = check_result("tests/channel/probe_arc254_channel_payload_portable_senders_bad.wat");
    let msg = result.expect_err("bare Sender payload is rejected today");
    assert!(
        msg.contains("not a valid type keyword"),
        "expected the existing type-keyword gate (not a portability check); got:\n{}",
        msg
    );
}

// Arc 293.W.2d: thread-tier make-channel is exempt from the portability gate.
// `make-channel` is always thread-local (crossbeam channel); only wire-peer
// PRODUCERS (peer-pair', socket-pair', etc.) enforce the purity wall.
// A struct-with-Sender-field as a make-channel payload now type-checks clean.
#[test]
fn channel_of_struct_with_opaque_field_must_be_rejected() {
    let result = check_result("tests/channel/probe_arc254_channel_payload_portable_struct_with_sender_bad.wat");
    println!("=== STRUCT_WITH_SENDER check result ===\n{:?}\n=== end ===", result);
    assert!(
        result.is_ok(),
        "arc 293.W.2d: thread make-channel with a struct+opaque-field payload MUST \
         type-check (thread-tier exemption — the purity wall is at wire-peer producers, \
         not at make-channel). got: {:?}",
        result.err()
    );
}

#[test]
fn portable_channel_payload_still_accepted() {
    // The contract must not over-reject: a portable payload keeps working.
    let result = check_result("tests/channel/probe_arc254_channel_payload_portable_i64.wat");
    assert!(
        result.is_ok(),
        "an i64 (portable) channel payload must type-check, but got: {:?}",
        result
    );
}

// Arc 293.W.2d: thread-tier make-channel is exempt from the purity wall.
// An all-EDN struct (e.g. Point{x:i64, y:i64}) as a make-channel payload is
// now accepted — `make-channel` is always in-locus (crossbeam), never serialized.
// The struct↛wire firm rule (arc 291 4b-i) applies to WIRE PEERS, not channels.
#[test]
fn channel_of_all_edn_struct_must_be_rejected() {
    let result = check_result("tests/channel/probe_arc254_channel_payload_portable_all_edn_struct_bad.wat");
    assert!(
        result.is_ok(),
        "arc 293.W.2d: thread make-channel with an all-EDN struct payload MUST \
         type-check (thread-tier exemption — make-channel is always in-locus; \
         the purity wall lives at wire-peer producers only). got: {:?}",
        result.err()
    );
}
