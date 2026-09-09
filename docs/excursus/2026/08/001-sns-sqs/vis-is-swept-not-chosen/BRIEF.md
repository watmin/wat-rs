# BRIEF — `vis` is swept, not chosen

Make `vis` a `run-with` parameter with today's value as the default, then sweep it to find where
n=2000 completes. `wat-scripts/fanout/circuit.wat` only.

**The sweep names the constant. Do not choose one.**

## Read in order

1. **`DESIGN.md`** — the failure it unblocks and why the band is not picked
   by reasoning.
2. **`docs/excursus/2026/08/001-sns-sqs/a-reconnect-is-not-an-abandonment/SCORE.md`** — the evidence: one stuck batch per queue, every
   counter near zero, workers still inside a tick.
3. **`circuit.wat`, the `vis` binding in `run-with`** — currently
   `(if <any drop rate> 200000000 1000000000000)`. **This becomes the default, not the value.**
4. **`circuit.wat:522`** — `limit-ms (:wat::i64::/ vis 1000000)`, the retry bound that inherits it.
5. **`circuit.wat:2116`** — how `sub-cap` was threaded as a parameter. **Copy that shape.**
6. **`:user::main`'s argv path** — where `sub-cap` and `fill-first?` are parsed. `vis-ms` joins them.

## The work

**1. `vis-ms <- :wat::core::i64` on `run-with`**, appended after the existing parameters.

**2. `0` means "today's default"** — the existing `(if <drop> 200000000 1000000000000)`. Any other
value is used as milliseconds and converted to nanos. This keeps every existing wrapper and every
floor test byte-identical without threading a real value through them.

**3. `:user::main` accepts it** as the next argv position after `fill-first?`, defaulting to `0`.

**4. Nothing else changes.** `limit-ms` keeps its derivation from `vis`; the fork in the DESIGN is
about whether that survives the sweep, and pre-empting it would waste the measurement.

## The sweep

Once the rows pass, on a **quiet box**:

```bash
for v in 1000 2000 5000 10000 30000 100000; do
  ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true $v
done
```

Report per point: completed or `drained-never`, `drain`, `distinct`, `dup`, `seen-skipped`,
`ack-retries`, `ack-exhausted`.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `sqs.wat`, no `wat/`, no `StatsResponse` — no ripple.

## STOP triggers

- **STOP-1** — if `0` cannot mean "the existing default", **STOP and say so.** Do not change what
  existing callers get; every wrapper and floor test must be unchanged.
- **STOP-2** — if the no-args run differs in **any** field, **STOP.**
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not decouple `limit-ms` from `vis`.** That is the fork's second branch and the
  sweep decides whether it is needed.
- **STOP-5** — **do not pick a value to ship.** Report the sweep. The constant is the builder's call.
- **STOP-6** — **leave the tree parsing.**
- **STOP-7** — if **no** `vis` completes n=2000, **STOP and report the whole sweep.** That is the
  finding — the coupling is wrong — and it is worth more than a passing run.

## What is deliberately NOT gated

⚠ `drain` varies run to run, and **more so under load**: I measured n=500 at 549 ms on a quiet box
and 1122 ms while a floor was running. **Sweep on a quiet box**, and check `ps` first.
