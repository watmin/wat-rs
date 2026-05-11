# Arc 170 slice 1f-λ Phase B2 — EXPECTATIONS (sonnet scorecard)

**Spawn 2 of Phase B.** Pattern B2 from the BRIEF: wat-cli subprocess
tests + example smoke tests (12 tests total). Embedded `const FOO_PROGRAM: &str = r#"..."#` strings need migration to canonical wat shape; Rust test scaffolding stays.

## Independent prediction

**Runtime band:** 25-60 min sonnet. 12 tests, mechanical const-string
replacement, no Rust-scaffolding rewrites. Faster per-test than B1
since each test is one self-contained const.

**Hard cap:** 120 min (2× upper). If sonnet hits cap with work pending,
kill via TaskStop + score Mode B-time-violation.

## Scope (3 files)

| File | Failing tests | Pattern |
|---|---|---|
| `crates/wat-cli/tests/wat_cli.rs` | 10 | Update each failing test's `const FOO_PROGRAM: &str = r#"..."#`; leave Rust test scaffolding (Command::new, stdin pipe, stdout assertions) intact unless an assertion needs to update for the EDN-only stdio contract |
| `examples/with-loader/tests/smoke.rs` | 1 | Update embedded wat const |
| `examples/with-lru/tests/smoke.rs` | 1 | Update embedded wat const |

## Migration target per const (mandatory; surface diff per file in SCORE)

The substrate's check-error message (`src/check.rs:732+`) documents
this. The canonical migration shape:

| Retired form | Canonical replacement |
|---|---|
| `(:user::main (stdin :wat::io::IOReader) (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :wat::core::nil)` | `(:user::main -> :wat::core::nil)` (no params; argv ambient) |
| 4-arg variant with `(argv :wat::core::Vector<wat::core::String>)` | Same `[] -> :nil`; access argv via `(:wat::runtime::argv)` inside body |
| `(:wat::io::IOReader/read-line stdin)` | `(:wat::kernel::readln -> :T)` where T is the expected type (`:wat::core::String` for raw line text per slice 1f-ι contract) |
| `(:wat::io::IOWriter/print stdout x)` / `(:wat::io::IOWriter/println stdout x)` | `(:wat::kernel::println x)` |
| `(:wat::io::IOWriter/print stderr x)` / `(:wat::io::IOWriter/println stderr x)` | `(:wat::kernel::eprintln x)` |
| `(:wat::kernel::fork-program ...)` / `(:wat::kernel::fork-program-ast ...)` | `(:wat::kernel::spawn-process worker-fn)` (per Phase B1 pattern) — unlikely in B2 const programs but flag if encountered |

## Required reading (load-bearing)

1. `docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-1F-LAMBDA-PHASE-B.md` — Pattern B2 section
2. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1F-LAMBDA-PHASE-B1.md` — disposition-table approach + canonical-home reference
3. `tests/wat_arc170_program_contracts.rs` — canonical post-arc-170 wat shapes (`:user::main -> :nil`, `(:wat::runtime::argv)` ambient, etc.)
4. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1F-IOTA.md` — println/readln EDN contract (the stdio replacement target)
5. `crates/wat-cli/tests/wat_cli.rs` — read in full to inventory all const programs (failing AND passing — if a passing test's const is already migrated, that's the worked example)

## Disposition approach (mirror B1, adapted for const strings)

For EACH failing test in scope:

1. Read the test's const program string
2. Identify ALL retired forms (legacy main signature, IOReader/IOWriter calls, fork-program/spawn-program callsites)
3. Apply the migration table above to the const string in-place
4. If the test's Rust assertion checks RAW stdout text:
   - If migration changes the on-wire EDN encoding (e.g., printed `42` becomes `42\n` EDN-encoded), update the assertion
   - If migration preserves observable output, leave assertion intact
   - Surface assertion changes in SCORE as honest delta
5. If a test scenario is fundamentally obsolete on the new surface (e.g., test exercises a path that doesn't exist post-arc-170), `git rm` the test fn or the whole file; record in SCORE

## What sonnet should produce

1. **Code changes:**
   - Failing tests' const programs migrated to canonical shape
   - Rust test scaffolding adjusted ONLY where assertions need updating
   - Any wholly-obsolete tests deleted (with rationale)
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1F-LAMBDA-PHASE-B2.md`
   - Per-test migration summary (test name, what changed in the const, did assertion update)
   - Workspace failure delta
   - Honest deltas (≥ 3 categories)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- No substrate Rust edits (substrate is settled)
- No expansion of scenarios (12 originals only)
- No reshape of canonical home `tests/wat_arc170_program_contracts.rs` (already at 20 tests; T1-T16 + sub-variants — do NOT touch)
- No mixing of B1's typed-channel pattern into B2's subprocess pattern
- No commit (orchestrator handles)
- No deferral language in SCORE — INSCRIPTION-grade discipline per FM 11

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | All 10 wat-cli failing tests migrated | grep / cargo test |
| B | `examples/with-loader/tests/smoke.rs` migrated | cargo test |
| C | `examples/with-lru/tests/smoke.rs` migrated | cargo test |
| D | All B2 tests in scope pass | `cargo test --release -p wat-cli --test wat_cli` + smoke tests show 0 failed for B2 scope |
| E | Workspace BareLegacy* failure count drops by 12 (or by N for any obsolete-deletes; rationalize delta) | workspace test count |
| F | `cargo check --release` green | clean |
| G | Per-test migration table in SCORE | manual review |
| H | Honest deltas surfaced (≥ 3 categories) | per FM 5 |

**8 rows.**

## Tools required

- Read / Edit / Bash (cargo, git)
- No Agent invocations (single-agent sweep)

## Verification commands (sonnet runs these)

```bash
# Baseline at start
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Per-package verification mid-sweep
cargo test --release -p wat-cli --test wat_cli 2>&1 | tail -5
cargo test --release -p with-loader-example --test smoke 2>&1 | tail -5
cargo test --release -p with-lru-example --test smoke 2>&1 | tail -5

# Final workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post B1): 2168 passed / 16 failed
- Post B2 prediction: ≥ 2168 passed / ≤ 4 failed (12 B2 originals close;
  4 `slice4_*` heterogeneous-dispatch failures remain, independent of arc 170)

## Honest delta categories (anticipated)

1. **Assertion updates for EDN-only stdio.** `:wat::kernel::println x` for non-string `x` emits EDN-encoded form (e.g., i64 `42` → `42\n`, String `"foo"` → `"foo"\n` with quotes). Tests asserting on RAW stdout may need quote-handling updates.

2. **argv ambient migration.** Tests that exercised `argv` as a `:user::main` parameter migrate to `(:wat::runtime::argv)` inside the body. Surface which tests this applies to.

3. **Obsolete test deletes.** Per B1 precedent, some scenarios may be obsolete on the new surface (e.g., tests for the 4-arg signature itself, or for stdio paths that don't exist EDN-encoded). Delete with rationale.

4. **Anything unexpected** — surfaced during reading.
