//! Arc 253 reproduce probe (FM-2-bis disconfirming) — the lock-step invariant:
//! after the SOLE sender drops, `try_recv` MUST report `Disconnected` (never
//! `Empty`). The flake caught under coverage instrumentation (comms `try_recv`'s
//! `poll(timeout=0)` intermittently returned `Empty`) violates this.
//!
//! This probe doubles as a ROOT DISCRIMINATOR:
//!   - `Empty` returned (no EMFILE)  => poll / io_uring read-race in try_recv.
//!   - `pair()` fails with EMFILE     => the write-fd LEAKS at drop (fd exhaustion);
//!     POLLHUP can never fire while a leaked write-end holds the pipe open = the
//!     suppressed-signal half of the lock-step class (the arc-170 orphan-leak root).
//!
//! Run: cargo test --release -p wat --test probe_lock_step_try_recv_disconnect -- --nocapture
//! RED at HEAD if the class is live; GREEN once arc 253 annihilates it.

use wat::comms::process::pair;
use wat::comms::TryRecvError;

#[test]
fn try_recv_disconnected_after_sole_sender_drop_under_load() {
    const ITERS: usize = 50_000;
    let mut empties = 0usize;
    let mut pair_failures = 0usize;
    let mut first_empty: Option<usize> = None;
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
        drop(tx); // sole sender closes -> read-end POLLHUP -> Disconnected (lock-step)
        match rx.try_recv() {
            Err(TryRecvError::Disconnected) => {} // correct
            Err(TryRecvError::Empty) => {
                empties += 1;
                if first_empty.is_none() {
                    first_empty = Some(i);
                }
            }
            other => panic!("iter {i}: unexpected try_recv result {other:?}"),
        }
    }

    assert!(
        empties == 0 && pair_failures == 0,
        "LOCK-STEP VIOLATION reproduced over {ITERS} iters: \
         {empties} Empty-instead-of-Disconnected (first @ {first_empty:?}); \
         {pair_failures} pair() failures (first @ {first_pairfail:?}). \
         ROOT: Empty>0 => poll/io_uring read-race; EMFILE pair-fail => write-fd leak."
    );
}
