# EXPECTATIONS — the outcome crosses, the resource stays

Written **before** the strike. Binary gate, currently failing in both our hands. The enum half has
already been measured green at n=12.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **the failing run completes** | `… circuit.wat 2000 4 3 8192 true` | `total=8000;distinct=8000;dup=0;seen-skipped=0` — no `drained-never` |
| 2 | nullary closed enums | `grep -nE 'defenum :fanout::(Seen\|Queue)Retry' -A 3 circuit.wat` | `:Got []`, `:Exhausted [attempts <- i64]`; **no `:- [`, no Peer, no Reply** |
| 3 | keyword-matched in the child | read the three call sites | `(:fanout::SeenRetry::Exhausted _a)` style arms — **no hash-destructure, no `_` standing in for a variant** |
| 4 | no droppable flag survives | `grep -nE 'a1 \|a2 \|a3 \|mm1\|mm2\|mm3' circuit.wat` | **no hits** |
| 5 | every site names `Exhausted` | read the three matches | each has an `Exhausted` arm; none discards it |
| 6 | honest counters | summary + phases + failure string | one per site; **`gave-back` gone** |
| 7 | counters reach the failure path | if row 1 fails, the `drained-never` string | every counter present (`:2153`) |
| 8 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; counters additive |
| 9 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 10 | curve not disturbed | n=500 and n=1000 fill-first | within **±15 %** of 3802 / 3244 |
| 11 | scripts load | `cargo nextest run --release every_wat_scripts_file_loads` | green |
| 12 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed (± added) |

⚠ **Row 1 is the stone.** Rows 2–5 stop the shape regrowing a fourth time.

⚠ **Row 3 can be satisfied dishonestly** and the checker will suggest how. Hash-destructure builds
green and destroys the stone — `MatchShape::Open` determines no variant shape and tracks
exhaustiveness by wildcard alone. If a match will not close, that is STOP-1, not a licence.

⚠ **Row 2 is the guard against draft four.** A Peer or Reply reappearing inside a variant is the
collision that killed three drafts; the grep must come back clean.

## Runtime prediction

**60–90 minutes.** The enum half is already built and measured; the ladders and the rename are the
work. Row 1's run is ~4 minutes when it fails, well under when it passes; the floor is the long pole.

## Trap-doors named in advance

- **Positional variant constructors.** `(:fanout::SeenRetry::Exhausted 3)`, not `:attempts 3`.
- **Seed threading.** Carry `seed'` forward, or every worker draws identical jitter — a synchronised
  retry storm, the opposite of what jitter is for.
- **Per-batch, not per-message.** All three cover a `receive :limit 10` batch. One increment is one
  *batch*; say so where it is counted or the number reads 10× small.
- **`vis` is nanoseconds.** `1e12` is 1000 s, `2e8` is 200 ms. Off by 1000 gives a 1 s bound that
  passes at n≤1000 and fails at depth — the current symptom, different cause.
- **Renaming `gave-back`** touches five places plus the failure string. Four are compile errors if
  missed; **the failure string is a `format` template and will not fail to compile.**

## What this stone does NOT claim

⚠ **Does not fix the class.** The backoff formula stays in three copies with `100` hardcoded in two.
The real fix is one generic `defn` in `wat/` — the builder's ruling, and this stone earns it.

⚠ **Does not explain the 25 % slope** (4348 → 3244), measured with `gave-back=0` throughout.

★ If row 1 passes, the honest next question is not "how fast is n=2000" but **"what do the
exhaustion counters read there?"** Passing while abandoning nothing clears the wall; passing while
the counters climb only moves it.
