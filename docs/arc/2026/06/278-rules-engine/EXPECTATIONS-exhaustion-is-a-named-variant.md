# EXPECTATIONS — exhaustion is a named variant

Written **before** the strike. The gate is binary and **currently failing in both our hands**, and
the mechanism behind it is measured, not inferred.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **the failing run completes** | `… circuit.wat 2000 4 3 8192 true` | `total=8000;distinct=8000;dup=0;seen-skipped=0` — no `drained-never` |
| 2 | two **closed** enums | `grep -nE 'defenum :fanout::(Seen\|Queue)Retry' circuit.wat` | both present, **neither carries `:- [`** |
| 3 | keyword-matched in the child | read the three call sites | `(:fanout::SeenRetry::Exhausted …)` style arms — **no hash-destructure, no bare `_` standing in for a variant** |
| 4 | no droppable flag survives | `grep -nE 'a1 \|a2 \|a3 \|mm1\|mm2\|mm3' circuit.wat` | **no hits** |
| 5 | every site names `Exhausted` | read the three matches | each has an `Exhausted` arm; **none discards it** |
| 6 | honest counters | summary + phases + failure string | one per site, named for what it measures; **`gave-back` is gone** |
| 7 | counters reach the failure path | if row 1 fails, the `drained-never` string | every counter present, as `ack-retries`/`gave-back` are today (`:2153`) |
| 8 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; new counters additive |
| 9 | chaos unchanged | `scripts/floor.sh`, count `drop` tests | **38 drop tests pass**, same assertions |
| 10 | curve not disturbed | n=500 and n=1000 fill-first | within **±15 %** of 3802 / 3244 |
| 11 | scripts load | `cargo nextest run --release every_wat_scripts_file_loads` | green, probe included |
| 12 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± tests added) |

⚠ **Row 1 is the stone.** Rows 2–5 are what stop the shape regrowing a fourth time.

⚠ **Row 3 is the one that can be satisfied dishonestly.** The checker will *suggest*
hash-destructure whenever a match goes `Open`. Accepting that suggestion produces a green build and
**destroys the stone** — `MatchShape::Open` determines no variant shape and tracks exhaustiveness by
wildcard alone, so it is exactly as droppable as the `bool`. If a match will not close, that is
STOP-1, not a licence to destructure.

★ **Row 10 is deliberately narrow.** An earlier ±10 % band covered n=100 and n=250 — drains of
~100–240 ms on a shared box — and grok and I each threw one outlier, at a different n, in opposite
directions, while n=500 and n=1000 sat inside both times. Only the two resolvable points are banded,
at ±15 %.

## Runtime prediction

**60–90 minutes.** Two enums, two ladders rewritten to the ack's shape, one rename threaded
Record → DisruptsResponse → sum → summary → phases → failure string. Row 1's run is ~4 minutes when
it fails and should be well under when it passes; the floor is the long pole.

## Trap-doors named in advance

- **Positional variant constructors.** `(:fanout::SeenRetry::Got peer reply)`, not `:peer … :reply …`.
  The keyword form is an arity error, verified this session.
- **Seed threading.** Each retry carries `seed'` forward. Reusing the original seed makes every
  worker draw identical jitter — a synchronised retry storm, the opposite of what jitter is for.
- **Per-batch, not per-message.** All three cover a whole `receive :limit 10` batch. One counter
  increment is one *batch*. Say so where it is counted, or the number reads 10× small.
- **`vis` is nanoseconds.** `1000000000000` is 1000 s; `200000000` is 200 ms. An off-by-1000 yields a
  1 s bound that passes at n≤1000 and fails at depth — the current symptom, different cause.
- **The check ladder's reply is needed.** `once` yields `Option<CheckResponse>`; `Got` must carry it.
  The mark site ignores its reply — fine, but it must still match.
- **Renaming `gave-back` touches five places** plus the failure string. Four are compile errors if
  missed; the failure string is a `format` template and **will not fail to compile.**

## What this stone does NOT claim

⚠ **It does not fix the class.** The backoff formula stays in three copies (`:1204`, `:1540`,
`:675`) with `100` hardcoded in two, because an `EmptyEnv` child cannot see the file's own `defn`s.
The real fix is one generic top-level `defn` in `wat/` — **the builder's ruling, on the precedent's
own terms, and not on the table while the code that would move is broken.**

⚠ **It does not explain the 25 % slope** (4348 → 3244). Measured with `gave-back=0` on every point,
so it has a different cause and stays open.

★ If row 1 passes, the honest next question is not "how fast is n=2000" but **"what do the
exhaustion counters read there?"** A run that passes while abandoning nothing has cleared the wall;
a run that passes while its counters climb has merely moved it.
