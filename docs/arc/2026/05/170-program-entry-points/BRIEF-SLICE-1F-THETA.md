# Arc 170 slice 1f-θ — BRIEF (hermetic test-body deadlock triage)

> ⚠️ **STALE — DO NOT EXECUTE THIS BRIEF AS-IS.**
> The premise (fix deadlocks by tweaking flat-let bind order) was wrong.
> Sonnet flailed and was killed by user. The correct fix is STRUCTURAL per
> `/complectens` — layered named helpers, per-helper deftests, 3-7 line
> final test bodies, top-down dependency graph in one file.
>
> **Supersedes:** `BRIEF-SLICE-1F-THETA-V2.md` (V2; reflects complectens
> discipline + references canonical example).
>
> Original content below preserved as historical record per "what is
> inscribed is inscribed."
>
> ---

**Sonnet pattern-apply.** Fixes the test-body deadlocks introduced in slices 1f-β-i/ii/iii (the wat-side trio test files I authored). These deadlocks are the **root cause** of the test-binary orphan leak documented at `FOLLOWUPS-TEST-BINARY-LEAK.md` (HEAD `287c0f5`).

**Direction context** (user 2026-05-10): *"do we attack these leaks or do we attack the failing tests? I think resolving the failing tests will resolve the leak."* Correct — Tier 4 of the followup framework is the root-cause fix; Tiers 1-3 are workarounds.

## Slice surface

> *"Fix the hermetic test-body deadlocks I shipped in 1f-β-i/ii/iii."*

12 failing tests across the StdIn/StdOut/StdErr trio. The canonical correct pattern exists in `scope-drop-shutdown` (passes in all three test files). Apply the same shape to the 12 broken tests.

## Failing tests (12 = 4 per service × 3 services)

From `wat-tests/kernel/services/{stdin,stdout,stderr}.wat`:

| Test (per service) | Failure type |
|---|---|
| `spawn-shape` | structural scope-deadlock (flat let — `_ctrl-tx` Sender outlives `recv final-rx`) |
| `add-and-read` (stdin) / `add-and-write` (stdout/stderr) | channel-recv deadlock |
| `multi-thread-routing` | routing-table interaction deadlock |
| `remove-drops-entry` | cleanup race |

Passing reference: `scope-drop-shutdown` — uses nested `let` so the inner ControlTx scope drops before the outer `recv final-rx`.

## The canonical pattern (from passing test)

Per slice 1f-δ SCORE § Honest delta #3:

> *"`scope-drop-shutdown` correctly uses a nested let to ensure `_ctrl-tx` drops before `recv`."*

For `spawn-shape` × 3: the test body has a flat `let` where the ControlTx Sender is bound alongside `recv final-rx`. The Sender doesn't drop until ALL bindings drop (end of let scope) — meaning the service program never sees control-channel disconnect, so it never sends the final `()` value, so `recv final-rx` blocks forever.

**Fix shape:**
```diff
- (let [thr-and-ctrl (... spawn ...)
-       _ctrl-tx (second thr-and-ctrl)
-       final-rx (Thread/output (first thr-and-ctrl))
-       _final (recv final-rx)        ;; deadlock: _ctrl-tx hasn't dropped
-       result (Thread/join-result thr)]
-   ...)

+ (let [final-rx
+         (let [thr-and-ctrl (... spawn ...)
+               _ctrl-tx (second thr-and-ctrl)]
+           ;; _ctrl-tx drops here when inner let scope exits
+           (Thread/output (first thr-and-ctrl)))
+       _final (recv final-rx)        ;; now succeeds: control-tx dropped → service exits → final () sent
+       result (Thread/join-result thr)]
+   ...)
```

For `add-and-read`/`add-and-write`/`multi-thread-routing`/`remove-drops-entry` × 3: similar but the specific blocking pattern varies by test. Sonnet should read each, identify the deadlock (channel recv that won't complete because Sender hasn't dropped, OR a race in cleanup), and apply the analogous shape.

## Scope

### Edits

Touch only the 3 test files:
- `wat-tests/kernel/services/stdin.wat`
- `wat-tests/kernel/services/stdout.wat`
- `wat-tests/kernel/services/stderr.wat`

