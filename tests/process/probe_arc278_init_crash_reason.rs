//! Arc 278 — `:init` (startup) crash reason parity, BOTH loci.
//!
//! THE FLAW: a `defservice` whose `:init` crashes MASKS its reason — unlike a serve-loop crash
//! (honest). The address / `Status::Started` was made available to the owner BEFORE `:init` ran,
//! so `/start` completed before the crash and the failure only surfaced at the owner's independent
//! dial, disconnected from any reason channel:
//!   - THREAD: address parent-minted, `/start` returned immediately; `:init` crashed before the
//!     serve loop's `accept'`, so the owner's `connect'` rendezvous DEADLOCKED forever.
//!   - PROCESS: the child sent `Status::Started` before running `:init`; `/start` succeeded, then
//!     `:init` crashed and the owner's later `connect'` got a bare ECONNREFUSED with the reason
//!     discarded to the child's stderr.
//!
//! THE FIX (arc 278): run `:init` BEFORE `Status::Started` is sent, and wait for Started on the
//! crash-aware lineage `recv'`, on BOTH tiers. An `:init` crash then dies before Started → the
//! launch handshake's `recv'` raises the reason (`Crashed(reason)`), exactly like the honest
//! serve-loop-crash path. `/start` fails FAST carrying the reason instead of deadlocking/discarding.
//!
//! RED at HEAD: PROCESS → bare ECONNREFUSED, no sentinel; THREAD → hangs forever.
//! GREEN: both raise fast with the reason carrying `BOOM-INIT-SENTINEL-99`.
//!
//! Run SERIALLY (spawns threads/processes):
//!   `cargo test --release -p wat --test process init_crash_reason -- --test-threads=1`

use std::sync::mpsc;
use std::time::Duration;
use wat::freeze::call_beside;

const SENTINEL: &str = "BOOM-INIT-SENTINEL-99";

/// PROCESS locus (PRIMARY assertion). The owner starts a crashing-`:init` service on `(process)`
/// and dials it; the call MUST RAISE with the reason carrying the sentinel. No hang at HEAD (bare
/// ECONNREFUSED), so this runs unbounded.
#[test]
fn process_init_crash_surfaces_reason_to_owner() {
    match call_beside(file!(), ":user::compute-process") {
        Ok(v) => panic!(
            "expected :user::compute-process to RAISE (the :init crashed); got Ok({v:?})"
        ),
        Err(e) => {
            let text = format!("{e:?}");
            assert!(
                // rune:lint(loose-assert) — the raised text embeds machine-specific absolute
                // paths (startup_beside/file!()); the sentinel substring is the stable signal.
                text.contains(SENTINEL),
                "process :init crash must surface its reason to the owner (fail /start fast with \
                 the reason), not collapse to a bare ECONNREFUSED. got: {text}"
            );
        }
    }
}

/// THREAD locus. Prove the DEADLOCK is GONE: run the owner's call on a worker thread and wait on a
/// bounded (10s) join. GREEN → it RETURNS (raising with the sentinel) well within the bound; the
/// pre-fix HEAD hung forever → the timeout arm fires and fails the test. The 10s bound is far above
/// the sub-second raise path, so a GREEN run never races it.
#[test]
fn thread_init_crash_does_not_deadlock_and_surfaces_reason() {
    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    let _worker = std::thread::spawn(move || {
        // Stringify inside the worker so only a String crosses the channel (no Send bounds on
        // Value/RuntimeError to reason about).
        let verdict = match call_beside(file!(), ":user::compute") {
            Ok(v) => Err(format!(
                "expected :user::compute to RAISE (the :init crashed); got Ok({v:?})"
            )),
            Err(e) => Ok(format!("{e:?}")),
        };
        let _ = tx.send(verdict);
    });

    match rx.recv_timeout(Duration::from_secs(10)) {
        Err(mpsc::RecvTimeoutError::Timeout) => panic!(
            "THREAD :init crash DEADLOCKED — the owner's call did not return within 10s (the \
             pre-fix hang: a bound-but-never-accepted address). The fix must make /start raise fast."
        ),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("worker thread died without producing a verdict")
        }
        Ok(Err(msg)) => panic!("{msg}"),
        Ok(Ok(text)) => assert!(
            // rune:lint(loose-assert) — see the process test: machine-specific paths in the text.
            text.contains(SENTINEL),
            "thread :init crash must surface its reason (no deadlock, fail fast with the reason). \
             got: {text}"
        ),
    }
}
