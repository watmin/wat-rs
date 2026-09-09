# SCORE (independent re-run) — STONE purgare: clippy to ZERO

Independent re-run of the rider's SCORE. Floor not run.

```
cargo clippy --release --all-targets -- -D warnings
CLIPPY_EXIT=0
```

The five named `dead_code` items are gone from `*.rs` (`pattern_coverage`,
`Coverage::Wildcard`, `try_match_pattern_ast`, `substitute_many`,
`MatchArm::Binding::ident_span`). `fn substitute` (singular) remains at
`src/runtime.rs:13610`. No `#[allow(dead_code)]` in the diff. Remaining
`ident_span` hit is the unrelated local in `src/macros/expand.rs:584`.

Working tree matches the TABLE's three files:

```
src/check.rs      | 452 ----
src/match_arm.rs  |   7 +-
src/runtime.rs    | 147 ----
3 files changed, 1 insertion(+), 605 deletions(-)
```

## a2_a_variant count

Confirmed independently: **15 passed, 5298 skipped, EXIT 0** — not the
brief's 16. Pre-existing discrepancy, as the rider reported. Not a
regression of this delete.

## STOP rows (from this re-run)

| STOP | this re-run |
|---|---|
| STOP-1 behavioural change | not re-proven by a behavioural test here; the clippy-zero bar and the delete-only diff are consistent with "already unreachable" |
| STOP-2 cascade campaign | 6 sites as tabled; under ~20 |
| STOP-3 allow-list | **held** — none in the diff |
| STOP-4 hidden callers | **held** for the five names in `*.rs`; comments-only leftovers as the TABLE flagged |

No commit. Clippy is green at `-D warnings`.
