# EXPECTATIONS — the window gets a test

Written **before** the strike. Every "before" is a run of mine on `8264fddcc`.

| # | what | expected |
|---|---|---|
| 1 | **★★ the new test is deterministic** | run it **×6**: `distinct = 1` every time |
| 2 | **★★ the window is actually forced** | the same 6 runs report `total = 2` — **reported, not gated** |
| 3 | ⛔ the gate catches loss, not duplication | the assertion is on `distinct` alone; `total`/`dup` are `eprintln!` |
| 4 | the two existing redelivery tests still pass | unchanged, on the floor |
| 5 | **the floor** | `5215 passed`, 22 skipped — **one more test, and it is NOT ignored** |
| 6 | the rename is complete | `grep -c "delay-ms"` finds no bare `delay-ms`; 6 `mk-worker` sites carry both delays |
| 7 | chaos cells unaffected | check-drop ×6 and mark-drop ×6: `distinct=100` |
| 8 | rate-0 invariant | circuit ×5: `total=8000; distinct=8000` |
| 9 | timings | report only. Before: publish `48163–50664` |

### Before-state, recorded verbatim

```
row 4/5  Summary [ 360.893s] 5214 passed, 22 skipped   .floor/2026-09-05T11-00-56Z/
row 7    check-drop ×12: 12/12 distinct=100, gave-back 3/12
         mark-drop ×6: 6/6 distinct=100
row 8    total=8000; distinct=8000; dup=0 ×5
row 9    publish 48163 48457 48599 49145 50664
```

## ⛔ ROW 5 IS 5215, NOT 5214 — AND NOT IGNORED

Every chaos cell in this arc is `#[ignore]`d because it needs a drop rate. **This one is not.**
It is deterministic timing, no chaos, so it belongs on the floor where it will actually run. A
`22 skipped` that becomes `23` means it was ignored, and that is a failed strike.

## ⛔ ROW 2 IS REPORTED BECAUSE OF WHAT IT WOULD COST TO GATE

`total = 2` is what the design does today. **A future change that made the woken worker re-check
and skip would give `total = 1` — strictly better — and a gate would red on it.** Report it, and
say in the SCORE what it was.

★ This is the first stone written under the rule the last SCORE recorded: *a row must state what
must HOLD, not what was last observed.* Row 1 gates the invariant; row 2 reports the number.

## RUNTIME PREDICTION

30–45 min. The rename is 6 sites in one file; the nap relocation is small; the fixture and test
copy siblings that already exist.

## TRAP DOORS, NAMED

1. **Napping in the wrong place.** After the mark, this reproduces the *absorbed* test and row 2
   reads `total=1` — green-looking, and testing nothing new.
2. **Gating `total`.** STOP-2. It is the exact error this arc has now made three times.
3. **Ignoring the new test.** Row 5 catches it.
4. **A margin too tight.** 200 ms vis against a 350 ms nap is the sibling fixture's own margin;
   if `distinct` varies across 6 runs, that is STOP-3, not something to paper over with a retry.
