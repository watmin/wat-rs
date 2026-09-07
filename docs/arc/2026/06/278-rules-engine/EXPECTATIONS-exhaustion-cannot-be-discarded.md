# EXPECTATIONS — exhaustion cannot be discarded

Written **before** the strike. The gate is binary and **currently failing in both our hands**, and
this time the mechanism is measured rather than inferred.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **the failing run completes** | `… circuit.wat 2000 4 3 8192 true` | `total=8000;distinct=8000;dup=0;seen-skipped=0` — no `drained-never` |
| 2 | one outcome type, both peer types | `grep -n 'RetryOutcome' circuit.wat` | one `defenum :- [P R]`; constructed and matched at **Seen** and **Queue** |
| 3 | no droppable flag survives | `grep -nE 'a1 |a2 |a3 |mm1|mm2|mm3|second (aa|mm)' circuit.wat` | **no hits** — the ladders are gone |
| 4 | every site names `Exhausted` | read the three call sites | each is a `match` with an `Exhausted` arm; **none discards it** |
| 5 | honest counters | read summary + phases + failure string | one per site, named for what it measures; **`gave-back` is gone** |
| 6 | counters reach the failure path | if row 1 fails, the `drained-never` string | every counter present — as `ack-retries`/`gave-back` are today |
| 7 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; new counters additive |
| 8 | chaos unchanged | `scripts/floor.sh`, count `drop` tests | **38 drop tests pass**, same assertions |
| 9 | curve not disturbed | n=500 and n=1000 fill-first | within **±15 %** of 3802 / 3244 |
| 10 | scripts load | `cargo nextest run --release every_wat_scripts_file_loads` | green, probe included |
| 11 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± tests added) |

⚠ **Row 1 is the stone.** Rows 2–4 are what stop the shape regrowing a fourth time.

★ **Row 9 is deliberately narrower than last time.** The previous stone's ±10 % band covered n=100
and n=250, whose drains are ~100–240 ms on a shared box; grok and I each threw one outlier, at a
different n, in opposite directions, while n=500 and n=1000 sat inside both times. **The band was
tighter than the instrument.** Only the two resolvable points are banded now, and at ±15 %.

## Runtime prediction

**60–90 minutes.** Larger than the last stone: one new parametric enum, two ladders rewritten to the
ack's loop shape, one counter rename threaded through Record → DisruptsResponse → sum → summary →
phases → failure string. Row 1's run is ~4 minutes when it fails and should be well under when it
passes; the floor is the long pole.

## Trap-doors named in advance

- **Seed threading.** Each retry must carry `seed'` forward. Reusing the original seed makes every
  worker draw identical jitter — a synchronised retry storm, the opposite of what jitter is for.
- **Per-batch, not per-message.** All of these cover a whole `receive :limit 10` batch. One counter
  increment is one *batch*. Say so where it is counted, or the number reads 10× small.
- **`vis` is nanoseconds.** `1000000000000` is 1000 s; `200000000` is 200 ms. An off-by-1000 in the
  divisor yields a 1 s bound that passes at n≤1000 and fails at depth — the current symptom with a
  different cause.
- **The check ladder returns a reply the caller needs.** `once` yields `Option<CheckResponse>`;
  `Got` must carry it. The mark site ignores its reply — that is fine, but it must still match.
- **Renaming `gave-back` touches five places** (Record field, `DisruptsResponse`, `sum-disrupts`,
  summary, phases) plus the failure string. Missing one is a compile error, not a silent drift —
  but check the failure string, which is a `format` template and will not fail to compile.

## What this stone does NOT claim

⚠ **It does not explain the 25 % slope** (4348 → 3244, n=100→1000). That was measured with
`gave-back=0` on every point, so the slope has a different cause and stays open.

⚠ **It does not de-duplicate the backoff formula**, which now exists in three copies with `100`
hardcoded in two. Named, recorded, and its own stone.

★ And if row 1 passes, the honest next question is not "how fast is n=2000" but **"what do the
exhaustion counters read there?"** A run that passes while abandoning nothing has cleared the wall;
a run that passes while its counters climb has merely moved it.
