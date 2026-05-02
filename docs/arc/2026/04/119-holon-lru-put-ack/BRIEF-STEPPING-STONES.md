# Arc 119 — Sonnet Brief: Stepping-stone debugging (deadlock isolation)

**Status:** durable record of the brief sent to sonnet for arc
119's deadlock isolation via stepping-stone tests. Same brief
stays as the reference for re-attempts and post-mortems.

## Provenance + goal

After arc 119 step 7's first agent attempt, four wat-tests in
`crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
hang somewhere in their bodies. The hang is structural — not
just slow compilation — and it's not caught by arc 117's
existing scope-deadlock check. We need to:

1. **Isolate which slice introduces the hang** by writing
   stepping-stone tests that build up step-by-step. The
   first slice that hangs is where the deadlock pattern
   lives.
2. **Document each slice's pass/fail** so the next arc (a
   compile-time deadlock check) can model its detection rule
   on the structural shape that fails.

## Reference: how the lab does this

`/home/watmin/work/holon/holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`
is the canonical stepping-stone pattern. Five files:
`004-step-A-rundb-alone.wat` through `004-step-E-reporter-fires-once.wat`.
Each file adds ONE piece of complexity. Each makes a binary
claim — "if this hangs, the bug is in THIS layer."

Read those files first. The arc 119 stepping stones follow the
same shape applied to HologramCacheService instead of the lab's
rundb+cache.

## Anchor docs (read in order)

1. `/home/watmin/work/holon/holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/004-step-A-rundb-alone.wat`
   through `004-step-E-reporter-fires-once.wat` — the canonical
   pattern.
2. `/home/watmin/work/holon/wat-rs/docs/SERVICE-PROGRAMS.md`
   §§ "The lockstep", "Step 1", "Audience boundary" — the
   service-shutdown discipline.
3. `/home/watmin/work/holon/wat-rs/docs/CONVENTIONS.md`
   § "Caller-perspective verification" — the layer the tests
   stand at (consumer surface, not raw protocol).
4. `/home/watmin/work/holon/wat-rs/.claude/skills/vocare/SKILL.md`
   — the ward that defends caller-perspective. Self-check each
   slice with vocare's lens before moving to the next.
5. `/home/watmin/work/holon/wat-rs/crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
   — the substrate file (post-arc-119 reshape; uncommitted in
   working tree). Read the helper verb signatures —
   `HologramCacheService/get`, `HologramCacheService/put` —
   and the `Spawn`, `ReqTxPool`, `ReqTx`, `GetReplyTx`,
   `GetReplyRx`, `PutAckTx`, `PutAckRx`, `Entry` typealiases.
6. `/home/watmin/work/holon/wat-rs/crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   — the existing wat-tests file. Read `test-step1-spawn-join`
   and `test-step2-counted-recv` (both passing at HEAD —
   smallest known-good shapes). Steps 3-6 are the recovered
   hanging tests; do NOT model after them.
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md`
   — locked target shape for the post-arc-119 protocol
   (batch-of-one Get/Put, variant-scoped channel families).

## Scope — five stepping stones

Create these files under
`crates/wat-holon-lru/wat-tests/proofs/arc-119/` (NEW
subdirectory; the test runner discovers it recursively):

