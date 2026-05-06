# Arc 130 — HolonLRU Test Rebuild EXPECTATIONS (sweep 2b)

**Drafted 2026-05-06.** Pre-handoff scorecard for the HolonLRU
test rebuild + `:should-panic` retirement (sweep 2b of arc 130).

**Brief:** `BRIEF-HOLON-LRU-TEST-REBUILD.md`
**Output:** EDITS to 3 test files + ~250-word written report.
NO substrate edits. NO commits.

## Setup — workspace state pre-spawn

- Sweep 2a (substrate reshape) shipped Mode A clean; working
  tree dirty with `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  in NEW shape (Handle/DriverPair/Reply enum/pair-by-index).
- Workspace test profile: 16 wat-holon-lru tests fail with
  TYPE-MISMATCH errors. ALL OTHER WORKSPACE TESTS PASS.
- The 16 failing tests breakdown:
  - 9 `:should-panic("channel-pair-deadlock")` in main test file
    (substring no longer matches; failing)
  - 5 non-:should-panic tests in main file (type errors directly)
  - 1 in `proofs/arc-119/step-A-spawn-shutdown.wat`
  - 1 in `proofs/arc-119/step-B-single-put.wat` (was :should-panic,
    now substring mismatch)
- wat-lru's CacheService.wat test rebuild from this afternoon is
  the canonical template (5 layers Mode A clean; factory prelude;
  tuple-out adaptations).

## Hard scorecard (12 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EXACTLY 3 files modified per `git diff --stat`: main test file (`HologramCacheService.wat`), step-A proof, step-B proof. NO substrate (substrate is sweep 2a's work in working tree; should be IDENTICAL to its post-2a state in this sweep's output). NO Rust source. NO other crate. |
| 2 | Main test file rebuilt from empty | The prior 731-LOC monolithic structure with 14 deftests is GONE; replaced with a `make-deftest` factory + 5+ layered deftests. Verifiable: `git diff` shows the prior structure REMOVED entirely. |
| 3 | Layer count + naming | At minimum Layers 0..4 (spawn-and-drop, helper-get-empty, helper-put-one, helper-put-then-get, helper-get-many-keys) shipped. Optional Layers 5-6 (eviction, multi-client) acceptable if relevant scenarios from old test file warranted preserving. Each layer is a `:test::hcs-*` named helper + a sibling factory invocation. |
| 4 | Per-helper deftest discipline | Every named `:test::hcs-*` helper has its own deftest, with the SKILL's Level-3-taste exemption acceptable for sub-helpers used in exactly one place. |
| 5 | Top-down dependency graph | No forward references. For each helper at line N, all helpers it calls are defined at lines < N. |
| 6 | Body line budget | Helper outer let*: 3-7 bindings. Deftest body: 3-7 lines. Spell's "deftest body 3-7 lines" rule. |
| 7 | Helper-verb usage | Layer 1+ uses `:wat::holon::lru::HologramCacheService::get` / `put` (post-sweep-2a) as primary interface. NO raw `:wat::kernel::send`/`:wat::kernel::recv`. |
| 8 | **All 9 LRU `:should-panic` annotations RETIRED** | Verifiable: `grep -rn ":should-panic" crates/wat-holon-lru/wat-tests/` shows ZERO matches in the rebuilt files. (The 1 in arc-122 wat-sqlite is the only remaining `:should-panic` in the workspace; that's the mechanism's own self-test, NOT LRU-related.) |
| 9 | Proof step-A updated | `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat` uses NEW substrate shape (HandlePool<Handle>; pop-finish-join lifecycle). Test passes. Educational shape preserved. |
| 10 | Proof step-B updated + :should-panic retired | `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` uses NEW shape; the `:should-panic("channel-pair-deadlock")` annotation is GONE; the test passes naturally (helper verb's send-AND-recv contract handles the lifecycle cleanly). |
| 11 | **Workspace at 0 failed** | `cargo test --release --workspace` shows 0 failed across ALL crates. The wat-holon-lru crate's 16 failing tests resolve to passing; no new failures elsewhere. |
| 12 | Honest report | Per BRIEF reporting requirements. Layer roll-up; proof updates; workspace verification; path classification; honest deltas. |

**Hard verdict:** all 12 must hold. Rows 8 + 11 are the
load-bearing rows (the LRU :should-panic crutches retire AND
the workspace ships clean).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 13 | LOC budget | Main test file: 200-450 LOC (sweep 1's wat-lru rebuild was 325; HolonLRU may be slightly larger if 5-6 layers ship). Proofs: ±20% of current size. >500 LOC main = re-evaluate. |
| 14 | Pattern fidelity to wat-lru rebuild | Factory prelude pattern matches sweep 1's structure. Tuple-out pattern applied where scope-deadlock check would fire. Honest deltas flagged. |
| 15 | clippy clean | wat-source-only edits; no Rust delta. |
| 16 | No-grinding discipline | Sonnet does NOT modify substrate. Does NOT add :ignore as substitute for :should-panic. STOPS at first red and reports rather than iterating. |

## Independent prediction

- **Most likely (~65%) — Mode A clean ship.** wat-lru's rebuild is
  a complete template; the two adaptations (factory + tuple-out)
  are documented. HolonLRU substrate is sound (sweep 2a Mode A
  clean). Mechanical pattern-application + retire annotations.
  ~30-45 min wall-clock.
- **Mode B at Layer 1-3 (~15%):** the helper-verb path on HolonLRU
  has a bug post-substrate-reshape that wat-lru didn't surface
  (e.g., HolonAST-keyed cache vs String-keyed cache interaction).
  Honest STOP; surface; orchestrator decides whether to fix
  substrate (rare; sweep 2a was Mode A) or rescope.
- **Mode B at Layer 5-6 (~10%):** eviction or multi-client edge
  case surfaces a substrate gap. Same handling.
- **Mode C complectens violation (~10%):** sonnet ships
  monolithic deftests, missing per-helper deftests, forward refs,
  or other shape violations. Reland with sharper brief.

## Time-box

90 minutes wall-clock (1.5× the predicted upper-bound of 60 min).
3 files + retire annotations is more work than sweep 1's
single-file rebuild. If wakeup fires and sonnet hasn't completed:
TaskStop + Mode B-time-violation score with overrun as data.

## What sonnet's success unlocks (forward progress only)

**Mode A clean**:
- Sweep 2a + sweep 2b commit ATOMICALLY as one HolonLRU cleanup ship
- Arc 130 slice 1 INSCRIPTION + slice 2 INSCRIPTION can be drafted
  (or one combined INSCRIPTION covering both)
- Arc 130 v1 closure within reach (slice 3 = closure paperwork)
- Arc 109 K.holon-lru slice (#195) becomes tractable post-arc-130
- Arc 109 v1 milestone closer

**Mode B/C**: surface the gap; orchestrator adjusts brief; reland.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Diff via `git diff --stat` → expect 4 files (1 substrate from
  sweep 2a + 3 test files from sweep 2b).
- Read the rebuilt main test file from top to bottom — verify
  layer ordering, no forward refs, helper+deftest pairing,
  body line counts.
- Run `cargo test --release --workspace` locally → confirm 0-failed.
- Write `SCORE-HOLON-LRU-SUBSTRATE-RESHAPE.md` (sweep 2a) + `SCORE-HOLON-LRU-TEST-REBUILD.md` (sweep 2b) as siblings.
- Commit ALL changes (substrate from 2a + tests from 2b + both
  SCORE docs) AS ONE ATOMIC COMMIT per `feedback_no_broken_commits.md`.

## Why this brief matters for the cooperation

User direction 2026-05-06: cleanup HolonLRU. Sweep 2a shipped the
substrate side; this brief ships the test side. Together they
prove arc 130's premise: pair-by-index discipline propagates;
the deadlock-pattern tests retire because the deadlock pattern
is GONE from the substrate.

The mutual-agreement chain:
- User → Orchestrator: cleanup direction
- Orchestrator → Sonnet: rebuild tests; retire :should-panic;
  mirror wat-lru's pattern; concrete typing
- Sonnet → Reality: 3 files updated; 9 :should-panic crutches
  gone; workspace 0-failed

Mode A clean = the discipline propagates from wat-lru to HolonLRU
end-to-end. Arc 130's substrate-as-teacher cascade closes.
