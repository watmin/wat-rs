# EXPECTATIONS — Stone S-C.3 — base/holonic macro split

Paired with `BRIEF-STONE-S-C3.md`. Orchestrator scores against an INDEPENDENT local re-run.

## Independent runtime prediction

**60–100 min Mode A.** Constructor rename + 2-arg base mint; macro rename + base macro mint;
recordtype parents; migration cascade (small — most callers become base + pass); 18-contract probe.
Wakeup time-box: **2× upper = 200 min** (the cascade + macro authorship can run long).

## Scorecard verification (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 |
| 2 | **probe 18/18 (LOAD-BEARING)** | `--test probe_arc237_sC3_macro_split 2>&1 \| grep "test result"` | `18 passed; 0 failed` (was RED — PRELUDE failed: macro absent) |
| 3 | **lib baseline (LOAD-BEARING)** | `--lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 4 | `=` / `same-data?` unaffected | `--test probe_arc238_eq_completeness` + `--test probe_arc237_sC2d_same_data` | 8/8 + 6/6 |
| 5 | defrecord surface (post-migration) | `--test probe_arc227_stone2_defrecord` | 35/35 |
| 6 | **workspace clean** | `cargo test --release --no-fail-fast 2>&1 \| grep -c "FAILED"` | 0 |
| 7 | scope | `git status --short` | `src/runtime.rs` + `wat/Record.wat` + the probe + migrated caller files + SCORE; NO holon-rs |

**FM-9 on the claim:** independently re-run rows 2 + 3 + 6. The load-bearing contracts to confirm
MEASURE the split:
- `base_to_holon_errors` — base really has no holon flavor (not silently ok).
- `holonic_to_holon_ok` — holonic flavor preserved.
- `liskov_base_into_holon_rejected` — the STATIC proof: a base-defined record is rejected at a
  `:wat::holon::Record` param at CHECK time. **This is the heart of the stone — verify it's a check
  error, not a runtime pass.**
- `cross_flavor_same_data_true` + `cross_flavor_eq_false` — the two verbs behave per the split.

Spot-read: base macro emits NO holon_form block; recordtype parents are `:wat::Record` (base) vs
`:wat::holon::Record` (holonic).

## Mode classification
- **Mode A:** all rows green; cascade was mechanical (each break classified base/holonic by the rule); ≤ STOP-3.
- **Mode B:** `:wat::Record::def` builds holonic (flip wrong); base macro emits holon_form; wrong
  recordtype parent (contract 13 fails); a caller wrongly migrated; a probe contract weakened;
  baseline/​workspace not clean. Any → re-brief.
- **Time-violation:** wakeup fires with Sonnet running → `TaskStop` + Mode-B-time.

## Trap-doors (mirror BRIEF)
1. unmarked `:wat::Record::def` ≠ base → REJECT. 2. base macro emits holon machinery → REJECT.
3. wrong recordtype parent → REJECT. 4. over-migration to holonic → REJECT. 5. weakened contract → REJECT.
6. holon-rs touched → REJECT.

## On green
Atomic commit: `src/runtime.rs` + `wat/Record.wat` + `tests/probe_arc237_sC3_macro_split.rs` +
migrated caller files + `SCORE-STONE-S-C3.md` as ONE commit (S-C.3 flip + S-D cascade together —
mid-cascade brokenness never lands). Then USER-GUIDE (base vs holonic def; the type-distinction) +
tracker advance: records flavor thread CLOSED; NEXT = 237.7 (arithmetic tail) or arc-237 INSCRIPTION
fold. (S-D is absorbed here — the migration IS the cascade.)
