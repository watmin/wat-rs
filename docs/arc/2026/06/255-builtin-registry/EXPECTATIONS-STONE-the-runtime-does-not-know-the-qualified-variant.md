# EXPECTATIONS — STONE: the qualified builtin-variant spelling. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the probe FAILS first | run it pre-change | `PatternMatchFailed` on the qualified arms | probe 5, run this session |
| 2 | the probe passes after | rebuild, run it | all four variants match | the six guards |
| 3 | the bare spelling still works | a bare-spelling probe | unchanged | STOP-1, additive only |
| 4 | user enums unaffected | `binary_id(wat::types)` | green | probe 1 — they already worked |
| 5 | the loader gate | `-E 'test(every_wat_scripts_file_loads)'` | pass | the probe is a `wat-scripts/` file |
| 6 | scoped suites | the four `binary_id` runs | green | Option/Result are everywhere |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement

**None, and derived.** No registration, no retirement, no property grade, no corpus change — the
corpus still spells everything the bare way and this stone does not migrate it.

```
registry 556 · GAP_A · GAP_B · DEBT 121 · TYPES_UNCHECKED 10 · corpus 37   ← ALL UNCHANGED
```

⚠ **The corpus stays at 37 and `:wat::core::None` stays an artifact.** This stone makes the exit
*possible*; the codemod is what takes it. A report claiming the artifact count moved is wrong.

## Runtime

**20-35 min.** Four guards are mechanical; rooms 4's two sites need judgment and may correctly
end in "not extended, here is why."

## Trap doors, named in advance

1. **A probe first seen green.** The single most likely way to ship nothing. It MUST raise before
   the change. `[[feedback_a_green_test_can_prove_nothing]]`, and this campaign's own round-trip
   probe once returned a false perfect 386/386 by comparing through a lossy projection.
2. **Tidying `src/check.rs`.** 21 sites carry the same spellings and are already correct. STOP-2.
3. **Adding an unreachable arm in room 4.** The shadowed-residue defect, 52 of which this campaign
   has deleted. STOP-4 — report, do not tidy.
4. **Removing the bare spelling because the qualified one now works.** STOP-1. The corpus is still
   entirely on the bare form; removing it here would red the floor at 6346 sites and invert the
   anneal.