Within each file, edit the 4 deadlocking test bodies. Leave `scope-drop-shutdown` (the canonical reference) unchanged.

### Out of scope

- No substrate Rust edits
- No new test cases — just fix the bodies of the 12 broken ones
- No changes to `:wat::test::deftest-hermetic` macro
- No changes to slice 1f-γ orchestrator or trio service definitions

## Pre-flight verification

```bash
grep -n "scope-drop-shutdown" wat-tests/kernel/services/stdin.wat | head -3
# Read the passing test as the template; check structure of broken tests
```

The orchestrator pre-flighted: passing test exists in all 3 files; broken tests follow visible patterns. Sonnet should sample 1 broken test + the passing reference to confirm the pattern, then apply to all 12.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | All 12 hermetic tests pass when run individually | `cargo test --test test deftest_<name>` for each succeeds |
| B | `cargo check --release` green | clean |
| C | Workspace failure count drops by ≥ 12 (post-1f-ζ baseline 2151/48) | toward 2163/36 or better |
| D | Pass count rises by ≥ 12 | same |
| E | No regression of pre-existing passing tests | re-baseline |
| F | Only 3 files modified (the trio test files) | git status |
| G | Test bodies fixed via canonical nested-let / explicit-Sender-drop pattern | inline shape matches the spec |
| H | Honest deltas surfaced | per FM 5 |

**8 rows.**

## Predicted runtime

**30-60 min sonnet.** Pattern is bounded (12 tests, ~3-5 lines each). Read passing reference once → mint mental template → apply 12 times. Some tests may have non-obvious deadlock shapes; surface as honest delta if any test needs substantive rewrite beyond the template.

**Hard cap:** 120 min.

## Honest delta categories (anticipated)

1. **Non-uniform deadlock patterns** — `spawn-shape` is scope-deadlock (Sender lifetime). `add-and-read`/`add-and-write` might be different (channel-recv-forever for a different reason). Sonnet may need to identify each test's specific deadlock + apply the appropriate shape (drop-before-recv, explicit-Sender-drop, etc.).

2. **Test bodies may have OTHER bugs** — once the deadlock is fixed, an assertion might fail revealing a separate body issue. Surface count of "deadlock-fixed but other-issue remains" tests.

3. **`scope-drop-shutdown` reference may not directly apply to all** — if a test's pattern doesn't match scope-drop-shutdown's shape closely, sonnet should sample the data path (what's sent, what's received, where the block is) and reason from first principles.

4. **Workspace count uncertainty** — fixing 12 tests should drop failures by 12, but if these tests were chain-blocking others, more recovery may surface.

## What to NOT do

- No substrate Rust edits
- No timeout / SIGTERM-handler installations (Tier 1/2 from followup; separate concerns)
- No fork-program-ast PDEATHSIG (Tier 3 from followup; separate)
- No changes to deftest-hermetic macro
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Verification

```
cargo check --release 2>&1 | tail -3
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Verify individual test passes
cargo test --release --test test deftest_stdin_test_spawn_shape
```

Post-1f-ζ baseline: **2151 passed / 48 failed**. Expected post-1f-θ: failure count drops by ≥ 12 toward 2163/36 (or better if chain-unblocks).

## Reference

- `wat-tests/kernel/services/stdin.wat` — passing `scope-drop-shutdown` test (canonical reference)
- Slice 1f-δ SCORE § Honest delta #3 — discovery of scope-deadlock pattern
- FOLLOWUPS-TEST-BINARY-LEAK.md (HEAD `287c0f5`) — root-cause framework; this slice executes Tier 4
- Predecessors: 1f-β-i/ii/iii (the slices that introduced these deadlock-prone test bodies)

## Path forward post-slice-1f-θ

1. Orchestrator scores; atomic-commits deliverable + SCORE; pushes
2. **Verify leak resolved** — run `cargo test --release --workspace`; check for orphans
3. **Remaining ~36 failures** — split between retired-verb tests (sibling slice) + wat-cli echo + OOM-SIGKILL infra
4. **Sibling slice — restore retired `spawn-program` / `fork-program-ast`** (bridge pattern; ~22 BareLegacy* failures)
5. **Arc 170 INSCRIPTION** — once baseline near-zero
