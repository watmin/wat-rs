# DESIGN — a fired deadline hands back a live peer

Builder: *"the primitive should redial - draw stone 2"*, after ruling
*"peer should remember its address"* and confirming D1-(1) and D2-(b) as the reasoned directions.

**Drawn 2026-09-15. NOT STRUCK.** Stone 2 of three.

## WHY — the measured defect, and it is silent

`FINDING-a-fired-deadline-desyncs-the-connection-forever.md`. One connection, tags to make a stale
reply visible:

```
call-1(tag=11,dl=100)=DeadlineFired                the caller gave up first
call-2(tag=22,dl=5000)=Answered(tag=11) sent=22    ← got call 1's reply
call-3(tag=33,dl=5000)=Answered(tag=22) sent=33    ← got call 2's reply
call-4-GENERATED=Ok(tag=33) sent=44                ← the generated method, same shift
```

Permanent, silent, and the **generated client method inherits it** — so every `<S>::<Op>` call in the
system is exposed. `circuit.wat:1000` redials by hand in exactly one place; the macro does not.

⭐ **And it is NOT process-only. Measured this session:**
`wat-scripts/scratch-pad/probe-a-slow-peer-desyncs-thread-tier.wat` — the identical four lines on the
thread tier. A crossbeam channel queues an abandoned reply exactly as a stream does. ⚠ The builder's
ruling that *"the thread tier doesn't need chaos"* was about **fault injection**; this is a live bug
class on both tiers and the ruling does not cover it.

## ⛔ THE ONE CONTRACT DECISION

**`DeadlineFired` means: your call timed out, AND your handle has been re-established.
If it could not be, the answer is `Lost`.**

No new variant, no arity change, **no change to any of the 13 existing match arms** — because the
repair happens *inside the handle*:

```
pub type PeerCell = Arc<ThreadOwnedCell<Option<Peer>>>     src/kernel/spawn.rs:142
```

The wat-visible peer is a **shared, swappable cell**, not a value. So `call-by-deadline` re-dials and
replaces the cell's contents; every holder of that handle — including the caller of a generated
client method, which returns a nullary `RecvOutcome::TimedOut` and could never have carried a fresh
peer — is immediately working again. `with_mut` is the existing operation (precedent:
`src/runtime.rs:25845` already does `with_mut(… |opt_peer| opt_peer.take())`).

⭑ **The redial's failure is where the honesty lives.** Three outcomes, all from existing variants:

| the deadline fires and… | answer | meaning |
|---|---|---|
| redial succeeds | `DeadlineFired` | timed out; **your handle is good** |
| redial fails | `Lost [cause]` | the peer is gone — not merely slow |
| `dialed-from` is `None` | ⛔ see below | cannot be repaired |

**That is a strictly stronger contract than today with no type change.** Today `DeadlineFired` hands
back a booby trap; after this it hands back a connection or tells you the peer died.

### ⛔⛔ AND THE THIRD ROW IS A GAP I PUT IN STONE 1

`a-peer-remembers-its-address`'s DESIGN says, in my words: *"`Peer::from_thread` — thread tier —
`None` — none exists."* **That is false.** `ThreadAddress` exists (`src/kernel/address.rs:97`) with its
own `connect` (`:106`), so a thread-tier dial holds an address and drops it — the *same* omission
stone 1 was drawn to repair, on the other tier, and my DESIGN instructed the executor to bake it in.

★ So this stone also **finishes stone 1**: `from_thread` takes the optional address and
`ThreadAddress::connect` passes `Some(self.clone())`. `dialed-from` needs no type change —
`Address` is transport-blind (`Address { inner: Box::new(ThreadAddress { tx }) }`, `:303`), so a thread
dial simply returns `Some`.

⚠ **Shipping the redial without this would make the primitive repair only half its callers** — and
"silently repairs, sometimes" is worse than a uniform failure, because nobody can reason about which
half they are in. One coherent idea: *a fired deadline hands back a working handle, on either tier.*

## Out of scope = REJECTED

- **The publisher's arm.** Stone 3, and trivial once this lands: it stops needing to redial at all.
- **`DeadlineFired [peer]` — carrying the fresh peer in the variant.** Rejected on measurement: the
  generated client method maps it to `RecvOutcome::TimedOut`, which is **nullary**, so a carried peer
  could never reach the caller that matters. It would also churn 13 arms to deliver nothing.
- **Linear/affine handles.** Unneeded — this hands back a good peer rather than forbidding a bad one.
- **Repairing `recv-by-deadline`.** ⚠ It may share the defect (the owner path). **Unmeasured.** Say so
  in the SCORE rather than assuming either way; it is its own stone.

## Blast radius

```
wat/service.wat        call-by-deadline (:4210) — the 3 DeadlineFired sites (:4233 :4237 :4241)
src/kernel/peer.rs     from_thread takes the address
src/kernel/address.rs  ThreadAddress::connect (:106) passes Some(self.clone())
src/kernel/*, runtime  the other from_thread callers pass None, each deliberately
13 existing match arms  0 — the contract strengthens without changing shape
```

## Trap-doors named up front

1. ⛔⛔ **A silent in-place swap is HIDDEN STATE** — `sequi`'s exact complaint. It is accepted here for
   one reason only: **the peer is already a shared mutable cell, not a value.** The type says handle,
   and handles that reconnect underneath you are what a connection pool is. ⭑ But it must be
   *observable*: the SCORE must say how a reader learns a redial happened. If nothing records it, we
   have fixed a corruption and created an invisible reconnect.
2. ⛔ **The cell is an `Arc` — a swap is visible to EVERY holder.** Correct for this defect (a stale
   peer is stale for all of them) and it must be deliberate, not incidental.
3. ⚠ **A redial is a round trip on a path that just lost one.** Deadline budgets are measured in this
   corpus; report the cost.
4. **`ThreadOwnedCell` is thread-owned.** The swap must happen on the owning thread —
   `call-by-deadline` runs on the caller's, which should be that thread. Verify, do not assume.
5. **Do not let the swap resurrect a dead peer.** If the redial succeeds but the service is actually
   gone, the next call must still fail honestly.
6. **`cargo build --release` does not compile tests.** Use `cargo nextest run --release --no-run`.
