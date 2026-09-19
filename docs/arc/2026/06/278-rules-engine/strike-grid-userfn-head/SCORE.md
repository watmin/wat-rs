# SCORE — the `userfn-head` axis is green, and it reddens when the cure is reverted

The standing fixture is the deliverable. Post-cure Clara | oracle | native agree at items=5.
Reverting `rule-produces` to the colon-strip makes the port check name the five missing `Out`
rows. The revert changed the file (31 lines). Restored, green again. No `src/rete/kernel/`,
no `wat/rete/` in the landed tree.

## Scorecard

| # | result |
|---|---|
| 1 ★ axis runs | **HOLD.** `#grid/Result` at `[5]`, 10-element `:derived` and `:oracle-derived`, both `-ns`. Quoted below. |
| 2 ★ native == oracle post-cure | **HOLD.** Byte-identical vectors. |
| 3 ★ Clara agrees | **HOLD.** `check-grid-three-way.sh userfn-head`: `clara=10 native=10 oracle=10 ALL THREE MATCH`. |
| 4 ★ count is anti-vacuity | **HOLD.** Expected 10 = `2 * items`, written in the header before the run. `port_check` PASS after restore. |
| 5 ★ axis REDDENS on reverted cure | **HOLD.** Revert verified: `git diff --stat` 31 lines, colon-strip back at `stratify.wat:62-64`. Port check: 14 axes agreed; `userfn-head` MISMATCH, oracle missing the five `enc(1,*)` Out rows. Quoted below. Not re-run. |
| 6 ★ restored | **HOLD.** `git checkout HEAD -- wat/rete/oracle/stratify.wat`. Port check PASS 8.222s. |
| 7 ★ floor | **HOLD.** `Summary [ 485.774s] 5475 tests run: 5475 passed (3 slow), 22 skipped`. `.floor/2026-09-07T11-45-57Z/`. Count unchanged: the axis lives inside walking tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 no cure touch / no gen- | **HOLD.** Landed diff is two new grid files and three registration rows. |

★ load-bearing. **Row 5 is the proof the axis would have caught the flaw. Row 2 is only the post-cure picture.**

## Post-cure `#grid/Result`

```
#grid/Result {:axis "userfn-head" :size #wat.core/PersistentVector [5] :derived #wat.core/PersistentVector [0 1 2 3 4 1000000000000000 1000000000000001 1000000000000002 1000000000000003 1000000000000004] :native-ns 163705 :oracle-derived #wat.core/PersistentVector [0 1 2 3 4 1000000000000000 1000000000000001 1000000000000002 1000000000000003 1000000000000004] :oracle-ns 16138665}
```

`check-grid-three-way.sh userfn-head` (exit 0):

```
userfn-head          clara=10   native=10   oracle=10    ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (3s)
```

## Row 5 — reverted colon-strip, captured, not re-run

Revert: `git checkout '21a5f8514^' -- wat/rete/oracle/stratify.wat`
Diff vs HEAD: `wat/rete/oracle/stratify.wat | 31 +++++++------------------------`. Colon-strip present.

```
1 of 15 grid axes FAILED the native-vs-oracle port check (14 axes agreed before the failures below):
  userfn-head (size [5]): ⛔ PORT BUG — NATIVE AND $ORACLE DISAGREE.
      native (10 elems): 0 1 2 3 4 1000000000000000 1000000000000001 1000000000000002 1000000000000003 1000000000000004
      oracle (5 elems): 0 1 2 3 4
      only in native: 1000000000000000 1000000000000001 1000000000000002 1000000000000003 1000000000000004
      only in oracle: 
```

The five `enc(1,*)` values are the Out rows. Rate survived; Out dropped. The other 14 axes agreed — the colon-strip was correct for every record-head axis.

## Items = 5, stated

Expected count `2 * 5 = 10` from the header formula before the run. `Bad` matches `k = -1`, which no key in `[0, 5)` can take. Liveness size is 2.

## Landing

`userfn-head.{wat,clj}`, three registration rows. Static `.clj`, no `gen-`. Do not commit unless asked.
