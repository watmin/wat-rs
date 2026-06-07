#!/usr/bin/env bash
# green-gate.sh — the routine verification gate for wat-rs dev work.
#
# Usage:
#   ./scripts/green-gate.sh
#
# Runs FOUR checks, in order, and gates on all of them:
#   1. ./scripts/gen-test-mods.sh --check            (every grouped test dir's
#      mod.rs is current — no test file silently undeclared / ignored)
#   2. cargo build --release --tests --workspace   (compile ALL test units)
#   3. cargo test  --release --lib -p wat           (the lib run baseline)
#   4. ./scripts/integration-run.sh                  (the leak-contained
#      integration tier: every non-leaky-signal test binary, each run in its
#      own setsid session with timeout + reap; exit 0 iff every binary passes)
#
# Exit code: 0 only if ALL pass; non-zero otherwise (lets you gate commits).
#
# Why this script exists (arc 239 #566 — the span-rot class-fix):
#   The tracked green-metric used to be `cargo test --lib` alone, which compiles
#   ONLY src/lib.rs. Every tests/*.rs and crates/*/tests is a separate compile
#   unit the lib build never touches — so signature / span-coordinate drift piled
#   up INVISIBLY behind the metric until arc 239's first full `cargo build --tests
#   --workspace` surfaced 21 compile errors across 15 targets. Adding the
#   test-BUILD to the routine gate closes that visibility gap WITHOUT running the
#   leaky process tests.
#
# Why check 3 is the CONTAINED tier, not the raw `cargo test --workspace` RUN
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

echo "== green-gate 1/4: gen-test-mods.sh --check (grouped test mod.rs lists current) =="
./scripts/gen-test-mods.sh --check

echo "== green-gate 2/4: cargo build --release --tests --workspace (compile all test units) =="
cargo build --release --tests --workspace

echo "== green-gate 3/4: cargo test --release --lib -p wat (lib run baseline) =="
cargo test --release --lib -p wat

echo "== green-gate 4/4: integration-run.sh (the leak-contained integration tier) =="
./scripts/integration-run.sh

echo "== green-gate: PASS (mod-lists current + test-build clean + lib baseline green + integration tier green) =="
