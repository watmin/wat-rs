# DESIGN — the publisher gives up in time, not in tries

Builder: *"let's get it in motion"* — stone 3 of the chain confirmed at D1-(1), after
`a-peer-remembers-its-address` and `a-fired-deadline-hands-back-a-live-peer`.

**Drawn 2026-09-15. NOT STRUCK.**

## WHY — measured after stone 2, not assumed

With the slow-topic injector armed (`:delay-bp 10000 :delay-ms 11000`), via the record:

```
exit=2      "publisher stats closed" ×3
```

**The publisher still dies.** Stone 2 repaired the *handle*; the publisher's `TimedOut` arm
(`circuit.wat:2270`) still `assertion-failed!`s, so the process still goes. Two defects, both in the
same fold (`circuit.wat:2274`, `(range 0 256)`):

1. ⛔ **The give-up has no form.** On `TimedOut`/`Lost`/`Stopped`/`Closed` it raises. That is the
   crash the builder named.
2. ⛔ **The bound is an attempt count, and stone 2 just made it unusable.** Each timeout now costs
   the 10 s deadline **plus a redial**, so 256 attempts is a **~43-minute** bound. Simply not-raising
   would convert a fast death into a very slow one. ⭑ Stone 2 changed the regime; this bound expired
   with it.

## ⭑ TWO STRUCK RULINGS GOVERN, AND THEY ARE NOT IN CONFLICT

They very nearly got mixed in this DESIGN, so both are cited:

| ruling | governs | says |
|---|---|---|
| `a-give-up-has-no-form-for-attempts` | **parent-side pollers** (`poll-until-*`, `join-publishers`) | `Verdict` has **no `Exhausted` variant, ever** — a poller must not give up on a count |
| `exhaustion-is-a-named-variant` | **service-internal retry ladders** (the worker's check/mark/ack) | every ladder returns a **closed enum whose `Exhausted` the match must name**, and the laggards get *"the vis-bounded backoff the ack path already has"* |

The publisher's fold is the **second** kind. So the shape is the worker's, not the pollers'.

⭑ **And the ack ladder's bound is derived from a DOMAIN FACT, not a number:**
`limit-ms = vis-ns / 1000000` because *"after vis expiry the message is visible again and retrying is
pointless"* (`circuit.wat:667–669`), then `elapsed >= limit-ms → SeenRetry::Exhausted 0` (`:684–686`).

## ⛔ THE ONE CONTRACT DECISION

**The publisher's publish ladder returns a closed enum whose exhaustion variant the match must name,
and that variant carries ELAPSED and CEILING milliseconds — never an attempt count.**

That honours both rulings at once: a *named* variant the match cannot drop (`exhaustion-is-a-named-variant`)
whose payload is *time* (the spirit of `a-give-up-has-no-form-for-attempts`).

⚠ **The publisher has no `vis` to derive a bound from.** Its job is "publish n messages"; nothing about
a slow topic makes retrying *pointless*, only expensive. So the ceiling must be **declared**, and
`:fanout::Input` is now the place a declared knob lives — a field, `None`-defaulting to today's
effective behaviour. A derived bound would be invented; a declared one is honest.

## Out of scope = REJECTED

- ⭐ **The worker's `(range 0 256)` at `:747`.** Checked, not skipped: it **already has the blessed
  shape** — it returns `:fanout::SeenRetry::Exhausted` rather than raising, per
  `exhaustion-is-a-named-variant`. ⚠ Its *count*-shaped payload is the older spelling, and changing it
  would re-litigate a struck stone's ruling on a path with no measured defect. **Named here as the
  sibling that was examined and found sound**, so nobody reads this stone as a one-sided fix.
- **`require!`.** It raises on `Stalled`/`Ceiling` by design — the *caller's* choice
  (`require-stopped` / `stop-faced` is the same pair). The publisher needs the **faced** consumer, not
  a change to `require!`.
- **`recv-by-deadline`.** Still unmeasured after stone 2; its own stone.
- **The stdin-blocking diagnostic.** Invoking without a piped record blocks and then reports a
  `TimedOut` on an internal stdio peer — confusing, already flagged in
  `the-harness-takes-a-record/SCORE.md` finding 1. Not this stone.

## Blast radius

```
wat-scripts/fanout/circuit.wat   the publisher's fold (:2274) + its 6 raising arms (:2263-:2271),
                                 a closed outcome enum, its faced consumer, one Input field,
                                 and the phases line so the give-up is observable
elsewhere                        0 expected — VERIFY, do not assume
```

## Trap-doors named up front

1. ⛔ **A ceiling that is shorter than one deadline is a no-op ladder.** The publisher's generated
   deadline is 10 s; a ceiling below that gives up before the first attempt can finish. State the
   relationship, and gate on it.
2. ⛔ **Do not let the give-up become silent.** A publisher that quietly stops publishing turns a
   crash into missing data, and the completion check (`distinct=8000`) would then fail with no reason
   attached. The exhaustion must reach the report.
3. ⚠ **`:fanout::Input` gained fields last stone; every construction site must grow** — the record's
   fields are mandatory and `:user::run*`/`run-p*`/`run-chaos*`/`run-drop*`/`delay-full-rate` all
   construct it. That is the same "durable fields are mandatory" blast the injector hit at 7 files.
4. **The fold's accumulator is a nested Tuple.** ⚠ Threading a clock through it re-opens what
   `the-waiter-folds-carry-a-named-aggregate` closed; prefer reading the clock at the boundary, or say
   why not.
5. **`cargo build --release` does not compile tests.**
