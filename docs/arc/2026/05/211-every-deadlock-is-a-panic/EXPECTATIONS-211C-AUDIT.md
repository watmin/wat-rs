# Arc 211c — EXPECTATIONS (orchestrator's independent prediction)

## Independent prediction

- **Runtime band:** 45–60 min Mode A. 11 targets × ~3 min/target run + 10 min catalog + 15 min write-up.
- **Lines changed:** 0 (investigation only)
- **New files:** 1 (SCORE-211C-AUDIT.md, likely 300–500 lines with verbatim panic output)
- **Workspace failure delta:** 0 (no code changes; same 11 targets)
- **Surprises expected:** 3–5 (audit reveals unexpected categorizations; some probes may hang differently than predicted; orphan leaks may worsen during test runs)

## Predicted categorization (orchestrator hypothesis; sonnet replaces with honest evidence)

| Target | Predicted category | Reasoning |
|---|---|---|
| `probe_plain_panic_produces_structured_edn` | D (assert on old format) | Name describes what 211b shipped |
| `probe_no_default_rust_panic_noise_on_stderr` | D (assert on old format) | Tests "no Rust default noise" — only meaningful with installed hook |
| `probe_runtime_err_stderr_visibility` | D or A | Visibility tests likely assert on text format |
| `probe_runtime_error_produces_structured_edn` | D | Name says "structured edn" — likely outdated assertion |
| `probe_lifeline_pipe_proof` | B (pre-existing flake) | Noted by 211a SCORE as flaky |
| `probe_run_hermetic_no_deadlock` | A or B | Hermetic runs may interact with dup-removal |
| `test` | mixed | wat::test! umbrella; could hide multiple subtests |
| `wat_arc113_cross_fork_cascade` | A or C | Cross-fork; pipe-stdio-sensitive |
| `wat_arc170_program_contracts` | mixed A+B | t14 is original live repro (A); other contracts may be B |
| `wat_run_sandboxed` | A or D | Sandboxed runs may exercise dup-removal path |
| `wat_cli` | A or C | wat-cli stdio pipes; could be regression |

**Predicted category counts:**
- A (dup regression): 2–4 targets
- B (pre-existing flake): 1–2 targets
- C (foundation): 0–1 targets
- D (assertion update needed): 4–5 targets
- E (other): 0–1 targets

If categorization is mostly D + B: 211d is a SHORT slice (assertion updates).
If categorization includes A: 211d either reverts the dup OR ships surgical fix.

## Scorecard predictions

| # | Criterion | Expected result |
|---|---|---|
| 1 | `panic_any!` sites cataloged with file:line + payload type | YES |
| 2 | All 11 targets investigated | YES |
| 3 | Each target has verbatim panic output captured | YES |
| 4 | Each target has explicit category (A/B/C/D/E) | YES |
| 5 | 211d worklist concrete + actionable | YES |
| 6 | Recommendation provided with four-questions reasoning | YES |
| 7 | No code changes / test edits made | YES |
| 8 | No regressions introduced | YES (investigation only) |

## Honest-delta watch (predicted surprises)

1. **Probe hangs different than predicted** — some "deadlock" probes time out cleanly; others hang past 90s. Sonnet's `timeout 90` should handle gracefully; if not, surface as STOP.

2. **Orphan process accumulation** — running these tests likely spawns orphans (per INTERSTITIAL § 2026-05-17 orphan investigation). Sonnet may need `pkill -9 -f "target/release/deps"` between test runs.

3. **More than 11 failing targets surface** — workspace flakes may rotate; pre-211c run may show 12-13 targets failing. Sonnet documents the rotation; doesn't treat it as a 211c-introduced regression.

4. **Category D being smaller than predicted** — maybe none of the probes were asserting on text format; maybe they were asserting on EDN structure that doesn't quite match what 211b produces. Sonnet's evidence overrides my hypothesis.

5. **Category A surfaces unexpected pattern** — dup-removal might have broken something orchestrator hasn't anticipated. The audit IS the way to find out.

6. **A panic_any! site emits something OTHER than AssertionPayload** — would mean some panic sites don't get the structured rendering. Sonnet notes this for follow-up; not blocking 211c.

## Mode classification

- **Mode A:** audit complete; all 11 targets investigated; SCORE comprehensive.
- **Mode B:** audit complete with caveats (e.g., one target genuinely hung past timeout; one target's output is hard to parse).
- **Mode B-time-violation:** ran >60 min. Investigate sonnet's path; the work shouldn't justify >60 min unless some test takes substantially longer than predicted.
- **Mode C:** STOP trigger hit during investigation.

## Calibration metadata

- Orchestrator confidence: MEDIUM. The work is well-bounded but discovery-shaped; surprises are expected.
- Risk factors: orphan accumulation, hang-prone tests, panic_any! sites that don't use AssertionPayload.
- Why 211c before 211d: 211d's decision (revert dup vs surgical fix vs assertion updates only) MUST be informed by honest evidence; without 211c, 211d would be speculation.

## Post-completion orchestrator actions

1. Read SCORE end-to-end
2. Read each target's verbatim panic output carefully
3. Re-classify any category that sonnet got wrong (if obvious from the evidence)
4. Identify the 211d action shape:
   - If mostly Category D: 211d is a small assertion-update sweep
   - If Category A present: 211d decides revert vs surgical
   - If Category C present: 211d may need to split (other arc for foundation; current arc for dup)
5. Commit SCORE atomically
6. Push
7. Mark task #363 complete; mark task #364 (211d) in_progress
8. Draft BRIEF for 211d based on findings

## Cross-references

- BRIEF-211C-AUDIT.md — work definition
- SCORE-211A-CTOR-INSTALL.md + SCORE-211B-PANIC-AS-EDN.md — preceding slices' calibration
- Arc 211 DESIGN — locked scope; 211c is the diagnostic step before 211d acts
- INTERSTITIAL § 2026-05-18 (later) — panic-as-EDN doctrine; 211c reads against
- Songs (per soundtrack): #1 The Other Side — pain as guide; #8 Hell Is Empty — revealing what's actually broken vs what we thought; #10 Bleed Me Dry — the audit makes the next cut precise
