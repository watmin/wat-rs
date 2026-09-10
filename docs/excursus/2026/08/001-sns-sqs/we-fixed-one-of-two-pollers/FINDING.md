# FINDING — we fixed one of two pollers, and the other costs 356 seconds

Builder, on seeing a captured red: *"'356 seconds'… what? we brought the loop down to like 20s I
thought?"* The standard run is **22.5 s total.** A 356-second *fill phase* is **16× the entire run.**

## The red, captured whole at `ae2c15622`

```
filled-never: last=[2010/0][2010/0][2010/0][2010/0] outbox=0 want=2000
              attempts=8000 elapsed=356669
circuit.wat:2652  :fanout::require!  ←  :fanout::poll-until-filled*
```

## ⛔ THE MECHANISM: `sweep-filled?` COMPARES WITH `=`, NOT `>=`

`:fanout::sweep-filled?`:

```
(:wat::core::and (:wat::core::= (:wat::core::first d) n)     ← visible must EQUAL n, exactly
                 (:wat::core::= (:wat::core::second d) 0))
```

**Every sub queue reported 2010 against `want=2000`.** With `=`, the exit condition is not merely
unmet — it is **permanently unreachable.** The poller then burns its entire budget of `n × m = 8000`
attempts and reports `filled-never`.

★ `>= n` is the correct comparison. **More-than-wanted is not a failure of *filling*.** Under `=`, an
overshoot is indistinguishable from an under-fill, and both cost the full budget.

## ⛔⛔ AND IT IS THE EXACT CLASS WE FIXED ON THE OTHER POLLER

`18a86fe27` replaced `poll-until-drained*`'s attempt budget with a **progress-bounded stall detector**,
because *"one verdict — `drained-never` — covers two different worlds: the system stopped delivering,
and the poller ran out of budget while measuring."* It now emits `drained-stalled` and
`drained-timeout` as distinct verdicts, and the file documents them at `:1444-1445`.

**`poll-until-filled*` was never touched.** It is still attempt-bounded, still single-verdict, and its
condition can be *unsatisfiable* rather than merely unmet — which is strictly worse than what the drain
poller had before the fix.

★★ **We fixed one of two pollers and never asked whether the sibling had the same defect.** That is the
same shape as the three magic constants found today — `mk-tw`'s 5 s against six 200 ms siblings,
`worker`'s 250 ms against `held-worker`'s 50 ms — **a fix applied to one site while its sibling kept the
defect.** Fourth instance today.

## ⭑ And the cost, which the code computes on itself

The poller's own accumulator is `rts' = rts + (count qclients) + 1` — it counts **5 crossings per
attempt** (4 sub queues + the topic).

```
8000 attempts × 5 crossings   = 40 000 round-trips in ONE failed fill
the entire SUCCESSFUL run     = 11 250
                                → 3.6× the whole run's budget, in one failed phase
8000 × 5 ms intended sleep    = 40 s
measured elapsed              = 356 s
                                → ~317 s is the poller's own stats traffic
```

★★★ Under the builder's **networking-first** ruling this is the sharpest number of the day: **a single
failed fill makes 40 000 process-boundary crossings.** At a cross-region RTT it would not finish.

## Two defects, and the second is the operational one

| | what | severity |
|---|---|---|
| **(a)** | **The overshoot** — 2010 delivered where 2000 wanted, exactly one batch of 10 | a correctness question, unexplained |
| **(b)** | **`=` instead of `>=`** — a benign overshoot becomes a 356 s hang and 40 000 crossings | the operational defect |

⚠ **(b) is real independent of (a).** Fix `>=` and the overshoot becomes a pass rather than a hang —
which is *not* the same as explaining it, and must not be allowed to hide it. **A fill that overshoots is
still a fact that wants a mechanism.**

★ And the coincidence nobody has claimed: the ack-drift law measured in the same stone is
`excess acked ids = 10 × ack-retries`, and this overshoot is exactly **10**. Both involve a batch of ten
being re-presented. **A hypothesis, not a finding.**

## What must be true of any fix

⛔ **The fill poller needs what the drain poller got, not just a changed operator.** Swapping `=` for
`>=` removes *this* unsatisfiable condition; it leaves an attempt-bounded budget whose verdict still
cannot distinguish *"the system stopped filling"* from *"the poller ran out of budget."* The drain
poller's own header records why that distinction was worth a stone.

⚠ And the sibling audit is now owed: **`sweep-unread?`, `snapshot-str`, and any other predicate shared
between the two pollers should be checked for the same one-sided fix.**
