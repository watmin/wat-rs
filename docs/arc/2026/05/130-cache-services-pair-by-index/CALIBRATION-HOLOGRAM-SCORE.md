# Arc 130 — Calibration sweep: HologramCacheService — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.

**Agent ID:** `a4dd1fbf445c20b3b`
**Runtime:** ~14 min (865s).

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single-file diff | **PASS** | Only `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` modified. `git status` confirms. No substrate file. No other tests. |
| 2 | Helpers added in prelude | **PASS+** | 12 new helpers across TWO `make-deftest` factories: `:deftest-hermetic` (4 helpers — pure lifecycle, no channel-pair) + `:deftest-service` (8 helpers — service-aware, includes channel-pair patterns that trigger arc 126). The two-prelude split is sonnet's invention to handle the mixed-outcome file (gap 1, see below). |
| 3 | Each step's body shrinks | **PASS+** | step1: 14→1; step2: 40→4; step3: 75→4; step4: 45→6; step5: 65→4; step6: 60→5. **All within 3-7 target.** Average shrink: 75%. |
| 4 | Per-helper deftests | **PASS** | 8 new helper deftests; each one names + proves a helper. 4 use `:deftest-hermetic` (clean pass), 4 use `:deftest-service` (consistent `:should-panic` per channel-pair pattern). |
| 5 | No forward references | **PASS** | File reads top-down. Prelude 1 → its proofs → step1 + step2 → Prelude 2 → its proofs → step3-6. Each helper references only earlier helpers within the same prelude. |
| 6 | **Outcomes preserved** | **PASS** | `cargo test --release -p wat-holon-lru --test test`: 22 passed; 0 failed (was 14). Original 6 step deftests preserve outcome (step1+step2 ok; step3-6 should-panic ok). 8 new deftests added: 4 clean pass, 4 should-panic. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications. No `git commit`. |
| 8 | Honest report | **PASS+** | ~600-word report. Surfaces THREE document gaps as honest deltas (see § "Calibration insights" below). Walks the four questions explicitly. |

**HARD VERDICT: 8 OF 8 PASS. The artifacts taught.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 9 | Helper count | **PASS** | 12 new helpers (predicted band 6-15). Mid-range. |
| 10 | Average body shrink | **PASS+** | Average body shrink: 75% (target ≥60%). Steps 3 + 5 shrunk >93%. |
| 11 | Workspace runtime | **PASS** | `cargo test --release --workspace` exit=0; 100 result blocks all `ok`; runtime within baseline. |
| 12 | Document gap report | **PASS+** | Three explicit gaps named with exact mechanism + reproduction conditions. NOT silent. |

**SOFT VERDICT: 4 OF 4 PASS.**

## Independent prediction calibration

The orchestrator predicted (in `CALIBRATION-HOLOGRAM-EXPECTATIONS.md`):

- **55% all-pass** ← matched, but with the document-gap framing strongly active.
- 20% uneven discipline.
- 10% per-helper deftests skipped.
- 8% outcome regression.
- 5% document ambiguity surfaced.
- 2% substrate quirk.

**Actual:** all 8 hard + 4 soft pass. AND three explicit document gaps surfaced. The 55% "most likely" path AND the 5% "ambiguity surfaced" path BOTH fired — the agent shipped clean AND named where the documents fell short. That's the load-bearing outcome of a calibration sweep.

## Calibration insights — the document gaps sonnet surfaced

This is the load-bearing data. The calibration sweep's purpose is to test whether the artifacts teach. Sonnet shipped clean AND named where they fell short. Three gaps:

### Gap 1 — two-prelude pattern undocumented

**The discovery:** the file mixes deftests that must pass cleanly (step1, step2 — hermetic, no channel-pair) with deftests that must `:should-panic` (step3-6 — channel-pair via helper-verb signatures). A SINGLE `make-deftest` prelude including helpers from both classes makes ALL deftests fire arc 126 at freeze (because the prelude is part of every freeze). Step1 + step2 would change outcome from "ok" to "should panic ok" — a regression we explicitly forbade.

