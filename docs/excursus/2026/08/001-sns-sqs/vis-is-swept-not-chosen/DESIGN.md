# DESIGN — `vis` is swept, not chosen

**`vis` becomes a `run-with` parameter, defaulting to today's value; then it is swept to find where
n=2000 completes.** `wat-scripts/fanout/circuit.wat` only.

The measurement decides the constant. I do not.

## WHY — the retry bound is 1000 s and the test's patience is 215 s

`a reconnect is not an abandonment` did what it was drawn for: `Lost`/`Closed` now redial **and
retry** instead of discarding. The failure line proves it:

```
[0/10][0/10][0/10][0/10]  outbox=0  attempts=8000  elapsed=280581
check-exhausted=0; mark-exhausted=0; ack-retries=1; ack-exhausted=0
```

★★★ **Exactly one stuck batch per queue, and every counter near zero.** Nothing abandoned, nothing
exhausted. The counters are written when a tick *returns*, so this is four workers **still inside a
tick**, riding a retry loop bounded at `limit-ms = vis-ns / 1e6` = **1 000 000 ms**, while the drain
gives up at 280 s.

The system is behaving correctly. **The bound is absurd.**

## ⛔ `vis` IS DOING TWO UNRELATED JOBS

| job | what it wants |
|---|---|
| **visibility timeout** | long enough that a healthy worker acks first; short enough that a stuck message is redelivered |
| **retry bound** | how long a client keeps trying before letting redelivery take over |

I coupled them in the ack stone — `limit-ms = vis-ns / 1e6`, on the reasoning *"after vis expiry
retrying is pointless."* **That reasoning still holds.** The consequence I did not follow through is
that the retry bound then **inherits** whatever `vis` is, and `vis` is 25× the run.

## ⛔ THE BAND, AND WHY I AM NOT PICKING A POINT IN IT

```
ack call deadline           200 ms      the system's own "a queue op should take this long"
worker processing         < 1 ms        measured, flat at every depth
drop-run vis              200 ms        chaos config — redelivery IS the recovery
NON-drop vis        1,000,000 ms        25× the whole run
drain patience        ~215,000 ms       n×m attempts at n=2000
```

`vis` must be **longer than a healthy ack cycle** (or live work is reclaimed) and **shorter than the
drain's patience** (or a stuck message cannot recover in-run). Today's value misses the second by
4.7×; the drop-run's 200 ms would miss the first.

★ That leaves a wide valid band — roughly 1 s to 100 s. **I have derived four constants today and
been wrong about four of them.** The measurements have not been wrong. So this stone parameterises
`vis` and sweeps it rather than reasoning to a number.

## ⛔ THE ONE CONTRACT DECISION

`vis-ms` joins `run-with` as a parameter. **Default is today's behaviour** — `1000000` on non-drop
runs, `200` when any drop rate is set — so every existing caller and every floor test is unchanged.
The sweep then varies it.

★★ **The gate is empirical and binary:** *there exists a `vis` at which `2000 4 3 8192 true`
completes, and the sweep names it.* Not "vis should be X".

## THE FORK — and both branches are informative

- **Some `vis` completes n=2000** → the coupling was fine and only the value was wrong. The sweep
  yields the number, and `ack-exhausted` at that value tells us the real abandonment rate.
- **No `vis` completes n=2000** → `limit-ms = vis-ns / 1e6` is the **wrong coupling**, and the retry
  bound needs to be independent of the visibility timeout. That is a finding no choice of constant
  could have produced.

⚠ **No prediction.** Ten mechanisms have died in this arc.

## WHAT THIS IS AND IS NOT

⚠ **It does not change the default.** Nothing about the shipped configuration moves; this adds a
knob and turns it.

⚠ **It does not explain the slope.** It unblocks the n=2000 point that `the dedupe map stops cloning
itself` could not measure — the PersistentMap win is confirmed at n=1000 (−18 %) and unconfirmed at
n=2000 for want of a completing run.

## OUT OF SCOPE — REJECTED

- **Decoupling the retry bound from `vis`.** That is the *second* branch of the fork above, and
  doing it now would pre-empt the measurement that decides whether it is needed.
- **`circuit.wat:2075`** and **`wat/query/mem.wat`** — the same O(N²) map clone. Named, untouched.
- **The 268-line `-tick` arm** and its 36 nested-Tuple chains. Its own stone; `sqs.wat` is the worked
  example now.
