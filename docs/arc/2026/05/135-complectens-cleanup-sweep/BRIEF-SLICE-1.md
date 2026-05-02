# Arc 135 Slice 1 — Sonnet Brief: service-template + console (top-level wat-tests)

**Goal:** apply the *complectēns* discipline to TWO top-level wat-test files in `wat-tests/`. Each becomes a top-down dependency graph: named helpers in `make-deftest` prelude(s), per-helper deftests, final scenario deftests with bodies 3-7 lines. Outcomes preserved.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** TWO files —
1. `wat-tests/service-template.wat` — flagship 🔴 (106-line monolithic deftest body).
2. `wat-tests/console.wat` — 🔴 (101-line monolithic deftest body) + 🟡 (44-line).

NO substrate changes. NO other test files. NO documentation. NO commits.

## Read in order — your contract

1. `.claude/skills/complectens/SKILL.md` — the spell. Four questions, severity levels, edge cases (TWO-PRELUDE PATTERN, cross-function tracing warning, pop-before-finish).
2. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md` — the discipline.
3. `docs/arc/2026/05/130-cache-services-pair-by-index/CALIBRATION-HOLOGRAM-SCORE.md` — the calibration sweep that validated this discipline AND surfaced the edge cases.
4. **The worked demonstrations:**
   - `crates/wat-lru/wat-tests/lru/CacheService.wat` — single-prelude form.
   - `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — two-prelude form for mixed-outcome files.
5. Then read the targets:
   - `wat-tests/service-template.wat`
   - `wat-tests/console.wat`

## What to produce

Both files rewritten in the *complectēns* shape:

1. **One file per file.** No new helper-deps file; helpers live in `make-deftest` prelude(s) inside the same file.
2. **Each step's deftest body 3-7 lines** composing named helpers from the prelude.
3. **Each new helper has its own deftest** proving it in isolation.
4. **Top-down dependency graph.** Helpers reference only earlier helpers. No forward references.
5. **Outcomes preserved.** If a deftest currently passes cleanly, it still does. If a deftest currently `:should-panic`s, it still does. Use the two-prelude pattern (per HologramCacheService demo) if a file has mixed-outcome deftests.

## Constraints

- ONLY `wat-tests/service-template.wat` and `wat-tests/console.wat` modified.
- NO substrate files. NO other test files. NO Rust files. NO documentation.
- `cargo test --release --workspace` exit=0 with same outcome counts (your additions count toward "more passing tests" — that's expected).
- NO commits, NO pushes.
- Apply the SKILL's edge-case guidance: do NOT factor `make-bounded-channel` allocations into helpers (that silences arc 126); DO pop before finish on lifecycle helpers; DO use two preludes if outcomes mix.

## What success looks like

1. Each file's deftest bodies shrunk to 3-7 lines. Average shrink ≥60%.
2. Per-helper deftests added; outcomes consistent with their helper's pattern.
3. `cargo test --release --workspace` exit=0; no regressions; new passes counted.
4. NO commits.

## Reporting back

Target ~300 words:

1. **Per-file body line-count: BEFORE → AFTER** for each existing deftest in both files. Format: `service-template.wat :svc::test-template-end-to-end: 106 → 5`.
2. **Helpers added** with name, params, body line-count, what each does in one sentence.
3. **Per-helper deftests added** with their outcome class (clean / `:should-panic` reason).
4. **Outcomes verified** — `cargo test --release --workspace` totals + per-file results for the two crates touched.
5. **Honest deltas** — anything the documents didn't anticipate, especially edge cases NOT already in the SKILL's "Edge cases" section.
6. **Four questions applied** to your output.

## What this slice tests (meta)

This is the second cast of *complectēns* on real code (after the HologramCacheService calibration). The hypothesis: the SKILL's three new edge-case sections (two-prelude, cross-function tracing, pop-before-finish) plus the worked demonstrations are sufficient teaching for ANY mixed-outcome service-test file.

If you ship clean — the artifacts work; the discipline propagates. If you encounter ambiguity, name it in your honest deltas. The next slice will benefit from your calibration data.

Begin by reading the artifacts in the order specified. Then plan the layering for both files (helpers may share a shape; preludes stay per-file). Then ship. Then verify.
