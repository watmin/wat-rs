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

use std::sync::Arc;
use wat::freeze::{startup_from_source, StartupError};
use wat::load::InMemoryLoader;

fn check_result(src: &str) -> Result<(), String> {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => Ok(()),
        Err(StartupError::Check(errs)) => Err(format!("{}", errs)),
        Err(other) => Err(format!("{}", other)),
    }
}

// A channel whose payload type is itself a Sender — an opaque, non-portable
// resource. Under the uniform serializable/portable contract this must be
// rejected at check time. (`d1 tx`/`d2 rx` bind-uses so neither half is unused.)
const CHANNEL_OF_SENDERS: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :wat::kernel::Sender<:wat::core::i64>)
                    d1 tx
                    d2 rx]
    nil))
"#;

// THE REAL GAP: a struct whose field is an opaque `Sender` is non-portable, but
// its NAME (`:my::Capsule`) parses as a valid type keyword — so it sails past the
// parse gate, and nothing checks the field's portability. This channel payload
// must be rejected at check time (254.1), but type-checks CLEAN at HEAD.
const CHANNEL_OF_STRUCT_WITH_SENDER: &str = r#"
(:wat::core::defstruct :my::Capsule [snd <- :wat::kernel::Sender<wat::core::i64>])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :my::Capsule)
                    d1 tx
                    d2 rx]
    nil))
"#;

// Control: an i64-payload channel is portable and MUST keep type-checking.
const CHANNEL_OF_I64: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :wat::core::i64)
                    d1 tx
                    d2 rx]
    nil))
"#;

// FINDING (2026-06-06, de-risk): a BARE `Sender<T>` payload is already rejected
// — but by the `parse_type_keyword` "valid type keyword" gate (check.rs:12779),
// NOT a portability check. So 254.1 is not greenfield; a payload-type gate
// exists. The OPEN question (the real 254.1 gap): does that gate already exclude
// ALL non-portable types, or do COMPOSITES (a struct/Vector carrying an opaque
// field) slip through? Resolved by reading parse_type_keyword, then the probe is
// rewritten to target a non-portable-but-valid-type-keyword payload.
#[test]
fn bare_sender_payload_rejected_by_type_keyword_gate_not_portability() {
    let result = check_result(CHANNEL_OF_SENDERS);
    let msg = result.expect_err("bare Sender payload is rejected today");
    assert!(
        msg.contains("not a valid type keyword"),
        "expected the existing type-keyword gate (not a portability check); got:\n{}",
        msg
    );
}

// FM-2-bis DISCONFIRMING probe (RED at HEAD): the composite gap. At HEAD this
// type-checks clean (gap open); after 254.1's is_portable_type gate it is
// rejected. Run un-ignored to confirm RED, then keep #[ignore]'d (baseline green)
// until 254.1 lands and un-ignores it.
#[test]
fn channel_of_struct_with_opaque_field_must_be_rejected() {
    let result = check_result(CHANNEL_OF_STRUCT_WITH_SENDER);
    println!("=== STRUCT_WITH_SENDER check result ===\n{:?}\n=== end ===", result);
    assert!(
        result.is_err(),
        "a struct-with-Sender-field channel payload type-checked CLEAN — the composite \
         portability gap is open (this is the RED-at-HEAD disconfirming probe for 254.1)"
    );
}

#[test]
fn portable_channel_payload_still_accepted() {
    // The contract must not over-reject: a portable payload keeps working.
    let result = check_result(CHANNEL_OF_I64);
    assert!(
        result.is_ok(),
        "an i64 (portable) channel payload must type-check, but got: {:?}",
        result
    );
}

// ── arc 291 strike-4b-i: the FIRM rule — struct ↛ wire, categorically ──────────
// An ALL-EDN struct (no opaque field) as a channel payload. At HEAD this is
// ACCEPTED — is_portable_type recurses the fields, finds them all portable, and
// lets the struct cross (254.1's "mirror the encoder" behaviour). The firm rule
// (arc 291 4b) makes a struct non-portable BY KIND: a struct shall never cross the
// wire; if you want that, you want a record. RED at HEAD; GREEN once
// is_portable_type(Struct) → false.
const CHANNEL_OF_ALL_EDN_STRUCT: &str = r#"
(:wat::core::defstruct :my::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :my::Point)
                    d1 tx
                    d2 rx]
    nil))
"#;

#[test]
fn channel_of_all_edn_struct_must_be_rejected() {
    let result = check_result(CHANNEL_OF_ALL_EDN_STRUCT);
    assert!(
        result.is_err(),
        "an ALL-EDN struct channel payload type-checked CLEAN — the firm struct↛wire \
         rule (arc 291 4b-i) is not yet enforced; a struct must be non-portable BY KIND"
    );
}