**Sonnet's solution:** TWO `make-deftest` factories in the same file. `:deftest-hermetic` (clean prelude — only Layer 0 helpers that don't allocate channel pairs); `:deftest-service` (service-aware prelude — Layer 1+ helpers that DO have channel-pair patterns, triggering arc 126 panic-as-expected).

**Why it's a gap:** the SKILL + REALIZATIONS describe the one-prelude model. Both implicitly assume all deftests in the file share the same outcome class. Neither anticipates the mixed-outcome case. A fresh agent reading only the documents would put all helpers in one prelude on first attempt, observe step1+step2 regressing, and need to invent the two-prelude split themselves.

**Refinement candidate:** add a section to the SKILL ("When a file has mixed outcome classes") that names the two-prelude pattern explicitly.

### Gap 2 — arc 126 silenced by cross-function indirection

**The discovery:** arc 126's `channel-pair-deadlock` check traces Sender/Receiver arguments back through `(:wat::core::first|second pair)` chains to a `make-bounded-channel` anchor. The trace stops at function-call boundaries (per check.rs line ~2135). Therefore: extracting `(make-bounded-channel ...)` + `(first pair)` + `(second pair)` into a helper function, and calling that helper to RETURN the (Sender, Receiver) tuple, **silences the check**. The same code shape that fires arc 126 inline does NOT fire when wrapped in a helper.

**Why it's a gap:** the SKILL says to extract helpers. The user reading the SKILL would naturally try to abstract channel allocation — cleaner abstraction, less boilerplate. But this accidentally defeats the deadlock-pattern detection. The runtime then HANGS (because the deadlock pattern still exists) instead of cleanly panicking at freeze. Sonnet hit this on second attempt; the test timed out.

**Refinement candidate:** the SKILL needs a "When extracting helpers, do NOT factor `make-bounded-channel` into a helper" warning. OR the substrate's arc 126 check needs to follow function returns (substrate arc — much bigger work).

### Gap 3 — `HandlePool::finish` requires pop-before-finish

**The discovery:** when `HCS::spawn 1` allocates a pool with N=1 slot and the test code calls `HandlePool::finish pool` WITHOUT first popping a handle, the substrate raises an "orphaned handles" runtime error. The lifecycle helper that wants to demonstrate spawn-and-shutdown MUST `HandlePool::pop` first (even if the popped handle is immediately dropped) before calling `finish`.

**Why it's a gap:** this is implicit from the substrate's runtime check. Not surfaced in the discipline documents. Low-stakes (the runtime error is clear when it fires) but worth naming for the spawn-and-shutdown lifecycle helper specifically.

**Refinement candidate:** add a sentence in the SKILL or in REALIZATIONS — "the lifecycle helper must pop before finish" — or in a new substrate-as-teacher hint that fires on this specific shape.

## What this calibration tells us

The clean-delegation hypothesis (from arc 126 REALIZATIONS): "us delegating to sonnet is proof our discipline is sound — I have taught you to teach others."

This sweep validates the hypothesis for the *complectēns* discipline specifically. Sonnet:

1. Read the artifacts cold (no conversation context).
2. Applied the discipline to a 570-line file.
3. Produced 731 lines (file grew from 570) with 22 deftests (up from 14), bodies 1-6 lines.
4. Preserved outcomes.
5. Named exactly where the documents fell short.

The artifacts teach. They also have known gaps that the calibration surfaced — those gaps become the next refinement.

## Next steps

1. Commit the rewrite + SCORE doc together.
2. Refine SKILL for Gaps 1 + 3 (Gap 2 is a substrate arc — open a follow-up arc reference or note it in arc 126's queued follow-ups).
3. Update arc 130 FOLLOWUPS to mark HologramCacheService as ✓ shipped.
4. The same brief shape now applies to the 8 other files in FOLLOWUPS. Each subsequent sweep can reference this SCORE as its calibration baseline.

## Cross-references

- `CALIBRATION-HOLOGRAM-BRIEF.md` — the contract.
- `CALIBRATION-HOLOGRAM-EXPECTATIONS.md` — the pre-handoff scorecard.
- `REALIZATIONS.md` — the discipline.
- `.claude/skills/complectens/SKILL.md` — the spell (refinements queued).
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — the deliverable.
- `FOLLOWUPS.md` — the violation queue.
