# SCORE (independent re-run) — STONE purgare: clippy to ZERO

Independent re-run of the rider's SCORE. Floor not run. No re-implementation.

HEAD is `418c06b78` DRAW(purgare) — **docs-only** (the BRIEF). The three-file
delete is still uncommitted, as the rider left it.

```
cargo clippy --release --all-targets -- -D warnings
    Finished `release` profile [optimized] target(s) in 0.08s
CLIPPY_EXIT=0
```

## Rows, re-run

| # | command | rider | this re-run |
|---|---|---|---|
| 1 | `cargo clippy --release --all-targets -- -D warnings` | EXIT 0 | **CLIPPY_EXIT=0** (cached release, no warnings emitted) |
| 2 | `-E 'test(a2_a_variant)'` | 15 passed, 5298 skipped | **15 passed, 0 failed, 5298 skipped, EXIT 0** |
| 3 | `-E 'test(p1_annotation)'` | 10 / 5303 | **10 passed, 0 failed, 5303 skipped, EXIT 0** |
| 4 | `-E 'test(p1b_a_parametric)'` | 4 / 5309 | **4 passed, 0 failed, 5309 skipped, EXIT 0** |
| 5 | `-E 'test(p2prereq)'` | 4 / 5309 | **4 passed, 0 failed, 5309 skipped, EXIT 0** |
| 6 | `-E 'test(p3_one_question)'` | 5 / 5308 | **5 passed, 0 failed, 5308 skipped, EXIT 0** |
| 7 | `-E 'test(a1_one_rule)'` | 4 / 5309 | **4 passed, 0 failed, 5309 skipped, EXIT 0** |
| 8 | `git diff --stat` three files | −452 / −6+1 / −147 | **`src/check.rs` −452; `src/match_arm.rs` −6 +1; `src/runtime.rs` −147** |

**8/8 match the rider on this re-run.** The brief's `a2_a_variant` **16** is
still the live **15**. Pre-existing discrepancy, not a regression of the delete.

## Names gone / names kept (source, this re-run)

- `fn pattern_coverage`, `Coverage::Wildcard`, `fn try_match_pattern_ast`,
  `fn substitute_many` — **zero hits** in `*.rs`.
- `enum Coverage` at `src/check.rs:6573` has Option/Result/EnumVariant only;
  no `Wildcard`.
- `MatchArm::Binding` is `{ ident, body }` — no `ident_span`. Remaining
  `ident_span` hit is the unrelated local at `src/macros/expand.rs:584`.
- `fn substitute` (singular) remains live at `src/runtime.rs:13610`.
- No `#[allow(dead_code)]` in `src/check.rs`, `src/match_arm.rs`, or
  `src/runtime.rs`, and none in the three-file diff.

## STOP rows (from this re-run)

| STOP | this re-run |
|---|---|
| STOP-1 behavioural change | not re-proven by a behavioural test here; delete-only diff + clippy-zero + the six filters unmoved are consistent with "already unreachable" |
| STOP-2 cascade campaign | 6 sites as tabled; under ~20 |
| STOP-3 allow-list | **held** — none in the three files or their diff |
| STOP-4 hidden callers | **held** for the five names in `*.rs`; comments-only leftovers as the TABLE flagged |

## Extra comment leftovers the TABLE did not list

TABLE named `src/check.rs:7466`, `:7779`, and `src/runtime.rs:13577`.
This re-run also sees `src/runtime.rs:8757` and `:8857` still naming
`pattern_coverage` in comments. Same class the TABLE already left alone
(STOP-1 / out of scope). Flagged, not edited.

No commit. Clippy is green at `-D warnings`.
