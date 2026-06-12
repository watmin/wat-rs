# Stone 259.S2a — the ThreadProg self-peer model

> Substone of arc 259 (The Forced Hand), spawn-redesign. Parent DESIGN:
> `docs/arc/2026/06/259-forced-hand/DESIGN.md` § "The spawn primitive" + § "The
> prog — per-tier contract".

## Why this stone

The DESIGN locks the per-tier prog asymmetry: **`ProcessProg` is a stdio
`:user::main`** (fresh fds → the child just *is* a stdio program; keep it) but
**`ThreadProg = [self: ThreadPeer] -> nil`** — a thread shares the parent's
ambient stdio, so it *cannot* use stdio for its data channel; it must be handed
its own `(rx, tx)` self-peer explicitly and drive it with `recv'`/`send'`. That
is the capability grant (Mark Miller): the channel crosses by explicit hand-off,
never via ambient abuse.

At HEAD the `:thread` arm of `spawn-program'` runs the **platform apply-loop**
(`src/kernel/spawn.rs:357-390`): it `recv`s a Value, applies a 1-ary fn, `send`s
the result, loops. The prog never sees a peer — the platform owns the channel.
S2a replaces that loop with the **self-peer hand-off**: build the prog its own
child-side peer, hand it over ONCE, let the prog own its lifecycle.

This is the hard new machinery the rest of Stone 2 rides on. It is sequenced
first because it is a contained, independently-provable unit; S2c (the host-type
defclause) then operates on a settled foundation.

## The contract (pinned)

### The child self-peer — `ThreadSelf'`

The dual of the parent's `Thread'` handle. For one connected channel:

```
:wat::kernel::Thread'<In,Out>      ; the PARENT's handle (exists today)
:wat::kernel::ThreadSelf'<In,Out>  ; the CHILD's view of its own end (NEW)
```

**Param convention (locked):** `<In,Out>` names the channel's two directions in
the **parent's frame** — `In` flows parent→child, `Out` flows child→parent. Both
ends share the SAME param list (no swap). The verbs then read as one rule:
`send'` = "what leaves *this* end", `recv'` = "what arrives at *this* end". So
the projection is the **mirror**:

| peer | `send'` payload | `recv'` returns |
|---|---|---|
| `Thread'<In,Out>`     (parent) | `In`  | `Out` |
| `ThreadSelf'<In,Out>` (child)  | `Out` | `In`  |

Obvious (one rule), Simple (one shared param list), Honest (the type describes
the channel, not a local view).

### The prog shape

`ThreadProg = [self <- :wat::kernel::ThreadSelf'<In,Out>] -> :wat::core::nil`,
applied **exactly once** with its self-peer. The prog owns its own loop if it
wants one; the platform no longer imposes one.

### The owner-thread invariant (the trap)

The `ThreadSelf'` opaque wraps `Arc<ThreadOwnedCell<Option<ThreadSelf<…>>>>`.
`ThreadOwnedCell` captures the **constructing thread's id** and panics on
cross-thread access. The prog runs on the *spawned* thread, so the self-peer
opaque MUST be constructed **inside the spawned thread's closure**, never on the
parent thread. (The raw `input_rx` / `output_tx` endpoints are `Send` — move them
into the closure, build the `ThreadSelf` + opaque there.) Getting this wrong is a
runtime panic, not a compile error — it is the one load-bearing subtlety.

## The five touches (probe-grounded)

The disconfirming probe `tests/nursery/probe_arc259_s2a_thread_self_peer.rs`
(CHECK-RED at HEAD: the verbs reject the `ThreadSelf'` head) drove the exact
scope:

1. **`src/kernel/peer.rs`** — `ThreadSelf<I,O>` struct: `input:
   comms::thread::Receiver<I>` (arrives at child), `output:
   comms::thread::Sender<O>` (leaves child); `recv(&self)->Result<I,RecvError>`,
   `send(&self, O)->Result<(),SendError<O>>`. Mirror of `Thread<I,O>` minus the
   `JoinHandle` (the child does not join itself).
2. **`src/kernel/spawn.rs`** — `THREAD_SELF_PEER_TYPE_PATH =
   ":wat::kernel::ThreadSelf'"` sentinel + `ThreadSelfPeerCell` alias; rewrite
   `spawn_thread_peer`: move `input_rx`+`output_tx` into the closure, construct
   the `ThreadSelf` opaque **inside** the closure (owner-thread invariant), call
   `apply_function(prog, vec![self_peer_value], …)` ONCE (delete the loop).
3. **`src/runtime.rs`** — `eval_peer_send_prime` / `eval_peer_recv_prime` gain a
   third arm for `THREAD_SELF_PEER_TYPE_PATH`: `send'` → `cell…output.send(val)`,
   `recv'` → `cell…input.recv()` (Value pass-through, exactly like the `Thread'`
   arm; no EDN bridge — thread tier carries live Values).
4. **`src/check.rs`** — `send'`/`recv'` projective inference accepts the
   `ThreadSelf'` head with the **mirror** projection (table above). Whatever else
   reads the peer-head set (e.g. `close'`) accepts `ThreadSelf'` too if the probe
   surface needs it.
5. The probe flips GREEN: `s2a_thread_prog_drives_self_peer` round-trips 42
   through the prog's self-peer.

## Out of scope — REJECTED, tracked to S2c

- **The host-type defclause + the 2-arg `(host prog)` sig + env-arg removal** —
  S2a keeps the existing 3-arg `(:thread env prog)` call shape; only the
  `:thread` *body* changes. The dispatch redesign is S2c.
- **The strict forced-hand typing** — S2a's check-side accepts `ThreadSelf'`
  projectively; making `ThreadOpts` *demand* a `ThreadProg` (wrong prog = type
  error) lands with the defclause in S2c.
- **The process tier** — untouched; `:process` keeps its forms-server stdio
  model.
- **The timing correction** (`wat.peer-started-at` stamping) — banked, per the
  parent DESIGN § "The corrected timing model".

## Naming

`ThreadSelf'` is the working name — the prog's view of its own channel end,
primed per the live `Thread'`/`Process'` migration convention (the `'` drops with
the rest at the migration's end). Intueri-confirmable at ward time; not blocking.
