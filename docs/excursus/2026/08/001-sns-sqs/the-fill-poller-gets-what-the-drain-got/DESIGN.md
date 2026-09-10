# DESIGN — the fill poller gets what the drain poller got

## Why

**Builder's ruling: "(b) sounds like priority."** From `43efddb6a`:

```
filled-never: last=[2010/0][2010/0][2010/0][2010/0] outbox=0 want=2000
              attempts=8000 elapsed=356669
```

**356 seconds in a phase, in a run whose entire normal duration is 22.5 s.** And the code prices its own
cost: `rts' = rts + (count qclients) + 1`, so **5 crossings per attempt × 8000 = 40 000 round-trips in
one failed fill** — **3.6× the whole successful run's budget of 11 250.**

**The mechanism is one operator.** `:fanout::sweep-filled?`:

```
(:wat::core::and (:wat::core::= (:wat::core::first d) n)     ← visible must EQUAL n, EXACTLY
                 (:wat::core::= (:wat::core::second d) 0))
```

Every queue reported **2010** against `want=2000`, so the condition is not unmet — it is
**unsatisfiable.** The poller then spends its whole budget proving something that can never become true.

## ⛔ And this is the class already fixed on the sibling

`18a86fe27` rebuilt `poll-until-drained*` because *"one verdict covers two different worlds: the system
stopped delivering, and the poller ran out of budget while measuring."* Its header now states the
property to preserve above all:

> *"⛔ THE CHECK CAN STILL GO RED… the ONLY path returning `""` is the completion test, and every other
> path is bounded — the stall arm by K, and the ceiling arm unconditionally by wall clock regardless of
> progress."*

**`poll-until-filled*` never received it.** It is attempt-bounded, single-verdict, and — worse than the
drain poller ever was — its completion test can be *unsatisfiable*.

★★ **Fourth instance today of a fix applied to one site while its sibling kept the defect**, after
`mk-tw`'s 5 s against six 200 ms siblings and `worker`'s 250 ms against `held-worker`'s 50 ms.

## What it delivers

The fill poller mirrors the drain poller: **give up on lack of progress, not on an attempt budget, and
say which world it is in.**

| verdict | meaning |
|---|---|
| `""` | the completion test — **`>=` n**, unchanged otherwise |
| `filled-unread` | a stats reply could not be read — outranks both, unchanged |
| `filled-stalled` | the progress signal did not move for K consecutive polls: **the system stopped filling** |
| `filled-timeout` | the wall ceiling was reached with **no** K-poll stall ever seen: slow, not stuck |

## ⭑ The progress signal, and why the obvious one is wrong

The drain poller uses **Σacks** — monotone because acks only increment. The obvious mirror for fill is
**Σvisible**, and it is **wrong**: with `fill-first? = false` the consumers are armed *before* the fill,
so `visible` falls as messages are consumed and the signal is not monotone.

★ **The honest signal is `Σ(visible + unacked + acks)`** — everything that ever arrived, whether still
present, claimed, or already consumed. Monotone under **both** `fill-first?` modes, and **free**:
`sweep-of` → `depth-of` already returns `(visible, unacked, acks)` per queue (`circuit.wat:1181`). One
sweep per iteration, exactly as today.

## The one contract decision

⛔ **`=` becomes `>=` on the visible term, and that is a semantic change, not a typo fix.** It declares
that **more-than-wanted is not a failure of filling.**

⚠ **And it must not hide the overshoot.** Under `>=`, the 2010 case becomes a **pass** — which is not the
same as explaining it. So the stone **reports the overshoot rather than swallowing it**: when the
completion test passes with `visible > n`, the summary carries the excess. A fill that delivers 2010 of
2000 is still a fact that wants a mechanism, and the poller must not be the thing that buries it.

## Out of scope = rejected

- **The overshoot's mechanism** (defect (a)). Unexplained, and it stays unexplained by this stone — which
  is why the excess must be *reported*. ★ The unclaimed coincidence: the ack-drift law measured at
  `ae2c15622` is `excess acked ids = 10 × ack-retries`, and this overshoot is exactly **10**.
- **`fill-first? = false` completion semantics.** If consumers drain during fill, `visible >= n` may never
  hold. That is today's behaviour and a separate question; the progress signal above is chosen so the
  *give-up* path is correct in both modes even though the *completion* path is unchanged.
- `Store::DeleteResponse::Success`'s missing count, the maintained-depth patches, `fill`'s `p=1` axis,
  tier-1 fault injection.

## ⚠ The sibling audit this stone owes

Having found one one-sided fix, the same question must be asked of every predicate the two pollers share:
**`sweep-unread?`, `snapshot-str`, `sweep-drained?`** — and of `poll-until-filled`'s budget expression at
`circuit.wat:2650` (`n × m`), which is the same "attempts, not progress" shape.

⚠ **Report the audit even where it finds nothing.** *"Checked, and the sibling is fine"* is the result
that stops this recurring — an unasked question is how four of these got here.

## ⚠ What must not be claimed

⚠ **No speedup.** A successful fill never reaches the give-up path, so the happy path is untouched and its
timings must not move. **The win is entirely in the failure path** — 356 s and 40 000 crossings become
bounded — and that path does not appear in a green run's numbers at all.
