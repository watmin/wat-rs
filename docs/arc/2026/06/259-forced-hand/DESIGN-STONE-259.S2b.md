# Stone 259.S2b — internal `close` via RAII Drop (thread tier)

> Substone of arc 259, spawn-redesign. Parent DESIGN: `DESIGN.md` § "THE CONVERGED
> MODEL" → "`close` is internal — RAII Drop + orchestration-explicit". Rides S2a.

## Why this stone

The converged model takes lifecycle out of the user's hands: the platform owns the
reap. The mechanism that does it **and** keeps the locked 2-arg sig is **RAII** — the
peer's `Drop` *is* the internal `close`: drain (drop the input Sender → the worker's
`recv'` raises via the cascade → the worker exits) **then** join. Hang-free by
construction (the worker can only block on cascade-aware `recv'`/`send'`, which raise
on disconnect). The user never holds the rope; a dropped peer reaps itself.

S2b is **additive**: the wat `close'` verb still works (it now routes through the same
internal reap), so the existing callers stay green. S2d later removes `close'` from the
user surface. S2b is **thread-tier only** — the `:process` reap (Pidfd `wait_status`,
exit code, lifeline) is its own lifecycle and is untouched here.

## The contract (pinned)

### The restructure

```rust
struct Thread<I,O> {
    input: Option<comms::thread::Sender<I>>,   // Option so Drop can take+drop it BEFORE join
    output: comms::thread::Receiver<O>,
    join: Option<std::thread::JoinHandle<()>>, // Option so Drop can take+join it idempotently
}

/// Drain then join — idempotent (Option::take). The ONE internal reap.
fn drain_and_join(&mut self) -> Option<std::thread::Result<()>> {
    drop(self.input.take());            // drain: worker's recv'/send' sees disconnect → raises → exits
    self.join.take().map(|j| j.join())  // join: synchronous; None if already reaped
}

impl Drop for Thread<I,O> {
    fn drop(&mut self) { let _ = self.drain_and_join(); }  // backstop; swallows (Drop can't propagate)
}
```

### The load-bearing invariant — drain BEFORE join

`drain_and_join` MUST drop the input Sender **before** calling `join`. If it joined
first, `join` would block waiting for the worker, but the worker is blocked on `recv'`
(input not yet dropped) → **deadlock**. Drain-before-join is the cascade-safety; it is
the exact order the arc-170 `ProcessJoinBeforeOutputDrain` walker already enforces for
the process tier.

### close' routes through the same reap (idempotent)

- **`close'` (wat verb, eval_peer_close_prime Thread' arm):** take the `Thread` from the
  cell Option → `thread.drain_and_join()` → if `Some(Err(_))` report a panic
  `RuntimeError`, else `nil`. The `Thread` then drops; its `Drop` calls `drain_and_join`
  again → `None` (already taken) → no-op. **Idempotent.**
- **Drop backstop (no `close'`):** the cell's Option still holds `Some(Thread)` → when the
  Arc/cell drops, `Thread::drop` reaps. The worker is reaped without any user call.

## The touches

1. **`src/kernel/peer.rs`** — `Thread<I,O>`: `input`/`join` → `Option`; add private
   `drain_and_join(&mut self)`; add `impl Drop`; **remove** the consuming
   `close(self) -> JoinHandle` and `join(self)` methods; update the `Debug` impl for the
   Option fields.
2. **`src/kernel/spawn.rs`** — `spawn_thread_peer`: construct
   `Thread { input: Some(input_tx), output: output_rx, join: Some(join_handle) }`. Update
   the in-module `spawn_thread_peer_echo_round_trip` test (it calls `peer.join()` — migrate
   to the new reap / `drop`). The S2b probe `s2b_drop_reaps_blocked_worker` flips
   RED→GREEN.
3. **`src/runtime.rs`** — `eval_peer_close_prime` Thread' arm: replace `peer.join()` with
   `thread.drain_and_join()` + panic reporting. (The `Process'` arm is untouched.)
4. **Caller sweep** — grep every caller of `Thread::close()` / `Thread::join()` (e.g.
   `select'` / `eval_peer_select_prime`, any kernel/test site) and migrate to
   `drain_and_join` (where the join result matters) or `drop` (where it does not).

## Verification — deterministic protocol (NOT disconfirm-at-HEAD)

S2b is a **synchronization** change; per the race-discipline (`feedback_race_fix_structural_not_reproduced`),
the structural fix (Drop joins → "unreaped worker" unrepresentable) is verified by a
**deterministic protocol test**, not a flaky race-repro:

- **`s2b_drop_reaps_blocked_worker`** (committed, RED at HEAD = `strong_count 4 vs baseline 2`):
  a self-peer worker blocks on `recv'`; dropping the peer without `close'` must drain→join,
  releasing the worker's captured `program_fn` clone → `strong_count` returns to baseline
  (deterministic, because join is synchronous). GREEN post-S2b.
- The existing `spawn_thread_peer_echo_round_trip` (migrated off `peer.join()`) proves the
  explicit-reap path still works.

## Out of scope — REJECTED, tracked downstream

- **Process-tier RAII** — `:process` keeps its `wait_status`/lifeline reap. (A sibling
  stone or folded later; not S2b.)
- **Removing user `close'`** — S2b keeps `close'` working (routed through the reap); S2d
  removes it from the user surface.
- **The parent → `Peer'` type unification** — the parent stays `Thread'` (now RAII-reaping);
  the full collapse of `Thread'`/`Process'` into `Peer'` + internal lifecycle is later.

## Done = green

- `cargo test --release -p wat --lib kernel::spawn` — `s2b_drop_reaps_blocked_worker` +
  `spawn_thread_peer_echo_round_trip` both pass.
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery probe_arc214_stone46aii_peer_verbs` +
  `probe_arc259_s2a` still green (close'/spawn unregressed).
- Full nursery SERIAL `--test-threads=1`: only the 4 known pre-existing reds.
