# EXPECTATIONS — Stone Value (independent scorecard, fixed BEFORE the strike)

The change is additive and gated behind a brand-new type name (`:wat::core::Value`) that no existing code
references, so it is **inert for every existing test** — the four floors must be UNCHANGED, and the probe must
flip from 3-ignored to 6-green.

| # | what | command | expected |
|---|------|---------|----------|
| 1 | the probe goes fully green | `cargo test --release -p wat --test probe_arc278_value_universal_top` | **6 passed; 0 failed; 0 ignored** (the 3 disconfirm asserts now pass; the 3 discipline asserts still pass) |
| 2 | lib floor unchanged | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | **930 passed / 36 failed** (the 36 are pre-existing; no new failure) |
| 3 | deftest floor unchanged | `cargo test --release --test test \| grep "test result"` | **264 / 1** (the 1 = `run_string_entry_direct`, pre-existing) |
| 4 | nursery floor unchanged | `cargo test --release -p wat --test nursery -- --test-threads=1 \| grep result` | **~893 / 4** (±3 fork flake — re-run a single failure before calling it real) |
| 5 | deporder floor unchanged | `cargo test --release --test test_stdlib_load_order \| grep result` | **1 / 0** |
| 6 | build clean | `cargo build --release` | compiles; warning count unchanged (~25 lib warnings, pre-existing) |

## Runtime prediction
~10–15 min wall, almost all of it the release build + the four floor runs. The edit itself is ≈6 lines in
`src/types.rs` + deleting 3 `#[ignore]` lines.

## Trap-door risks (named)
- **Down-leak (the only real risk).** If the rule is written so any `sup` (not strictly `":wat::core::Value"`)
  returns true, DOWN leaks and the discipline asserts (`down_*`, `narrow_*`) go RED. Row 1 catches it. This is
  STOP-2.
- **`assignable` routing differs from the brief.** If Path/Path acceptance does NOT consult `is_subtype` first,
  the root rule won't make WIDEN pass. Row 1's `widen_*` assert catches it. This is STOP-1.
- **Scope creep.** If the executor reaches for `check.rs` / `rete.wat` / registration to make the probe green,
  the scope assumption (root-rule-suffices) was wrong → STOP-3, surface, do not expand.
- **Nursery fork flake** (row 4, ±3). Re-run a lone nursery failure isolated before treating it as a regression.

## Acceptance
All six rows meet expected, weighed against the orchestrator's OWN re-run (not the executor's report) + a read
of the diff (`git diff` shows ONLY the `is_subtype` root rule + 3 removed `#[ignore]` lines — nothing else
moved). Then commit on green + push.
