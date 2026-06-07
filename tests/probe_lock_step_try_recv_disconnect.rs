//! Arc 253 lock-step probe — after the SOLE sender drops, `try_recv` MUST
//! return `None` (2-state contract; no panics, no Some values).
//!
//! Pre-arc-253, this probe was a ROOT DISCRIMINATOR for the timing race:
//!   - Old `Empty` returned (no EMFILE) => poll / io_uring read-race in try_recv.
//!   - `pair()` fails with EMFILE        => write-fd leak (fd exhaustion).
//!
//! Post-arc-253: `try_recv` returns `Option<T>`. Both old `Empty` and
//! `Disconnected` map to `None`. The race is structurally unrepresentable —
//! the two outcomes collapsed to one. This probe verifies no panic and no
//! `Some` value appears in 50_000 drop-then-try_recv iterations, and
//! guards the EMFILE/pair-failure class.
//!
//! Run: cargo test --release -p wat --test probe_lock_step_try_recv_disconnect -- --nocapture
//! GREEN: no Some(..), no pair() failures. (Always green post-arc-253.)

use wat::comms::process::pair;

#[test]
fn try_recv_none_after_sole_sender_drop_under_load() {
    const ITERS: usize = 50_000;
    let mut unexpected_some = 0usize;
    let mut pair_failures = 0usize;
    let mut first_some: Option<usize> = None;
    let mut first_pairfail: Option<(usize, String)> = None;

    for i in 0..ITERS {
        let (tx, rx) = match pair::<String>() {
            Ok(p) => p,
            Err(e) => {
                pair_failures += 1;
                if first_pairfail.is_none() {
                    first_pairfail = Some((i, format!("{e:?}")));
                }
                break; // fd exhaustion is the signal; stop accumulating
            }
        };
        drop(tx); // sole sender closes -> read-end POLLHUP -> None (2-state)
        match rx.try_recv() {
            None => {} // correct — the only valid outcome post-arc-253
            Some(_) => {
                // A value appeared with no send — impossible; indicates a
                // different bug (stale accumulator, pipe shared wrongly, etc.)
                unexpected_some += 1;
                if first_some.is_none() {
                    first_some = Some(i);
                }
            }
        }
    }

    assert!(
        unexpected_some == 0 && pair_failures == 0,
        "ARC-253 INVARIANT VIOLATED over {ITERS} iters: \
         {unexpected_some} Some-instead-of-None (first @ {first_some:?}); \
         {pair_failures} pair() failures (first @ {first_pairfail:?}). \
         ROOT: Some>0 => stale accumulator or shared pipe bug; EMFILE pair-fail => write-fd leak."
    );
}
