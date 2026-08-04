# BRIEF — P4: the `libc::raise` cascade tests become wat deftests

**This is P4 of `DESIGN-STONE-process-signal-owner-to-child.md` (strike order, line 381).** It was
blocked by P3; P3 shipped. Do not re-derive the design — the stone has it, and the shape is already
working on disk.

**Floor:** `4344 tests run: 4344 passed, 262 skipped` (my own `--release` re-run at `824feb14`).

## The work

Two Rust tests self-signal their own process to prove a blocked recv wakes when the substrate stops:

```
tests/process/shutdown_cascade_memory.rs   libc::raise(SIGTERM) :122   (thread-tier / crossbeam)
tests/process/shutdown_cascade_pipefd.rs   libc::raise(SIGTERM) :129   (process-tier / fd poll)
```

A thread is not a process. Both reach below the peer layer to poke `ReceiverInner` and `typed_recv`
directly, and both justify the self-signal in their own headers with *"nextest already gives every
test its own process"* — a runner configuration, not a wall.

**Replace them with wat deftests that spawn a child in a process locus, signal the CHILD, and assert
on what the child reports.** Then `git rm` the Rust files.

## Read in order

1. **`wat-tests/process/signal-user1-delivers-child-observes.wat`** — THE EXEMPLAR. Copy its shape
   exactly: `deftest` → `spawn-peer (:wat::spawn::process)` → `(:wat::kernel::signal child …)` →
   `send` to unblock the child → `recv` the child's own report → `assert-eq`. Every outcome faced;
   note it asserts on the CHILD's observation, never on the parent's `Delivered` alone — its header
   says why, and that reasoning carries here unchanged.
2. Its two siblings in `wat-tests/process/` for variations.
3. The two condemned files above — for **what they assert**, not how.

## The signal

`:wat::kernel::Signal::Terminate` (`types.rs:1807`). The stone's line 403: *"P4 exercises `Terminate`
end-to-end via the cascade tests."*

## What each test must still prove

The property, unchanged: **a blocked recv returns when the substrate stops** — it does not hang, and
it does not report a clean close.

Today a stop reaches wat as `RecvOutcome::Lost[cause]` carrying `LociDiedError::Stopped`. Assert that
structurally. (A `RecvOutcome::Stopped` variant is a separate strike; do not wait for it and do not
mint it here.)

## ⛔ STOP-1 — the thread tier may not be reachable, and there is same-day evidence

Do the **process-tier** file first. It is the one proven to wake.

For the **thread tier**: inside the child, a `:user::main` blocked on a thread-peer `recv` may never
wake, because `comms/thread.rs:200` selects against `shutdown_rx()`, which fires only when
`SHUTDOWN_TX` drops — and `runtime.rs:496` defers `trigger_shutdown()` until after `:user::main`
returns whenever `stdio_bootstrapped()` is true, which it is in a real child.

A probe hit exactly this today (`wat-scripts/scratch-pad/arc278-shutdown-cohort-probe-thread.wat`) —
the child stayed alive through SIGTERM and never reported.

**If the child cannot report the thread-tier outcome, STOP and say so.** Do not restructure the child
to dodge it, do not weaken the assertion, do not invent a third shape. Report which structures you
tried and what each did. That is a substrate finding and it is the orchestrator's to route.

## ⛔ Other STOPs

- **Do not add a `libc::raise` anywhere.** The headline gate is `grep -rn 'libc::raise' tests/` → the
  only surviving hit may be the doc comment in
  `tests/process/signal_kill_produces_close_outcome_signaled.wat`, whose `SUPERSEDED-BY: P4` note you
  should update to say P4 landed.
- **Delete the `<100ms` assertions** (`shutdown_cascade_memory.rs:139`, `shutdown_cascade_pipefd.rs:146`).
  Nothing derives 100; it asserts performance while the subject is correctness. "Did it hang" is
  nextest's slow-timeout.
- **Do not weaken an assertion to make a migration pass.** A wat test that proves less than the Rust
  one did is a finding to report, not a result to ship.
- **New wat tests go in `wat-tests/process/`**, beside the P3 exemplars.

## ★ THE DELIBERATE BREAK — one per test you land

Break the mechanism the test names (the poll arm, the select arm), confirm the new wat deftest goes
**RED**, restore byte-exact, confirm green. Report both with real output. A migrated test that passes
whether or not the mechanism works is worse than the one it replaced.

## Done means

- Each landed test is a wat deftest under `wat-tests/process/`, signalling a real child.
- Its Rust predecessor is `git rm`'d.
- One deliberate break per landed test, RED output shown, restored byte-exact (`git diff src/` empty).
- `cargo nextest run --release` **Summary verbatim** against `4344/4344/0/262`, any delta explained
  (a migrated test may move between binaries; say so).
- `cargo clippy --release --all-targets` clean.
- Every STOP hit named; if none, say so.

Run every verification in the FOREGROUND and block on it — your turn ends when the numbers are in
your hands, not when the command is launched. Do not commit, push, or stash.

If STOP-1 fires, landing the process-tier file alone is a complete result. Say plainly what is left.
