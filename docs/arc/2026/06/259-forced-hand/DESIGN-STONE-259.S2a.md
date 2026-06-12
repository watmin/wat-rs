# Stone 259.S2a — the ThreadProg self-peer model (on the unified `Peer`)

> Substone of arc 259 (The Forced Hand), spawn-redesign. Parent DESIGN:
> `docs/arc/2026/06/259-forced-hand/DESIGN.md` — esp. § "THE CONVERGED MODEL"
> (the unified pipes-only `Peer`, internal `close`, one entry point).
>
> **Supersedes** the earlier `ThreadSelf'` + mirror-projection draft of this file
> (committed `2529cce5`); re-authored onto the unified `Peer` after the design
> duet converged.

## Why this stone

The DESIGN locks the per-tier prog asymmetry: **`ProcessProg` is a stdio
`:user::main`** (fresh fds → the child just *is* a stdio program; keep it) but
**`ThreadProg = [self: Peer] -> nil`** — a thread shares the parent's ambient
stdio, so it *cannot* use stdio for its data channel; it must be handed its own
`(rx, tx)` self-peer explicitly and drive it with `recv'`/`send'`. That is the
capability grant (Mark Miller): the channel crosses by explicit hand-off, never
ambient abuse.

At HEAD the `:thread` arm of `spawn-program'` runs the **platform apply-loop**
(`src/kernel/spawn.rs:357-390`): it `recv`s a Value, applies a 1-ary fn, `send`s
the result, loops. The prog never sees a peer — the platform owns the channel.
S2a replaces that loop with the **self-peer hand-off**: build the prog its own
pipes-only `Peer`, hand it over ONCE, let the prog own its body.

S2a is the hard new machinery the rest of Stone 2 rides on. It keeps the existing
3-arg call shape (`:thread env prog`) and user `close'` — the dispatch redesign
(S2c) and the internal-`close`/RAII (S2b) come after.

## The contract (pinned)

### The unified peer — `Peer'<S,R>`

A **pipes-only** bidirectional endpoint — the same type the whole converged model
uses (no bespoke `ThreadSelf'`). Holds exactly the two pipe ends, nothing else:

```rust
struct Peer<S, O> { tx: comms::thread::Sender<S>, rx: comms::thread::Receiver<R> }
//   send'(s: S)  →  tx.send(s)        recv'() -> R  →  rx.recv()
```

**Uniform projection — one rule, no mirror.** For *every* peer, `<S,R>` reads
`<send-type, recv-type>`: `send'`→S, `recv'`→R.

| peer | `send'` payload | `recv'` returns |
|---|---|---|
| `Thread'<I,O>`  (parent handle, exists today) | `I` | `O` |
| `Peer'<S,R>`    (worker self-peer, NEW)        | `S` | `R` |

The worker is `Peer'<O,I>` — the param-swap of the parent's `Thread'<I,O>` (parent
sends I / recvs O; worker recvs I / sends O). No mirror projection: the duality
lives in the type's argument order, the verbs stay uniform. (For the echo probe
I=O=i64, so the worker is `Peer'<i64,i64>`.)

`Peer'` carries **no `JoinHandle`** — it is pipes only. Lifecycle is the parent's
(today: `Thread'` + `close'`; after S2b: internal RAII).

### The prog shape

`ThreadProg = [self <- :wat::kernel::Peer'<S,R>] -> :wat::core::nil`, applied
**exactly once** with its self-peer. The prog owns its own loop if it wants one;
the platform no longer imposes one.

### The owner-thread invariant (the trap)

The `Peer'` opaque wraps `Arc<ThreadOwnedCell<Option<Peer<…>>>>`.
`ThreadOwnedCell` captures the **constructing thread's id** and panics on
cross-thread access. The prog runs on the *spawned* thread, so the self-peer
opaque MUST be constructed **inside the spawned thread's closure**, never on the
parent thread. (The raw `input_rx` / `output_tx` endpoints are `Send` — move them
into the closure, build the `Peer` + opaque there.) Getting this wrong is a
runtime panic, not a compile error — the one load-bearing subtlety.

## The five touches (probe-grounded)

The disconfirming probe `tests/nursery/probe_arc259_s2a_thread_self_peer.rs` is
CHECK-RED at HEAD (`send'`/`recv'` reject the `Peer'` head:
`"expected peer (Thread'<I,O> | Process'<I,O>), got Peer'<...>"`). The five touches:

1. **`src/kernel/peer.rs`** — `Peer<S,R>` struct: `tx: comms::thread::Sender<S>`,
   `rx: comms::thread::Receiver<R>`; `send(&self, S)->Result<(),SendError<S>>`,
   `recv(&self)->Result<R,RecvError>`. Pipes only — no `JoinHandle` (mirror of
   `Thread<I,O>` minus the join).
