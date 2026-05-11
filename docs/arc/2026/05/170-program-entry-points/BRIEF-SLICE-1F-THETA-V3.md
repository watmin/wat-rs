# Arc 170 slice 1f-θ V3 — BRIEF (remove the poison; consumer-vantage tests)

**Opus.** V1 and V2 of this BRIEF both failed for the same reason: they tried to fix existing test files that violate BOTH `/complectens` AND `/vocare`. Sonnet kept fixating on preserving the existing implementer-vantage wire-protocol gymnastics. **V3 removes the poison entirely: delete the existing tests, write fresh consumer-vantage tests from scratch.**

**Supersedes:**
- `BRIEF-SLICE-1F-THETA.md` (V1 STALE) — flat-let bind-order tweak; killed
- `BRIEF-SLICE-1F-THETA-V2.md` (V2 SUPERSEDED) — complectens-shaped restructure of existing tests; killed because sonnet kept anchoring on the broken implementer-vantage shape

## What's poisoned

Three files contain implementer-vantage wire-protocol tests:
- `wat-tests/kernel/services/stdin.wat`
- `wat-tests/kernel/services/stdout.wat`
- `wat-tests/kernel/services/stderr.wat`

Each manually spawns the service, sends `Event::Add` with channel pairs, manages routing tables, sends `Event::Remove`, etc. **A consumer of `:wat::kernel::println` would NEVER write any of this.** Per vocare, these tests stand at the implementer vantage when the recommended vantage for the service is the AMBIENT consumer surface.

These files also have monolithic let bodies with 10+ anonymous sequential bindings — Level 1 complectens lies.

**Both spells fail simultaneously.** The fix is not to repair the existing files; it is to delete them and write tests at the right vantage following the right discipline.

## Mission

> *"Delete the implementer-vantage trio tests. Write fresh consumer-vantage tests for the ambient stdio surface."*

The consumer surface for the trio services is:
- `(:wat::kernel::println v)` — write `v` (EDN-encoded) to stdout
- `(:wat::kernel::eprintln v)` — write `v` (EDN-encoded) to stderr
- `(:wat::kernel::readln)` — read next EDN form from stdin

Consumer tests verify these primitives WORK from the consumer's vantage. The trio services are mechanism the consumer never touches.

## Read first (load-bearing)

1. **`.claude/skills/vocare/SKILL.md`** — caller-vantage discipline
2. **`.claude/skills/complectens/SKILL.md`** — layered-helper discipline
3. **`crates/wat-lru/wat-tests/lru/CacheService.wat`** — canonical complectens example
4. **`tests/wat_arc170_slice_1f_gamma_orchestrator.rs`** — Rust integration tests for the orchestrator; shows the OS-pipe capture pattern for verifying ambient stdio
5. **`src/thread_io.rs:182, :211, :240`** — `eval_kernel_println` / `eprintln` / `readln` arms; the actual consumer surface

## The vantage decision (the design call)

Per vocare § "How to read a test through vocare":

**Who calls `:wat::kernel::println` in production?**
- A wat program's `:user::main` (canonical no-arg shape per slice 1e)
- Body uses `(:wat::kernel::println "hello")` to emit output
- The orchestrator + trio services route the output to fd 1

**The test's call shape should match this caller.** A test that calls `:wat::kernel::println` from a wat program's body and captures the resulting fd-1 output is consumer-vantage. A test that hand-builds Event::Add + Event::Write events is implementer-vantage.

## Scope

### Delete (3 files; -913 lines)

```bash
rm wat-tests/kernel/services/stdin.wat
rm wat-tests/kernel/services/stdout.wat
rm wat-tests/kernel/services/stderr.wat
```

### Author (1 new file; ~150-250 lines)

Path: `wat-tests/kernel/services/ambient-stdio.wat`

One file. Top-down dependency graph per complectens. Layered helpers + per-helper deftests + 3-7 line final test bodies.

**Proposed layer structure:**

| Layer | Name | What it tests |
|---|---|---|
| 0 | `:test::println-emits-line` | `(:wat::kernel::println "hello")` writes "hello\n" to captured stdout |
| 1 | `:test::println-emits-edn` | `(:wat::kernel::println 42)` writes "42\n" (EDN encoding for i64) |
| 2 | `:test::eprintln-emits-line` | `(:wat::kernel::eprintln "error")` writes to captured stderr (not stdout) |
| 3 | `:test::println-multi-line` | Two `println` calls produce two lines in order |
| 4 | `:test::readln-parses-edn` | Send EDN to captured stdin; `(:wat::kernel::readln)` returns parsed HolonAST |

Each layer has its own `(:deftest-ambient :test::test-N-name (:test::layer-N-helper))` deftest.

### Edge case — running these tests: MUST be deftest-hermetic

**Use `deftest-hermetic`. NOT `deftest`. NOT `deftest-source`.**

