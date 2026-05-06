# Arc 130 — Test File Rebuild EXPECTATIONS

**Drafted 2026-05-06.** Pre-handoff scorecard for the
post-arc-110-aware test file rebuild.

**Brief:** `BRIEF-TEST-FILE-REBUILD.md`
**Output:** ONE file rewritten from empty
(`crates/wat-lru/wat-tests/lru/CacheService.wat`) +
~250-word written report.

## Setup — workspace state pre-spawn

- Last commit `b4dbd45` (substrate consumer sweep + SCORE).
- Workspace: 1 failing test
  (`deftest_wat_lru_test_lru_raw_send_no_recv` — currently fails
  with arc 110's `"reply-tx disconnected"` panic). Layer 0 in the
  same file passes.
- Substrate is in SOUND state post-Vector/length sweep. Helper
  verbs `:wat::lru::get` + `:wat::lru::put` exist with their
  arc-110-aware send-AND-recv internals. Pair-by-index pool +
  Reply<V> enum unification all in place.
- Sonnet has no conversation memory of any prior sweep on this
  file. Walks in cold; reads the artifacts; builds the test file
  from empty.

## Hard scorecard (12 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | One-file diff | Exactly 1 file modified: `crates/wat-lru/wat-tests/lru/CacheService.wat`. NO substrate. NO other crate. NO Rust. |
| 2 | File rebuilt from empty | The file's content is a fresh write — no inheritance from the prior 98 LOC; the prior `:wat-lru::test-lru-spawn-and-drop` and `:wat-lru::test-lru-raw-send-no-recv` deftests are GONE. (Spot-check: `git diff` shows the prior 98 LOC REMOVED entirely, replaced with the new structure.) |
| 3 | Layer count + naming | Either ALL suggested layers (0..3 minimum, ideally 0..5) present, OR layers 0..N present with a clear stop-at-failure at Layer N+1. Each layer is a `:test::lru-*` named helper + a sibling `(:wat::test::deftest ...)` referencing it. |
| 4 | Per-helper deftest discipline | Every named `:test::lru-*` helper has its OWN deftest. Verifiable: count of `(:wat::core::define (:test::lru-*` defines = count of `(:wat::test::deftest :wat-lru::test-lru-*` deftests. (Spell's Level 2 mumble check.) |
| 5 | Top-down dependency graph | No forward references. Verifiable: for each helper at line N, all helpers it calls (matching `:test::lru-*`) are defined at lines < N. (Spell's Level 1 lie check.) |
| 6 | Body line budget | Each helper's body: single outer `let*`, 3-7 bindings. Each deftest's body: 3-7 lines. Spot-check: no helper body > 12 lines; no deftest body > 8 lines. (Spell's "deftest body 3-7 lines" rule.) |
| 7 | Time-limit discipline | Either each deftest carries `(:wat::test::time-limit ...)`, OR none do (relying on arc 132's 1000ms default). Consistent across the file. |
| 8 | Helper-verb usage | Layer 1+ uses `:wat::lru::get` and `:wat::lru::put` as the primary interface. Raw `:wat::kernel::send` / `:wat::kernel::recv` may appear ONLY if a layer's purpose is explicitly to probe substrate plumbing (justified in helper docstring). |
| 9 | Arc 110 contract honored | NO layer drops a handle's reply-rx without recv'ing first (the prior file's anti-pattern). Every send → recv pair in scope. |
| 10 | **`cargo test --release -p wat-lru`** | Mode A: all shipped layers report `... ok`; total wat-lru test count goes from `8 passed/1 failed` to `(8 + N) passed / 0 failed` where N = layers shipped + Layer 0. Mode B: the FIRST failing layer's deftest fails; layers above it pass; no layers were skipped past the failure. EITHER outcome is acceptable. |
| 11 | Layer-pass discipline + stop-at-first-red | Sonnet ran `cargo test` AFTER EACH LAYER was added (per the brief's Step 3 workflow). Verifiable in the report's pass/fail roll-up. If a layer fails, NO subsequent layers were added. |
| 12 | **No grinding** | Sonnet does NOT modify the substrate to make a failing layer pass. Sonnet does NOT iterate on a single failure beyond reporting it. The reland is a measurement, not a fix. |

**Hard verdict:** all 12 must hold. Rows 4-6 are the
complectens-discipline rows — load-bearing for proving the spell
teaches. Row 9 is the post-arc-110 lesson row. Row 12 is
load-bearing for the failure-engineering discipline.

## Soft scorecard (5 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 13 | LOC budget | Total file LOC: 100-300 (per-layer 15-30 LOC including helper + deftest, times 4-7 layers, plus a 20-30 LOC header comment block). >400 LOC = re-evaluate. |
| 14 | Header comment block | File starts with a short comment block naming the discipline (one paragraph) + the layered map (one line per layer). Mirrors the prior file's header style — but updated to reference Reply<V>'s GetResult/PutAck variants and arc 110's contract. |
| 15 | Final scenario layer | If Mode A: the final shipped layer is a multi-step scenario (e.g., put-then-get-many or eviction probe) that composes 2-3 prior helpers. The deftest body is 3-5 lines composing the layered helpers + assertions. |
| 16 | Diagnostic clarity (Mode B) | If Mode B: the failing layer's name is OBVIOUS about what it tests. A reader sees `lru-helper-put-then-get FAILED` and knows the bug is in the cross-verb composition (not in put alone, not in get alone). |
| 17 | Honest deltas | Sonnet flags any layer where the substrate's actual shape diverges from the brief's prediction. E.g., if `(:wat::lru::get handle <empty-vec>)` doesn't return what the brief expects, sonnet adapts the layer + reports the divergence. |

## Independent prediction

- **Most likely (~55%) — Mode A clean ship at 4-5 layers.** The
  substrate consumer sweep cleared the only-known
  substrate-vocabulary gap; helper verbs (`:wat::lru::get`,
  `:wat::lru::put`) are designed to do send-AND-recv per arc 110;
  the happy path should work end-to-end. ~30-45 min wall-clock
  for a 4-5 layer rebuild.
- **Mode B at Layer 4-5 (~20%):** a multi-key or eviction layer
  surfaces a substrate edge case — e.g., Reply enum routing for
  multiple in-flight requests, or LocalCache eviction interaction
  with reporter callbacks. Diagnostic clean; open follow-on arc.
- **Mode B at Layer 1-3 (~15%):** the helper verb path itself has
  a bug post-substrate-consumer-sweep. Probably reveals a missing
  cascade link beyond `Vector/length`. Layer N's name names the
  broken behavior.
- **Mode C — complectens violation (~10%):** sonnet ships
  monolithic deftest, skips per-helper deftests, forward-refs.
  Hard rows 4-6 fail. Reland with sharper brief.

## Time-box

45 minutes wall-clock (≈1.5× the predicted upper-bound of 30
min). If the wakeup fires and sonnet hasn't completed: TaskStop +
Mode B-time-violation score with the overrun as data.

## What sonnet's success unlocks (forward progress only)

**Mode A:** slice 1's test side ships. Slice 2 (HolonLRU mirror)
becomes the next sweep — same shape, different crate. Slice 3
(closure paperwork) follows.

**Mode B:** slice 1's test side ships through Layer N-1 cleanly;
Layer N's diagnostic is the next chain link in the cascade. Open
follow-on arc for the substrate fix; resume after.

**Mode C:** the complectens spell needs sharpening AGAIN; reland
with feedback baked in. The killed-sweep calibration set in
`complected-2026-05-02/` already preserves "what bad looks like";
this Mode C would add another data point.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Diff via `git diff --stat` → expect 1 file modified.
- Read the rewritten test file from top to bottom — verify
  layer ordering, no forward refs, helper+deftest pairing,
  body line counts.
- Run `cargo test --release -p wat-lru` locally → confirm
  sonnet's reported test totals.
- Score; commit `SCORE-TEST-FILE-REBUILD.md` as a sibling.

## Why this matters for the cooperation

User's direction 2026-05-06 names the rebuild approach.
Orchestrator's brief restates that approach for sonnet's
context-free execution. Sonnet's ship verifies the restatement.

The mutual-agreement chain:
- User → Orchestrator: "rewrite tests from ground up using the
  pattern; pass handles correctly per the post-arc-130 substrate"
- Orchestrator → Sonnet: this brief, with explicit layer plan +
  arc 110 contract awareness + complectens discipline
- Sonnet → Reality: the test file ships with N happy-path layers

Mode A = chain held; substrate honestly works for the happy path.
Mode B = chain held; surfaced a deeper substrate bug for follow-on.
Mode C = orchestrator → sonnet transmission was lossy; sharpen.

Each outcome calibrates the cooperation. The brief's shape IS
the orchestrator's understanding; sonnet's execution IS the
test of that understanding.
