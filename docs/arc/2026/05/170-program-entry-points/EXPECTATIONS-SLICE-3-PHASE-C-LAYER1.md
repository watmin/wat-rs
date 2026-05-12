# Arc 170 slice 3 phase C — EXPECTATIONS (sonnet scorecard)

**One spawn.** Author Layer 1 `:wat::test::run-hermetic` macro + one canonical test passing on the new entry point.

## Independent prediction

**Runtime band:** 60-120 min sonnet. Substantive design call between Path A (pure-wat composition) and Path B (new substrate verb); one canonical test authorship; verification.

**Hard cap:** 240 min. If sonnet hits cap without Layer 1 working, kill via TaskStop and score Mode B-time-violation.

## Scorecard (7 rows; sonnet self-scores then orchestrator verifies)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::run-hermetic` macro defined in `wat/test.wat` | `grep -n "wat::test::run-hermetic" wat/test.wat` shows the macro |
| B | Helper function/verb defined (Path A pure-wat OR Path B substrate) | grep / file inspection |
| C | One canonical test using the new macro passes | `cargo test --release --test <test_file> <test_name>` returns 0 failed |
| D | Workspace failure count UNCHANGED (stays at 0 from 2180/0 baseline) | `cargo test --release --workspace --no-fail-fast` total 0 failed |
| E | `cargo check --release` green | clean compile |
| F | SCORE doc explains the path chosen + rationale + honest deltas | manual review |
| G | NO consumer sweep happened (deftest/deftest-hermetic definitions unchanged) | `git diff wat/test.wat` shows ADDITION of run-hermetic; NO deletion or modification of deftest/deftest-hermetic defmacros |

**7 rows.** All must pass.

## Implementation approach (mirror precedent)

For the canonical test, mirror Phase A precedent from slice 1f-λ:
- Find ONE simple test scenario currently using `deftest` (e.g., a 2+2=4 assertion test)
- Author a NEW test that does the same thing using `:wat::test::run-hermetic`
- The new test goes in a NEW test file OR is appended to an existing canonical home (`tests/wat_arc170_program_contracts.rs` is the obvious candidate — sibling to T1-T16)
- Verify the new test passes
- Leave existing deftest-based tests unchanged

## What sonnet should produce

1. **Code changes:**
   - `wat/test.wat` — `:wat::test::run-hermetic` macro definition appended (do NOT modify existing deftest macros)
   - Helper function/verb (Path A: in wat; Path B: in src/ Rust)
   - One canonical test (new or appended) exercising the new macro
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-C-LAYER1.md`
   - Scorecard verification
   - Path A vs Path B decision + rationale
   - Honest deltas (≥ 3 categories)
   - Files modified
   - What's next (phase D path — Layer 2)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- Do NOT modify `deftest` / `deftest-hermetic` definitions in `wat/test.wat`
- Do NOT retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast`
- Do NOT touch BareLegacy* walker code
- Do NOT touch `Process<I,O>` struct field shape
- Do NOT sweep consumers (phase E)
- Do NOT mass-author Layer 2 (`run-hermetic-with-io`) — phase D only
- Do NOT use deferral language in SCORE — per FM 11
- If you find a substrate gap that makes Path A truly impossible AND Path B requires architectural decisions beyond your scope, STOP and report; do not workaround

## Tools required

- Read / Edit / Bash (cargo, git)
- Possibly Write for SCORE doc + new test file
- No Agent invocations (single-agent slice)

## Verification commands sonnet runs

```bash
# Baseline at start
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Layer 1 macro presence
grep -n "wat::test::run-hermetic\b" wat/test.wat

# Canonical test passes
cargo test --release --test <canonical_test_file> <canonical_test_name> 2>&1 | tail -5

# Final workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline: 2180 passed / 0 failed
- Post phase C: 2181+ passed / 0 failed (one new canonical test added; nothing else changes)

## Honest delta categories (anticipated)

1. **Path A vs Path B decision** — which one won; what blocked the other
2. **RunResult drain mechanism** — how stdout/stderr lines get to `:Vector<String>` (which substrate Rust does today via `test_runner.rs` parsing the child's pipe output)
3. **slice 1f-* service routing** — confirm `:wat::kernel::println` inside the worker fn routes to the captured child stdout pipe; surface any gaps
4. **Anything unexpected** — surfaced during authorship

## Sonnet kickoff prompt

The orchestrator's Agent call passes the BRIEF + EXPECTATIONS paths as required reading. No tool-availability preamble (per FM 16). Give the work directly.

Expected sonnet preamble:
"Execute arc 170 slice 3 phase C — Layer 1 testing-lib rebuild per BRIEF-SLICE-3-PHASE-C-LAYER1.md + EXPECTATIONS-SLICE-3-PHASE-C-LAYER1.md. Substrate-informed grounding: read both docs in full, then DESIGN.md slice 3 spec, then the current macros + sandbox.wat/hermetic.wat, then T4-T6 in tests/wat_arc170_program_contracts.rs. Author Layer 1 macro + ONE canonical test using it; verify workspace stays green; write SCORE; do NOT commit; do NOT sweep consumers; STOP if a substrate gap surfaces."
