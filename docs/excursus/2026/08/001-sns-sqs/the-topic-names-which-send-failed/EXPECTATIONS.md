# EXPECTATIONS — the topic names which send failed

Written **before** the strike. Measurement only: the three numbers are the deliverable.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **the three arms are counted separately** | read the diff | `inbox-lost`, `inbox-closed`, `inbox-timedout` — three `:durable` counters, incremented in the arms that already exist, **never summed into one** |
| 2 | ★ **the report line carries all three** | run n=2000 `vis-ms=1000` | three named numbers in the circuit's report, beside `full-retries` |
| 3 | ★ **they account for the rejections** | the same run | `inbox-lost + inbox-closed + inbox-timedout` **equals** the publisher's rejection count, **or** the SCORE names the gap. A gap is a **finding**, not a failure — it means a fourth source of `Accepted 0` |
| 4 | **no behaviour changed** | read the diff | the `Accepted 0` reply, the redial, the retry semantics and `:max-entries [msgs 10]` all identical |
| 5 | **existing entries unaffected** | the no-arg run and each wrapper | reported completeness fields identical to today |
| 6 | **blast radius** | `git diff --stat` | `sns-fanout.wat` + `circuit.wat`. **8 sites**: producers `:221 :235 :249 :250`, consumers `:720 :728` and `circuit.wat:1100 :1110` |
| 7 | **the stats sentinels are left alone** | `git diff` | `:235 :249 :250` keep their `-1` values — same collapse, different surface, out of scope |
| 8 | **the box was quiet for the measured run** | the SCORE | load stated at start |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5235 passed, 22 skipped, **0 FAIL, 0 TIMEOUT**, quiet box |

## The rows that carry it

★ **Row 1 is the stone.** One number covering `Lost`, `Closed` and `TimedOut` is why no fix can be
chosen: three worlds print the same line. Each implicates something different — a deadline, connection
lifetime, or a genuine reliability bug in a run reporting `dup=0`.

★★ **Row 3 gates my own analysis, not the executor's work.** I claim every `Accepted 0` comes from
those three arms. If the counters do not add up, I missed one — and I would rather that surface here
than after a fix is built on the claim.

⚠ **Row 4 is what keeps this a measurement.** Any temptation to also *fix* the arm the numbers
implicate belongs to the next stone. A measurement that changes what it measures is not a measurement.

## Runtime prediction

**30–50 minutes.** Three counters, four producer sites, four consumer sites, one report line. One
n=2000 run at ~41 s. The floor is the long pole.

## Trap-doors

- **The counters live on `:durable`**, so they survive a tick; a `:ephemeral` counter would reset.
- **`StatsResponse::Ok` arity 2 → 5 breaks every consumer** — the two in `sns-fanout.wat` and the two
  in `circuit.wat`. The compiler will name them; do not assume there are only two.
- **Increment where the arm already is**, not in a wrapper that guesses which arm ran.
- **`stats`'s own `-1 -1` arms are not the publish arms.** Do not conflate them while editing nearby.
- **A zero is a result.** If `inbox-lost` is 0 across the run, that is worth stating, not hiding.

## What this stone does NOT claim

⚠ It does **not** fix anything. It names which failure dominates so the next stone can.
⚠ It does **not** touch `setup` (the builder's ruling), the batch contract, or the drain.
