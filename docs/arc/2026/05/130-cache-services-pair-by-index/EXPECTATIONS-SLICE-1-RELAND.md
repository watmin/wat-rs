# Arc 130 Slice 1 — RELAND Pre-handoff expectations

**Drafted 2026-05-02 (evening)** for the reland after the
original sweep's complectens-violating test file.

**Brief:** `BRIEF-SLICE-1-RELAND.md`
**Output:** ONE file modified (`crates/wat-lru/wat-tests/lru/CacheService.wat`
fully rewritten from empty) + ~250-word written report.

## Setup — workspace state pre-spawn

- LRU substrate is in the prior sweep's reshaped state
  (HandlePool<Handle>, unified Reply<V> enum, simplified
  Request enum, helper-verbs take Handle). Suspected bug at
  `loop-step` lines 387-391 (driver removes slot on `Ok(None)`
  from select). NOT REVERTING for this reland — the bug, if
  real, must surface at a stepping stone.
- LRU test file content from prior sweep: 4-helper /
  5-deftest layered structure; 4 of 5 tests failing with
  "reply channel closed — driver dropped reply-tx?". This
  poisoned file gets WIPED in the reland's Step 1.
- Workspace post-prior-sweep: 4 failing tests in wat-lru.
  After reland's wipe, workspace returns to compilable
  state with 0 failing tests in wat-lru (the wiped file has
  zero deftests; cargo test reports "0 passed").
- Sonnet has no conversation memory of the prior sweep.
  Walks in cold; reads the artifacts; builds the test file
  from empty.

