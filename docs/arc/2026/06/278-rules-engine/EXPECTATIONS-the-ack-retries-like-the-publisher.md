# EXPECTATIONS — the ack retries like the publisher does

Written **before** the strike. The gate is binary and **already failing today**, in both our hands.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | **the failing run completes** | `… circuit.wat 2000 4 3 8192 true` | `total=8000;distinct=8000;dup=0;seen-skipped=0` — no `drained-never` |
| 2 | `ack-retries` is reported | same, read summary + phases | `ack-retries=` present in **both** |
| 3 | the counter is readable at depth | n=2000 vs n=100 | n=2000 `> 0`; n=100 at or near `0` |
| 4 | the bound is derived | read the `limit-ms` expression | `vis-ns / 1000000`, with the comment saying why — **not a literal** |
| 5 | backoff is the shared one | `grep -n 'backoff-delay' circuit.wat` | now **two** call sites: publisher and ack |
| 6 | no-args unchanged | `… circuit.wat` | summary identical to today, field for field |
| 7 | chaos unchanged | `cargo nextest run --release drop` (and the four `:user::drop-*`) | green, same assertions |
| 8 | curve is not disturbed | n=100/250/500/1000 fill-first | `pairs/sec` within ~10 % of 4348 / 4167 / 3802 / 3244 |
| 9 | the curve extends | n=2000 fill-first | a `drain` and a `pairs/sec` **for the first time** |
| 10 | scripts load | `cargo nextest run --release every_wat_scripts_file_loads` | green |
| 11 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± tests added) |

⚠ **Row 1 is the stone.** Everything else supports it.

⚠ **Row 7 is the one that can quietly go wrong.** Drop runs rely on exhausting the ack and letting
vis expiry redeliver. The vis-derived bound is designed to preserve that **by construction** — if it
does not, the derivation is wrong, and the answer is a better derivation, not a flag. STOP-1.

⚠ **Row 8 guards against banking a coincidence.** No ack is abandoned at n ≤ 1000 today, so this
stone should cost nothing there. A material change either way is a finding to explain.

## Runtime prediction

**45–70 minutes.** One recursive function in the publisher's existing shape, one counter threaded
through the `gave-back` groove, one derived bound. Row 1's run alone is ~4 minutes when it fails and
should be well under that when it passes; the floor is the long pole.

## Trap-doors named in advance

- **Seed threading.** `backoff-delay` returns `(seed', delay)`. Dropping `seed'` and reusing the
  original seed makes every worker draw the **same** jitter every attempt — a synchronised retry
  storm, which is the opposite of what jitter is for. The publisher threads it; copy that.
- **Per-batch, not per-message.** The ack covers a whole `receive :limit 10` batch (`:ids ids`).
  One `ack-retries` increment is one *batch* retry. Say so where it is counted, or the number reads
  10× smaller than a reader expects.
- **`vis` is nanoseconds.** `1000000000000` ns is 1000 s; `200000000` ns is 200 ms. An off-by-1000
  in the divisor gives a 1 s bound on normal runs — which would look like a pass at n≤1000 and fail
  at depth, exactly the current symptom, with a different cause.
- **The redial closure.** Today `once-a` calls `(redial-q)` on `Lost`/`Closed`. The loop must keep
  that, and must not redial on `DeadlineFired` *and* count it as a `Lost` — they are different
  states.
- **n=2000 takes ~4 minutes to fail** and hits the drain's `n×m` bound at 8000 attempts. If row 1
  fails, that is the cost of each attempt; budget for it rather than shortening the bound.

## What this stone does NOT claim

⚠ **It does not explain the 25 % slope** (4348 → 3244 across n=100→1000). That was measured with
zero abandonments, so it has a different cause and remains open.

⚠ **It does not make the ack path correct in general** — it makes it correct **up to the redelivery
window**, which is the only guarantee the system actually offers. A queue that is dead for longer
than `vis` still loses the ack, and the drain's liveness bound is what says so.

★ And if row 1 passes, the honest next question is not "how fast is n=2000" but **"what does
`ack-retries` read there?"** — a run that passes only by retrying thousands of times has found the
contention wall, not cleared it.
