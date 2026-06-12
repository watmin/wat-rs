# BRIEF — Stone 259.S2b: internal `close` via RAII Drop (thread tier)

**The work, in one paragraph.** Give the thread-tier `Thread'` peer a custom `Drop` that
reaps its worker — **drain** (drop the input `Sender` so the worker's `recv'` raises and
the worker exits) **then join** — so a peer that simply leaves scope is reaped with no
explicit `close'`. Route the `close'` verb through the same internal reap (idempotent), so
it keeps working. The committed probe `s2b_drop_reaps_blocked_worker` flips RED→GREEN.

**Read in order (the rooms):**
1. `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S2b.md` — the pinned contract
   (the restructure, the drain-before-join invariant, idempotency).
2. `src/kernel/spawn.rs` (in-module test) `s2b_drop_reaps_blocked_worker` — the GREEN
   target (currently RED: `strong_count 4 vs baseline 2`). Make it pass.
3. `src/kernel/peer.rs:59-122` — `Thread<I,O>` struct + `close(self)`/`join(self)` methods
   + `Debug`. Restructure: `input: Option<Sender<I>>`, `join: Option<JoinHandle<()>>`
   (output unchanged); add the private `drain_and_join(&mut self) -> Option<thread::Result<()>>`;
   add `impl Drop` calling it (swallow); **remove** the consuming `close(self)`/`join(self)`;
   fix `Debug` for the Option fields.
4. `src/kernel/spawn.rs` — `spawn_thread_peer`: the `let peer = Thread { input: input_tx,
   output: output_rx, join: join_handle }` constructor → Some-wrap `input` + `join`. The
   in-module `spawn_thread_peer_echo_round_trip` test calls `peer.join()` — migrate it to
   the new reap (`drain_and_join` or `drop`).
5. `src/runtime.rs` — `eval_peer_close_prime` Thread' arm: replace `peer.join()` (the
   `Some(peer) => peer.join()...` site) with `thread.drain_and_join()` + panic reporting
   (`Some(Err(_))` → the existing "Thread peer join failed (thread panicked)" RuntimeError;
   else `nil`). Leave the `Process'` arm untouched.
6. **Caller sweep** — `grep -rn "\.close()\|\.join()" src/ | grep -i thread` and check
   `eval_peer_select_prime` (select') + any kernel/test site that consumes a `Thread` via
   `close()`/`join()`. Migrate each to `drain_and_join` (if the join result is used) or
   `drop` (if not).

**Implementation sketch (peer.rs — the heart):**
```rust
pub struct Thread<I: Send + 'static, O: Send + 'static> {
    pub(crate) input: Option<crate::comms::thread::Sender<I>>,
    pub(crate) output: crate::comms::thread::Receiver<O>,
    pub(crate) join: Option<std::thread::JoinHandle<()>>,
}
impl<I: Send + 'static, O: Send + 'static> Thread<I, O> {
    pub fn send(&self, value: I) -> Result<(), SendError<I>> {
        // input is now Option; the worker is alive while it is Some
        self.input.as_ref().map(|s| s.send(value)).unwrap_or(Ok(()))   // or an explicit closed-error
    }
    pub fn recv(&self) -> Result<O, RecvError> { self.output.recv() }
    /// Drain THEN join — idempotent. The ONE internal reap.
    pub(crate) fn drain_and_join(&mut self) -> Option<std::thread::Result<()>> {
        drop(self.input.take());            // drain FIRST: worker's recv' raises → worker exits
        self.join.take().map(|j| j.join())  // THEN join (synchronous); None if already reaped
    }
}
impl<I: Send + 'static, O: Send + 'static> Drop for Thread<I, O> {
    fn drop(&mut self) { let _ = self.drain_and_join(); }  // backstop; Drop cannot propagate
}
```

**Blast radius:** `src/kernel/peer.rs` (the `Thread` type), `src/kernel/spawn.rs`
(constructor + the echo test), `src/runtime.rs` (the `close'`/`select'` Thread' arms).
No new wat files; no parser/check changes; the `:process` tier and the `Peer'` worker
self-peer (S2a) are untouched.

**STOP triggers (halt + report; do not work around):**
- **STOP-1 (the deadlock trap):** `drain_and_join` MUST drop the input `Sender` BEFORE
  calling `join`. Joining first deadlocks (join waits for the worker; the worker waits for
  the input drop). If a code path would join-before-drain, STOP and report.
- **STOP-2 (double-join):** `drain_and_join` must be idempotent via `Option::take`, so
  `close'` (which calls it) followed by `Drop` (which calls it again) does NOT double-join
  or panic. If you cannot make both paths safe with one idempotent reap, STOP and report.
- **STOP-3:** if making the probe green requires touching the `:process` tier, the `Peer'`
  worker self-peer, the parser, or the type checker, STOP and report — S2b's scope is the
  thread-tier `Thread'` reap only.

**Done = green:**
- `cargo test --release -p wat --lib kernel::spawn` → `s2b_drop_reaps_blocked_worker` (now
  `strong_count == baseline`) + `spawn_thread_peer_echo_round_trip` both pass.
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery probe_arc214_stone46aii_peer_verbs` +
  `probe_arc259_s2a` still green.
- `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known
  pre-existing reds (arc-255 reflection ×2 + undefined-builtin ×2).

**Mirror for shape:** the existing `Thread::close`/`join` (peer.rs:95-110) is the logic you
are folding into `drain_and_join`; the `ProcessPeerBundle` drop-order invariant
(spawn.rs, the `// INVARIANT: declaration order` comment) is the same drain-before-signal
discipline for the process tier.
