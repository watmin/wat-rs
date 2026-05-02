# Arc 130 — Calibration sweep: HologramCacheService

**Goal:** apply the *complectēns* discipline (`/.claude/skills/complectens/SKILL.md`) to `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`. Restructure the file as a top-down dependency graph: each deftest body shrinks to 3-7 lines composing named helpers; every helper has a deftest proving it.

**This sweep is the validation of the discipline itself.** The artifacts (REALIZATIONS + the spell + the calibration set + the worked demonstration) MUST teach a fresh agent to ship clean compositional work without conversation context. If you ship clean, the documents communicate. If you don't, the documents fall short — and the gap becomes the next arc's work.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** ONE file — `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` (currently 570 lines, 6 deftests). NO substrate changes (the `wat/holon/lru/HologramCacheService.wat` substrate file is OFF-LIMITS). NO other test files. NO documentation. NO commits.

## Read in order

These artifacts are your contract. Read them cover-to-cover, in this order, before touching the test file:

1. `.claude/skills/complectens/SKILL.md` — the spell. The discipline named, the four questions, the levels of severity, what to look for.
2. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md` — the doctrine. Why this discipline exists. Three rules. The worked example (the failure that named the lesson).
3. `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/README.md` — what bad looks like. The failed sonnet sweep frozen at the moment the discipline broke.
4. `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/test.wat` — the monolithic test that triggered the lesson. ~30-binding deftest body. NO diagnostic surface on failure.
5. `crates/wat-lru/wat-tests/lru/CacheService.wat` — the worked demonstration. Five layered deftests; each body 1-6 lines; named helpers in a `make-deftest` prelude. THIS is the shape you produce.

Then read the target:

6. `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — your target. 570 lines, 6 step deftests. PARTIALLY compositional already (uses `make-deftest :deftest-hermetic` with helper preludes for `trivial-worker`, `count-recv`, `counter-worker`) but each step's deftest BODY is monolithic (10-30 sequential let* bindings). Apply the discipline.

## What to produce

A rewritten `HologramCacheService.wat` where:

1. **Each step's deftest body is 3-7 lines.** Currently each step's body is 10-30 bindings. Extract each unit of work into a named helper in the `make-deftest :deftest-hermetic` prelude. The deftest body composes those helpers.

2. **Each new helper carries its own deftest.** When you introduce `:test::lru-spawn-and-shutdown`, write `(:deftest-hermetic :test::test-lru-spawn-and-shutdown ...)` proving it works in isolation. Bottom-up proofs THEN top-down composition.

3. **Top-down dependency graph.** Helpers in the prelude appear before the helpers / deftests that use them. NO forward references. The reader traces the file top-to-bottom without jumping.

4. **Outcomes preserved.** Each step's existing pass/`:should-panic` outcome is unchanged. Step 1 + step 2 still pass cleanly. Step 3-6 still `:should-panic("channel-pair-deadlock")`. The compositional rewrite does not change runtime behavior — only structural shape.

## Constraints

- ONE file modified: `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`. Substrate file at `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` is OFF-LIMITS.
- NO new files (no helper-deps file split — *complectēns* requires ONE file).
- NO documentation work.
- NO commits, NO pushes. Working tree stays modified.
- The `:wat::test::should-panic` annotations on step3-6 STAY. The substrate's helper-verb signatures still trigger arc 126 at freeze; that panic is intentional until arc 130's substrate reshape lands (separate work; not this sweep). Per-helper deftests for helpers that call those signatures will ALSO `:should-panic` on the same substring. Helpers that don't touch the helper-verb call sites (e.g., a pure spawn-and-shutdown lifecycle) pass cleanly.

## What success looks like

1. `cargo test --release -p wat-holon-lru --test test`: same outcome counts as baseline (14 passed today; the new helper deftests will add to the count — that's the point). 0 failed. The 5 existing `:should-panic` tests still PASS via panic-as-expected. Any new helper deftests pass cleanly OR `:should-panic` consistently with their helper-verb usage.

2. `cargo test --release --workspace`: green. Same pass-count as baseline + your additions. NO regressions.

3. Each step deftest body line-count: substantially shrunk. Empirical target: 3-7 source lines (excluding empty lines) for the body of each step's deftest. The complexity moves into named helpers in the prelude.

4. No forward references. `grep -n` from each helper to its callees confirms callees are defined ABOVE.

5. NO commits.

## Reporting back

Target ~300 words:

1. **Helpers added.** List each new helper added to the `make-deftest :deftest-hermetic` prelude. For each: name, parameter count, body line-count, what it does in one sentence.

2. **Deftest body line-count: BEFORE → AFTER.** For each of the 6 step deftests, the source-line count of its body (excluding the surrounding `(:deftest-hermetic :name ...)` wrapping). Format: `step1: 14 → 5`, etc.

3. **Per-helper deftests added.** For each new helper, the deftest that proves it. Format: `:test::test-N — proves :test::helper-N (line-count of deftest body)`.

4. **Outcomes verified.** `cargo test -p wat-holon-lru --test test` totals + the explicit `... should panic ... ok` lines for the existing step3-6 tests. Confirm the new helper deftests' outcomes (which pass / which `:should-panic`).

5. **Honest deltas.** Anything you needed to do that the artifacts didn't explicitly cover. Any place where the documents were unclear, ambiguous, or didn't anticipate a case.

6. **The four questions, applied to your output.** Walk through each one against the rewritten file. Where each holds; where any concession had to be made.

## What this sweep is testing (meta)

This sweep IS the validation of arc 130's REALIZATIONS, the *complectēns* spell, and the calibration artifacts together. The hypothesis: a fresh agent reading only those artifacts (no conversation context) can ship a compositional test rewrite that matches the worked demonstration's discipline.

If you ship clean: the documents teach.

If you encounter ambiguity: the documents have gaps. Surface those gaps in your honest deltas — they become the next arc's work (artifact refinement).

You are calibrating both your work AND the discipline's documentation. Both records matter.

Begin by reading the artifacts in the order specified above. Then read the target. Then plan the layering before editing. Then ship.
