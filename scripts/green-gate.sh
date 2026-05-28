#!/usr/bin/env bash
# green-gate.sh — the routine verification gate for wat-rs dev work.
#
# Usage:
#   ./scripts/green-gate.sh
#
# Runs TWO checks, in order, and gates on both:
#   1. cargo build --release --tests --workspace   (compile ALL test units)
#   2. cargo test  --release --lib -p wat           (the lib run baseline)
#
# Exit code: 0 only if BOTH pass; non-zero otherwise (lets you gate commits).
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
# Why NOT the full `cargo test --workspace` RUN (momentary, 2026-05-27):
#   The full workspace RUN leaks processes — the ambient-stdio / fork / lifeline
#   integration tests spawn children + threads that time out and leak. That is
#   exactly the instability arc 170 (program entry points / stdio-trio / spawn-
#   fork reshape) exists to fix. Until 170 closes the leaks, this gate stays
#   build-for-the-workspace + run-for-the-lib. ONCE 170 lands, add the full
#   `cargo test --release --workspace` RUN here and this comment retires.
#   See memory: feedback_green_gate_lib_and_build.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== green-gate 1/2: cargo build --release --tests --workspace (compile all test units) =="
cargo build --release --tests --workspace

echo "== green-gate 2/2: cargo test --release --lib -p wat (lib run baseline) =="
cargo test --release --lib -p wat

echo "== green-gate: PASS (test-build clean + lib baseline green) =="
