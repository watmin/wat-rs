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
//! Uses `make-bounded-channel ... 1` (the surviving DEPTH-1 form; unbounded +
//! bounded(N) are condemned per the Mini-TCP doctrine — arc 254 §contract).

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
  (:wat::core::let [[tx rx] (:wat::kernel::make-bounded-channel :wat::kernel::Sender<:wat::core::i64> 1)
                    d1 tx
                    d2 rx]
    nil))
"#;

// Control: an i64-payload channel is portable and MUST keep type-checking.
const CHANNEL_OF_I64: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-bounded-channel :wat::core::i64 1)
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
