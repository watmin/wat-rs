# EXPECTATIONS — no client call can hang

⛔ **Supersedes the earlier file of this name.** Written before the strike; every "before" is a
run of mine on `fba0b4777`.

| # | what | expected |
|---|---|---|
| 1 | **★★ a silent peer no longer hangs a caller** | a probe calling a generated method against `:fanout::Hold` (*"never settles"*) returns **`TimedOut` within ~10 s**. Before: **forever** |
| 2 | **★★ the migration is complete** | the floor **type-checks** — a missed site is a non-exhaustive match and fails to compile |
| 3 | **the floor** | `5215 passed`, 22 skipped. ⚠ a red **naming a surface** is row 6, not a failure |
| 4 | codemod idempotent | re-run changes **0 bytes**; show the `diff` of the two `git diff`s |
| 5 | census reported | matches touched vs matches with a catch-all (untouched), **occurrences not lines** |
| 6 | ⛔ **which surfaces need a longer deadline?** | report every deadline raise with its elapsed time. Expected: none |
| 7 | existing chaos unaffected | check/mark/recv/ack-drop ×3 each: `distinct=100` |
| 8 | rate-0 | circuit ×5: `total=8000; distinct=8000` |
| 9 | timings | report only. Before: publish `47784–49856` |

### Before-state, recorded verbatim

```
row 2/5  643 RecvOutcome::Message arms across 282 .wat files
         Lost-arm bodies: 245 assertion-failed! (+470 None args), ~22 nil/Tuple/connect
row 3    Summary [ 359.980s] 5215 passed, 22 skipped   .floor/2026-09-05T11-32-03Z/
row 7    check/mark/recv/ack-drop: distinct=100 on every run measured
row 8    total=8000; distinct=8000; dup=0 ×5
row 9    publish 47784 47790 48170 49124 49856
```

## ⛔ ROW 2 IS FREE, AND THAT IS THE POINT

Adding a variant makes every non-exhaustive match **fail to type-check**. **The compiler is the
census.** A missed site cannot ship — which is precisely why this shape is one-shottable and the
raise-inside-the-macro shape was not.

## ⛔ ROW 1 NEEDS A PEER THAT IS ALIVE AND SILENT

Not a dead peer — `Lost` covers that and works. `:fanout::Hold` (`circuit.wat:142`) never
settles, and `:user::deadline-redial-is-fresh` already drives it. **Demonstrate the hang before
repairing it**, as with the crash.

## RUNTIME PREDICTION

2–3 h — the largest stone of this run. The variant and the macro edit are small; the codemod and
its census are most of it, and the compiler will name every site the codemod missed.

## TRAP DOORS, NAMED

1. **Peer-kind at the caller's locus.** STOP-1 — the one thing unproven anywhere.
2. **A raise that prints nothing useful.** The message must name **surface, verb and deadline**;
   "call timed out" repeats the empty-ARM failure this arc paid for twice.
3. **Finishing the migration by hand** because the codemod missed a few. STOP-4 — the misses are
   the finder's bug, and the finder is the artifact.
4. **A green floor proving row 1.** It does not; nothing in the floor holds a silent peer open.
