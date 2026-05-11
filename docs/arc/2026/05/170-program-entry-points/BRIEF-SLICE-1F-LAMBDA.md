# Arc 170 slice 1f-λ — BRIEF (retired fork/spawn-program test rebuild)

**Opus (Phase A) + Sonnet (Phase B).** This slice closes the 28
`BareLegacyMainSignature` / `BareLegacyForkProgram` /
`BareLegacySpawnProgram` failures from the workspace by **deleting +
replacing** tests that embed the retired surface, NOT by migrating
them in place. Same shape as slice 1f-θ V3 — the existing tests are
implementer-vantage poison on a retired API; the honest move is fresh
consumer-vantage tests on the canonical `spawn-process(fn)` /
`spawn-thread(fn)` surface.

## What's retired (substrate-verified)

The substrate's own check-error message dictates the migration target
(`src/check.rs:736`):

> `:wat::kernel::fork-program-ast` is retired (arc 170 slice 2);
> canonical replacement is `:wat::kernel::spawn-process` (fn-input
> surface). The fn IS the program — substrate handles closure
> extraction + fork internally; user passes a fn directly that
> satisfies `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil`.

Similar for `spawn-program{-ast}` retiring under `spawn-thread`
(parent's world) / `spawn-process` (forked OS process). And `:user::main`'s
4-arg shape `(stdin stdout stderr argv)` retires under canonical
`[] -> :wat::core::nil` with argv ambient via `(:wat::runtime::argv)`
and stdio via the three kernel services (println/eprintln/readln per
slice 1f-ι).

## Why rebuild, not migrate (the design call)

Per slice 1f-θ V3's INSCRIBED precedent: when test files encode a
retired surface DEEPLY (string-embedded inner-src, retired Process/stdin
pipe access, retired IOReader/IOWriter calls), migration ≠ a pattern
sweep. Some tests have no meaningful migration:

- `fork_program_parse_error_surfaces_as_exit_3` — the new
  `spawn-process(fn)` surface has NO parse-error-on-spawn path because
  the fn is wat-compiled in parent's world. The scenario is obsolete.

The honest move per `/vocare` (consumer-vantage) + `/complectens`
(layered helpers) is: delete the poison; write fresh tests at the
canonical surface for scenarios that still have meaning.

## Scope

### Failing tests in scope (28 tests across 7 files in 5 packages)

| Package | Test file | Tests | Bucket |
|---|---|---|---|
| `wat` | `tests/wat_arc104_fork_program.rs` | 4 | **PHASE A** (stepping stone) |
| `wat` | `tests/wat_arc103_spawn_program.rs` | 6 | PHASE B |
| `wat` | `tests/wat_fork.rs` | 10 | PHASE B |
| `wat-cli` | `crates/wat-cli/tests/wat_cli.rs` | 10 | PHASE B (wat-cli subset) |
| `with-loader-example` | `examples/with-loader/tests/smoke.rs` | 1 | PHASE B |
| `with-lru-example` | `examples/with-lru/tests/smoke.rs` | 1 | PHASE B |

**Note:** the 4 `slice4_*` failures in `tests/wat_polymorphic_arithmetic.rs`
are INDEPENDENT of arc 170 (heterogeneous-dispatch; arc 146 territory)
and OUT of slice 1f-λ scope.

### What each test exercises (scenario inventory)

Phase A (`wat_arc104_fork_program.rs` — 4 tests):
1. `fork_program_round_trip_via_pipes` — parent sends Ping, child reads, doubles, writes back, parent reads response
2. `fork_program_child_writes_stdout_parent_reads_line` — child writes to stdout, parent captures
3. `fork_program_clean_exit_code_via_wait_child` — child reads stdin to EOF, exits cleanly
4. `fork_program_parse_error_surfaces_as_exit_3` — **likely obsolete** (parse happens in parent's world; no spawn-time parse-error path)

Phase B scenarios surface during Phase B authorship; sonnet inventories
when reading each file's failure list.

## Phase A — opus stepping stone (this session)

**Goal:** rebuild `tests/wat_arc104_fork_program.rs` from scratch as
consumer-vantage tests on `:wat::kernel::spawn-process(fn)`. The
rebuilt file becomes the canonical pattern reference for Phase B.

### Required reading (load-bearing)

1. **`tests/wat_arc170_program_contracts.rs`** — the post-arc-170
   contract tests; T4-T8 demonstrate spawn-process(fn) patterns
   (inline-lambda, factory-fn, etc.)
2. **`tests/wat_spawn_fn.rs`** — spawn-thread fn-shape variants;
   shows the `(in :Receiver<I> out :Sender<O>) -> :nil` worker shape
3. **`.claude/skills/vocare/SKILL.md`** — caller-vantage discipline
4. **`.claude/skills/complectens/SKILL.md`** — layered-helper discipline
5. **`src/check.rs:732+`** — substrate's full BareLegacy* diagnostic
   bodies (they document the migration shape)

### Approach

For each of the 4 scenarios in `wat_arc104_fork_program.rs`:

1. Read the existing test (recognize the SCENARIO it preserves)
2. Decide: does the scenario survive under spawn-process(fn)?
   - YES → write a fresh consumer-vantage test on the new surface
   - NO → delete with rationale in the SCORE's honest-deltas section
3. Compose using complectens: named helpers (if needed) for non-trivial
   setup; test bodies stay ≤ 7 lines for assertion-clarity

### Phase A deliverable

- `tests/wat_arc104_fork_program.rs` rewritten (or possibly renamed —
  surface to user if a new name better reflects the post-rebuild
  scope; e.g., `wat_arc170_spawn_process_examples.rs`)
- All Phase A tests pass; failure count drops by 4 (from 36 to 32)
- `cargo check --release` green
- 4 honest-delta categories anticipated:
  1. Number of scenarios that survived (e.g., 3/4 if parse-error-test deletes)
  2. Worker-fn shape used (inline-lambda vs factory-fn — pick whichever's
     simpler per test; surface choice)
  3. EDN-over-pipes I/O encoding — `(send tx ...)` value types worth surfacing
  4. Anything else surfaced during authorship

## Phase B — sonnet pattern replication

**Goal:** replicate Phase A's pattern across the remaining 5 files / ~24
tests.

### Approach

Sonnet reads Phase A's rebuilt file as the canonical example. For each
of the remaining files, sonnet:

1. Inventories scenarios (existing test count + what each tests)
2. For each scenario: writes fresh consumer-vantage test OR records
   "obsolete; delete" in honest deltas
3. Verifies `cargo test --release` for each touched file
4. Reports honest deltas (scenarios surviving vs not, worker-fn shape
   patterns, EDN encoding details)

### Phase B BRIEF + EXPECTATIONS

Authored AFTER Phase A ships (so the canonical pattern is concrete
and grep-able in the brief). Predicted Phase B runtime: 60-120 min
sonnet (~24 tests, pattern uniform once Phase A settles).

## Out of scope (Phase A and Phase B)

- No substrate Rust edits — the substrate's spawn-process / spawn-thread
  are settled; tests adapt to them, not the reverse
- No new test scenarios beyond what the existing 28 tests preserve
  (or replace one-for-one with the canonical replacement; don't expand
  scope mid-rebuild)
- No fixing `slice4_*` heterogeneous failures (separate arc 146 work)
- No modifications to `:user::main` substrate validator behavior

## Ship criteria (Phase A only; Phase B has its own scorecard)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `tests/wat_arc104_fork_program.rs` rewritten (or replaced); zero `BareLegacy*` errors from this file | grep |
| B | Each surviving scenario has its own #[test] fn; obsolete scenarios deleted with rationale in SCORE | manual review |
| C | All Phase A tests pass | `cargo test --release --test wat_arc104_fork_program` (or new name) |
| D | Workspace failure count drops by Phase-A's scenario-survivor count (≤ 4) | cargo test count |
| E | `cargo check --release` green | clean |
| F | Phase A's canonical pattern is grep-able in Phase B's BRIEF — i.e., named helpers, layered structure, test-body composition all visible | manual review |
| G | Honest deltas surfaced | per FM 5 |

**7 rows.**

## Predicted runtime

**Phase A: 30-60 min opus.** 4 scenarios; design judgment per scenario;
authorship is constrained by Phase A's required reading + patterns.

**Phase A hard cap:** 120 min.

**Phase B: 60-120 min sonnet** (predicted after Phase A ships).

## Path forward post-1f-λ

1. Phase A ships → SCORE → atomic-commit → push
2. Phase B BRIEF + EXPECTATIONS authored with Phase A pattern as reference
3. Phase B sonnet sweep
4. Slice 1f-μ if scope remains (likely subsumed by Phase B since wat-cli
   tests + examples share the same migration shape)
5. Triage 4 `slice4_*` heterogeneous failures (arc 146 follow-up)
6. Arc 170 INSCRIPTION — DESIGN names arc 170 as blocker for arc 109 v1
   milestone closure

## Cross-references

- DESIGN: [`DESIGN.md`](./DESIGN.md) § "The API — spawn-* fn"
- Prior slice SCOREs: [`SCORE-SLICE-1F-IOTA.md`](./SCORE-SLICE-1F-IOTA.md),
  [`SCORE-SLICE-1F-THETA-V3.md`](./SCORE-SLICE-1F-THETA-V3.md)
- Substrate diagnostic source: `src/check.rs:732+` (BareLegacy*
  diagnostic bodies)
- Canonical worked examples: `tests/wat_arc170_program_contracts.rs`,
  `tests/wat_spawn_fn.rs`
- Spell library: `.claude/skills/vocare/SKILL.md`,
  `.claude/skills/complectens/SKILL.md`
- User direction 2026-05-10: *"go make println and readln work — it'll
  break a bunch of existing tests which is correct — we must fix them
  after we make the contract work"* (slice 1f-ι contract; this slice
  closes the consumer breakage)
- 1f-θ V3 precedent: poison → fresh consumer-vantage tests (the
  inscribed pattern)
