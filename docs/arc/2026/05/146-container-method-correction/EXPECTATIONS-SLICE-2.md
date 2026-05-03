# Arc 146 Slice 2 — Pre-handoff expectations

**Drafted 2026-05-03.** First migration via the dispatch mechanism.
Predicted SMALL slice (Mode A ~70%; Mode B-arc144-test-breakage
~15%; Mode B-load-order ~10%; Mode C ~5%).

**Brief:** `BRIEF-SLICE-2.md`
**Output:** EDITS to `src/runtime.rs` + `src/check.rs` +
`src/stdlib.rs` + NEW `wat/core.wat`. ~150-300 LOC + report.
**No new test file** — the load-bearing proof is the existing
slice 6 canary turning GREEN.

## Setup — workspace state pre-spawn

- Slice 1 closed (`5ae33d1`) + slice 1b rename closed (`d59f285`).
  Substrate has `Dispatch` entity + `define-dispatch` form +
  reflection wired.
- Q3 from slice 1: dispatch wins over primitives in lookup_form
  precedence. The dispatch will take effect at the moment it's
  registered, even before retirement of the old machinery.
- 1 in-flight uncommitted file (CacheService.wat — arc 130;
  ignore).
- Workspace baseline (FM 9): all green except slice 6 length
  canary.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to `src/runtime.rs` (3 new fns + 3 dispatch arms; eval_length + 1 dispatch arm DELETED) + `src/check.rs` (3 new env.register; 1 dispatch arm + infer_length + 1 register DELETED) + `src/stdlib.rs` (1 WatSource entry added). NEW `wat/core.wat`. NO new test files. NO other Rust changes. |
| 2 | 3 per-Type impls | `eval_vector_length`, `eval_hashmap_length`, `eval_hashset_length` all present; each takes 1 arg, type-checks its container, returns `:i64`. |
| 3 | 3 dispatch arms | Added to eval_list_call's keyword switch (BELOW the dispatch_registry guard from slice 1). |
| 4 | 3 TypeScheme registrations | In `register_builtins`; each is `forall T. ContainerShape -> :i64` (or 2 type-params for HashMap). Adjacent to where the OLD `:wat::core::length` registration was (which is also DELETED). |
| 5 | `wat/core.wat` exists | Header comment + the `define-dispatch` declaration for `:wat::core::length` with 3 arms pointing at the per-Type impls. |
| 6 | `wat/core.wat` registered | Added to STDLIB_FILES at a position BEFORE `wat/runtime.wat`. |
| 7 | Old machinery RETIRED | `eval_length` + `infer_length` functions DELETED; their dispatch arms DELETED; the arc 144 slice 3 TypeScheme registration for `:wat::core::length` DELETED. |
| 8 | **LENGTH CANARY GREEN** | `cargo test --release --test wat_arc143_define_alias` 3/3. The `define_alias_length_to_user_size_delegates_correctly` test was 2/3 pre-slice; now 3/3. THIS IS THE LOAD-BEARING ROW. |
| 9 | All other baseline tests pass | `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` should still pass (or be adapted per Q2 — sonnet's report names what was updated); `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8. |
| 10 | Honest report | ~250-word report covers all required sections; Q1+Q2+Q3 decisions named with rationales. |

**Hard verdict:** all 10 must pass. Row 8 is THE load-bearing
proof: arc 130 → arc 143 → arc 144 → arc 146 cascade closes its
first chain link.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget (150-300) | Total slice diff including retirements + new code. |
| 12 | Style consistency | New per-Type impls mirror the existing eval_length shape (split per container). New TypeScheme registrations mirror existing register_builtins entries. wat/core.wat header mirrors wat/list.wat shape. |
| 13 | clippy clean | No new warnings. |
| 14 | Workspace failure profile shrinks | Pre-slice: length canary + CacheService.wat noise. Post-slice: ONLY the CacheService.wat noise (length canary closed). |

## Independent prediction

- **Most likely (~70%) — Mode A clean ship.** Brief is detailed +
  pre-flighted; the work is mechanical (mirror existing patterns;
  retire what's named). ~10-20 min wall-clock (smaller scope than
  slice 1; the mechanism does the heavy lifting).
- **Surprise on arc 144 hardcoded_primitives test (~15%) — Mode
  B-arc144-test.** The test asserts something about
  `:wat::core::length`'s scheme that breaks when the scheme is
  retired. Sonnet adapts (Q2) + reports.
- **Surprise on stdlib load order (~10%) — Mode B-load-order.**
  wat/core.wat needs to load AFTER something specific (e.g., the
  per-Type schemes need to be registered before parser sees the
  arm references). If hit: surface clean; orchestrator decides
  scope.
- **Borrow / type friction (~5%) — Mode C.** New per-Type impls
  may surface friction with the existing eval/value machinery.
  Adapts.

**Time-box: 40 min cap (2× upper-bound 20 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 4 file changes expected (3
   modified, 1 new).
4. Read the 3 new per-Type impls + their dispatch arms.
5. Read the 3 new TypeScheme registrations.
6. Read wat/core.wat content.
7. Verify retirements via `git diff src/runtime.rs src/check.rs`
   — confirm eval_length + infer_length + dispatch arms +
   length scheme are GONE.
8. **Run wat_arc143_define_alias — confirm 3/3 (load-bearing).**
9. Run all baseline tests — confirm zero regression (or Q2 update).
10. Run workspace tests — confirm shrunk failure profile.
11. Run clippy.
12. Score; commit `SCORE-SLICE-2.md`.

## What this slice unblocks

- **Slices 3-6** — same shape for empty?, contains?, get, conj
  families. Each ~10-15 min once the proof is in.
- **Slice 7** — pure rename family (no dispatch needed) — different
  shape; lighter.
- **Slice 8** — closure paperwork.
- **Arc 144 slice 4** — verification simpler post-slice-2 (the
  length canary that arc 144's slice 4 was originally to verify
  is now green).
- **Arc 130 RELAND v2** — the next chain link in the cascade
  becomes accessible.

The substrate's first poorly-defined primitive becomes properly
defined. Foundation strengthens by one primitive. Per § 12: the
slow path is the right path; each migration compounds.

The proof of the methodology: orchestrator + sonnet + substrate-
informed brief + clean retirement = working migration with the
load-bearing canary closed.
