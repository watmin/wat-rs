//! Arc 214 — `1b-ii-α` FM-2-bis probe: the crash reason arrives THROUGH the
//! io_uring receiver (the Err arm), so `recv'` auto-raises it.
//!
//! # The dogfood claim under test
//!
//! The builder's north star: io_uring is the substrate's ONLY io-select loop,
//! and the 3-fd cross-process IPC (`in` / `Ok` / `Err`) in a cap-4 ring is the
//! proving point that dogfoods the autoscaling TCO loop. Stone `1b-ii-α` folds
//! the `Err` channel (the child's fd 2 — today a SEPARATE plain `libc::pipe`
//! drained by `ProcessPeerBundle::take_crash_reason`) into the io_uring receiver
//! as a 3rd `POLL_ADD` arm. Once it is an arm of the ring, a crashed child's
//! `#wat.kernel/ProcessPanics` reason arrives through `recv` itself — so the
//! `recv'` verb AUTO-RAISES it (closing Q1: the substrate raises on the user's
//! behalf; no user-facing crash verb, no second `take_crash_reason` call).
//!
//! # Why this is RED at HEAD (the isolated gap)
//!
//! At HEAD the `Err` channel is NOT an io_uring arm: a crashed child closes its
//! stdout (fd 1), the comms `Receiver` sees EOF, and `bundle.peer.recv()`
//! returns a BARE `RecvError` (no reason). `eval_peer_recv_prime` maps that to a
//! generic `MalformedForm { reason: "recv failed: process channel disconnected" }`.
//! The actual cause (`DivisionByZero`) is only reachable via the separate
//! plain-pipe `take_crash_reason` — NOT through `recv'`. So the load-bearing
//! assertion below (the raised error names the cause) FAILS at HEAD on exactly
//! the gap, with everything around it (spawn, send', the crash, the disconnect)
//! already working. After `1b-ii-α` it goes GREEN.
//!
//! Companion HEAD-behavior tests (the same crash, read via `take_crash_reason`):
//! `spawn_program_prime_process_runtime_error_emits_diagnostic` (already green).
//!
//! # Containment
//!
//! Forks a `:process` child — run under setsid + timeout, single-threaded:
//!   setsid timeout 180 cargo test --release --test kernel \
//!     probe_arc214_alpha_crash_autoraise -- --ignored --test-threads=1


/// FM-2-bis (α): `recv'` on a crashed `:process` child must auto-raise the
/// child's crash reason — proving the `Err` channel arrived through the io_uring
/// receiver, not a separate plain-pipe drain.
///
/// `#[ignore]` — process-tier probe; run under setsid + timeout, `--test-threads=1`.
#[test]
#[ignore = "process-tier FM-2-bis probe (arc 214 1b-ii-α): run via setsid timeout 180 cargo test --release --test kernel probe_arc214_alpha_crash_autoraise -- --ignored --test-threads=1"]
fn alpha_recv_prime_autoraises_child_crash_reason() {
    unimplemented!("arc 214 1b-ii-α: io_uring Err arm carries crash reason; on unlock assert_eq! the exact DivisionByZero crash EDN");
}
