# NOTE — a callee's `:deadline-ms` is INERT; a caller-side deadline is real

**Filed 2026-09-13** while drawing `the-store-can-fail/` (the store-fault injector). Filed here, flat at
the excursus root, because **a finding is not a task** (this directory's README) and because minting or
aiming an arc is the builder's ruling.

## The fact, measured twice

`wat/service.wat:2442` documents the clause:

> *"Optional `:deadline-ms` on the service. Default 10000. Never off — a deadline …"*

**Declaring it on the service being CALLED does nothing to what its caller waits.**
`wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat` — a `:satisfies`-mode service declaring
`:deadline-ms 300`, whose `ping` handler parks on a 1-hour timer so no reply is ever sent:

```
outcome=TimedOut;elapsed-ms=10000;declared=300
```

**10 000 ms observed against 300 declared.** The generated client method uses the default.

★ And the other half, found by the strike's STOP-5: **a CALLER-side deadline is real.**
`call-by-deadline` on the peer with **20 000 ms** waits out a callee that takes 10 000, and receives the
real reply. So the deadline that governs a round-trip is the **caller's**, and the callee's declaration is
not consulted.

## Why it matters, concretely

Two consequences already paid for:

1. **A fault injector cannot shorten its own timeout.** `the-store-can-fail`'s proxy wanted
   `:deadline-ms 200` so a suppressed reply would cost 200 ms. It cannot; every dropped reply costs the
   full **10 s**, which is now a sizing constraint on every chaos harness that uses it.
2. ⛔ **Two 10 000 ms deadlines RACE.** The generated `Queue/send` is also 10 000, so when the store times
   out at 10 000 the client sees *its own* `TimedOut` rather than the queue's real answer. The strike had
   to call with **20 000** to observe `Accepted 0` at all. **Anyone measuring a downstream timeout through
   a generated client method will see the wrong outcome unless they widen the caller's deadline.**

## What is NOT established

⚠ **Where `:deadline-ms` IS honoured is unmeasured.** The plausible reading — that it governs calls the
service's own handlers *make* — was **not tested**, and this note deliberately does not assert it. Nor is
it established whether the clause is inert only in `:satisfies` mode (where the surface owns the client
methods) or in `:ops` mode too. **Both are one probe each; neither has been run.**

⛔ So the honest statement is narrow: *a `:satisfies`-mode callee's `:deadline-ms` does not change what its
caller waits.* Do not widen it into "`:deadline-ms` does nothing" without the probes.

## The shape of a fix, if the builder wants one

A clause that is accepted, documented, and has no observable effect in the mode it is most likely to be
written in is a **convention wearing a wall's clothes**. Three candidate dispositions, none taken here:

1. **Honour it** — the surface's generated client method reads the target service's declaration.
2. **Refuse it** — `defservice` rejects `:deadline-ms` in `:satisfies` mode with a diagnostic naming the
   caller-side alternative. Cheapest, and makes the silence impossible.
3. **Rename it** to say whose deadline it is (`:outbound-deadline-ms`?), if reading (1) is what it means.

★ (2) is the extirpare-shaped answer: if the clause cannot do what a reader will expect, the mistake
should be unwritable rather than silent.

## Provenance

- Probe: `wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat` (committed `004457a5d`).
- Drawn in: `the-store-can-fail/DESIGN.md` §"THE COST IS FIXED AT 10 s AND I PROBED WHY".
- Caller-side half: `the-store-can-fail/SCORE.md` §STOP-5, graded `acc11e5b9`.
- Kin: `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md` — the other
  place a declaration promises more than the substrate delivers.
