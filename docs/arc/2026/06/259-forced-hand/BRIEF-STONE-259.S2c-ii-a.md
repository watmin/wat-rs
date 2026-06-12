# BRIEF — Stone 259.S2c-ii-a: the apply-loop PURGE

**The work, in one paragraph.** Annihilate the legacy apply-loop thread model. After
this, a `:thread` prog MUST be a self-peer prog `[self <- :wat::kernel::Peer'<S,R>] ->
:wat::core::nil`; the apply-loop form `[I] -> O` is REJECTED at check. Remove the
transitional dual-mode branches (the S2a/S2c-i `rune:exigere` apply-loop paths), and
migrate every apply-loop caller to the self-peer form. The committed probe
`s2cii_a_apply_loop_prog_rejected` flips RED→GREEN.

**Read in order (the rooms):**
1. `tests/nursery/probe_arc259_s2cii_a_applyloop_purged.rs` — the GREEN target (RED at
   HEAD: the apply-loop prog is still accepted).
2. `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S2c-ii.md` — the campaign + the
   purge ruling.

**Annihilate the apply-loop branches:**
- `src/kernel/spawn.rs` `spawn_thread_peer`: DELETE the `is_self_peer_model` dispatch and
  the entire apply-loop `else` arm + its `rune:exigere`. The spawned closure now
  ALWAYS does the self-peer handoff (construct the `Peer'` opaque inside the thread, call
  the prog ONCE). The apply-loop `loop { recv → apply → send }` is gone.
- `src/check.rs` `infer_thread_prog_type`: DELETE the legacy apply-loop projection (the
  non-`Peer'` `[I]->O → Thread'<I,O>` arm + its `rune:exigere`). ONLY a self-peer prog
  `[Peer'<S,R>] -> nil` is valid → `Thread'<R,S>`. A non-`Peer'` prog must produce a
  CLEAR error: `"spawn-program' :thread expects a self-peer prog [Peer'<S,R>] -> nil; got <T>"`.

**Migrate every apply-loop caller to self-peer — THE SWAP (mechanical, peer-type-preserving):**

An apply-loop prog `(:wat::core::fn [input <- I] -> O input)` yields `Thread'<I,O>`. Its
self-peer equivalent is `(:wat::core::fn [self <- :wat::kernel::Peer'<O,I>] -> :wat::core::nil
(:wat::kernel::send' self (:wat::kernel::recv' self)))` — which ALSO yields `Thread'<I,O>`
(`Peer'<S,R>` with S=O, R=I → `Thread'<R,S>` = `Thread'<I,O>`). So the peer type is
identical and ALL downstream `send'`/`recv'`/`select'`/annotation assertions are unchanged
— only the spawn's prog argument swaps. For the symmetric `[i64]->i64` case (every site),
the self-peer prog is `[self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :nil
(send' self (recv' self))`.

Per the four-questions decision (`rewrite where the verified capability survives; retire
only duplicates`):
- `tests/nursery/probe_arc214_stone46aii_peer_verbs.rs` — **p1** (`probe_1_thread_round_trip_via_verbs`,
  a single round-trip) is a DUPLICATE of `probe_arc259_s2a`'s self-peer round-trip → **RETIRE
  it** (delete the test). **p2** (`probe_2_recv_projects_o...`) + **p3** (`probe_3_send_checks_i...`)
  → **SWAP** the apply-loop prog for the self-peer prog; the `recv'`-projects-O / `send'`-checks-I
  negative assertions are unchanged.
- `tests/nursery/probe_arc214_stone46i_typed_peer.rs` — all 3 → **SWAP** (the `Thread'<i64,i64>`
  inference assertions hold under the self-peer prog).
- `tests/nursery/probe_arc214_stone46b_select_prime.rs` — both → **SWAP** (the `select'` multiplex
  is unchanged; only the spawned progs swap).
- **Rust lib tests using apply-loop progs** — grep `spawn_thread_peer(` + `[input <- ` in
  `src/kernel/spawn.rs` / `src/kernel/peer.rs`: `spawn_thread_peer_echo_round_trip` (spawn.rs)
  spawns an apply-loop echo via the Rust fn — it now gets a self-peer arg; **rewrite it to a
  self-peer prog** (or RETIRE if redundant with `s2b_drop_reaps_blocked_worker`'s self-peer
  coverage — your call, state which). The `peer.rs` `thread_peer_round_trip` constructs the
  `Thread` peer with MANUAL channels (not via `spawn_thread_peer`) so it does NOT run a prog —
  verify it's unaffected; if so leave it.

**STOP triggers (halt + report; do not work around):**
- **STOP-1:** if any apply-loop caller's prog is NOT a clean `[I]->O` identity/echo (e.g. it
  transforms the value, or sends multiple messages relying on the platform loop), STOP and
  report it — that one needs a by-eye semantic decision (a self-peer prog loops via NAMED
  recursion, not the anonymous swap), not the mechanical swap.
- **STOP-2:** if removing the apply-loop projection breaks the PROCESS tier or any `Peer'`/
  self-peer path, STOP — only the thread apply-loop dies; `:process` (forms) is untouched.

**Done = green:**
- `cargo test --release -p wat --test nursery probe_arc259_s2cii_a` → passes (apply-loop rejected).
- `cargo test --release -p wat --test nursery probe_arc259_s2a` + the rewritten
  `probe_arc214_stone46aii` / `_46i` / `_46b` → all green.
- `cargo test --release -p wat --lib kernel::spawn kernel::peer` → green (the rewritten/retired lib tests).
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known
  pre-existing reds (arc-255 reflection ×2 + undefined-builtin ×2).

**Note (banked, S2c-ii-b):** the self-peer multi-message server idiom is NAMED RECURSION
(`(defn :srv [self <- Peer'] -> nil (do (send' self (recv' self)) (:srv self)))`) — wat has no
`loop`/`recur`, and an anonymous prog can't self-call. Not needed for these single-message
tests; record it for the user-facing docs at the defclause stone.
