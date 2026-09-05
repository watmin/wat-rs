# EXPECTATIONS — a worker gives the message back

Written **before** the strike. Every "before" is a run of mine on `2f4573222`.

| # | what | expected |
|---|---|---|
| 1 | **★★ PROVOKE IT FIRST** — `check` knob on, worker unchanged, tiny ×6 | the crash **returns**: `claim deadline exhausted;depth=3;attempts=3` in ≥1 run. **Record the count** |
| 2 | **★★ after the fix, same config** ×6 | **0/6 crashes**, every run terminates |
| 3 | **⛔ nothing is lost** — same runs | `total = 100` and `distinct = 100` on every run |
| 4 | ⛔ nothing is duplicated | `dup = 0` on every run |
| 5 | mark-drop coverage kept | with the `mark` knob on and `check` off, tiny ×6 still **6/6**, `total=100` |
| 6 | rate-0 untouched | circuit ×5: `total=8000; distinct=8000; dup=0; seen-recorded=8000` |
| 7 | **the floor** | `5214 passed`, 19 skipped |
| 8 | blast radius | `git diff --stat` → `circuit.wat` only |
| 9 | timings | **report only, no gate.** Before: publish `45965–47486` |

### Before-state, recorded verbatim

```
row 1  0/6 crashes — because check has NO drop knob, not because it was fixed
row 5  6/6; total=100; distinct=100; dup=0; seen-recorded=100; seen-skipped 13–18
row 6  total=8000; distinct=8000; dup=0 ×5; seen-recorded=8000; seen-skipped 4–15
row 7  Summary [ 360.766s] 5214 passed, 19 skipped   .floor/2026-09-05T08-39-33Z/
row 9  publish 45965 46578 46581 47397 47486
```

## ⛔ ROW 1 IS NOT OPTIONAL AND CANNOT BE INFERRED

A green row 2 means nothing without row 1. **0/6 is already the before-state** — it is what the
tree reports today, and it is a lie told by an instrument that is looking elsewhere. Row 1 is
the only thing that distinguishes *repaired* from *unprovoked*.

If row 1 will not fire, that is **STOP-1**, not a pass.

## ⛔ ROWS 3 AND 4 ARE THE GUARD ON ROW 2

Row 2 alone is satisfiable by acking the envelope on exhaustion — the crash disappears and the
message is silently destroyed. **Row 3 is what makes row 2 mean something**, and row 4 stops the
opposite cheat. `total = distinct = 100` with `dup = 0`, or row 2 is not a pass.

## RUNTIME PREDICTION

40–60 min. The `check` knob mirrors `mark`'s exactly; the care is in the fold — returning the
accumulator unchanged, with `outs0` rather than `outs1`, and no ack.

## TRAP DOORS, NAMED

1. **Acking on give-back.** The single most likely wrong turn: it makes rows 1, 2 and 5 green
   and destroys messages. Row 3 is the only thing that catches it. STOP-2.
2. **Emitting `outs1` on the exhaustion path.** Nothing was checked, so nothing may be emitted;
   `outs1` here would be an outcome for work never claimed.
3. **A shared `hit?` between the two verbs.** If both read one rate, aiming at `check` re-aims
   away from `mark` and row 5 goes dark — the very failure this stone is about.
4. **A green floor proves nothing here.** The floor runs rate 0; rows 1–5 are all outside it.
