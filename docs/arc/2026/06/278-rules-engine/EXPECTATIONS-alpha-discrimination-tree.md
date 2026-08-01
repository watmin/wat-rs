# EXPECTATIONS — the alpha discrimination tree (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the superset invariant holds | the new invariant test at `[50 100]` | green — `walk(fact) ⊇ matcher's true set` for **every** fact |
| 2 | the tree actually discriminates | the new candidate-count test | mean candidates/fact **≈ 1**; with the tree bypassed, **≈ D** |
| 3 | `alpha:match` falls | `a0_depth_cost_split_at_equal_work` | depth50 column drops materially from **117.072 ms**; depth10 column does **not** regress |
| 4 | setup stays cheap | same table, `SETUP: indexes` row | ≤ ~2 ms at `[50 100]` (0.250 ms today) |
| 5 | nothing derived moved | `cargo nextest run --release` | the floor unchanged, Summary line read directly |
| 6 | the oracle did not move | `git diff --stat` | zero lines under `wat/`; `matcher.rs` unedited or additive-only |

## Independent prediction

- **Runtime:** 60–120 min. The node shape and the call site are given; the real work is the analyzer
  (resolving bind→field→constant chains through `classify_rete_clause`) and the partitioning build.
- **Row 3:** `facts × D → facts × ~1` predicts `alpha:match` 117 ms → single digits. **Treat that as
  a direction, not a number** — the last forecast on this engine was wrong low by 17 points, and a
  per-node walk has its own cost that the call-count ratio does not model.
- **The `[50 100]` grid cell:** may or may not flip off `:clara`. It is **not** a scorecard row, on
  purpose. Report it; do not chase it.

## Trap-doors named in advance

- **Row 2 is the one that matters.** A tree where every alpha lands on the wildcard edge is
  *perfectly correct* and buys nothing — rows 1, 5 and 6 would all still be green. The
  bypassed-comparison half of row 2 is what stops that from reading as success. This is the same
  vacuous-gate shape the whole arc has been fighting.
- **A subset is a silent wrong answer.** If row 1 fails, the temptation is to widen the invariant or
  special-case the failing fact. Both encode a real defect away. The count differentials might not
  even catch it — a dropped derivation can leave cardinality intact.
- **The analyzer's incompleteness is not a bug.** Conditions it cannot decompose belong on the
  wildcard edge. A rider "improving" coverage by guessing at an unfamiliar shape is how row 1 breaks.
- **`Value` as a hash key must agree with the matcher's equality.** If the tree's `HashMap<Value, _>`
  and `alpha_match_inner`'s `==` disagree for any value kind (float, nested aggregate, keyword vs
  string), the walk silently misroutes. That is the most likely cause of a row-1 failure.
- **Test-build inflation.** The phase census runs `#[cfg(test)]` with live timers, ~1.45×. Read
  rows 3 and 4 as **proportions**; the grid's `:native-ns` remains the release truth.
- **Do not re-run the whole grid** — that is the orchestrator's weigh.

## What would make me reject the strike outright

A failing superset invariant "fixed" by relaxing it; a second private condition parser instead of
`classify_rete_clause`; any edit under `wat/`; a candidate-count assertion with no bypassed
comparison behind it; or `alpha:match` improved while `:accuracy` on any axis moved.
