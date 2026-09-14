# FINDING — a fired deadline desyncs the connection FOREVER, and silently

**Found 2026-09-14.** Drawn while building the latency injector the builder asked for
(*"let's continue to find ways to inject chaotic networking into our circuit"*). The injector was not
built. The first probe found this instead.

**Severity: silent wrong answers on the core client path.** Not a crash, not a hang — a caller that
receives someone else's reply and is told it succeeded.

## The measurement

`wat-scripts/scratch-pad/probe-a-slow-peer-desyncs-the-next-call.wat`. A service whose handler parks
600 ms on a timer channel (mora-legal, inlined — a forked child cannot see parent helpers). One
client, one connection, four calls, each carrying a `tag` so a stale reply is visible:

```
call-1(tag=11,dl=100)=DeadlineFired                the caller gave up first
call-2(tag=22,dl=5000)=Answered(tag=11) sent=22    ← got call 1's reply
call-3(tag=33,dl=5000)=Answered(tag=22) sent=33    ← got call 2's reply
call-4-GENERATED=Ok(tag=33) sent=44                ← the GENERATED client method, same shift
```

Three facts, each measured:

1. **The shift is permanent.** Not one surplus reply — every subsequent call on that connection reads
   the previous call's answer, forever. Nothing resyncs.
2. **It is silent.** The result is `CallOutcome::Answered` carrying a well-typed `GoResponse::Ok`.
   **No outcome variant expresses "this reply is not yours."** The type system cannot catch it: the
   value is perfectly valid, it is merely *someone else's*. Only the `tag` echoed in the request made
   it visible at all.
3. ⛔ **The generated client method inherits it.** Call 4 goes through `:slow::Svc/go` — the macro's
   own path, the one every service-to-service call in the system uses — and reports `Ok`.

## The mechanism, and why the generated path cannot escape it

`call-by-deadline` (`wat/service.wat:4109`) races the peer against a timer. When the timer wins it
returns `DeadlineFired` and **abandons the in-flight reply on the wire**. The peer handle is returned
to the caller unchanged, so the next `recv` on it reads the abandoned frame.

The generated client method (`wat/service.wat:2708`–`:2715`) maps
`CallOutcome::DeadlineFired → RecvOutcome::TimedOut` and hands the same peer back. **It does not
redial, does not drain, does not mark the peer.** So every `<S>::<Op>` call inherits the hazard.

⚠ `call-by-deadline`'s header comment documents the tier subtlety in detail and says **nothing** about
the peer being unusable afterwards.

## ⭑ The corpus already reasoned about this and could not test it

`the-gate-methods-face-an-outcome/DESIGN.md` ruled **re-recv over re-ask** specifically to avoid it:

> ⛔ **But a re-ask can leave a surplus ack queued on the lineage peer.** If the first ack was merely
> *slow* rather than lost … the loop reads one, and the second stays on the wire … **That is
> character-for-character the frame-desync class that produced today's crash**, and it is silent until
> it detonates somewhere else.

That ruling was made blind — nothing in the tree could produce a slow peer
(`FINDING-we-cannot-induce-a-slow-peer.md`). The reasoning was right, and **understated**: the DESIGN
expected *one* surplus frame; the measurement shows a *permanent* one-frame shift.

## The one place already mitigated, by hand

`wat-scripts/fanout/circuit.wat:1000` — the fanout worker **redials** on `DeadlineFired`, and its
comment (`:1010`) states the rule: *"redial-and-retry, which is right for Lost/Closed/DeadlineFired —
the peer is reachable again and the call may succeed."*

⭑ So the correct disposition is **known and applied in exactly one hand-written place**, while the
macro that generates every other client call does not do it. That is a convention — the bottom rung —
and this finding is what a convention looks like when it is not held.

## ⛔⛔ The interaction with D1-a, which is in flight RIGHT NOW

D1-a converts a raise escaping an op handler into "keep serving". Several of the arms D1-c will
migrate are `TimedOut` arms that currently **`assertion-failed!`** — i.e. today they kill the process,
which **destroys the desynced connection and prevents the corruption.**

★ **The raise is currently masking this bug.** Make it non-fatal without also resyncing, and a loud
crash becomes a silent wrong answer. That is strictly worse.

⚠ Concretely: the publisher at 1 ms × p=3 dies because `-run` asserts on `TimedOut`
(`the-benchmark-has-more-than-one-publisher/SCORE.md`). Its death is the only reason that run does not
silently publish under shifted replies.

**So D1-c's rule must be: a migrated `TimedOut` arm REDIALS; it does not merely continue.**
`circuit.wat:1000` is the shape.

## The fix shape (a direction — the builder's ruling)

Climbing the ladder rather than adding a rule to remember:

1. **Convention** — "redial after `DeadlineFired`". This is today's state in one file. It failed.
2. **Check** — `call-by-deadline` drains or resyncs before returning `DeadlineFired`. ⚠ It cannot know
   how long to wait for a frame that may never come; draining *is* a second deadline.
3. ⭑ **No form** — `DeadlineFired` **consumes** the peer, or carries a fresh one. If the outcome that
   abandons a frame makes the stale handle unreachable, the mistake cannot be written down. That is
   the rung that matches this defect's shape: the bug is *using a handle that is no longer valid*, and
   the type currently says it is fine.

## What this does NOT claim

Not measured: whether the same shift occurs across a **process** boundary under real load rather than
a deliberate 600 ms park (the probe's service is process-tier, but the delay is synthetic); whether
`recv-by-deadline` on the owner path has the same hazard; and whether any *shipped* run has ever hit
it. The corruption is silent, so **"we have never seen it" is not evidence that it has not happened.**
