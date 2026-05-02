# Arc 130 — Calibration sweep: HologramCacheService — Pre-handoff expectations

**Written:** 2026-05-03, AFTER spawning sonnet, BEFORE deliverable.

**Brief:** `CALIBRATION-HOLOGRAM-BRIEF.md`
**Target:** `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
**Output:** rewritten test file + ~300-word report.

## Setup — workspace state pre-spawn

- Baseline `cargo test --release -p wat-holon-lru --test test`: 14 passed; 0 failed; 5 of those are `:should-panic("channel-pair-deadlock")` from step3-6 + step-B; step1 + step2 + 7 LocalCache tests pass cleanly.
- `cargo test --release --workspace` green.
- `wat-lru/wat-tests/lru/CacheService.wat` ships the worked compositional demonstration (5 layered deftests, all `:should-panic`).
- `complectens` SKILL committed at `.claude/skills/complectens/SKILL.md`.
- arc 130 REALIZATIONS + complected/ calibration set in `docs/arc/2026/05/130-cache-services-pair-by-index/`.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single-file diff | Only `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` modified. NO substrate file. NO other tests. NO docs. NO Rust files. |
| 2 | Helpers added in prelude | The `make-deftest :deftest-hermetic` prelude grows new named helpers (above the existing `trivial-worker`, `count-recv`, `counter-worker`). Helpers cover spawn-and-shutdown lifecycle, single-Put, single-Get, multi-Put, eviction-step, etc. |
| 3 | Each step's body shrinks | Each of step1-step6's deftest body is 3-7 source lines (body of `(:deftest-hermetic :name BODY)`, excluding the wrapping). Step1 was already short (~7 lines); step3-6 were 20-30 lines each. |
| 4 | Per-helper deftests | EACH new helper added to the prelude has its own `(:deftest-hermetic ...)` proving it. Bottom-up proofs THEN top-down composition. |
| 5 | No forward references | Every helper in the prelude references only helpers defined ABOVE it in the prelude. The deftests reference only helpers defined in the prelude. `grep -n` confirms top-down. |
| 6 | **Outcomes preserved** | `cargo test --release -p wat-holon-lru --test test` exit=0; the existing 5 `:should-panic` outcomes still pass via panic; step1 + step2 + 7 LocalCache still pass cleanly; new helper deftests have consistent outcomes (some pass, some `:should-panic` on the same substring depending on whether they touch the helper-verb call sites). |
| 7 | No commits | Working tree shows uncommitted modifications. No `git commit` or `git push`. |
| 8 | Honest report | ~300-word report includes: (a) helpers added with line-counts; (b) BEFORE → AFTER body line-count for each of the 6 steps; (c) per-helper deftest list; (d) outcomes verified; (e) honest deltas — gaps in the documents; (f) the four questions applied to the output. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Helper count | 6-15 new helpers added. Fewer = layering not deep enough; more = over-decomposition. |
| 10 | Average body shrink | Average step body shrinks ≥60% (e.g., 25 lines → 10 or fewer). |
| 11 | Workspace runtime | `cargo test --release --workspace` total runtime stays within ~10s of baseline. The added per-helper deftests cost a small amount; substantial growth = something is firing extra. |
| 12 | Document gap report | The "honest deltas" section names AT LEAST one place where the documents could be clearer, OR explicitly says "no gaps found" with a sentence on why. Either is honest; silence is a soft fail. |

## Independent prediction

- **Most likely (~55%):** all 8 hard + 3-4 soft pass. The artifacts teach; sonnet ships clean. 6-12 new helpers, body shrinks 60-80%, workspace green.
- **Discipline applied unevenly (~20%):** 7 of 8 hard pass; deftest bodies shrink unevenly (some good, some still 10+ lines). Specific steps resist clean decomposition because of cross-binding state (e.g., k1, v1, k2, v2 across multiple Put calls in step3). Surfaces a discipline edge case.
- **Per-helper deftests skipped (~10%):** new helpers exist but lack their own deftests. Hard row 4 fails. Discipline degraded to "named factoring" without proof tree. Document gap: the `complectens` SKILL emphasizes both rules but maybe not strongly enough.
- **Outcome regression (~8%):** test counts diverge from baseline. Some `:should-panic` test stops panicking, OR some clean test starts panicking. Hard row 6 fails. Likely cause: helper signature subtly changed the deadlock pattern.
- **Document ambiguity surfaced (~5%):** sonnet can't proceed because the documents leave a real ambiguity. Reports back without modifying the file (or modifies partially). The report itself is the deliverable. Documents need refinement; that becomes the next arc.
- **Edge-case substrate quirk (~2%):** an interaction we didn't anticipate (e.g., make-deftest prelude size limits, hermetic forking quirk).

## Methodology

After agent reports back:

1. Read this file FIRST.
2. Score each row with concrete evidence.
3. `git diff --stat` → ONE file modified.
4. `cargo test --release -p wat-holon-lru --test test 2>&1 | tail -20` for outcomes.
5. `cargo test --release --workspace 2>&1 | tail -3` for full workspace.
6. Read the rewritten file's structure top-down; verify no forward references; verify per-helper deftests exist; verify body line-counts.
7. Read the honest deltas: what document gaps did sonnet surface?
8. Score; commit `CALIBRATION-HOLOGRAM-SCORE.md`.

## What this calibration tells us

- **All hard pass + clean honest deltas** → the discipline propagates cleanly via the artifacts. The grimoire works. The next arc spawning a sonnet for a similar discipline-application task can dispatch with confidence.
- **Hard pass with surface-able document gaps** → the artifacts MOSTLY teach; refinements ship as a follow-up commit. Each refinement makes the next dispatch cleaner.
- **Hard fail** → the artifacts have a substantive gap. Diagnose which artifact (SKILL? REALIZATIONS? complected/ README? worked demo?) was the load-bearing failure. Fix it. Re-spawn.

The clean-delegation hypothesis (from arc 126's REALIZATIONS): "us delegating to sonnet is proof our discipline is sound." This sweep tests that hypothesis against the *complectēns* discipline specifically.

## What follows

- **All clean:** commit the rewrite + this expectations doc + the score doc together. Update arc 130 status to reflect the calibration result.
- **Gaps surfaced:** ship the rewrite, then ship the document refinements as a follow-up commit referencing the SCORE doc as motivation.
- **Hard regression:** revert sonnet's deliverable (or stash); diagnose the document gap; re-spawn after refinement.
