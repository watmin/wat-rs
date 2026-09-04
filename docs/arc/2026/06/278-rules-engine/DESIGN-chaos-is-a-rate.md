# DESIGN — chaos is a rate

**Stone 3c.** The disruptor. Opt-in, seeded, self-arming, rate 0 by default.

> Builder, 2026-09-04: *"we'd have an internal `-disrupt` call... its invoked randomly?... and
> when it gets invoked it just attacks the select pool?"*
>
> *"wat has been earning its marks by combat — we study our enemy as the inquisitor and we strike as
> the shadowdancer... we strike when we know we have the kill."*

## WHY

58 failure arms across the circuit's three files have **never been taken.** `Lost`, `Closed`,
`Stopped` are matched everywhere because the substrate makes them unignorable — and nothing drives
them. `experiri`: *a surface that cannot be reached is a promise the system does not keep.*

3c-pre (`17e80d01c`) took the gate down: `Closed` recovers, the dead-peer wall still fires, and a
real sever was survived end-to-end with `dup=0`. What remains is that the sever was **one poison
wired into worker start** — a proof instrument, not a fault domain.

**This stone is the difference between "a reap is survivable" and "chaos is a rate."**

## THE FOUR PROBES — every mechanism proven before the strike

| probe | result |
|---|---|
| `probe-rand-is-usable-from-wat.wat` | `same-seed=REPLAYS; rate-hits=97/1000` — **SEEDED-CHAOS-IS-REPLAYABLE** |
| `probe-disrupt-reaps-and-reacquires.wat` | `before=ok:1/d=0; after=ok:2/d=1` — an arm poisons its own peer, re-acquires, keeps serving |
| `probe-closed-is-recoverable.wat` | `a-REDIAL=ok` — a severed connection is re-dialable |
| `probe-reply-drop-is-userland.wat` | `call2-RETURNED=LOST` — 3d needs no reactor surgery *(not this stone)* |

Independently re-run by the executor on this tree; strings match.

## ⛔ `:wat::kernel::close` IS DEAD AS A MECHANISM — from disk, twice

```
src/runtime.rs:25160   #[restricted_to(":wat::kernel::close", ":wat::kernel::")]
tests/kernel/probe_arc259_s2d_internal_only_close.wat.bad     ← arc 259 already banked it
```

Kernel-only **and** it reaps a spawned *child*; a dialed `Peer` is a different type. **Do not
reach for it. It is not an alternative to weigh — it has no form.**

The reachable mechanism is the one the tree already uses: **reap by speaking too loudly.** An
oversized *frame* severs the sender's own connection.

## WHAT IT DELIVERS

An opt-in internal arm on the circuit's services:

```
:durable [… disrupt-rate-bp <- i64      ;; basis points. 0 = off. Crosses the wire.
             disrupt-seed    <- i64      ;; threaded; the run replays
             disrupt-lo-ms   <- i64      ;; re-arm delay window
             disrupt-hi-ms   <- i64]
```

- **`start` arms `-disrupt` only when `rate-bp > 0`.** Rate 0 arms *nothing* — no timer, no cost.
- **`-disrupt`** draws from the seed; on a hit it poisons its own peer with an oversized frame,
  re-acquires, and **threads the fresh peer into state**; then draws a delay and **re-arms itself**.
- **`SelfOutcome` has no reply field** — *"Internal arms have no caller, so they have no reply
  field — the mistake cannot be written."* A disruptor that could answer a caller has no form.
- **Seed and rate live on the `Record`**, not on a client feature: they cross the wire and survive
  hibernation, so a chaotic run is replayable by its seed.

## ⛔ THE ONE CONTRACT DECISION

**`-disrupt` re-arms itself from a seeded draw. A disruptor that fires once is the probe, not the
stone.**

`probe-disrupt-reaps-and-reacquires.wat` returns **empty alarms** — it proves a reap is survivable
and stops. If this stone ships without the re-arm it has changed the spelling and not the fault
domain: one sever is a test case; a *rate* is chaos.

## ⛔ SEVEN CARRIES — settled, so they are not rediscovered mid-strike

1. **The oversized-frame tear is PROCESS-LOCUS.** Thread-locus in-process channels do not tear.
   3c-pre's captured red (`.floor/2026-09-04T01-53-16Z/`) was exactly that. **Do not demand drops on
   thread-locus tests, and do not invent a second disrupt mechanism to make thread look like
   process.** This is the fourth locus asymmetry (S28) and it is a property, not a bug to route
   around.
2. **The probe is one-shot.** See the contract decision.
3. **Do not copy 3c-pre's always-on poison.** That was a proof instrument in worker start. **A floor
   that suddenly severs every process worker is the 3c-pre ARM all over again.**
4. **Trap 3 is a rejection criterion**: the fresh peer goes into state, or `Closed` becomes an
   infinite loop **that looks like a hang** — the failure mode the last four stones removed.
5. **The grant rides `process/post-spawn`, before `:init` dials** (`sns-fanout.wat:22-27`). A
   stranger is bounced until granted. **Already paid for once**, in the disrupt probe.
6. **Do not braid 3d.** The `None`-reply → `LOST` path is proven and is a separate stone;
   `wat/service.wat` stays the one-form macro, untouched.
7. **Name every outcome.** The disrupt probe's *outer* helper names `LOST`/`STOPPED`/`CLOSED`; its
   `hit` arm still collapses `Sink/ping` to `-1`/`-2` (`:123-124`). The verdict cell saved it —
   `ok:2/d=1` cannot be a hidden `-2`. **The BRIEF's exemplar must not carry that flaw.** Third
   collapsed fallback this session, after `q-depth`'s `(Tuple 1 1)` and the wire probe's `-1`.

## FILES

`wat-scripts/fanout/circuit.wat` (the worker) · `wat-scripts/topic/sns-fanout.wat` (the
topic-worker) · one scratch probe for the rate.

**No `wat/service.wat`. No `src/`. No codemod.** The disruptor is an ordinary internal arm.

## OUT OF SCOPE = REJECTED

- **`:wat::kernel::close`.** No form. Not an alternative.
- **3d, the reply-drop.** Proven userland; its own stone.
- **Promotion of `-disrupt` into `defservice`.** The `sqs.wat:3-5` precedent governs: *built in
  userland, promoted when it demonstrates excellence, and that promotion is the builder's ruling.*
  A substrate feature minted on a guess is what this arc has spent eight stones removing.
- **Making thread locus tear.** Carry 1.

## THE PROOF

1. **★ Rate 0 arms nothing.** With the default, `-disrupt` never fires — and the floor is unchanged.
   Not "fires and does nothing": **no alarm is armed at all.**
2. **★ Chaos is a rate.** With a rate set, `-disrupt` fires **many times** across a run — report the
   count. One firing fails this row.
3. **★ The seed replays.** Two runs, same seed → **the same disruption count at the same points.**
   A chaotic run that cannot be replayed cannot be debugged.
4. **★ The invariant survives chaos.** With the rate on, `total=8000; distinct=8000; dup=0`.
   ⛔ If `dup > 0` appears, **that is a finding, not a failure** — at-least-once permits it and R69
   says our detector was blind to it for nine stones. Report the number; do not tune it away.
5. **The floor**, Summary line, `5213/5213`, with the default rate.
