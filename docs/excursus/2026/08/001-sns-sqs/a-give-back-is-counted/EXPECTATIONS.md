# EXPECTATIONS — a give-back is counted

Written **before** the strike. Every "before" is a run of mine on `cc7f592b8`.

| # | what | expected |
|---|---|---|
| 1 | **★★ the counter fires** — check-drop cell, **×12** | `gave-back > 0` in **≥ 1 run**. Report the full distribution, all twelve |
| 2 | **★★ THE ROW WE COULD NOT WRITE BEFORE** — in every run where `gave-back > 0` | `total = 100`, `distinct = 100`, `dup = 0` |
| 3 | ⛔ no chaos, no give-backs | rate-0 ×5: `gave-back = 0` in **all five** |
| 4 | mark-drop unaffected | `r2_drop_after_tiny` ×6: 6/6, `total=100; distinct=100; dup=0` |
| 5 | rate-0 invariant | `total=8000; distinct=8000; dup=0; seen-recorded=8000` ×5 |
| 6 | **the floor** | `5214 passed`, **20 skipped** |
| 7 | blast radius | `git diff --stat` → `circuit.wat` only |
| 8 | timings | report only, no gate. Before: publish `45771–47039` |

### Before-state, recorded verbatim

```
row 1/2  no such field exists — grep "gave-back" → 0 hits
row 3/5  total=8000; distinct=8000; dup=0 ×5; seen-recorded=8000
row 4    6/6; total=100; distinct=100; dup=0; seen-recorded=100; seen-skipped 15–17
row 6    Summary [ 362.221s] 5214 passed, 20 skipped   .floor/2026-09-05T09-30-41Z/
row 8    publish 45771 46113 46544 46920 47039
```

⚠ **20 skipped is correct.** `drop_check_tiny` was added during the previous grading; 19 is the
stale number.

## ⛔ ROW 1 IS TWELVE RUNS ON PURPOSE

Exhaustion measured ~**1 in 6**. Six runs could plausibly show zero give-backs and prove nothing
— which is precisely the hole this stone closes. **Twelve runs, and report every one**, so the
rate is visible rather than inferred from a single hit.

If `gave-back` is 0 across all twelve, that is **not a pass and not a failure** — it is a STOP
worth reporting, because either the counter is not wired or exhaustion is rarer than measured,
and those are different worlds.

## ⛔ ROW 2 IS THE POINT OF THE WHOLE STONE

Row 1 only proves the counter moves. **Row 2 is what the counter is FOR**: it makes the previous
stone's claim — *a give-back loses nothing* — checkable for the first time, on exactly the runs
where the path was taken.

A green row 2 with `gave-back = 0` everywhere is **vacuous** and must be reported as such, not as
a pass.

## RUNTIME PREDICTION

30–45 min. One field through four records and a format string. The care is in the fold: the
counter lives in worker state while the fold threads a 3-tuple.

## TRAP DOORS, NAMED

1. **A counter that counts too much.** If `gave-back` also increments on `PeerGone` or a clean
   check, it stops meaning what its name says and rows 1–3 all go green for the wrong reason.
   STOP-2.
2. **Incrementing where the envelope is retried, not given back.** `a1`/`a2` are *retries*; only
   the exhausted `a3` is a give-back.
3. **The `held-worker` stub.** It must gain the field to type-check and must keep returning zero
   — it has no give-back path, and a non-zero there would be fiction.
4. **A green floor proves nothing about rows 1–3.** The floor runs rate 0 with the chaos cells
   ignored.
