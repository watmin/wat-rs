# DESIGN — the topic names which send failed

**Measurement only.** Three counters where there is one. No behaviour change, no new machinery.

## Why — 86 % of publish attempts fail, and we cannot say why

n=2000, `vis-ms=1000`, my own run:

```
setup=12491  fill=17090  arm=8  drain=2439  collect=5915  total=37981
  33%          45%                 6.4%       16%

publish-calls=200   publish-attempts=1447   full-retries=1247   asleep=8421
```

★★ **`fill` is 45 % of the run and 49 % of `fill` is the publisher waiting.** The drain — eleven dead
mechanisms' worth of attention — is 6.4 %.

And the rejections are not capacity. `sns-fanout.wat:186-208`: every `Accepted 0` sits on
`RecvOutcome::Lost`, `Closed`, or `TimedOut` of the topic's **inbox send**, each carrying the same
comment:

> *"Do not claim Accepted n — the inbox write is unknowable. Accepted 0 is the caller's retry."*

The success path returns `Accepted floor` (`:115`, `floor = pairs / nsubs`). So **1247 of 1447 attempts
were failed inbox sends**, each costing a `connect` redial plus a guessed backoff.

★★★ This is not "the fill phase is slow". It is **"the transport fails most of the time and the retry
is expensive."**

## ⛔ What we cannot say, and why it blocks every fix

``full-retries`` collapses three unrelated failures into one number:

| arm | what it means | what it would implicate |
|---|---|---|
| `TimedOut` | the send deadline expired | the deadline, and the write path rebuilt today |
| `Closed` | the peer hung up | connection lifetime / reuse |
| `Lost` | the peer died | a reliability bug, in a run reporting `dup=0 distinct=8000` |

**Three worlds print the same line.** Guessing `TimedOut` and tuning a deadline, when the truth is
`Closed`, is tuning a constant against the wrong failure — the exact shape of the eleven dead
mechanisms.

## The one contract decision

**`:demo::Topic::StatsResponse::Ok` grows from 2 fields to 5** — `depth`, `ticks`, plus
`inbox-lost`, `inbox-closed`, `inbox-timedout`. Three `i64` counters on the topic's `:durable`,
incremented in the arms that already exist.

Nothing else changes: the retry semantics, the `Accepted 0` reply, the redial, and the batch contract
`:max-entries [msgs 10]` all stay exactly as they are.

## ★ The row that gates my own analysis

The three counters must **sum to the publisher's observed rejections**. If
`inbox-lost + inbox-closed + inbox-timedout ≠ full-retries`, there is a **fourth** source of
`Accepted 0` that this DESIGN did not find — and that is a more valuable result than the split itself.

## OUT OF SCOPE — REJECTED

- **Fixing whatever the arms reveal.** That is the next stone and it needs this measurement first.
- **The `-1 -1` sentinels on `stats`'s own failure arms** (`:235 :249 :250`). Same collapse, different
  surface; naming it here would widen the stone past a measurement.
- **Raising `:max-entries [msgs 10]`.** A batch limit is contract, never raised to fit a caller.
- **`setup`** — the builder's ruling: cold boot, not this session.

## Files

`wat-scripts/topic/sns-fanout.wat` (the counters, the three arms, the stats reply) and
`wat-scripts/fanout/circuit.wat` (two consumers + the report line). **8 sites, 2 files.**
