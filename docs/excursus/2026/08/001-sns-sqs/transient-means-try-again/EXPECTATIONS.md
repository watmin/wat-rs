# EXPECTATIONS — transient means try again

Written before the strike, on `c1a07bc99`.

| # | what | expected |
|---|---|---|
| 1 | ★★ **a transient store error no longer kills the queue** | honest wrapper fails N times then succeeds: the queue **retries and completes**. `total`/`distinct` intact, **no `Lost` to the caller** |
| 2 | ★★ **an exhausted budget still dies, and says so** | wrapper fails forever: assertion names **transient, exhausted after N** — not the old generic string |
| 3 | ★★ **the other variants die by name** | a `:Constraint` and a `:Fatal` each assert with **their own** message |
| 4 | ⛔ **no `_` arm on a store response survives** | `grep` the three sites: every store response is matched **exhaustively** |
| 5 | ⛔ **the floor** | `5215 passed`, 22 skipped |
| 6 | ⛔ **no perf regression** | `publish + drain` ×5. Before: median **23672 ms**. A retry that never fires must cost nothing |
| 7 | blast radius | `sqs.wat` only, plus a `scratch-pad/` probe |

### Before-state, recorded verbatim

```
row 1/2  probe-entry-three-of-ten: store=Transient; send=Lost; total=9; distinct=9
         (queue dies; one message silently lost)
row 5    Summary [371.648s] 5215 passed, 22 skipped   .floor/2026-09-06T01-52-00Z/
row 6    publish 23317 23497 23568 23299 23591; drain 191 175 235 232 233
         publish+drain median 23672 ms
```

## ⛔ ROW 6 IS NOT DECORATION

A retry loop on a hot path can cost even when it never fires — an extra match arm, an extra
binding, a timer armed and cancelled. **`publish + drain` must not move.** This is the metric the
inbox cap cannot Goodhart, and it is the one the last two perf stones were measured on.

## ⛔ ROW 1 NEEDS AN HONEST STORE, NOT THE STEP-1 WRAPPER

`probe-entry-three-of-ten.wat`'s wrapper **applies k of n and then reports `:Transient`** — a
lying store, built deliberately for step 1. Retrying against it would re-apply the k that landed
and measure the wrong thing.

★ Row 1 needs a wrapper that fails **without applying anything**, N times, then succeeds. That is
what a real `SQLITE_BUSY` looks like, and it is the only case this stone claims to fix.

## RUNTIME PREDICTION

60–90 min. Three sites, one shape. The care is that the retry lives inside a service arm
(STOP-1) and that no `_` survives (row 4).

## TRAP DOORS

1. **Retrying `:Constraint` or `:Fatal`.** STOP-2 — turns a clear failure into a slow one.
2. **A narrower catch-all.** Row 4 — `(_ …)` replaced by `(_ …)` over three variants is the same
   defect with a smaller mouth.
3. **Testing against the lying wrapper.** Row 1 — it re-applies committed rows and measures step
   2's question instead of this one.
4. **A retry that costs when it never fires.** Row 6.