## Hard scorecard (12 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | One-file diff | Exactly 1 file modified: `crates/wat-lru/wat-tests/lru/CacheService.wat`. No substrate. No other crate. No Rust. |
| 2 | File rebuilt from empty | The file's content is a fresh write — no inherited Layer 0 / Layer 1 / Layer 2 / Final structure from the prior sweep. The header comment block + the layered helpers below are NEW. (Spot-check: `git diff` shows the prior `make-deftest` factory + 4 helpers + 5 deftests REMOVED, and a new structure inserted.) |
| 3 | Layer count + naming | Either ALL 8 suggested layers present, OR 1..7 layers present with a clear stop-at-failure at Layer N. Each layer is a `:test::lru-*` named helper + a sibling `(:wat::test::deftest ...)` referencing it. |
| 4 | Per-helper deftest discipline | Every named helper has its OWN deftest. Verifiable: count of `(:wat::core::define (:test::lru-*` defines = count of `(:wat::test::deftest :wat-lru::test-lru-*` deftests. (Spell's Level 2 mumble check.) |
| 5 | Top-down dependency graph | No forward references. Verifiable: for each helper at line N, all helpers it calls (matching `:test::lru-*`) are defined at lines < N. (Spell's Level 1 lie check.) |
| 6 | Body line budget | Each helper's body: single outer `let*`, 3-7 bindings. Each deftest's body: 3-7 lines. Spot-check: no helper body > 12 lines; no deftest body > 8 lines. (Spell's "deftest body 3-7 lines" rule.) |
| 7 | Time-limit discipline | Either each deftest carries `(:wat::test::time-limit "200ms")`, OR none do (relying on arc 132's default). Consistent across the file. |
| 8 | **`cargo test --release -p wat-lru`** | Exit=0 if Mode A (all layers pass) OR exit=non-zero if Mode B (stopped at first red). EITHER outcome is acceptable. The TOTAL test count in the file matches the layer count (3 LocalCache + 4 HolonKey from the other test files + N layers from this file). No `should panic` markers. |
| 9 | Layer-pass discipline | Sonnet ran `cargo test` AFTER EACH LAYER was added (per the brief's Step 3 workflow). Verifiable in the report's pass/fail roll-up — sonnet should describe at least one intermediate run, not just the final. |
| 10 | Stop-at-first-red | If a layer fails, sonnet STOPS adding layers. Verifiable: the failing layer is the LAST defined layer; no layers attempt to use the broken behavior. |
| 11 | **No grinding** | Sonnet does NOT modify the substrate to make a failing layer pass. Sonnet does NOT iterate on a single failure beyond reporting it. The reland is a measurement, not a fix. |
| 12 | Honest report | 250-word report includes: layer-by-layer pass/fail roll-up; cargo test totals; failing layer's mechanics (if any) + hypothesis on broken substrate behavior; the four-questions verdict on the test file YOU WROTE; file LOC + per-layer LOC. |

**Hard verdict:** all 12 rows must hold. Rows 4 + 5 + 6 are
the complectens-discipline rows — load-bearing for proving the
spell teaches. Row 11 is load-bearing for the failure-engineering
discipline (failure is data, not a thing to fix in-flight).

## Soft scorecard (5 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 13 | LOC budget | Total file LOC: 100-250 (per-layer 15-30 LOC including helper + deftest, times ~7 layers, plus a 20-30 LOC header comment block). >300 LOC = re-evaluate. |
| 14 | Header comment block | File starts with a short comment block naming the discipline (one paragraph) + the layered map (one line per layer). Mirrors step-A-spawn-shutdown.wat's header style. |
| 15 | Final scenario layer | If Mode A (all layers pass): the final layer is named `:test::lru-helper-put-then-get` (or equivalent) and its deftest body is 3-5 lines that compose Layer 5 + Layer 6's helpers + an assertion. The deftest at the bottom of the file IS the final scenario per the spell's "what happens" framing. |
| 16 | Diagnostic clarity (Mode B) | If Mode B (stopped at first red): the failing layer's name is OBVIOUS about what it tests. A reader sees "lru-raw-send-raw-recv FAILED" and knows the bug is in the raw-substrate channel cycle (not in helper verbs, not in pool mechanics, not in spawn). |
| 17 | Honest deltas | Sonnet flags any layer where the substrate's actual shape diverges from the brief's prediction. E.g., if `spawn 1 1` doesn't return the expected Tuple shape, sonnet adapts the brief's suggested layer + reports the divergence. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~45%) — Mode B at Layer 4:** the suspected
  substrate bug surfaces at the raw-send-raw-recv layer.
  Layer 4 fails with "reply channel closed" or similar.
  Layers 5..7 not written. The diagnostic is clean — the bug
  lives in the driver's select+remove-on-disconnect logic at
  loop-step lines 387-391. We open a follow-on arc to fix it.

- **Second-most-likely (~25%) — Mode B at Layer 3:** the
  bug surfaces earlier. Maybe send on a popped slot's req-tx
  doesn't reach the driver because the pool's internal channel
  shape interferes. Layer 3 fails. Diagnostic still clean.

- **Mode A (all layers pass) (~20%):** the substrate is
  intact; the prior sweep's failure was a test-shape issue
  (the helpers' inner-let* + finish + put ordering) that
  doesn't surface when each piece is tested in isolation.
  We learn the prior sweep's substrate reshape is sound; the
  test file rebuild is the slice 1 deliverable. Slice 2
  proceeds.

- **Mode B at Layer 0/1/2 (~5%):** the bug is in spawn or
  pool mechanics themselves. Surprising; would suggest the
  reshape broke the lifecycle. Open follow-on arc.

- **Sonnet violates complectens (~5%):** sonnet writes a
  monolithic deftest, or skips per-helper deftests, or
  forward-refs. Hard rows 4-6 fail. Reland with sharper
  brief (point at SKILL § severity-levels more emphatically).

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` → expect 1 file modified.
4. Read the rewritten test file from top to bottom — verify
   layer ordering, no forward refs, helper+deftest pairing,
   body line counts.
5. Run `cargo test --release -p wat-lru` locally → confirm
   sonnet's reported test totals.
6. If Mode A: verify all layers pass; the file is the worked
   demonstration; close slice 1 cleanly.
7. If Mode B: verify the failing layer's diagnostic is clean;
   open a follow-on arc for the substrate fix; mark slice 1
   as "discipline confirmed; substrate gap surfaced at Layer
   N; substrate fix in arc <next>".
8. Score; commit `SCORE-SLICE-1-RELAND.md` as a sibling.

## Why this reland matters for the chain

The prior sweep proved that sonnet, given a comprehensive
substrate-reshape brief, can ship a substrate reshape AND a
complectens-violating test file in the same sweep. That's
not the artifacts-as-teaching discipline working — that's
sonnet pattern-matching the prior file's structure WITHOUT
internalizing the spell.

This reland tests something stricter: given the spell + the
worked examples + a brief that says "WIPE the existing file
and BUILD bottom-up from empty," can sonnet construct a
test file that holds the discipline AND surfaces the
substrate's actual behavior at each layer?

If yes, the spell teaches. If yes AND substrate is intact
(Mode A), the failure-engineering chain produces a doubly-
clean ship. If yes AND substrate has a gap (Mode B), the
chain produces a clean diagnostic + a follow-on arc.

Both are wins. The reland is the calibration that proves
the discipline propagates to fresh agents EVEN against
inertia from prior file structures.

## What we learn

- **Mode A + clean discipline:** spell teaches; substrate
  reshape is sound. Slice 2 proceeds. The complectens
  spell + the worked examples + the bottom-up-stepping-stones
  brief is the canonical pattern for future substrate sweeps.

- **Mode B + clean discipline:** spell teaches; substrate
  has a real bug at Layer N. Open arc to fix it. Slice 1
  re-runs after fix.

- **Mode A + complectens violation:** substrate works but
  spell didn't propagate fully. Sharpen brief; reland
  again. Note which violations slipped through (specifies
  what to add to the spell or the brief for the next sweep).

- **Mode B + complectens violation:** the worst case;
  substrate broken AND spell didn't propagate. Reland with
  the sharpest brief possible; if it still fails, the spell
  needs structural revision.
