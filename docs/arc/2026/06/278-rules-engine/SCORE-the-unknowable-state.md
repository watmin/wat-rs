# SCORE — the unknowable state

**NOT STRUCK.** The specified fault has no live-service form. Executor: grok, 2026-09-04.
Verified by me.

```
Summary [ 360.811s] 5213 tests run: 5213 passed (4 slow), 15 skipped
FLOOR=0        .floor/2026-09-04T05-42-24Z/        default no-drop
```

`seen-dups` did not move. **The stone's premise was mine and it was false.**

## ⛔ MY PROBE MEASURED THE RIGHT STRING AND I READ THE WRONG THING INTO IT

`probe-reply-drop-is-userland.wat` printed `call2-RETURNED=LOST`, and I concluded *"a `None` reply
surfaces as a clean, handleable `LOST` — 3d is a userland stone."*

The liveness check I never ran, connected now, 3/3:

```
call1=ok:1
call2-RETURNED=LOST
served-after=-1 ; redial=REFUSED ; verdict=service-DIED-the-LOST-was-death
```

**The caller saw `LOST` because the peer was gone**, not because a reply was omitted on a living
connection.

★ **And the instrument was in the file.** `:cd::served-count` is defined at
`probe-reply-drop-is-userland.wat:89` **and never called.** I wrote a liveness probe into the
fixture, did not connect it, and drew a conclusion the unconnected instrument would have refused.

That is R69's shape — *"`body-key` had been sitting there the whole time, defined and never called.
The instrument existed. Nobody connected it."* — **committed by me, in a probe, roughly two hours
after I wrote that sentence into a SCORE.**

`LOST` names what the caller got. It does not name **why**, and two worlds print it identically.
Third instrument-collapse of the session, and the first where the collapse was in my *reasoning*
rather than in a sentinel.

## ★ THE FINDING — two spellings of `None`, opposite consequences, and the type cannot tell them apart

| spelling | consequence |
|---|---|
| `(:wat::core::None :Reply)` — **typed** | **the service dies** |
| `:wat::core::None` — **untyped** | **deferred reply**: the caller blocks, *the service lives* |

The untyped form is **live, load-bearing code**. `sqs.wat:584-585` is the queue's long-poll park:

```wat
(:wat::service::Outcome::Continue s-a
  :wat::core::None
  …)
```

The caller waits; the tick answers it later. **That is how long polling works in this tree.**

And `wat/service.wat:53` gives the intent: *"`:NoReply` — a cast / a fired self-op with **no client
to reply to** (OTP `{noreply,S}`)."* `None` means *there is no caller* — not *don't answer the
caller you have.* Internal arms already express "no caller" structurally, via `SelfOutcome`, which
has no reply field at all.

★ **So a mode is distinguished by a type ascription.** `Option<R>` admits both spellings and cannot
say which you wrote; one defers, one is fatal. **That is this campaign's defect class exactly** —
`wait-ns 0`, `after(Duration(0))`, `q-depth`'s `(Tuple 1 1)`, `take-one`'s hidden visibility: a mode
carried by something other than a constructor. **S32**, and rung 3 is available — the fatal spelling
should be a check error, not a runtime death.

**Same class as `:wat::kernel::close`: not an alternative to try harder at.** Recorded so no one
re-proposes it.

## The executor stopped correctly, and that is the result

- Did **not** tune the rate or seed to manufacture a number.
- Did **not** repair the worker.
- Did **not** touch `:353-360` or `:402-419`.
- Reported *"turning the knobs on (12×2×2, 2000 bp) dies the same way: `seen` gone, `Connection
  refused` on stats, `drained-never: last=[1/3][2/6]`"* — **service death, not the after-write cell**,
  and said so rather than reporting a number.

Default remains no-drop. Rate 0 this session: `seen-dups=0; distinct=8000; dup=0`. Floor green. Both
placements sit in the `Record` and do not arm.

## What survives

**The predicted stranding is UNRUN, not disproven.** The mechanism stands exactly as written:

> A claims → `First` → ledger written → reply dropped → A does not ack and emits `outs0`.
> Visibility expires, B claims → `Dup` → `first? = false` → B emits nothing either. **No outcome is
> ever emitted for that message** → `distinct < 8000`. Not a duplicate — a **stranding**.

It was never reached, because the drop never landed on a live peer. It is not evidence and it is not
refuted; it is a hypothesis still waiting for a fault that can carry it.

## What 3d needs before it can be re-drawn

A fault that produces **work-done, caller-informed-of-failure, service-alive.** The three mechanisms
now known to be unavailable or wrong:

- **`:wat::kernel::close`** — kernel-only, and reaps a spawned child, not a dialed peer.
- **typed `None` reply** — kills the service.
- **untyped `None` reply** — the caller blocks; the service lives but nobody is told anything.

3c's sever is the closest thing that works, and the tracker already explains why it cannot produce a
duplicate: **arms run to completion and an alarm fires between them.** The remaining candidate is a
fault *inside* the reply-send — which is the reactor, which is the 3120-line single-form macro.
**That is the honest next question, and it is not this stone.**

## Still open

- **S32** — two `None` spellings, opposite consequences, no type distinction.
- **S31** — `claimed` is `:ephemeral`.
- **3d, re-scoped** — needs a mechanism that does not exist yet in userland.
- **Stone D2** · **Stone C** · **S15**–**S32**.
