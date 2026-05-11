# Arc 170 slice 1f-λ Phase B1 — EXPECTATIONS (sonnet scorecard)

**Spawn 1 of Phase B.** Pattern B1 from the BRIEF: kernel-API tests in
`tests/wat_arc103_spawn_program.rs` (6 tests) + `tests/wat_fork.rs`
(10 tests). 16 tests total; pattern matches Phase A.

## Independent prediction

**Runtime band:** 50-90 min sonnet. 16 tests; per-scenario inventory +
REPLACE-or-DELETE disposition; consolidation likely (Phase A reduced
4 tests to 2 + 4 deletions by reusing T4-T6 coverage).

**Hard cap:** 180 min (2× upper bound). If sonnet hits cap with work
still pending, kill via TaskStop and score Mode B-time-violation.

## Scorecard (rows sonnet should self-score then orchestrator verifies)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `tests/wat_arc103_spawn_program.rs` dispositioned (file deleted; survivors consolidated to canonical home) | `ls tests/wat_arc103_spawn_program.rs` returns "no such file" |
| B | `tests/wat_fork.rs` dispositioned (file deleted; survivors consolidated to canonical home) | `ls tests/wat_fork.rs` returns "no such file" |
| C | Canonical home `tests/wat_arc170_program_contracts.rs` extended with T-numbered tests for surviving scenarios | `grep -c "^fn t" tests/wat_arc170_program_contracts.rs` shows count > 17 |
| D | Disposition table in SCORE: every original test has REPLACE / DELETE / CONSOLIDATE disposition with rationale | manual review |
| E | All B1 tests in canonical home pass | `cargo test --release --test wat_arc170_program_contracts` shows 0 failed |
| F | Workspace BareLegacy* failure count drops by ≥ 14 (16 originals minus expected consolidation) | `cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result"` total failures |
| G | `cargo check --release` green | clean compile |
| H | Honest deltas surfaced (≥ 3 categories) | per FM 5 (no-deferral discipline) |

**8 rows.**

## Disposition approach (mandatory; mirror Phase A)

For EACH test in the 2 input files:

1. **Read the existing test** (recognize the SCENARIO it preserves).
2. **Cross-reference canonical home T-numbered tests (T1-T13)** — does
   this scenario duplicate something already covered?
3. **Decide disposition:**
   - **CONSOLIDATE** — scenario already covered by T1-T13 → DELETE the
     original; record disposition in SCORE
   - **REPLACE** — scenario survives but isn't covered → write a fresh
     T-numbered test in canonical home matching Phase A's pattern
     (worker-fn defn or inline-lambda; spawn-process call AST; parent-side
     I/O via helpers)
   - **DELETE** — scenario obsolete on the new surface (e.g., parse-error-on-spawn
     doesn't exist post-arc-170) → record rationale in SCORE
4. **When the input file has zero surviving in-file tests,** DELETE the
   entire file (`git rm`).

## What sonnet should produce

1. **Code changes:**
   - 2 input files deleted via `git rm` (after all scenarios dispositioned)
   - `tests/wat_arc170_program_contracts.rs` extended with new T-numbered tests
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1F-LAMBDA-PHASE-B1.md`
   - Disposition table (one row per original test; columns: original test, disposition, rationale)
   - Workspace failure delta
   - Honest deltas (≥ 3 categories)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- No substrate Rust edits (substrate is settled)
- No expansion of scenarios (no "while we're here, let's add a test for X" — original 28 only)
- No reshape of existing canonical home tests (T1-T13) — those are settled
- No file rename ceremony (the canonical home is the canonical home;
  don't propose a rename mid-sweep)
- No mixing of B1 (kernel-API pattern) with B2 (wat-cli const-string pattern) — B2 ships in spawn 2
- No commit (orchestrator handles)
- No deferral language in SCORE — INSCRIPTION-grade discipline per FM 11
  (this is a SCORE, not an INSCRIPTION, but the no-deferral rule applies
  to Phase B as a Phase A successor)

## Tools required

- Read / Edit / Bash (cargo, git)
- No Agent invocations (single-agent sweep)

## Verification commands (sonnet runs these during work)

```bash
# Baseline (run once at start)
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Per-file scenario inventory (run for each input file)
grep "^fn " tests/wat_arc103_spawn_program.rs
grep "^fn " tests/wat_fork.rs

# After each batch of REPLACE additions, verify canonical home
cargo test --release --test wat_arc170_program_contracts 2>&1 | tail -10

# Final workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post Phase A): 2165 passed / 32 failed
- Post B1 prediction: ≥ 2167 passed / ≤ 18 failed (16 B1 originals close;
  some replaced as T-numbered passes; some consolidated as deletions)

## Honest delta categories (anticipated)

1. **Consolidation count** — how many of the 16 originals consolidated
   into existing T1-T13 vs needing fresh T-numbered tests
2. **Worker-fn shape variance** — keyword defn vs inline lambda vs
   factory-with-capture — which patterns each input file's scenarios
   exercise
3. **Scenarios that didn't survive** — count + reasons (per Phase A
   precedent: parse-error-on-spawn was obsolete)
4. **Anything unexpected** — surfaced during reading

## Sonnet kickoff prompt template

The orchestrator's Agent call passes this prompt (no preamble about
tool availability per FM 16):

```
Execute Phase B1 of arc 170 slice 1f-λ per:

- BRIEF: docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-1F-LAMBDA-PHASE-B.md (Pattern B1 section)
- EXPECTATIONS: docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-1F-LAMBDA-PHASE-B1.md
- Phase A canonical reference: tests/wat_arc170_program_contracts.rs (T4-T6, T12, T13 + helpers)
- Phase A SCORE for the disposition-table approach: docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1F-LAMBDA-PHASE-A.md

Goal: disposition all 16 tests in tests/wat_arc103_spawn_program.rs
+ tests/wat_fork.rs (CONSOLIDATE / REPLACE / DELETE per scenario);
delete the input files; extend canonical home with surviving
scenarios; ship a SCORE with disposition table.

Do NOT commit; orchestrator atomic-commits after verifying.
```
