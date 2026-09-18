# SCORE — mode parity gate (after REVIEW-2)

Two edits. Floor re-run at this state. No `src/` change. No cure.

## What changed

1. Both live arms call `soundness_holds` / `liveness_holds` — the same
   predicates the mutation stubs use. One encoding.
2. Non-vacuity is one `assert_eq!` on the exact name list. No `.contains`.

## Floor (this round, whole)

```
Summary [ 451.372s] 5427 tests run: 5425 passed (1 slow), 2 failed, 21 skipped
  FAIL  wat::cli mode_parity::mode_parity_empty
  FAIL  wat::cli mode_parity::mode_parity_deep_freeze_recursion
```

Exactly the two arms. Nothing else. Clippy `--all-targets --release` rc=0.

## Arms at HEAD

- SOUNDNESS: `--check` Accepted, run Rejected → `soundness_holds` false
- LIVENESS: `--check` DiedBySignal(6), run Accepted → `liveness_holds` false

Mutation stubs still green: a SOUNDNESS cure greens only SOUNDNESS; a
LIVENESS cure greens only LIVENESS. Blinding the predicate now blinds
the arm.
