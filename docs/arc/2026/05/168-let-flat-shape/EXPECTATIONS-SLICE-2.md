# Arc 168 slice 2 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-120 minutes (sonnet agent).**

Reasoning: ~563 sites per slice 1 SCORE measurement. Mechanical
recipe is settled (3 binder shapes covered). Substrate-as-teacher
loop is well-rehearsed (arc 167 slice 3 precedent at similar scale,
plus arcs 154 / 159 sweeps).

This is the FIRST sonnet sweep in arc 168 — the calibration data
the permission investigation paid for. Sonnet's empirical viability
was verified post-restart: the inline `cargo test ... | grep "^test
result" | awk '...'` pipeline runs cleanly via sonnet (probe
agentId a960c0573085058e0 returned `passed: 15 failed: 0` for the
arc 168 test bin).

**Time-box (2× upper-bound): 240 minutes.** If sonnet still
iterating at 120 min, in-flight check; hard cap at 240.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Workspace failures dropped to 0 | inline pipeline output `passed: N failed: 0` | ✓ |
| B — wat/ stdlib swept | `grep -rEn '\(:wat::core::let\b.*\(\(' wat/`: 0 hits | ✓ |
| C — wat-tests/ user-source swept | walker fires nowhere in wat-tests/ runs; cargo test green for wat-tests-driven binaries | ✓ |
| D — tests/wat_*.rs embedded strings swept | `grep -rEn '\(:wat::core::let\s*\(\(' tests/`: 0 hits | ✓ |
| E — crates/*/wat-tests/ swept | parallel grep clean | ✓ |
| F — crates/*/wat/ swept | parallel grep clean | ✓ |
| G — Slice 1 substrate untouched | `git diff src/` shows zero changes vs slice 1 final | ✓ |
| H — Walker untouched | `walk_for_legacy_let_bindings` unchanged | ✓ |
| I — Canonical parser untouched | `parse_let_binding` (the bare-Symbol/Vector branch) unchanged | ✓ |
| J — Tests/assertions untouched | NO test logic edits, NO assertion edits — ONLY translation of let bindings inside test fixtures | ✓ |
| K — Mechanical translation only | each migration is binder-shape change + body unchanged; no semantic reshapes | ✓ |
| L — Slice branch on remote | branch carries N WIP commits with progress; main untouched | ✓ |
| M — Single-body remains valid (regression) | tests using single-form let body still pass after their bindings migrate | ✓ |
| N — Sonnet ran the inline pipeline cleanly | report references the EXACT verification command without embellishment | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Site that doesn't fit the recipe.** A let form might have unusual
  nesting, escaped quoting, or be testing legacy syntax intentionally.
  STOP and report; orchestrator decides.
- **Test fails post-migration that should pass.** If mechanical
  translation is applied correctly but the test still fails, that's
  a real substrate-consumer dependency. STOP and report; do NOT
  modify substrate, do NOT rewrite the assertion.
- **More sites than predicted (~563).** If the count is materially
  off (>700 or <400), surface as an honest delta — could indicate a
  scope mismatch with slice 1's measurement.
- **Walker fires on stdlib.** Slice 1 SCORE confirmed walker scoping
  holds for stdlib via `freeze.rs` user-source pre-pass. If sonnet
  finds stdlib forms firing the walker, that's a substrate gap.
  STOP and report.
- **Sonnet adds shell glue to the verification commands.** If sonnet
  pipes through `head -N`, adds `2>&1` (extra), `; echo $?`, etc. —
  surface as a discipline failure in SCORE. Memory
  `feedback_script_invocation_no_embellishment.md` captures this rule.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 60-120 min band.

Sites by region: wat/ ___, wat-tests/ ___, tests/wat_*.rs ___,
crates/*/ ___, examples/ ___.

Honest deltas surfaced: ___ (count + brief).

Sonnet behavior on inline pipeline: clean / embellished / hallucinated
denial.

## What's next (orchestrator-side, post-slice-2)

When slice 2 ships green:
- Slice 3 BRIEF + EXPECTATIONS for substrate retirement (opus —
  delete walker, delete legacy parser arms in eval_let / infer_let,
  delete `is_typed_single` branch in parse_let_binding)
- Predicted: 30-60 min opus

Then slice 4 (lib unit-test fixture sweep — arc 167 slice 4b
precedent), then slice 5 (closure paperwork).

## SCORE artifact

Sonnet's report writes to chat; orchestrator commits SCORE-SLICE-2.md
to slice branch after scoring all rows + reviewing the diff.
