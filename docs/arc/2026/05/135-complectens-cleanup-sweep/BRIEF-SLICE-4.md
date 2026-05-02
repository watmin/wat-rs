# Arc 135 Slice 4 — Sonnet Brief: suspect-tier — phase-2 judgment

**Goal:** apply the *complectēns* discipline to the four suspect-tier files. These are at the body-line threshold (~30-43 lines); some may exempt under phase-2 judgment as inherently complex with documented justification, others compose cleanly. Make the call per file.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** FOUR files —
1. `wat-tests/test.wat` — 🟡×3 (42, 36, 31 line deftests). Meta-tests of the test framework's assertion failure paths. The "construct a deliberate failure → check it surfaces correctly" shape may resist clean decomposition.
2. `wat-tests/stream.wat` — 🟡 (31-line deftest). Stream pipeline test.
3. `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` — 🟡×2 (35-line deftests). Cache state assertions; may be inherently complex match expressions.
4. `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` — 🟡 (43-line deftest). Already in stepping-stone shape; the body IS the proof's content.

NO substrate changes. NO other test files. NO documentation. NO commits.

## Phase-2 judgment

Per the SKILL: line-count is a candidate flag, NOT a verdict. Some 30-line deftest bodies are inherently complex (long `match` expressions on enum state; tight assertion sequences). Read each before deciding to refactor.

For each file, decide:

- **Refactor** — the body has accidental complexity; named helpers shrink it cleanly. Apply the discipline as in slices 1-3.
- **Exempt** — the body is inherently complex. Document the justification in a comment at the top of the deftest (1-2 lines) explaining WHY it doesn't fit the discipline. Move on.

Honest exemption is acceptable. Forced refactoring of inherently-complex tests can MAKE them worse (the discipline is for SCENARIO complexity, not assertion complexity).

## Read in order

1. `docs/arc/2026/05/135-complectens-cleanup-sweep/BRIEF-SLICE-1.md` for the standard shape.
2. Slices 1-3 SCORE docs for calibration data.
3. `.claude/skills/complectens/SKILL.md` (Severity levels — Level 3 taste discussion is relevant for exemptions).
4. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`.
5. Targets.

## Constraints

- ONLY the four files modified.
- NO substrate. NO commits.
- Outcomes preserved.
- Each exempted deftest gets a short justification comment.

## Report (target ~250 words)

For each of the four files:

1. **Verdict:** REFACTOR or EXEMPT. If EXEMPT, the one-line justification.
2. If REFACTOR: BEFORE → AFTER body line counts; helpers added; per-helper deftests.
3. If EXEMPT: comment text added.

Plus the standard outcomes verification + honest deltas + four questions.
