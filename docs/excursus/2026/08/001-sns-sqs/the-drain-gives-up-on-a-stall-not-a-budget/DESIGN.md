# DESIGN — the drain gives up on a stall, not a budget

## Why

`poll-until-drained*` is bounded in **attempts**, and its verdict cannot distinguish two completely
different worlds:

- *the system stopped delivering* (a correctness failure), and
- *the poller ran out of budget while measuring* (a harness artifact).

Both print `drained-never`. Measured this session (`088f2669f`), the second is what has been
happening:

| condition | result |
|---|---|
| alone on a quiet box, `sub-cap` 32 / 64 / 128 / 256 | **PASS, all four** |
| 8 concurrent copies of the identical command | **4 of 8 FAIL** |

And the arithmetic that names the mechanism was already inside the error message:

```
100 attempts × 5 ms intended sleep  =   500 ms
elapsed, measured                   =  1627–2083 ms
```

**Two to three times the intended budget is spent observing.** Each attempt makes
`count(qclients) + 1 = 3` stats round-trips, so a run burns ~300 of them into the very queue
processes the workers need in order to progress.

Three hypotheses died establishing this, all recorded in
`the-chaos-gate-does-not-exist/FINDING.md`: backpressure via `sub-cap` (refuted — 32 passes alone),
`outbox=19` being seed-determined (refuted — 9/10/19 with the same seed), and the budget losing a
race against redelivery (refuted — `vis-ms` 200/50/20 gives 4/8, 4/8, 3/8).

## What it delivers

A verdict that says which world it is, and a loop that is **load-independent by construction**:
a slow box polls longer instead of failing.

Three distinct outcomes replace one ambiguous one:

| verdict | meaning |
|---|---|
| `""` (drained) | unchanged — every sub queue empty **and** topic inbox 0 |
| `drained-stalled` | **no delivery progress for K consecutive polls.** The system stopped. A real finding |
| `drained-timeout` | a generous wall-clock ceiling hit **while still progressing**. Slow, not stuck |

## ⭑ The progress signal is already arriving, free

`:fanout::depth-of` calls `Queue/stats` and receives the **whole 19-field `:queue::Stats`**, then
keeps two fields:

```
((:queue::Queue::StatsResponse::Ok qst)
  (:wat::core::Tuple (:queue::Stats/visible qst) (:queue::Stats/unacked qst)))
```

`(:queue::Stats/acks qst)` is in that same reply and is **monotone — acks only ever increment.**
Summing it across `qclients` gives a delivery-progress signal at **zero additional round-trips**, so
the observer effect does not worsen. The poller has been throwing away the instrument it needed.

★ Fourth time this session the exact instrument was already in hand and something weaker was used —
after the `attempts`/`elapsed` arithmetic in this very error message, `refused`/`publish-calls` on the
report line, and `attempts = calls + retries + partials` for partial accepts.

## The one contract decision

**`acks` detects STALL. It does not detect COMPLETION.** Under redelivery an entry is delivered and
acked more than once, so `Σ acks` can exceed `n×m` and is not an equality test. The completion
condition is unchanged: `sweep-drained?` **and** `box = 0`. `acks` answers only *"did anything move
since the last poll?"*

## Out of scope = rejected

- **Reducing the poll's cost or cadence.** The observer effect is real and measured (2–3× the
  budget), but making the poll cheaper or rarer is a *different* change with its own measurement.
  This stone makes the verdict honest; it does not make the poller cheap. **Affirmatively cut.**
- **The seven chaos tests' assertions and ignore status.** They cannot be settled until the check is
  trustworthy — that is what this stone delivers. Next stone, not this one.
- **Wiring the inbox into fault injection** (`circuit.wat:2178`'s three hardcoded zeros). Still the
  highest-value item on the list and still separate.
- ⚠ **Answering whether the 9–19 stuck entries would ever deliver.** This stone makes the question
  *answerable* — a `drained-stalled` verdict says no, a `drained-timeout` says probably — but it does
  not answer it. That is the point of building the instrument first.

## ⚠ Consequence that must be stated, not discovered

**Changing the poll loop changes every future `drain` number's comparability to banked baselines.**
`drain` is measured *through* this poller, so its wall time moves when the loop changes. Given the
instrument's ~1 % session offset and that we are no longer chasing sub-1 % effects, this is
acceptable — but the SCORE must **re-baseline** rather than compare against 1194 / 2506 / 5305, and
no later stone may cite a pre-change `drain` figure against a post-change one.

## Baseline for the correctness rows

`circuit.wat 50 2 2 32 false 0 0 1000 42` — passes alone, fails 3–4 of 8 under self-contention.
That asymmetry is the thing this stone must remove: **the same scenario must reach the same verdict
whether the box is idle or contended.**
