#!/usr/bin/env bash
# coverage-gate.sh — the rune-aware warded-home coverage gate (arc 252.1).
#
# Managed cargo-llvm-cov (the leak-safe + CORRECT-correlation engine) -> LCOV ->
# rune-aware analyzer over the warded homes. Doctrine: 100%-minus-argued-runes per
# warded file (docs/COVERAGE-RUNE.md): every uncovered region is tested or carries a
# `// rune:coverage(<cat>) — <reason>`.
#
# WHY managed mode (not hand-rolled `show-env`): hand-rolled show-env + a separate
# `report` mis-manages profraw state (stale-profdata; release-vs-dev profile splits
# break separate-binary correlation -> comms/* read 0%). cargo-llvm-cov's managed
# `--no-report` accumulation + one `report` correlates every tier correctly (comms/*
# 0% -> 52-67%). Proven 2026-06-06.
#
# Tiers (leak-safe; NEVER --workspace): lib + the wat-corpus + every homed [[test]]
# group (auto-discovered from tests/*/). The leaky #[ignore]'d process tests are
# excluded (their non-ignored siblings exercise comms to ~52-67%); a genuinely
# leaky-only path earns rune:coverage(proves-elsewhere) citing the contained run.
#
# Usage: scripts/coverage-gate.sh [--blocks]   (--blocks lists every uncovered block)
set -euo pipefail
cd "$(dirname "$0")/.."
LCOV="target/coverage.lcov"

# Auto-discover the homed [[test]] groups (tests/<group>/mod.rs).
group_args=()
for d in tests/*/; do
    [[ -f "${d}mod.rs" ]] && group_args+=(--test "$(basename "$d")")
done

echo "== coverage-gate 1/3: managed accumulate (clean + lib + corpus + ${#group_args[@]} group flags) =="
# A COVERAGE gate MEASURES; it does not enforce pass/fail (green-gate does that). A
# flaky/failing test still contributes the coverage of the lines it ran, so tolerate a
# non-zero exit here and report it — never abort the measurement on a test outcome.
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report --release -p wat --lib              || echo "  WARN: lib had test failures — coverage still collected"
cargo llvm-cov --no-report --release -p wat --test test       || echo "  WARN: corpus had test failures — coverage still collected"
cargo llvm-cov --no-report --release -p wat "${group_args[@]}" || echo "  WARN: a group had test failures (e.g. a flake) — coverage still collected"

echo "== coverage-gate 2/3: emit LCOV =="
cargo llvm-cov report --release --lcov --output-path "$LCOV"

echo "== coverage-gate 3/3: rune-aware warded-home check (100%-or-runed) =="
python3 scripts/coverage_rune_check.py "$LCOV" "$@"
