# Arc 253 — Lock-step or it races (the hidden-concurrency class)

**Status:** OPEN 2026-06-06 (STUB). Name PROVISIONAL — intueri-cast at design.

**Origin (builder, 2026-06-06):** *"all of wat must be lock-step — we found us not doing
that via tests — this kind of problem has been hiding from us."* Surfaced when the arc-252
coverage gate ran the test surface under llvm-cov instrumentation: the slower timing flushed
a race that the normal (fast) test runs never hit.

## The principle

**Every wait in wat must be lock-step: it arrives via the wire (a blocking `poll(2)`/fd-event),
never via a guess (a `timeout=0` snapshot, a `sleep`, a spin).** And lock-step has TWO halves,
joined by POLLHUP:
- **The waiter** blocks on the wire (`poll(timeout=-1)`). ✓ The lifeline/shutdown worker does
  this correctly (`src/runtime.rs:287-316`; its own doc: *"poll(2) over pipe FDs is the Linux
  primitive that gives lock-step"*).
- **The signal must actually fire.** `POLLHUP` fires only when EVERY write-end of a pipe is
  closed. If one write-fd leaks (hand-managed `into_raw_fd` + a missed manual close, a dup not
  dropped, a fork-inherited fd), POLLHUP is suppressed — and the perfectly-correct lock-step
  waiter blocks forever. The waiting half is useless if the signalling half leaks.

This is `mora`'s domain ("suffers no mora — every wait must arrive via the wire; sleep is a
guess; guesses race") + `conformare`'s ("the wrong shape must be uncompilable") — lock-step must
be STRUCTURAL (RAII + blocking poll), not hand-discipline.

## The two instances found (one class, not proven the same bug)

1. **Non-lock-step waiter — `try_recv` `poll(timeout=0)`** (`src/comms/process.rs:354`,
   `src/typed_channel.rs:400`). A non-blocking snapshot is a legitimate contract for "is data
   ready NOW" (Empty is a valid answer) — BUT its disconnect detection must be deterministic, and
   it mixes `poll(timeout=0)` with io_uring reads (process.rs uses io_uring for the wake). THE
   FLAKE: `tests/comms/process.rs:153` `probe_slice3d1_try_recv_disconnected_after_sender_drop`
   intermittently returns `Empty` instead of `Disconnected` after a same-thread `drop(tx)` — the
   poll didn't see POLLHUP. Caught only under llvm-cov instrumentation.

2. **fd-lifecycle by hand — `into_raw_fd` + manual close + `mem::forget`** (`src/spawn_process.rs`
   ~158-241, 381; `src/fork.rs`). `into_raw_fd()` disables `OwnedFd`'s RAII Drop → every branch
   must manually close every fd or it leaks. A leaked write-end suppresses POLLHUP → a child's
   lock-step lifeline `poll(-1)` never wakes → orphan → THE arc-170 PROCESS LEAK that the whole
   setsid+pkill containment apparatus exists to work around. (Hypothesis — the leaked-fd root is
   not yet reproduced; the hand-lifecycle is the leak-prone surface.)

## Why it hid

`poll(timeout=0)` masks the bug as a wrong-but-not-hung `Empty` (no crash, just occasionally
wrong); the hand-fd-close fails only on the branch that misses; both are timing-rare. Fast test
runs never lose the race. The arc-252 coverage gate (instrumented = slow) was the first thing to
apply enough timing pressure to flush it. **The test surface earning its keep.**

## The hunt (the arc)

1. **Audit non-lock-step waiters** across wat (`poll(timeout=0)`, any `sleep`/spin/timed guess in
   a readiness path). Cast `mora`. Each: make it lock-step (blocking poll), OR prove the
   non-blocking snapshot is contract-correct AND its disconnect detection is deterministic.
2. **Make fd-lifecycle structural (RAII).** Eliminate `into_raw_fd` + hand-close where possible;
   the fork/spawn fds should be owned by `OwnedFd` whose Drop closes them, with the child's
   post-fork `close_inherited_fds_above_stdio` the only deliberate exception. The wrong shape (a
   write-fd that can outlive its intended close) should be uncompilable.
3. **Reproduce the flake** deterministically (run the comms try_recv test under load/instrumentation
   in a loop) to confirm the io_uring+poll(timeout=0) mechanism, then fix at the root.
4. **Connect to the leak:** once fd-lifecycle is RAII + POLLHUP is guaranteed, re-test whether the
   arc-170 `#[ignore]`'d process tests can be un-ignored (the containment workaround retired at root).

## DECISION 2026-06-06 — collapse try_recv to 2-state (four-questions: A beats B)

Grounding refined the target. The flake lives in the Rust-API `Receiver::try_recv`'s
3-state (`TryRecvError::{Empty, Disconnected}`). But:
- `RecvOutcome` (the verb path, typed_channel.rs:176) has **no Empty** — the wat verb
  `(:wat::kernel::try-recv)` maps both empty + disconnected to **`Ok(None)`** (by design;
  eval_kernel_try_recv:43 "Disconnected as a stand-in … matches the empty"). So wat
  programs already cannot distinguish them; the flake never reaches the wat surface.
- Nothing needs the 3rd state: `Select` uses its own io_uring `POLL_ADD` (not try_recv);
  the select-then-drain pattern (wat-telemetry) only needs "value? no → skip"; only a
  Rust test asserts the Empty/Disconnected split.

So `Empty` is the asymmetric, unused state where the flake hides. Per
[[feedback_asymmetries_meet_high_bar]] the bar to KEEP it isn't met → **eliminate it.**

**FOUR-QUESTIONS: (A) collapse to 2-state (Value / not-value) beats (B) make Empty honest
everywhere.** A wins Simple (removes a state vs adds one through the whole stack), Honest
*by construction* (no distinction to lie about; B's honesty is hostage to fixing the
unreproduced poll race), and it's the asymmetry kill. B adds complexity for a distinction
no caller uses, atop a race we can't yet make deterministic.

THE KILL (structural — deletes the flake's home, not patches it):
1. `Receiver::try_recv` → 2-state (`Value` / `NoValue`), matching `RecvOutcome` + the wat
   `Ok(Some)/Ok(None)` surface. No Empty vs Disconnected → no snapshot race to lose.
2. crossbeam path (typed_channel.rs:428) made consistent with the 2-state contract.
3. grow `mora` to hunt the snapshot-guess class (not just pauses) — close the blind spot.
4. update the 3-state tests to the 2-state contract.

NOTE: this kills INSTANCE 1 (the try_recv non-lock-step guess). INSTANCE 2 (spawn_process
`into_raw_fd` hand-fd-lifecycle = the arc-170 ORPHAN leak) is SEPARATE + still open — the
comms-pair fd-path proved clean (50k no-leak), so the orphan leak is a distinct spawn-path
investigation (reproduce orphans + RAII the fd lifecycle).

## FINDING 2026-06-06 (recolligere grounding) — the collapse was scoped to `comms/`; the χ chokepoint is still 3-state (benign)

Grounding the question "did we resolve 253?" surfaced that the breadcrumb's
"`TryRecvError` removed" is true for `src/comms/` but NOT codebase-wide:
- `src/typed_channel.rs:536` still `pub use crossbeam_channel::{… TryRecvError}`.
- `src/typed_channel.rs:591` — the arc-213-χ `Receiver<T>::try_recv` still returns
  the 3-state `Result<T, TryRecvError>`. This is a SEPARATE `Receiver` from
  `comms::Receiver` (the one collapsed in `d3150a04`); it is live —
  `value/value.rs:920` `InThread(Receiver<SpawnOutcome>)`.

**BENIGN — not a live instance of the racing class** (axis-2 test):
- its sole live caller is `runtime.rs:18491` `eval_handle_pool_pop`, which maps BOTH
  error arms to one outcome (`Err(_)` → "no handles left") — the Empty/Disconnected
  distinction is never consumed, so there is no racing two-way result;
- `HandlePool::pop` runs at WIRING time (claim the committed count BEFORE any thread
  runs) — single-threaded, not a concurrent snapshot-guess.

So instance 1's RACE is annihilated. What remains is an ASYMMETRY (two `Receiver`
types, two `try_recv` contracts) per [[feedback_asymmetries_meet_high_bar]]. 253's
eventual INSCRIPTION must either (a) collapse the χ `try_recv` to the 2-state
contract too, or (b) rune it with this "sole caller collapses both arms +
wiring-time non-concurrent" justification. Ledger item, non-blocking.

## Cross-references

- `tests/comms/process.rs:153` — the flaky probe (the disconfirming evidence).
- `src/runtime.rs:287-316` — the CORRECT lock-step model (poll timeout=-1).
- `src/comms/process.rs:360` (try_recv) + `src/spawn_process.rs` (into_raw_fd) — the two instances.
- arc 170 (process entry points; the leak + the containment apparatus); arc 252 (the coverage gate that flushed it).
- grimoire `mora` (lock-step-via-the-wire) + `conformare` (uncompilable wrong shape).
- `feedback_lock_step_via_pipe` (the discipline this arc enforces substrate-wide).
