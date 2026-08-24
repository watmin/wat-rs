# EXPECTATIONS — STONE: the binder must be universal

> Written BEFORE the strike. Every row re-run by the orchestrator independently.

## Baseline, measured this session at `72c0334c4`

| fact | measured |
|---|---|
| floor | 5016/5016, 0 FAIL, 19 skipped |
| clippy | 0 under `-D warnings` |
| `(:wat::eval-ast! :- [T] expr)` | **FAILS** — `takes exactly 1 argument; got 3` |
| `(:wat::eval-edn! :- [] "42")` | **FAILS identically** — proves scope is not "generic verbs" |
| `(:wat::eval-edn! "42")` | works |
| `peel_param_spec` callers | 27 across 6 files; the ten eval forms are not among them |

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | **generic + binder** | probe row 1 | evaluates; typed `Result` returned — *load-bearing* |
| 2 | empty binder ≡ absent | probe row 2 | identical to the no-binder call |
| 3 | no-regression | probe row 3 | unchanged |
| 4 | structural, not one-armed | probe row 4 | a second form behaves the same |
| 5 | the chamber unblocks | run the design's own repro | query file → `eval-ast! :- [T]` → `compile` succeeds |
| 6 | one peel, one place | read the diff | **helpers byte-identical**; a single `peel_param_spec` call added |
| 7 | floor | `scripts/floor.sh` | **5016 + the probe rows**, accounted BY NAME |
| 8 | clippy | `-D warnings` | 0 |
| 9 | blast radius | `git diff --stat` | `runtime.rs` + probe + fixture. Nothing else. |

Row 6 is scored by reading. No test can fail because the fix was applied ten times instead of once —
and ten times is the shape that leaves form eleven broken.

## Independent prediction

**15–30 minutes.** One grouped match arm plus a four-row probe. The risk is not the edit; it is
whether the ten arms group cleanly — if any carries a guard or a different signature, STOP-1 fires
and the shape changes. Wake-up at 2× = 60 min.

## Trap doors, named before the strike

1. **The arms may not be adjacent or uniform.** `eval::walk` sits inside the cluster but is not an
   `eval-…!` form and may differ. If it does not group, it is still in scope — report it.
2. **Double-peel.** If any helper ALREADY peels (the design measured 0, but the arms are thin
   dispatchers and the helpers were read only at their arity checks), peeling again is harmless for
   `:- []` but would silently drop a real binder. Check before assuming.
3. **The binder is discarded — is that right for `eval-with-defs!`?** It takes `<form> <defs>` and is
   generic. If its `T` is load-bearing at runtime rather than only at check time, discarding is
   wrong. This is the one form where "types are erased" deserves a second look.
4. **Row 5 may surface a SECOND defect.** The chamber probe was proven only up to the binder. If it
   now fails further along, that is a new finding, not this stone's failure — report and stop.

## What would make me reject a green report

- Rows 1-4 green with the fix applied inside the helpers. That passes every test and leaves the
  eleventh form broken — it is the shape the design explicitly cut.
- Row 7 at anything but 5016 + the new rows, without a test-by-test account.
- A claim that all ten forms are covered without naming the five the message-grep cannot see.
