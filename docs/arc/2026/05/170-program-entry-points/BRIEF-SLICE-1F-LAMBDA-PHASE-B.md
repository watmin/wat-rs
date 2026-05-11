# Arc 170 slice 1f-λ Phase B — BRIEF (consumer-vantage sweep)

**Sonnet (two spawns: B1 then B2).** Phase A landed the canonical
spawn-process pattern at `tests/wat_arc170_program_contracts.rs`
(T4-T6, T12, T13). Phase B replicates that pattern across the
remaining 24 failures in 5 test files. Two distinct migration
patterns; one sonnet spawn per pattern.

## Phase A reference (load-bearing — read first)

- `tests/wat_arc170_program_contracts.rs` — canonical home
  - T4: `spawn-process(named-defn-fn)` round-trip via typed channels
  - T5: `spawn-process(inline-lambda fn)` round-trip
  - T6: `spawn-process(factory-fn with single-level capture)`
  - T12: `spawn-process(fn)` — child emits without recv'ing first
  - T13: `spawn-process(fn)` — child exits clean on parent tx-drop
- File-local helpers (lines 156-220): `drive_typed_recv`,
  `unwrap_sender_inner`, `unwrap_receiver_inner`, `process_tx_field`,
  `process_rx_field`, `process_handle_field`, `wait_child_exit_ok`
- [`SCORE-SLICE-1F-LAMBDA-PHASE-A.md`](./SCORE-SLICE-1F-LAMBDA-PHASE-A.md) — the disposition-table approach (REPLACE / DELETE per scenario)

## Two patterns — different shapes, same slice

### Pattern B1 — kernel-API tests (16 tests, 2 files)

These test the substrate's spawn-process / spawn-thread surface
directly through Rust (typed_channel API). Same shape as Phase A.

| File | Tests | Disposition approach |
|---|---|---|
| `tests/wat_arc103_spawn_program.rs` | 6 | Per-scenario inventory; REPLACE survivors as T14+ in canonical home OR consolidate into existing T4-T13 if scenario already covered; DELETE obsolete |
| `tests/wat_fork.rs` | 10 | Per-scenario inventory; REPLACE survivors as T-numbered in canonical home; DELETE obsolete (e.g., any test exercising substrate parse-error-on-spawn) |

**Pattern:** delete each file entirely once its scenarios are dispositioned
(consolidate to canonical home OR delete-as-obsolete). Net: 2 files
deleted, N new tests in canonical home (likely N < 16 due to
consolidation per Phase A precedent).

**Migration target (per scenario in B1):**
- `:wat::kernel::fork-program-ast` / `fork-program` / `spawn-program-ast`
  / `spawn-program` callsites → `:wat::kernel::spawn-process worker-fn`
  where worker-fn satisfies `[rx <- Receiver<I> tx <- Sender<O>] -> :nil`
- `:wat::kernel::Process/stdin` / `Process/stdout` pipe access →
  parent-side `process_tx_field` / `process_rx_field` + `drive_typed_recv`
- `wait_child` exit-code tests → `wait_child_exit_ok(handle)` helper
- `:user::main` 4-arg shape → canonical `[] -> :nil` (in tests' inner
  wat source) OR drop the inner-source entirely (worker-fn becomes a
  top-level `defn` in the test's outer wat program)

### Pattern B2 — wat-cli + examples (12 tests, 3 files)

These spawn the `wat` CLI as a subprocess and assert on stdout/stderr.
The Rust test scaffolding stays intact; only the embedded wat const
programs need migration.

| File | Tests | Disposition approach |
|---|---|---|
| `crates/wat-cli/tests/wat_cli.rs` | 10 | Update each `const FOO_PROGRAM: &str = r#"..."#` to the canonical wat shape; leave Rust scaffolding unchanged unless an assertion needs to update |
| `examples/with-loader/tests/smoke.rs` | 1 | Same — update embedded wat source |
| `examples/with-lru/tests/smoke.rs` | 1 | Same |

**Migration target (per const in B2):**
- `:user::main` 4-arg shape → canonical `[] -> :nil`
- `:wat::io::IOReader/read-line stdin` → `(:wat::kernel::readln -> :T)`
- `:wat::io::IOWriter/print stdout x` / `IOWriter/println stdout x` →
  `(:wat::kernel::println x)`
- `:wat::io::IOWriter/println stderr x` → `(:wat::kernel::eprintln x)`
- argv parameter → `(:wat::runtime::argv)` ambient (where used)

The Rust assertions on stdout/stderr should still hold IF the embedded
program emits the same observable output. If an assertion needs to
update (e.g., EDN-encoded vs raw text per slice 1f-ι contract), surface
that as an honest delta.

## Required reading

1. `tests/wat_arc170_program_contracts.rs` — Phase A canonical reference
   (T4-T6, T12, T13 + helpers)
2. [`SCORE-SLICE-1F-LAMBDA-PHASE-A.md`](./SCORE-SLICE-1F-LAMBDA-PHASE-A.md) — disposition-table approach
3. `src/check.rs:732+` — substrate BareLegacy* diagnostic bodies (migration
   targets documented in the diagnostic text)
4. `.claude/skills/vocare/SKILL.md` — caller-vantage discipline
5. `.claude/skills/complectens/SKILL.md` — layered-helper discipline
6. [`SCORE-SLICE-1F-IOTA.md`](./SCORE-SLICE-1F-IOTA.md) — println/readln EDN contract (Pattern B2's stdio replacement target)

## Out of scope

- 4 `slice4_*` heterogeneous-dispatch failures (independent of arc 170)
- Substrate Rust edits — substrate is settled; tests adapt to it
- New scenarios beyond what existing 28 tests preserve
- Renames of test files (consolidation to canonical home is OK; renames
  for stylistic reasons are not)

## Predicted runtime

**B1 (spawn 1):** 50-90 min sonnet. 16 tests, kernel-API pattern
matching Phase A. Per-scenario inventory + REPLACE-or-DELETE
disposition. Some consolidation likely (per Phase A precedent).

**B1 hard cap:** 180 min.

**B2 (spawn 2):** 30-60 min sonnet. 12 tests, mechanical const-string
replacement. Faster per-test than B1; total bounded by 12 const updates
+ verification.

**B2 hard cap:** 120 min.

**Total Phase B:** 80-150 min sonnet across two spawns.

## Path forward post-Phase B

1. B1 ships → SCORE → atomic-commit → push → B2 BRIEF authored
2. B2 ships → SCORE → atomic-commit → push
3. Workspace at zero failures (or 4 remaining for the `slice4_*` arc-146 work)
4. Triage `slice4_*` (separate arc 146 territory)
5. Arc 170 INSCRIPTION (DESIGN names arc 170 as blocker for arc 109 v1
   milestone closure)

## Cross-references

- BRIEF (Phase A + slice umbrella): [`BRIEF-SLICE-1F-LAMBDA.md`](./BRIEF-SLICE-1F-LAMBDA.md)
- Phase A SCORE: [`SCORE-SLICE-1F-LAMBDA-PHASE-A.md`](./SCORE-SLICE-1F-LAMBDA-PHASE-A.md)
- Slice 1f-ι contract: [`SCORE-SLICE-1F-IOTA.md`](./SCORE-SLICE-1F-IOTA.md)
- 1f-θ V3 precedent: [`SCORE-SLICE-1F-THETA-V3.md`](./SCORE-SLICE-1F-THETA-V3.md)
- DESIGN: [`DESIGN.md`](./DESIGN.md)
