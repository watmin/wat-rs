# Arc 130 — HolonLRU Substrate Reshape EXPECTATIONS

**Drafted 2026-05-06.** Pre-handoff scorecard for the
HolonLRU substrate reshape (sweep 2a).

**Brief:** `BRIEF-HOLON-LRU-SUBSTRATE-RESHAPE.md`
**Output:** EDITS to 1 substrate file
(`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`)
+ ~250-word written report.

## Setup — workspace state pre-spawn

- Last commit `8c96ce2` (wat-lru test file rebuild Mode A clean).
- Workspace currently at **0 failed tests** (workspace milestone
  achieved this afternoon).
- HolonLRU substrate state: OLD shape — `GetReplyTx/Rx/Pair` +
  `PutAckTx/Rx/Channel` typealiases; helper verbs take 3 channel
  ends; per-call channel allocation in consumer tests trips arc
  126's compile-time deadlock check.
- HolonLRU consumer tests: 26 passed / 0 failed including 9
  `:should-panic("channel-pair-deadlock")` tests passing via
  expected panic.
- Working-tree-clean discipline: sonnet should NOT commit. Sweep
  2b will run immediately after; orchestrator commits sweep 2a
  + sweep 2b atomically per `feedback_no_broken_commits.md`.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | One-file diff | EXACTLY 1 file modified: `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`. NO test files. NO Rust source. NO other crate. |
| 2 | OLD typealiases retired | `GetReplyTx`, `GetReplyRx`, `GetReplyPair`, `PutAckTx`, `PutAckRx`, `PutAckChannel` are GONE from the substrate. Verifiable: `grep -nE "GetReplyTx\|GetReplyRx\|GetReplyPair\|PutAckTx\|PutAckRx\|PutAckChannel"` returns zero matches in the substrate file (or only inside historical-comment lines, not active typealiases). |
| 3 | NEW typealiases introduced | `Reply` enum (variants `GetResult` + `PutAck`), `ReplyTx`, `ReplyRx`, `ReplyChannel`, `Handle`, `DriverPair` all defined. Concrete (no `<V>`). Verifiable via grep. |
| 4 | Spawn factory reshaped | Pre-allocates N (ReqChannel, ReplyChannel) pairs; builds `HandlePool<Handle>`; builds `Vector<DriverPair>` for driver. spawn return type: `(HandlePool<Handle>, Thread<unit,unit>)` — same shape as wat-lru. |
| 5 | Driver loop reshaped | select fires at index `i`; same index locates the matching `DriverPair`'s ReplyTx; sends `Reply::GetResult` or `Reply::PutAck` on that ReplyTx. Mirrors wat-lru's loop-step structure. |
| 6 | Helper verb signatures changed | `HologramCacheService/get` takes `(handle :Handle, probes :Vector<HolonAST>)` instead of 3 channel ends + probes. `HologramCacheService/put` takes `(handle :Handle, entries :Vector<Entry>)` instead of 3 channel ends + entries. Each helper does send-AND-recv internally per arc 110. |
| 7 | Substrate parses cleanly | `cargo test --release -p wat-holon-lru --test test 2>&1` shows NO parse-error / syntax-error / "unknown form" errors attributable to the substrate file. (Type errors in CONSUMER tests are expected; parse errors in substrate are not.) |
| 8 | Substrate self-types cleanly | The substrate file's internal references resolve correctly: helper-verb bodies type-check against new typealiases; spawn body type-checks; driver loop body type-checks. NO substrate-internal type errors. |
| 9 | Consumer test failures shape (Mode A predicted) | The 9 `:should-panic` tests + arc-119 proof + 9 other HolonLRU consumer tests will fail with errors that are TYPE-MISMATCH ("expected GetReplyTx, got ...") OR "unknown function" (calling old verb signatures) — NOT parser errors, NOT panics other than the structural ones. **Honest delta documented in report.** |
| 10 | Honest report | ~250-word report covers pre-flight crawl, section-by-section edits, file LOC delta, verification, path classification, honest deltas. |

