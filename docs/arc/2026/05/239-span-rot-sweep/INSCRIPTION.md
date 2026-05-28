# Arc 239 — INSCRIPTION (span-arity test-rot sweep + the visibility-gap class-fix)

**Closed 2026-05-27.** Surfaced while scoring arc-237 Stone 237.7a: a full
`cargo build --tests --workspace` revealed **21 compile errors across 15
integration-test targets** — invisible because the tracked green-metric was
`cargo test --lib` (834/0), which compiles ONLY `src/lib.rs`. Every `tests/*.rs`
and `crates/*/tests` is a separate compile unit the lib build never touched, so
span-coordinate signature drift (arc 138 / 233) piled up behind the metric.

## What shipped

- **Span-arity cascade FIXED** (`de7a2fcf`) — appended `Span::unknown()` (the
  test-scaffolding idiom) at each `expected Span, found Value` site across the
  15 targets + the one records-split `E0026` destructure. Pure arg-threading;
  no behavior change. `cargo build --tests --workspace` → 0 errors.
- **3 stale-world tests updated** (`e7809aad`) to current-correct substrate
  (`wat_arc144` length→empty? exemplar; `probe_arc234_stone15` → holonic variant;
  `wat_arc201` → structured rendering). FM-9 green 19/19.
- **`#566` — the class-fix** (this closure): `scripts/green-gate.sh` runs
  `cargo build --release --tests --workspace` (compile ALL test units) +
  `cargo test --release --lib -p wat` (the run baseline), gating on both.
  Adding the test-BUILD to the routine gate closes the visibility gap so
  signature/span drift can't silently re-rot behind `--lib` again. The full
  `cargo test --workspace` RUN stays OUT of the gate while it leaks processes
  (arc 170's domain) — the script's header documents when it returns. Memory:
  `feedback_green_gate_lib_and_build`.

## The STOP-and-report (per arc 239's own BRIEF) → arc 240

The first full `cargo test --workspace` after the compile-rot fix surfaced **69
runtime failures** that are NOT span-arity / records-destructure. Arc 239's BRIEF
mandated STOP-and-report for exactly those ("the arcs-closing-in-on-themselves
outlier"). They were promoted to **arc 240 (runtime-rot remediation), now CLOSED**
(`8d08eda2`): consumer-`.wat` drift + clean substrate gaps fixed; every non-drift
failure affirmatively scoped to its owning open arc (119/130/170/213) with
KNOWN-BROKEN markers. arc 240 is 239's child; it wound first (spawn-block).

## Verification (FM-9)

`./scripts/green-gate.sh` → PASS (test-build 0 errors + lib 834/0, exit 0).

## Doctrine carried forward

- **The `--lib`-only metric is a visibility trap.** A green-gate must compile
  every test unit, not just the lib. `green-gate.sh` is the standing antidote.
- Span-coordinate drift is mechanical to fix (the error names each site) but
  invisible without the test-build — failure-engineering the CLASS (the gate),
  not just the symptoms (the 21 sites).

Cross-ref: BRIEF.md (the ledger); SCORE.md + SCORE-stale-tests.md; arc 240
(child, runtime-rot); `feedback_green_gate_lib_and_build`.
