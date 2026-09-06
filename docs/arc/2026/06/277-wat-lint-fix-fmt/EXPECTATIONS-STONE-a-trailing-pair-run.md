# EXPECTATIONS — STONE: a trailing run of pairs

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★ `(assoc m :k x)` | `m` breaks (compound); **`:k x` on ONE line** |
| 3 | ★★ `(get X :k)` | no pair run — `X` breaks, `:k` on its own line, **nothing compound rides** |
| 4 | ★ the four-pair record | one pair per line, values aligned — **no regression** |
| 5 | ★★ `defservice` | the ATOM positional still rides — **no regression**, and via the ordinary rule |
| 6 | ★ `(HashMap :- [K V] :some-kw "s")` | type-args slot, then **one pair on one line** |
| 7 | idempotent | `IDEMPOTENT=true` on rows 2-6 and every existing fixture |
| 8 | no rule names a column | `grep -c 'col' wat-scripts/fmt/rules/*.wat` → **0 in every file** |
| 9 | ruled shapes hold | `generic-fn`, `foldl-bare`, `type-ctor`, `type-nested`, `let-two`, `let-complex`, `half-broken`, `all-four`, `claim-demo`, `unruled-*` |
| 10 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0 |
| 11 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 12 | every fixture TYPE-CHECKS | `wat --check` clean on each new fixture **before** the floor |
| 13 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 14 | ★★★ **the 614 doc examples** | changed / still-over-120 / worst shape verbatim — **and the 132-column line MUST BE GONE** |
| 15 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 16 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 30-50 min. One rule's condition, one clause deleted.

## Trap-doors named in advance

- **Row 14 is the gate, and it is not a repeat.** The previous run's single remaining over-120 line
  IS this stone's defect. Green everywhere else with that line surviving means the stone did not land.
- **Row 3 is what row 2 alone cannot prove.** A rule that treats every trailing keyword as a run
  passes row 2 and fails row 3 — `(get X :k)` has an ODD trailing run and must not match.
- **Row 5's failure looks like success** if the deleted clause is quietly re-added. `defservice` must
  work *because its positional is an atom*, not because of a special case.
- **Row 12 exists because the last stone lost a floor run to a fixture** whose declared return type
  was wrong. Check fixtures before the floor, not after.
- **The vacuous green:** row 11 prints the comment COUNT.