| File | What it adds | Pass claim |
|---|---|---|
| `step-A-spawn-shutdown.wat` | spawn HologramCacheService, pop ONE req-tx, drop scope, join. NO put / get. | post-arc-119 spawn/shutdown lifecycle is intact |
| `step-B-single-put.wat` | A + ONE put-batch-of-one via `HologramCacheService/put` helper, verify the call returns | single put cycle (send Request::Put, driver acks, scope drops, shutdown clean) |
| `step-C-single-get.wat` | B + ONE get-batch-of-one via `HologramCacheService/get` helper, verify returned `Vec<Option<HolonAST>>` | single get cycle; channel reuse on req-tx works between calls |
| `step-D-three-puts.wat` | C with 3 sequential puts (no get) at cap=2 | multi-put on shared (ack-tx, ack-rx) — the bounded(1) buffer cycles N times correctly |
| `step-E-three-puts-two-gets.wat` | D + 2 gets (matches the recovered step-6's full scenario) | full eviction round-trip; tells us if the hang is in puts-then-gets specifically |

Each file: one `(:wat::test::deftest ...)` form. Each test is
INDEPENDENT — owns its own spawn, scope, and shutdown.

## Use the helper verbs (caller-perspective)

This is non-negotiable per `CONVENTIONS.md` § "Caller-perspective
verification" + the vocare ward. Tests at this layer call:

- `(:wat::holon::lru::HologramCacheService/spawn count cap reporter cadence)`
- `(:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx entries)`
- `(:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx probes)`

NOT raw `(:wat::kernel::send req-tx (Request::Put k v))`. The
recovered hanging tests use raw protocol; the stepping stones
must NOT reproduce that pattern. If you find yourself reaching
for `Request::Put` or `Request::Get` directly, stop — that's
the wrong vantage. The helper verbs encode the protocol.

## Use deftest-hermetic

HologramCacheService spawns driver threads. In-process tests
hit thread-ownership issues; use the hermetic (forked-child)
variant. Either:

- Define a per-file `(:wat::test::make-deftest :deftest-hermetic ())`
  with empty prelude, then `(:deftest-hermetic :name body)` per
  test, OR
- Use `(:wat::test::deftest-hermetic :name () body)` directly.

Mirror the existing tests' style.

## Validation — per-step running

Arc 121 + 122 just shipped per-deftest `#[test] fn` emission.
You can run JUST one stepping stone:

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release -p wat-holon-lru step_A_spawn_shutdown
cargo test --release -p wat-holon-lru step_B_single_put
# ... etc
```

The sanitized name maps `:` → `_`, `-` → `_`. Run each step
INDIVIDUALLY. Don't run the whole crate's tests (the recovered
step-3/4/5/6 in the existing wat-tests file STILL hang — those
will hang the whole binary if you run them).

## Process

1. Write step A.
2. `cargo test --release -p wat-holon-lru step_A_spawn_shutdown`.
3. If it passes, write step B.
4. Run step B alone.
5. Continue to C, D, E.
6. The FIRST slice that hangs is the deadlock-class indicator.
7. **Do NOT proceed past a hanging slice.** Stop, report, mark
   the slice as hanging in your report.

If a slice fails for a reason OTHER than hang (e.g., type
error, assertion failure with a clear diagnostic), that's
substrate-as-teacher feedback — fix the test in-place against
the diagnostic, then proceed. Substrate-level errors point at
real wat-side discipline gaps.

If a slice HANGS — kill it (Ctrl-C in your shell, or
`pkill -f wat-holon-lru` from another shell) and report. Don't
keep it running.

## Discipline anchors

- **Caller-perspective.** Helper verbs only. No raw
  `Request::Put` / `Request::Get` constructors.
- **Hermetic forking.** Each test in its own subprocess.
- **Service-shutdown discipline.** Outer scope holds Thread;
  inner scope owns Senders. Inner exits → senders drop →
  driver disconnects → outer joins clean. Mirror
  `test-step1-spawn-join`'s shape.
- **One thing per file.** Step A doesn't try to be step B.
  Resist the urge to combine.
- **Run after each slice.** Don't write all 5 then run them
  all at once. Each step is its own checkpoint.

## Reporting back

For EACH slice:

1. The file's path
2. `git diff --stat <path>` line count
3. cargo test outcome — passed, failed-with-error, or HUNG
4. If failed-with-error: the exact error
5. If hung: which scope was active when you killed it (which
   binding is the test currently inside?)

For the hanging slice (whichever one is first), include:

- The structural shape that's NEW in this slice vs the
  previous passing slice.
- Your hypothesis on what's making it hang (from inspection
  + diagnostic stream, not running).

The orchestrator uses this to design the next arc — a
compile-time deadlock check that catches the structural
pattern this slice demonstrates.

## What this brief is testing (meta)

This brief is a deliberate test of the substrate-as-teacher
discipline. The user wants to see if you can build five
caller-perspective stepping-stone tests by reading:

- proof_004's pattern in the lab
- SERVICE-PROGRAMS.md / CONVENTIONS.md
- The substrate file's helper verb signatures
- The existing test-step1 / test-step2 shapes

If you can — the docs work. The patterns work. The substrate
teaches. If you can't — the gap surfaces and gets fixed.

Be honest. If something in the docs reads ambiguously, flag
it. If a pattern seems missing, flag it. Don't invent a shape
to fit — surface the gap.

## Constraints

- ONE new directory: `crates/wat-holon-lru/wat-tests/proofs/arc-119/`
- FIVE new files: `step-A-spawn-shutdown.wat` through `step-E-three-puts-two-gets.wat`
- Do NOT touch existing files
- Do NOT touch substrate (`crates/*/wat/`)
- Do NOT delete or modify the existing recovered wat-tests
- Do NOT touch `holon-lab-trading/`

Working directory: `/home/watmin/work/holon/wat-rs/`.