**Hard verdict:** all 10 must hold (with row 9 explicitly allowing
expected consumer failures by name). Rows 7-9 are the load-bearing
verification rows.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | -50 to +100 vs current ~538 LOC. Some lines retire (old typealiases + old verb signatures); some lines add (new typealiases + new verb bodies). Net change should be modest given the wat-lru template's similar overall LOC (~527). >+200 LOC = re-evaluate; >-150 = re-evaluate. |
| 12 | Pattern fidelity to wat-lru | Spawn factory body, driver loop body, helper verb body all mirror wat-lru's structure with HolonLRU-specific concrete typing (HolonAST instead of K,V). Honest divergences flagged in report. |
| 13 | clippy clean | wat-source-only edit; no Rust delta; clippy unchanged. |
| 14 | No-grinding discipline | Sonnet does NOT modify consumer tests to make them pass. Does NOT modify Rust source. Does NOT add new substrate primitives. The substrate edits stay in the SHAPE of wat-lru's template. |

## Independent prediction

- **Most likely (~70%) — Mode A clean ship.** wat-lru's
  substrate is the template; HolonLRU's reshape is mechanical
  pattern-application with concrete-typing adaptation. Substrate
  parses + self-types; consumer tests fail expectedly. ~30-45 min
  wall-clock.
- **Mode B-substrate-internal-bug (~15%):** sonnet introduces a
  bug in the substrate (e.g., driver-loop indexing wrong, helper
  verb body referencing a typealias incorrectly). Substrate
  parses but self-type-check fails OR substrate produces internal
  errors. Honest report; orchestrator adjusts brief.
- **Mode C-unexpected-consumer-failure-shape (~10%):** consumers
  fail for a reason that's NOT type-mismatch (e.g., load-order
  problem; runtime errors during freeze pipeline). Surfaces a
  cascade gap.
- **Mode D-pattern-divergence (~5%):** HolonLRU has a structural
  difference from wat-lru that the brief didn't anticipate
  (e.g., concrete typing complications). Sonnet honestly flags;
  brief adjusts.

## Time-box

90 minutes wall-clock (1.5× the predicted upper-bound of 60 min).
If wakeup fires and sonnet hasn't completed: TaskStop +
Mode B-time-violation score with overrun as data.

## What sonnet's success unlocks (forward progress only)

**Mode A clean**: Sweep 2b (test rebuild + retire `:should-panic`)
becomes the next brief. The wat-lru rebuild's worked patterns
(factory prelude, tuple-out for scope-deadlock) propagate to
HolonLRU. After 2b ships clean, both 2a + 2b commit atomically.

**Mode B/C/D**: surface gap; orchestrator drafts adjustment;
relands when ready.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Diff via `git diff --stat` → expect 1 file modified.
- Read the rewritten substrate file from top to bottom — verify
  pattern fidelity to wat-lru's template.
- Run `cargo test --release -p wat-holon-lru --test test` locally
  → confirm sonnet's reported failure shape (type errors, not
  parser errors).
- Score; **DO NOT COMMIT YET** — sweep 2b runs next; commits
  happen atomically together.

## Why this brief matters for the cooperation

User direction 2026-05-06: "lets get holon-lru cleaned up." The
four-questions check resolved to sequential (sweep 2a substrate
+ sweep 2b tests) over bundled. This brief is sweep 2a.

The mutual-agreement chain:
- User → Orchestrator: cleanup direction
- Orchestrator → Sonnet (this brief): mirror wat-lru substrate
  pattern in HolonLRU; substrate-only; concrete typing
- Sonnet → Reality: HolonLRU substrate ships in new shape; consumer
  failures expected pending sweep 2b

If sonnet ships Mode A clean, the chain holds for the substrate
side. If Mode B/C/D, the diagnostic clean-fires and sweep 2b
gets briefed against the actual state.
