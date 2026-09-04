# SCORE — the call site reads as English

**STRUCK.** Executor: grok, 2026-09-04. Every row re-run by me.

```
Summary [ 370.454s] 5214 tests run: 5214 passed (4 slow), 15 skipped
FLOOR=0        my own run, 0 FAIL/TIMEOUT · circuit distinct=8000; dup=0 ×5
```

## ★ ROW 1 — the quotation, which is the whole contract decision

`circuit.wat:217`:

```wat
(:wat::service::Alarm :delay (:wat::time::Milliseconds 1) :op :-tick)
```

*"alarm, delay 1 milliseconds, op `-tick`."* One reading. No docstring.

And **row 2, the proof the rename bit** — `service.wat:1620-1623`:

```wat
(:wat::kernel::after
  (:wat::program::Env/peer-kind (:wat::program::env))
  (:wat::service::Alarm/delay ~arm-alarm-sym)
  (:wat::service::Alarm/op   ~arm-alarm-sym))
```

**One `after` — the intrinsic.** The field is `delay`. The stutter of two `after`s as different parts
of speech is gone, and that was the sentence the whole stone was for.

| # | row | result |
|---|---|---|
| 1 | ★★ the call site reads aloud | ✅ quoted above |
| 2 | ★★ the stutter is gone | ✅ `service.wat:1620-1623` |
| 3 | ★ the finder's census | ✅ reported before apply — **and it refuted mine, twice** |
| 4 | both codemods idempotent | ✅ re-apply 0 changes; form rules 0 after |
| 5 | `Microsecond` renamed | ✅ — and it was **not** zero |
| 6 | purity unchanged | ✅ |
| 7 | scope | ✅ `service.wat` three hunks: docstring, field, accessor. **No R1 seam** |
| 8 | the floor | ✅ **5214/5214, my run** |
| 9 | the circuit | ✅ `dup=0` ×5 |

## ⛔ MY CENSUS WAS WRONG IN BOTH DIRECTIONS — the ninth

| | mine | finder |
|---|---|---|
| `:after` | 74 | **46 tokens, of which 45 are `Alarm`'s** |
| constructor sites | 99 | **125 across 42 files** |
| `Microsecond` | **0** | **1** |

**And the `Microsecond` miss is diagnosed: `wat-tests/time.wat:191` — `(:wat::time::Microseconds 1)`.
I omitted `wat-tests/` again.** Same directory, second time this campaign; the first cost the
`NonZeroDuration` stone its "zero literal zero-durations" claim.

My 74-vs-46 gap I **did not** diagnose. I hypothesised substring inflation (`:after-drain`,
`-flush-after-ms`) and checked it: **0 hits.** The hypothesis is dead and the cause is undetermined.
Recording that rather than inventing a second theory — this session has already spent four framings
on a question whose answer was "the form was malformed."

★ **The form filter is what made STOP-1 not fire.** The one leftover token —
`(:user::metric :after Tp nil-u)` in `probe_ex001_sortkey_boundary.wat` — is **not `Alarm`'s**, and a
token-matching rename would have swept it. My BRIEF predicted exactly this risk and the codemod
handled it; the difference between 46 tokens and 45 forms is the entire reason that file still works.

## BOOTSTRAP — needed, and cheaper than expected

No new `fix.wat` verb, so **no stash dance**: corpus first under the old binary, `src/` to match, then
`cargo build --release` froze the new stdlib. The procedure R1 never reached, used and reported.

## Still open

- **R1, the seam** — first draft known wrong (`Peer :- [R O]`, five sites not ten). Re-draw from a
  cold read; the corrections are in `SCORE-the-reactor-grows-a-seam.md`.
- **Behind the seam:** 3d, the select pool, server-side handle killing.
- **S33/S34 · S15–S32 · the arc-109 phantom-form NOTE.**
