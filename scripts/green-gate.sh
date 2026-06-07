#!/usr/bin/env bash
# green-gate.sh — the routine verification gate for wat-rs dev work.
#
# Usage:
#   ./scripts/green-gate.sh
#
# Runs THREE checks, in order, and gates on all of them:
#   1. cargo build --release --tests --workspace   (compile ALL test units)
#   2. cargo test  --release --lib -p wat           (the lib run baseline)
#   3. ./scripts/integration-run.sh                  (the leak-contained
#      integration tier: every non-leaky-signal test binary, each run in its
#      own setsid session with timeout + reap; exit 0 iff every binary passes)
#
# Exit code: 0 only if ALL pass; non-zero otherwise (lets you gate commits).
#
# Why there is no longer a "mod.rs lists current" gate (was check 1/4):
#   The grouped-test mod lists used to be a hand-run script (gen-test-mods.sh)
#   guarded by a --check gate. That whole class is annihilated by build.rs, which
#   generates each group's mod list into OUT_DIR on EVERY build — there is no
#   committed list to drift and no script to forget. Check 1 below
#   (`cargo build --tests`) runs build.rs, so auto-discovery is inherently
#   exercised: a newly-dropped test file that fails to compile fails this gate.
#
# Why check 1 is the full test BUILD (arc 239 #566 — the span-rot class-fix):
#   The tracked green-metric used to be `cargo test --lib` alone, which compiles
#   ONLY src/lib.rs. Every tests/*.rs and crates/*/tests is a separate compile
#   unit the lib build never touches — so signature / span-coordinate drift piled
#   up INVISIBLY behind the metric until arc 239's first full `cargo build --tests
#   --workspace` surfaced 21 compile errors across 15 targets. The test-BUILD in
#   the routine gate closes that visibility gap WITHOUT running the leaky tests.
#
# Why check 2 is the CONTAINED tier, not the raw `cargo test --workspace` RUN
# (arc 245 — supersedes the 2026-05-27 "once 170 lands" note that lived here):
#   The raw workspace RUN leaks processes (the arc-170 process/stdio binaries).
#   The answer that actually shipped is scripts/integration-run.sh (Stone 245.7):
#   per-binary setsid sessions, timeouts, session reaps — the leak contract held
#   live across the full tier. The 245 FULL CLEAR (2026-06-06) drove that tier
#   from 147 failing tests to ZERO (1646/0/59 across 187 binaries); folding the
#   runner into this gate is the clear's endgame — the tier ran RED in the dark
#   for weeks precisely because no routine gate watched it. Now one does, and it
#   cannot rot silently again. The 67 leaky-signal binaries stay excluded by the
#   runner's heuristic; un-quieting them is that runner's documented concern.
#   See memory: feedback_green_gate_lib_and_build; the clear's ledger:
#   docs/arc/2026/06/245-wat-corpus-warding/TRIAGE-FULL-CLEAR.md.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== green-gate 1/3: cargo build --release --tests --workspace (compile all test units; runs build.rs) =="
cargo build --release --tests --workspace

echo "== green-gate 2/3: cargo test --release --lib -p wat (lib run baseline) =="
cargo test --release --lib -p wat

echo "== green-gate 3/3: integration-run.sh (the leak-contained integration tier) =="
./scripts/integration-run.sh

echo "== green-gate: PASS (test-build clean + lib baseline green + integration tier green) =="
