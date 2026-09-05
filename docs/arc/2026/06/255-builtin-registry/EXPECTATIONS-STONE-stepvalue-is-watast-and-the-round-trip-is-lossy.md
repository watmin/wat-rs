# EXPECTATIONS — STONE: `StepValue` faces `WatAST`. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the bug reproduces first | the probe, pre-change | rational → `"1/2"`, bigint → `"…N"` | measured this session |
| 2 | i64 control unaffected | same probe | `42` before AND after | measured this session |
| 3 | the bug is fixed | the probe, post-change | `1/2` and the bigint survive as themselves | the stone |
| 4 | the four conversions are GONE | `grep holon_to_watast src/runtime.rs` | 7 sites → 3 | 4 bridge sites deleted |
| 5 | `wat` binary | `-E 'binary_id(wat)'` | green | the stepper's own suite |
| 6 | types / value / loader | the three scoped runs | green | today's `step_*` rewrites live here |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement

**None, and derived.** No registration, no retirement, no property grade, no corpus change.

```
registry 556 · GAP_A · GAP_B · DEBT 121 · TYPES_UNCHECKED 10 · corpus 37   ← ALL UNCHANGED
```

## Runtime

**40-70 min.** Rooms 1 and 4 are the judgment; rooms 2 and 3 are mechanical once that is settled,
and rustc drives the rest.

## Trap doors, named in advance

1. **Inventing a holon rational/bigint leaf.** The tempting "fix" for the two SURPRISE arms, and it
   is wrong — holon-rs has no such leaf, and the real defect is round-tripping through holon at all.
   The input `WatAST` was already correct.
2. **Breaking the VSA path.** `try_recognize_holon_value` is `pub(crate)` and may serve callers who
   genuinely want a holon. STOP-1. Find them before changing the signature.
3. **A probe first seen green.** It must reproduce the corruption before the change. This campaign's
   round-trip probe once returned a false perfect 386/386 by comparing through a lossy projection —
   and *lossy projection* is precisely this stone's subject.
4. **Re-weakening a test rewritten hours ago.** The `step_*` suite was migrated to WatAST
   shape-matches this session under a no-weakening rule. If one must change again, it proves the
   same thing or it stops. STOP-5.
5. ⚠ **Declaring victory on the probe alone.** Rational and bigint are the two arms the source
   NAMES. There may be a third lossy arm it does not name — read every arm of
   `try_recognize_holon_value`, not just the two with comments.
