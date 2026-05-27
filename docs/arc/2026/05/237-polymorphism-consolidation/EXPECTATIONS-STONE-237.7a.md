# EXPECTATIONS — Stone 237.7a — `length` intrinsic. Orchestrator scores on independent local re-run.

## Independent runtime prediction

**20–40 min Mode A.** One scheme registration (mirror `type`) + one eval handler (mirror `eval_type`,
3-arm match) + one decl delete + a small cascade (call-site resolution shift). Wakeup time-box:
**2× upper = 80 min.**

## Scorecard (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 |
| 2 | **probe green (LOAD-BEARING)** | `--test probe_arc237_7a_length_intrinsic 2>&1 \| grep "test result"` | `6 passed; 0 failed` |
| 3 | **lib baseline (LOAD-BEARING)** | `--lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 4 | **workspace clean** | `cargo test --release --workspace --no-fail-fast 2>&1 \| grep -c "FAILED"` | 0 |
| 5 | **MECHANISM — decl gone** | `grep -c "define-dispatch :wat::core::length" wat/core.wat` | 0 |
| 6 | **MECHANISM — builtin present** | `grep -c '":wat::core::length"' src/check.rs src/runtime.rs` | ≥ 2 (scheme + eval arm) |
| 7 | other ops intact | `grep -c "define-dispatch :wat::core::\(empty?\|contains?\|get\|conj\)" wat/core.wat` | 4 (untouched) |
| 8 | scope | `git status --short` | src/runtime.rs + src/check.rs + wat/core.wat + the probe + SCORE; NO holon-rs; NO namespace renames |

**FM-9:** independently re-run rows 2 + 3 + 4, and rows 5/6 (the mechanism actually changed — not just
behavior preserved). The probe is a *regression guard* (green before AND after); rows 5–7 prove the
swap really happened and is isolated to `length`.

## Mode classification
- **Mode A:** all rows green; cascade mechanical; ≤ STOP-3.
- **Mode B:** probe red, baseline/workspace regressed, decl not deleted, registry/other-ops touched,
  holon-rs touched, or a namespace renamed. Any → re-brief.
- **Time-violation:** wakeup fires with Sonnet running → `TaskStop` + Mode-B-time.

## On green
Atomic commit: `src/runtime.rs` + `src/check.rs` + `wat/core.wat` + the probe + `SCORE-STONE-237.7a.md`
as ONE commit. Then advance the tracker: 237.7a shipped (length = intrinsic; recipe proven); NEXT =
237.7b (empty?/contains?/get/conj/assoc).
