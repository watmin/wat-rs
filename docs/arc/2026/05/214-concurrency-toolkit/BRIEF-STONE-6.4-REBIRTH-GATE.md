# BRIEF — Stone 6.4: THE REBIRTH GATE (the fork-zombie shutdown infra dies)

> The class (live-diagnosed via gdb, 2026-06-07): OnceLock'd global
> infrastructure with an attendant worker thread does not survive fork —
> the state survives, the thread doesn't, and the idempotence guard turns
> rebirth into a lie. The fix has two layers: (1) THE GUARD CAN NO LONGER
> LIE (pid-aware), (2) every fork child passes THE REBIRTH GATE (the
> canonical post-fork sequence rebuilds attendant-bearing globals).
> fork+exec is the banked top-rung arc — NOT this stone.

## Required reading (in order)
1. `tests/comms/shutdown_cascade_memory.rs` — whole, INCLUDING the
   `#[ignore]` attestation (the full diagnosis lives there).
2. `src/runtime.rs:213-~330` — init_shutdown_signal_with_inputs + the
   statics (SHUTDOWN_RX OnceLock, SHUTDOWN_TX_PTR AtomicPtr, the wake pipe,
   the worker spawn). The TX already uses the swappable AtomicPtr pattern —
   you are extending that pattern to the RX + adding a pid.
3. `src/process/child.rs` — run_in_fork (bare child body, ~:113) +
   child_post_fork_init_preserving (~:247, the canonical 5-step).
4. `src/comms/mod.rs:28-31` + wherever comms WIRES the shutdown receiver
   into its select (scout `src/comms/thread.rs`) — you must verify the
   wiring reads the rx at a point that sees the post-rebirth swap for
   CHILD-CREATED channels (channels created post-fork must select the NEW
   rx). If the wiring caches the OLD rx in a way a getter-swap cannot fix
   for child-created channels: STOP-1, report the exact shape.
5. `src/channel/transfer.rs:175` + `:263` area — the `.get()` consumers.

## The work

### 1. The pid-aware guard (runtime.rs)
- `SHUTDOWN_RX: OnceLock<Receiver<()>>` → the TX's pattern:
  `static SHUTDOWN_RX_PTR: AtomicPtr<crossbeam_channel::Receiver<()>>` +
  `pub(crate) fn shutdown_rx() -> Option<&'static crossbeam_channel::Receiver<()>>`
  (load; null → None). Sweep every `.get()` consumer to the getter
  (transfer.rs sites, the comms wiring site, any freeze use).
- `static SHUTDOWN_INIT_PID: AtomicI32 = AtomicI32::new(0)`.
- `init_shutdown_signal_with_inputs` guard becomes:
  initialized AND `SHUTDOWN_INIT_PID == getpid()` → no-op;
  otherwise REBUILD: fresh channel + wake pipe + worker thread; swap
  RX ptr + TX ptr (Box::into_raw; the OLD boxes/copies LEAK BY DESIGN —
  they are the fork-child's inherited process-local copies; one-line
  comment says so); close the OLD inherited wake write fd if present
  (the new fd is stored BEFORE handler installation in the child sequence
  — note the ordering in a comment); store the new wake fd; store getpid().
  The doc on the fn rewrites: "Idempotent within a process; FORK-AWARE
  across them — a clone3 child's first call rebuilds (the inherited worker
  thread does not exist; the inherited state is a zombie). The 2026-06-07
  live diagnosis is the WHY (SCORE-STONE-6.3 § the live catch)."

### 2. The gate (src/process/child.rs)
- `pub(crate) fn rebirth_substrate_after_fork()` — THE REBIRTH GATE.
  Doc-contract (this is load-bearing prose — write it carefully):
  * Every substrate global with an ATTENDANT (worker thread, fd reader,
    held lock) must rebirth here — the state forks, the attendant doesn't.
  * Current inventory: the shutdown infra (this stone). The service trio
    needs no entry — children boot their own universe (bootstrap, 8.3).
  * The pre-gate region (clone3-return → this call) is constrained to
    async-signal-safe operations.
  * The top rung — fork+exec, a fresh address space — is the banked arc
    the 214 INSCRIPTION cites.
  Body: call `crate::runtime::init_shutdown_signal()` (the pid-aware guard
  does the work).
- Call it from: (a) `child_post_fork_init_preserving` — FIRST step, before
  signal-handler installation (the handler must see the new wake fd);
  (b) `run_in_fork`'s child branch, first line of the child body wrapper.

### 3. The detectors come alive
- REMOVE both `#[ignore]` attributes (tests/comms/shutdown_cascade_memory.rs
  + shutdown_cascade_pipefd.rs). Their manual `init_shutdown_signal()` calls
  are now harmless (pid-aware: the rebirth already ran via run_in_fork; the
  manual call no-ops correctly).

## Gates
1. **THE GATE**: `setsid timeout 240 cargo test --release --test comms --
   --test-threads=1` → completes (no timeout) with **52 passed / 0 failed /
   6 ignored** — both detectors GREEN IN-SUITE (the zombie path exercised:
   earlier tests init in the parent, the children rebirth). RED today
   (timeout 124).
2. Each detector also green ALONE:
   `setsid timeout 120 cargo test --release --test comms shutdown_cascade -- --test-threads=1 --include-ignored`
   (run BEFORE removing ignores to sanity-probe, then plain after).
3. `cargo test --release --lib -p wat` → 943/0/1.
4. `cargo test --release --test nursery` → 865/4/4.
5. Enveloped: channel_pipes 23/0 · gamma 5/0 · hermetic 2/0.
6. `cargo check --all-targets` 0 · `cargo clippy --release --lib -p wat`
   no new findings.

## STOP triggers (rejection criteria)
- STOP-1: the comms shutdown wiring caches the rx such that the getter-swap
  cannot serve child-created channels. Report the wiring shape verbatim.
- STOP-2: any enveloped run times out (the timeout IS your diagnostic —
  capture which test, report; ship nothing).
- STOP-3: the OnceLock→AtomicPtr sweep finds a consumer whose semantics
  genuinely depended on OnceLock (set-once races). Report it.

## Constraints
- Work only in /home/watmin/work/holon/wat-rs/. Commit NOTHING.
- Every process-tier run ENVELOPED (the 6.3 flight's envelope violation is
  the cautionary tale — the bare run converted a 120s diagnostic into an
  eternal hang).