The slice's mission is to **assert that hermetic continues to work after the arc 170 migration**. Using non-hermetic `deftest` would test only the in-process orchestrator path, which entirely SKIPS the forked-child path. That defeats the purpose of the slice — we'd ship "working tests" that don't exercise what we're verifying.

The forked-child orchestrator path under test:
1. `deftest-hermetic` → `run-sandboxed-hermetic-ast` → `fork-program-ast` forks a child
2. Child boots `invoke_user_main_orchestrated` (slice 1f-γ)
3. Child's orchestrator spawns the trio services + registers thread-0
4. Child's `:user::main` calls `(:wat::kernel::println v)` etc
5. Routes through trio → child's fd 1
6. Parent drains via OS pipe → returns `RunResult { stdout: Vector<String>, stderr: Vector<String> }`
7. Assertions on the vectors

**Every test in `ambient-stdio.wat` MUST use `deftest-hermetic`.** Earlier in this slice's iteration, an agent suggested switching to non-hermetic because "in-memory IOReader doesn't need real stdio isolation." That suggestion is REJECTED — the in-memory path doesn't exercise fork/orchestrator-boot/service-spawn/dup-fds/drain-pipe; those ARE what the slice exists to verify.

The `:wat::test::run-hermetic-ast` Layer 1 wrapper from `wat/test.wat` returns the `RunResult`. Assertions consume `Vector<String>` fields.

### Out of scope

- No substrate Rust edits
- No changes to the orchestrator or trio service definitions
- No restoration of the deleted tests (they are the poison)
- No tests at the implementer vantage (those would require `rune:vocare(substrate-primitive-reference)` and go in a different file — separate slice if ever needed)
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | 3 old trio test files deleted | `ls wat-tests/kernel/services/` shows none |
| B | New `ambient-stdio.wat` exists with `make-deftest` factory + layered helpers | grep |
| C | Each layer (0-4 or as redesigned) has its own deftest | grep |
| D | Final deftest bodies ≤ 7 lines | manual review |
| E | No deftest body exceeds ~10 anonymous sequential bindings (complectens § Level 1 lie) | manual review |
| F | Tests use consumer surface only (`:wat::kernel::println` / `eprintln` / `readln`) — no `Event::Add`, no direct `StdInService::spawn`, no routing-table manipulation | grep |
| G | All new tests pass | cargo test |
| H | Workspace failure count drops by ≥ 12 from 2151/48 baseline | cargo test count |
| I | `cargo check --release` green | clean |
| J | Top-down dependency graph: no helper references a helper defined LATER | manual review |
| K | Honest deltas surfaced | per FM 5 |

**11 rows.**

## Honest delta categories (anticipated)

1. **`deftest-hermetic` vs `deftest`** — recommend hermetic (forks subprocess; clean orchestrator boot per test); if deftest works in-process per slice 1f-ζ's `src/spawn.rs` ambient install, that's also acceptable. Pick the simpler path; surface choice.

2. **`readln` testing requires stdin pre-seeding** — `run-hermetic-ast` takes `stdin : Vector<String>` as second arg. Tests that exercise `readln` pass an EDN-encoded stdin; tests that don't exercise readln pass empty vec.

3. **EDN encoding details** — `(:wat::kernel::println "hello")` outputs `"hello"\n` (with quotes; EDN string syntax). `(:wat::kernel::println 42)` outputs `42\n` (raw integer; no quotes). Assertions need to match the EDN-encoded form, not the raw value.

4. **The original tests' INTENT may have been wire-protocol pedagogy** — if so, the right place is a separate file with `rune:vocare(substrate-primitive-reference)`. NOT this slice; if such pedagogy is needed, a future slice can mint it alongside `wat-rs/wat-tests/service-template.wat`. Do not author it here.

5. **`Vector<String>` assertion ergonomics** — assertions on stdout/stderr lines need to compare `Vector<String>` values; existing `assert-eq` may need a `Vector::equal?` form or similar. If awkward, surface.

## Predicted runtime

**180-300 min opus.** Read both spells (load-bearing), read canonical example, read orchestrator integration test pattern, design fresh layer structure, author one file, verify. The design call (vantage + layer structure) is the substantive work; the wat code is mechanical once the design is settled.

**Hard cap:** 600 min.

## Reference

- `.claude/skills/vocare/SKILL.md` — caller-vantage discipline
- `.claude/skills/complectens/SKILL.md` — layered-helper discipline
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — canonical complectens example
- `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` — orchestrator integration tests
- `src/thread_io.rs:182, :211, :240` — substrate eval arms for the consumer surface
- V1 + V2 BRIEFs (STALE/SUPERSEDED) — historical record of why this V3 framing emerged

## Path forward post-slice-1f-θ V3

1. Orchestrator scores; atomic-commits deliverable + SCORE; pushes
2. Verify leak resolved (root-cause fix); workspace clean; no orphans
3. Remaining failures (~36) — sibling slice for retired verbs + heterogeneous triage
4. Arc 170 INSCRIPTION
