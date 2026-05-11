# Arc 170 slice 1f-κ — SCORE (readln contract migration)

**Result:** Mode A clean. Direct execution (sub-spawn-threshold). 1 test migrated to the slice-1f-ι contract; the predicted `row_e_readln_roundtrip` failure closes.
**Runtime:** ~2 min orchestrator (no sonnet spawn).
**Files:** 1 modified — `tests/wat_arc170_slice_1f_gamma_orchestrator.rs`.

**Workspace: 2162/37 → 2163/36** — +1 pass / -1 failure exactly as predicted by SCORE-SLICE-1F-IOTA.

## § The migration

```diff
- [_form (:wat::kernel::readln)]
+ [_s (:wat::kernel::readln -> :wat::core::String)]
```

Plus a 3-line comment update: pre-ι "hands the HolonAST back via readln" → post-ι "passes the raw line; the substrate parses + coerces to T per readln's `-> :T` annotation". The slice-1f-ι contract is now visible in the test prose.

## § Scope sizing — why no sonnet spawn

The triage in SCORE-1F-IOTA projected ~10-15 readln-contract migrations. **Actual: 1 test.** All other readln-suspects were blocked behind the legacy `:user::main` 4-arg signature (`BareLegacyMainSignature` check error) — those fail at parse time before readln contract evaluation, so the readln issue never surfaces. Once slice 1f-λ migrates main signatures, additional readln consumers may surface; if so, they get bundled into 1f-λ or a follow-up.

Per recovery doc § 5 — proactive slicing — this slice was a STEPPING STONE: a 1-test fix proving the readln migration pattern (annotation addition + comment refresh) before the larger 1f-λ legacy-main sweep operates on settled ground.

## § Lessons captured

1. **Projection-vs-reality calibration.** SCORE-IOTA projected 10-15 1f-κ tests based on the workspace failure count; reality was 1. The "hidden behind a different check error" failure pattern collapses bucket size dramatically. Lesson: triage by ACTUAL failure text (capture `cargo test` panic message per failing test), not by test-name pattern matching.

2. **Sub-spawn-threshold direct execution.** A 5-line edit + comment refresh does not warrant the BRIEF + EXPECTATIONS + Agent ceremony. Direct orchestrator execution + tight SCORE is the right shape when scope collapses below the spawn floor.

3. **Stepping stones can be tiny.** Per `feedback_stepping_stones_proactive.md` the discipline isn't "every slice is a sonnet sweep" — it's "simple steps enable complex steps." A 1-test ship that proves the pattern + shrinks the failure count is a valid slice.

## § Files modified

- `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` — `row_e_readln_roundtrip` body: bare `(:wat::kernel::readln)` → `(:wat::kernel::readln -> :wat::core::String)`; comment block updated to reflect slice-1f-ι contract

## § What's next

1. **Slice 1f-λ** — legacy `:user::main` signature migrations (~22 tests; `BareLegacyMainSignature` + `BareLegacyForkProgram` failures). Largest fix-up bucket; sonnet-sized.
2. **Slice 1f-μ** — wat-cli + examples (raw-stdout / wat-cli echo / with_loader / with_lru / programs-are-atoms / sigterm-cascade). Pattern overlaps with 1f-λ since these also have legacy mains; scope may merge.
3. **Triage** — 4 `slice4_*` heterogeneous-dispatch failures (independent of arc 170).
4. **Arc 170 INSCRIPTION** when baseline is clean.

## § Cross-references

- Contract source: [`BRIEF-SLICE-1F-IOTA.md`](./BRIEF-SLICE-1F-IOTA.md)
- Substrate landing: [`SCORE-SLICE-1F-IOTA.md`](./SCORE-SLICE-1F-IOTA.md)
- User direction 2026-05-10: *"go make println and readln work — it'll break a bunch of existing tests which is correct — we must fix them after we make the contract work"*
