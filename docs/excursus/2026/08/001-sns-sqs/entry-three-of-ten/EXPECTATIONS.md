# EXPECTATIONS — entry three of ten

Written before the strike, on `f89d5cf59`.

| # | what | expected |
|---|---|---|
| 1 | ★★ **partial put duplicates** | store applies k of n, reports `:Transient`; queue retries the batch; **`total` > `distinct`**. Report both |
| 2 | ★★ **partial delete is safe** | store deletes k of n, reports `:Transient`; the retry re-deletes missing rows as no-ops; **nothing lost, nothing duplicated** |
| 3 | ★★ **what the surface can SAY** | the **literal** response value on each partial case, and what the queue can infer from it |
| 4 | ⛔ blast radius | `wat-scripts/scratch-pad/` only |
| 5 | floor untouched | `5215 passed`, 22 skipped — a probe cannot change it, and the run proves the tree was not disturbed |

### Before-state

```
row 5   Summary [376.670s] 5215 passed, 22 skipped   .floor/2026-09-06T01-12-08Z/
        (no prior measurement exists — per-entry batch failure has never been injected)
```

## ⛔ ROWS 1 AND 2 ARE A PREDICTION, NOT A REQUIREMENT

**Either outcome is a pass.** If partial `put` does **not** duplicate, my mechanism is wrong and
that is the finding. If partial `delete` **does** lose a message, that is a bigger finding still.

★ What would make this a failed strike is **not measuring `total`/`distinct`** — reporting that
the type is lossy without saying what the system does. The types were already read; the behaviour
is the new information.

## ⛔ ROW 3 IS WHAT EARNS STEP 2

If the store has an honest response for the partial case, the collapsed surfaces are fine and
step 2 shrinks. If it does not — if every available variant is a lie — **that is the argument for
per-entry outcomes**, made from evidence instead of from my reading of a type.

## RUNTIME PREDICTION

60–90 min. The wrapper is a forwarding service; the two cells are small. Most of the work is
making the partial write *actually partial*.

## TRAP DOORS

1. **A wrapper that fails without writing.** Models "the store did nothing" — already handled,
   and not this fault. The BRIEF says so twice.
2. **Reporting the type gap without the behaviour.** The types were read before the stone was
   drawn; only rows 1–2 are new.
3. **Fixing it.** STOP-3. Provoke before repairing — the crash stone is the precedent.
