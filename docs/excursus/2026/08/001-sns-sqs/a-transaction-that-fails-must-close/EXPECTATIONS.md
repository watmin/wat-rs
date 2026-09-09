# EXPECTATIONS — a transaction that fails must close

Written before the strike, on `ceb7a30dc`.

| # | what | expected |
|---|---|---|
| 1 | ★★ **the connection is no longer poisoned** | the STOP-6 probe's `put2` flips **`Fatal: cannot start a transaction within a transaction` → `Ok`** (or a fresh, unrelated error — **not** the transaction one) |
| 2 | ★★ **the original cause survives** | a failed put still reports **its own** error (`no such table: main`), not the rollback's outcome |
| 3 | ★ rollback exists at all three layers | `src/rust_deps/sqlite.rs`, `wat/sqlite.wat`, and called from both store error paths |
| 4 | ⛔ **the probe is now a GATE** | it runs on the floor, **not `#[ignore]`d**: tests **5216**, skipped stays **22** |
| 5 | ⛔ **the floor** | `5216 passed`, 22 skipped |
| 6 | ⛔ **the happy path is unchanged** | `publish + drain` ×5. Before: median **23672 ms**. A rollback that never fires must cost nothing |
| 7 | blast radius | the four named files only |

### Before-state, recorded verbatim

```
row 1/2  CONN=begin1=Ok;fail=Fatal:1:no such table: nosuch;
         begin2=Fatal:1:cannot start a transaction within a transaction;commit=Ok
         STORE=put1=Fatal:no such table: main;
         put2=Fatal:cannot start a transaction within a transaction
row 5    Summary [376.225s] 5215 passed, 22 skipped   .floor/2026-09-06T02-21-23Z/
row 6    publish 23317 23497 23568 23299 23591; drain 191 175 235 232 233 → median 23672
```

## ⛔ ROW 1 IS THE REFUTATION ROW AND IT IS ALREADY MEASURED FAILING

`put2` is `Fatal: cannot start a transaction within a transaction` today, on my runs and the
executor's, twice each. **If it does not flip, the rollback is not being called on the path that
matters** — report which path, do not tune the probe.

## ⛔ ROW 4 IS 5216, NOT 5215, AND SKIPPED STAYS 22

Every chaos cell in this arc is `#[ignore]`d because it needs a drop rate. **This one is not** —
deterministic, no chaos. A `23 skipped` means it was ignored, and the bug that took two stones to
find would regress silently.

★ **The measurement that found the bug is what keeps it fixed.**

## ⛔ ROW 2 GUARDS AGAINST THE INWARD COLLAPSE

A rollback that reports its own success and discards the original cause would be the same
information loss this arc has spent the day closing, pointed at ourselves. The caller must still
learn why **its** operation failed.

## RUNTIME PREDICTION

45–75 min. Three lines of Rust, five of wat, two error arms, and promoting the probe. A rebuild
is needed for both the `src/` and the `wat/` change.

## TRAP DOORS

1. **Returning the rollback's result.** Row 2.
2. **Leaving the probe in `scratch-pad/`.** Row 4 — it regresses silently the moment someone
   touches the error path.
3. **Adding the `:Transient` retry while here.** STOP-3; it is the next stone and mixing them
   makes neither measurable.
4. **A rollback that costs on the happy path.** Row 6.
