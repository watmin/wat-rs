//! Arc 254.0 — FM-2-bis disconfirming probe: ONE canonical channel constructor.
//!
//! THE CUT (Mini-TCP at depth 1 — arc 254 §contract; doctrine 2026-05-19):
//! there is exactly one channel and its depth is always 1. So the word
//! "bounded" and the capacity argument `N` are both noise. The whole channel
//! construction surface collapses to:
//!
//!     (:wat::kernel::make-channel :T)   — depth always 1, no capacity arg
//!
//! ANNIHILATED in the same swing:
//!   - `make-unbounded-channel`        — condemned graveyard (unbounded violates depth-1)
//!   - `make-bounded-channel :T N`     — the "bounded" name + the `N` knob (only N=1 ever used)
//!   - `make-bounded-queue` / `make-unbounded-queue` — PHANTOM verbs: type-check
//!     clean in check.rs but have NO runtime dispatch + ZERO call sites (tombstones
//!     that lie about being supported).
//!
//! Wrong-capacity becomes UNREPRESENTABLE (✅✅✅): there is no `N` argument to
//! get wrong, so there is no runtime gate and no checker gate to maintain.
//!
//! RED AT HEAD: `make-channel` does not exist (A fails); the four condemned verbs
//! all still resolve / type-check (B/C/D/E fail). GREEN AFTER 254.0: the cascade
//! mints `make-channel`, deletes the four condemned verbs, and these flip.
//!
//! Run un-ignored to confirm RED, then sonnet un-ignores after the cascade lands.

use wat::freeze::{startup_from_file, StartupError};

fn check_result(path: &str) -> Result<(), String> {
    match startup_from_file(path) {
        Ok(_) => Ok(()),
        Err(StartupError::Check(errs)) => Err(format!("{}", errs)),
        Err(other) => Err(format!("{}", other)),
    }
}

// A — MINT-CONFIRMER (GREEN after 254.0: make-channel is the one constructor).
#[test]
fn make_channel_is_the_one_constructor() {
    let result = check_result("tests/channel/probe_arc254_make_channel_i64.wat");
    assert!(
        result.is_ok(),
        "make-channel :T (the one canonical depth-1 constructor) must type-check; got: {:?}",
        result
    );
}

// B + C — CUT-CONFIRMERS for the channel verbs (GREEN after 254.0: condemned verbs gone).
#[test]
fn condemned_channel_verbs_are_annihilated() {
    let unbounded = check_result("tests/channel/probe_arc254_make_channel_unbounded.wat.bad");
    assert!(
        unbounded.is_err(),
        "make-unbounded-channel must no longer resolve (depth-1 doctrine); it type-checked clean"
    );
    let bounded_n = check_result("tests/channel/probe_arc254_make_channel_bounded_n.wat.bad");
    assert!(
        bounded_n.is_err(),
        "make-bounded-channel (name + N knob) must no longer resolve; it type-checked clean"
    );
}

// D + E — CUT-CONFIRMERS for the phantom queue verbs (GREEN after 254.0: phantoms now error).
#[test]
fn phantom_queue_verbs_are_annihilated() {
    let bounded_q = check_result("tests/channel/probe_arc254_make_channel_bounded_queue.wat.bad");
    assert!(
        bounded_q.is_err(),
        "make-bounded-queue (phantom: no runtime, no callers) must no longer type-check"
    );
    let unbounded_q = check_result("tests/channel/probe_arc254_make_channel_unbounded_queue.wat.bad");
    assert!(
        unbounded_q.is_err(),
        "make-unbounded-queue (phantom: no runtime, no callers) must no longer type-check"
    );
}
