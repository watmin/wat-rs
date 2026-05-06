# Arc 130 — Substrate consumer sweep EXPECTATIONS

**Drafted 2026-05-06.** Pre-handoff scorecard for the
`Vector/len` → `Vector/length` consumer sweep.

**Brief:** `BRIEF-SUBSTRATE-CONSUMER-SWEEP.md`
**Output:** EDITS to 2 substrate files (4 mechanical renames
total) + ~150-200 word written report.

## Setup — workspace state pre-spawn

- Working tree clean (last commit `030dc2e` —
  "what is inscribed is inscribed")
- Workspace baseline: `cargo test --release --workspace` shows
  1 failing test: `deftest_wat_lru_test_lru_raw_send_no_recv`
  with payload `unknown function: :wat::core::Vector/len` at
  `crates/wat-lru/wat/lru/CacheService.wat:219:33`
- Total Vector/len references in substrate: 4
  - `crates/wat-lru/wat/lru/CacheService.wat:219` (Get branch)
  - `crates/wat-lru/wat/lru/CacheService.wat:246` (Put branch)
  - `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:257` (Get)
  - `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:285` (Put)
- Substrate registers `Vector/length` per:
  - `wat/core.wat:13` Dispatch arm
  - `src/runtime.rs:2841` eval dispatch
  - `src/check.rs:11532` TypeScheme registration

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EXACTLY 2 files modified: `crates/wat-lru/wat/lru/CacheService.wat` + `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`. NO other files. NO test files. NO Rust source. NO wat substrate (`wat/core.wat` etc.) |
| 2 | All 4 sites swept | Verifiable: `git diff` shows exactly 4 lines changed (one per site listed in the brief's site table); each is `:wat::core::Vector/len` → `:wat::core::Vector/length`. No other content delta in those files. |
| 3 | Verbatim rename only | Each replaced line preserves all surrounding code, comments, indentation. The diff for each site is `-…Vector/len…` / `+…Vector/length…` with everything else identical. |
| 4 | Failing test now passes | `cargo test --release -p wat-lru deftest_wat_lru_test_lru_raw_send_no_recv` reports `... ok`. Before sweep: FAILED. After sweep: ok. |
| 5 | No previously-passing test breaks | `cargo test --release --workspace 2>&1 \| grep -E "test result\|FAILED"` shows the workspace test count's PASSING column equals (prior-passing + 1) and FAILING column equals (prior-failing - 1). No new failures. |
| 6 | The 6 :should-panic tests preserve state | The 6 deadlock-class tests in `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` (lines 632, 643, 652, 660, etc.) remain :should-panic and remain panicking on `channel-pair-deadlock` substring. They are NOT retired; that is slice 3 closure work. |
| 7 | No new substrate primitives or removals | The fix is rename-only. NO `register_builtin` additions. NO removals. The substrate's set of registered primitives is unchanged. |
| 8 | Honest report | ~150-200 word report covers: pre-flight crawl confirmation; edits made; verification numbers (passed/failed/ignored before vs after); :should-panic preservation; path classification (A/B/C); honest deltas if any. |

**Hard verdict:** all 8 must pass. Rows 4 + 5 + 6 are load-bearing
(the rename does what it says without side effects).

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Total LOC delta | 4 lines changed total (one per site). >4 lines = scope creep; investigate. |
| 10 | clippy clean | No new clippy warnings (this is wat-source-only edit; clippy applies to Rust; expected to be a no-op delta). |
| 11 | Stop-at-first-red discipline | If a verification step surfaced a problem, sonnet STOPS and reports rather than iterating to make it pass. Verifiable from the report's path classification. |

## Independent prediction

- **Most likely (~85%) — Mode A clean ship.** The brief's
  evidence is precise; the substrate definitely has
  `Vector/length`; the 4 sites are clearly identified; the
  failing test's payload literally names the missing primitive
  with file:line. Mechanical rename. ~5-10 min wall-clock.
- **Mode B (~10%):** the rename works but a previously-passing
  test breaks unexpectedly (cascade name-resolution issue I
  haven't anticipated). Sonnet stops + surfaces.
- **Mode C (~5%):** the rename DOESN'T fix the failing test
  (the substrate has a deeper bug beyond the rename). The
  brief's evidence was wrong; sonnet stops + surfaces.

## Time-box

20 minutes wall-clock (≈2× the predicted 10-min upper bound).
If wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with overrun as data.

## What sonnet's success unlocks (forward progress only)

- Layer 1 of the arc 130 slice 1 RELAND turns green
- Continuing Layers 2-7 of the original BRIEF-SLICE-1-RELAND
  becomes tractable (separate brief)
- Arc 130 slice 1's substrate consumer-sweep gap closes
- The cascade's `:wat::core::Vector/len` chain link closes
  (per arc 143's INSCRIPTION naming this as the next link)

## After sonnet completes

- Read this file FIRST
- Re-run the workspace test suite locally to verify report's
  numbers
- `git diff --stat` → expect 2 files modified, 4 lines net
- Read the diffs to verify the renames are mechanical (no
  surrounding context drift)
- Score each row of both scorecards explicitly
- Write `SCORE-SUBSTRATE-CONSUMER-SWEEP.md` as a sibling
- Commit + push BEFORE drafting any next brief (calibration
  preserved)

## Why this matters for the cooperation

User's framing 2026-05-06: "if i can teach you and you can
teach sonnet - then i have full clarity of my ask."

This brief is the orchestrator's restatement of the user's ask
in a form a context-free sonnet can execute. If sonnet ships
clean against this brief:
- User → Orchestrator transmission verified
- Orchestrator → Sonnet transmission verified
- Sonnet → Reality transmission verified

The chain holds. If any link fails, the failure surfaces which
transmission was lossy, and the discipline tightens.