2. **`src/kernel/spawn.rs`** — `PEER_TYPE_PATH = ":wat::kernel::Peer'"` sentinel +
   a cell alias; rewrite `spawn_thread_peer`: move `input_rx`+`output_tx` into the
   closure, construct the `Peer { tx: output_tx, rx: input_rx }` opaque **inside**
   the closure (owner-thread invariant), call `apply_function(prog, vec![self_peer_value], …)`
   ONCE (delete the loop). The parent still gets its `Thread'` handle (unchanged).
3. **`src/runtime.rs`** — `eval_peer_send_prime` / `eval_peer_recv_prime` gain a
   third arm for `PEER_TYPE_PATH`: `send'` → `cell…tx.send(val)`, `recv'` →
   `cell…rx.recv()` (Value pass-through, exactly like the `Thread'` arm; no EDN
   bridge — thread tier carries live Values).
4. **`src/check.rs`** — `send'`/`recv'` projective inference accepts the `Peer'`
   head with the **uniform** projection (`send'`→arg0, `recv'`→arg1 — identical to
   `Thread'`, NOT a special rule). Whatever else reads the peer-head set accepts
   `Peer'` too if the probe surface needs it.
5. The probe flips GREEN: `s2a_thread_prog_drives_self_peer` round-trips 42 through
   the prog's pipes-only `Peer'` self-peer.

## Out of scope — REJECTED, tracked downstream

- **The parent-side unification + internal `close` + RAII Drop** — S2a keeps the
  parent as `Thread'` with user `close'`. The pipes-only parent + RAII reap is S2b.
- **The host-type defclause + 2-arg `(host prog)` sig + env-arg removal** — S2a
  keeps the 3-arg `(:thread env prog)` shape; only the `:thread` *body* changes. S2c.
- **The strict forced-hand typing** — S2a's check-side accepts `Peer'`
  projectively; wrong-payload-is-a-type-error lands with the defclause in S2c.
- **The process tier** — untouched; `:process` keeps its forms-server stdio model.
- **`Peer'` for the parent / killing `Thread'`** — the full collapse of
  `Thread'`/`Process'` into `Peer` + internal lifecycle is the later unification;
  S2a only introduces `Peer'` as the worker self-peer.

## Naming

`Peer'` is the pipes-only bidirectional peer — the unified peer of the converged
model, primed per the live `Thread'`/`Process'` migration convention (the `'` drops
at the migration's end). Intueri-confirmable at ward time; not blocking.

## Shipped — weighed (2026-06-11)

Built by a shadowdancer, weighed against the disk by the orchestrator. Two honest
deltas from the prediction above:

1. **The apply-loop is NOT deleted — it becomes a TRANSITIONAL dual-mode** (correct,
   and better than this doc's original "delete the loop"). `spawn_thread_peer` now
   dispatches on the prog's first param type: `Peer'<S,R>` → self-peer handoff;
   anything else → the legacy apply-loop. Deleting the loop in S2a would break the
   arc-214 apply-loop callers — forcing S2d's migration into S2a. Keeping both is the
   honest stepping-stone (S2a *adds* the model, **S2d** migrates the callers + deletes
   the loop). Both transitional branches carry `rune:exigere(scope-affirmative)` naming
   S2d as their death (`kernel/spawn.rs` dispatch + `check.rs::infer_spawn_program_prime`
   legacy projection).
2. **A 5th touch the brief under-specified — `infer_spawn_program_prime`.** A self-peer
   prog `[Peer'<S,R>] -> nil` must infer the parent peer as **`Thread'<R,S>`** (the
   param-swap: parent sends R → worker recvs R; parent recvs S ← worker sends S), NOT
   `Thread'<Peer'<S,R>, nil>`. Without it the parent's `send' 42` mis-types. The
   shadowdancer caught this; the WEIGH confirmed the param-swap correct.

**Gates (orchestrator's own re-run):** the S2a probe → 42 ✓; arc-214 peer verbs 3/3 ✓;
lib `kernel::spawn` 1/1 ✓; `cargo build --release` clean (warnings only, all
pre-existing — `value_matches_type_pattern` is dead at clean HEAD, not this strike);
**full nursery SERIAL `--test-threads=1` = 840 passed / 4 failed**, the 4 being EXACTLY
the known pre-existing reds (arc-255 reflection ×2 + undefined-builtin ×2). The parallel
run flaked the thread/channel/spawn probes (the arc-170 spawn-race contention) — all
green serial. **NET REGRESSION = ZERO.** No STOP triggers hit.
