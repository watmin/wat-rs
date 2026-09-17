# BRIEF 7j ADDENDUM — #324 and the `:then` match collision: the ruling, and how to land it

**Read `BRIEF-7j-replay-batch-4j.md` first — it is still in force in full.** This addendum resolves the
STOP at #324 and supersedes only what it names. It exists because batch 4j stopped there, correctly,
rather than committing anything red.

## What happened, and what was ruled

grok's **#324** (`ab606b671`, *"fix(rete): D5 — a match arm's PATTERN is not a constructor call"*) fixes a
real diagnostic defect: the `:then` walker read a match arm's **pattern** as a constructor **call**, so an
illegal `:then` died with a phantom `RhsArityMismatch` instead of the fence's true refusal.

The fix is sound. Two of its five tests are not portable here:

| grok's test | on this tree |
|---|---|
| `the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error` | **PASSES** — grok agrees `:wat::core::match` is illegal in a `:then` |
| `the_banked_d5_repro_pair_both_load` | **PASSES** |
| `a_misspelled_constructor_in_a_match_arm_body_is_still_refused` | fails — **EDN golden format only**, not a defect |
| `the_bare_and_wrapped_then_spellings_compile_and_agree` | **fails — the collision** |
| `a_correct_constructor_in_a_match_arm_body_still_fires` | **fails — the collision** |

The collision is a **permanent, ruled, main-side design decision**: `250162a0e` (`SCORE(277)`, ACCEPTED
after one reland, floor 5189/5189) makes the `:then` item fence refuse `:wat::rete::core::match`
**outright**, for any spelling, exhaustive or not. Its reasoning, verbatim from that commit:

> a `:then` admits only what the fence can prove total, `total?`'s contract is head-level, match is a
> total HEAD, and a match's exhaustiveness belongs to ITS ARMS. The axis cannot tell an exhaustive match
> from a partial one, so admitting the good one means admitting both. It refuses both.

Verified by the orchestrator: that commit is an **ancestor of this batch's own start point**, is **absent
from grok's branch entirely**, and is **never revisited by grok's remaining ~300 commits**. It is live
here as `tests/rete/probe_then_match_is_refused.wat` (driven by `probe_then_fence_and_enum_name.rs:54`).
grok's opposing probes survive to grok's tip. **Neither side ever revisits it, so the fold rule does not
reach this** — there is no later commit to fold.

⛔ **BUILDER RULING, 2026-09-16, FOUR YES: OPTION A.** Rewrite the two colliding tests to assert refusal,
land the walker fix, regenerate the EDN golden. **Option B — reopening `250162a0e` to narrow the fence —
was weighed at 2 YES and is NOT ruled out for the future, but it is a `wat/` change needing its own stone
and it is NOT part of this batch.** ⛔ **Do not touch `wat/rete/compile.wat`.**

## How to land #324

1. **Cherry-pick as normal** (`git cherry-pick -x --no-commit`, per BRIEF-1 item 1).
2. **Resolve the two merge conflicts.** The previous executor resolved both correctly and verified them
   with a clean build; reproduce that reasoning rather than inventing new resolutions:
   - a `.wat` header conflict already superseded by #212's own prior disposition;
   - a `.rs` build error where grok's new call sites did not thread a `binds` parameter this tree's
     signature requires.
   State in the body what each conflict was and how it was resolved.
3. **Land the walker fix in `src/rete/validate/mod.rs` UNCHANGED.** This is the step's real content and
   main benefits from it regardless of the collision.
4. **Regenerate the EDN golden** for `a_misspelled_constructor_in_a_match_arm_body_is_still_refused` with
   `UPDATE_EDN=1`. ⛔ **Then PROVE the regeneration is convention-only** — diff before/after and confirm
   every field and span value is byte-identical, only the wrapper tag convention differing. That is
   #262's precedent from batch 4i, and it is the check that stops a "format fix" from silently moving a
   measured value.
5. **Rewrite these two tests to assert REFUSAL:**
   - `the_bare_and_wrapped_then_spellings_compile_and_agree`
   - `a_correct_constructor_in_a_match_arm_body_still_fires`

   ⛔ **Follow the shape of grok's own `the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error`**,
   which already passes here — that test is the template, written by grok, for exactly this assertion.

   ★ **The rewritten tests must still demonstrate the walker fix.** That is the whole point: BEFORE the
   fix the refusal came out as a phantom `RhsArityMismatch`; AFTER it, the refusal names the true Stone-C
   reason. **Assert on the diagnostic's content, not merely that it refused** — a test that only checks
   `!ok` would pass without the fix and would prove nothing.
6. **Decide the `.wat` fixtures by MEASUREMENT, not assumption.** `probe_arc278_match_arm_then_rete_bare.wat`
   and `_then_wrapped.wat` are inputs that will now be refused. Whether they stay `.wat` or become
   `.wat.bad` depends on what the gates actually require — check `every_tracked_wat_parses` and
   `every_ungated_wat_checks` against these paths and report what you found. Deliberately-refusing `.wat`
   fixtures already exist under `tests/rete/` (batch 4f landed three), so either answer may be correct.

## What the commit body MUST carry

- The divergence, named: **grok's tests were INVERTED, and why** — `250162a0e`, quoted or cited.
- That this is a **builder-ruled 4-YES divergence**, not an executor's judgment.
- That **grok agrees about the `:wat::core::` spelling**; only `:wat::rete::core::` collides.
- That the walker fix landed unchanged, and how the rewritten tests still prove it.
- The usual path-based verdict lines (this step touches `^src/`, so `census:` + `nested-program-gate:`
  are required, plus `lint-subset` / `kind(lib)` / `doctest`).

## ⛔ What NOT to do

- Do **NOT** touch `wat/rete/compile.wat` or any part of the fence. That is option B and it is not ruled.
- Do **NOT** weaken, delete, or ignore-mark `tests/rete/probe_then_match_is_refused.wat` or
  `probe_then_fence_and_enum_name.rs`. Main's ruling stays live and tested.
- Do **NOT** `#[ignore]` any of grok's five tests. Option C was weighed at 1 YES and refused.
- Do **NOT** skip #324 and proceed to #325. Option D was weighed at 1 YES and refused: #328 and #336 land
  on top of this walker, and a hole mid-range is the composition hazard the fold rule exists to prevent.

## Rows this addendum supersedes

`EXPECTATIONS-7j` is **not amended** — a contract is not edited after its results are seen (finding 34).
These rows are superseded here instead:

- **E7** for #324 only: its five named tests are now **5/5 green with two inverted**, not 5/5 as grok
  wrote them. The SCORE must show the inversion explicitly.
- **E19** gains a required disclosure: the SCORE must state that two of grok's assertions were inverted
  under a builder ruling, in the row that reports them green — not only in the log.
- **E22** applies with full force: this addendum is itself a deviation, and the SCORE reports how it went.

All other rows stand unchanged. After #324 lands, **resume #325 → #340 under BRIEF-7j as written** — its
warnings for **#329** (grok's own subject says it pushed a red floor) and **#332** (a finding, not a fix)
are still un-adjudicated and still apply.
