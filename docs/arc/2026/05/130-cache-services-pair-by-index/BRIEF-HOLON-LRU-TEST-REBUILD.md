# Arc 130 — HolonLRU Test Rebuild BRIEF (sweep 2b)

**Drafted 2026-05-06.** Sweep 2b of arc 130's HolonLRU cleanup.
Sweep 2a (substrate reshape) shipped Mode A clean ~10 min ago;
substrate is in working-tree-dirty state. Now: rebuild the test
files to use the new substrate shape + retire the 9 LRU-family
`:should-panic` annotations.

User direction 2026-05-06: "lets get holon-lru cleaned up." Per
`feedback_no_broken_commits.md` discipline, sweep 2a + sweep 2b
commit ATOMICALLY together when this sweep ships clean. Working
tree stays dirty between sweeps; orchestrator commits both at
end.

## The post-sweep-2a workspace state

- `wat-holon-lru` substrate (`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`)
  is in NEW shape: `Handle = (ReqTx, ReplyRx)`, `DriverPair = (ReqRx, ReplyTx)`,
  unified `Reply` enum (variants `GetResult` + `PutAck`),
  pair-by-index spawn, helper verbs take `Handle` (1 arg) instead
  of 3 channel ends.
- Workspace test profile: 16 wat-holon-lru tests fail with
  TYPE-MISMATCH errors at consumer call sites (expected by
  brief 2a). Otherwise 0 failed across workspace.
- Sweep 2a's diff is in working tree, uncommitted.

## Goal

Rebuild THREE test files using complectens + the new substrate
shape, retiring the 9 LRU `:should-panic` annotations. After this
sweep: workspace = 0 failed; working tree has substrate (2a) +
test files (2b) ready for atomic commit.

## Files in scope

1. **Main**: `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   (731 LOC, 14 deftests, 19 helper-verb call sites incl. 9
   `:should-panic("channel-pair-deadlock")` annotations to retire).
   **WIPE + rebuild bottom-up complectens-style.**
2. **Proof archetype**: `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat`
   (46 LOC, 0 :should-panic, fails post-reshape with type errors).
   **UPDATE in place to use new substrate shape.** This proof is
   referenced by sweep 1's BRIEF-TEST-FILE-REBUILD as Layer 0's
   archetype; preserve its educational shape, just adapt to new
   typealiases + helper-verb signatures.
3. **Proof step-B**: `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`
   (68 LOC, 1 `:should-panic("channel-pair-deadlock")` to retire,
   4 helper-verb call sites). **UPDATE in place** to use new
   substrate shape AND retire the `:should-panic`. Same
   educational-shape preservation.

## Pre-flight crawl (mandatory before editing)

1. **`crates/wat-lru/wat-tests/lru/CacheService.wat`** — your
   TEMPLATE for the main test file rebuild. Read it fully (~325
   LOC). Pay attention to:
   - The `(:wat::test::make-deftest :deftest-lru ...)` factory
     pattern (sweep 1's Adaptation 1 — sandbox-scope leak workaround)
   - The tuple-out pattern: `(driver, value)` returned from inner
     let* so spawn/pool/handle drop before outer Thread/join-result
     (sweep 1's Adaptation 2 — arc 117/126 scope-deadlock check)
   - The 5-layer plan: spawn-and-drop, helper-get-empty,
     helper-put-one, helper-put-then-get, helper-get-many-keys
   - Helper sub-extraction (Layer 3a, 4a, 4b) per Level-3-taste
     SKILL exemption
2. **`docs/arc/2026/05/130-cache-services-pair-by-index/SCORE-TEST-FILE-REBUILD.md`**
   — sweep 1's SCORE. Read "What sonnet's adaptations teach" in full.
3. **`crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`**
   — the NEW substrate (post-sweep-2a). Read the typealiases section
   + helper verb signatures + spawn return shape.
4. **`.claude/skills/complectens/SKILL.md`** — THE SPELL.
5. **`crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`**
   — the OLD test file you'll wipe. Note which scenarios it covers
   so the rebuilt file preserves equivalent coverage.
6. **`crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat`**
   + **`step-B-single-put.wat`** — the proofs to update in place.

## What to do

### Step 1 — Rebuild the main test file (complectens, bottom-up)

Wipe `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
and rebuild from empty using the wat-lru template's structure
adapted for HolonLRU:

- Factory prelude pattern: `(:wat::test::make-deftest :deftest-hcs
  ...)` (or similar; mirror wat-lru's `:deftest-lru`)
- Helper-verb primary interface: `:wat::holon::lru::HologramCacheService::get`
  / `put` (post-sweep-2a; take Handle + payload; do send-AND-recv
  internally per arc 110)
- Concrete typing: K=V=HolonAST throughout

**Suggested layer plan** (mirror wat-lru's, refine for HolonLRU):

- **Layer 0** — `:test::hcs-spawn-and-drop`. spawn → pop → finish
  → join lifecycle. No requests.
- **Layer 1** — `:test::hcs-helper-get-empty`. Single `:wat::holon::lru::HologramCacheService::get`
  on empty probes. Returns empty `Vec<Option<HolonAST>>`.
- **Layer 2** — `:test::hcs-helper-put-one`. Single
  `:wat::holon::lru::HologramCacheService::put` with one Entry.
- **Layer 3** — `:test::hcs-helper-put-then-get`. Composes Layers
  1+2: put one entry, get the same key, assert the result vec
  contains `(Some(value))`.
- **Layer 4** — `:test::hcs-helper-get-many-keys`. Multi-key
  probe alignment; presence pattern.
- **Optional Layer 5** — `:test::hcs-eviction`. cap=2; put 3
  distinct keys; first key should evict.
- **Optional Layer 6** — `:test::hcs-multi-client`. Spawn with
  count > 1; multiple handles operate on same cache (mirrors
  the old `test-hcs-spawn-2clients-put-get-verify` scenario).

Each layer = one helper + one deftest. Helper body: ONE outer
let*, 3-7 bindings. Deftest body: 3-7 lines composing the helper.
Top-down dependency graph; no forward references. Sub-helper
extraction (Level-3 taste) acceptable when used in exactly one
place per the SKILL's edge-case guidance.

NO `:should-panic` annotations. NO `:ignore` annotations. The
substrate's new shape (post-sweep-2a) means the deadlock pattern
is no longer required at the call site → arc 117/126's check
doesn't fire → tests pass naturally without panic crutches.

Run `cargo test --release -p wat-holon-lru --test test` after EACH
layer; STOP at first red.

### Step 2 — Update the proof files in place

Both proofs in `crates/wat-holon-lru/wat-tests/proofs/arc-119/`
are small (46 + 68 LOC) and currently broken with type errors.
Update mechanically:

- **step-A-spawn-shutdown.wat**: replace the OLD substrate
  references (e.g., direct ReqTxPool / per-call channel allocation)
  with NEW shape (HandlePool<Handle>, pop a handle, finish, join).
  The proof's TEACHING shape (single deftest, narrowest possible
  spawn-shutdown lifecycle test) is preserved.

- **step-B-single-put.wat**: same shape adaptation +
  RETIRE the `:should-panic("channel-pair-deadlock")` annotation
  at line 23. The new helper verb signature does send-AND-recv
  internally; the deadlock pattern is gone; the test passes
  naturally.

Pattern: read each proof in full, identify substrate references
to update, make minimal mechanical edits, run cargo test to
verify the proof still passes.

### Step 3 — Verification

After all three files updated, run:

```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail -10
```

EXPECTED: 0 failed across workspace. The wat-holon-lru crate's
26 → 16-fail intermediate state returns to all-passing (with the
9 :should-panic crutches retired and replaced by naturally-
passing tests).

If any test still fails, STOP and surface the failure mode. Do
NOT modify the substrate. Do NOT add :should-panic / :ignore
crutches.

## Constraints

- **Test-file-only edits.** THREE files modified (main + 2 proofs).
  NO substrate edits (sweep 2a's substrate is in working tree
  uncommitted; leave it alone). NO Rust source edits. NO other
  crates.
- **Retire all 9 LRU `:should-panic` annotations.** 8 in the main
  rebuilt file (don't re-add them) + 1 in step-B-single-put.wat
  (delete the existing line 23 annotation).
- **No `:ignore` annotations** added as a substitute. Tests should
  pass naturally post-rebuild.
- **STOP at first red after each layer.** Don't grind. The wat-lru
  rebuild's two adaptations (factory prelude + tuple-out) are
  documented; expect to use both.
- **No commits, no pushes.** Working tree stays dirty (substrate
  + tests); orchestrator commits sweep 2a + sweep 2b atomically
  AFTER this sweep verifies 0-failed workspace.
- **Substrate is OFF LIMITS.** If the substrate appears broken,
  the failing layer surfaces it; don't modify
  `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`.

## Out of scope

- Slice 1 / arc 130 INSCRIPTION (orchestrator paperwork after
  this commit)
- Any consumer code in lab repos
- Substrate further changes
- Adding new tests beyond the layer plan

## Reporting

Target ~250 words:

1. **Pre-flight crawl confirmation:** wat-lru template + sweep 1
   SCORE + new substrate + OLD test file + 2 proofs read.

2. **Main test file rebuild summary:**
   - Layer-by-layer pass/fail roll-up
   - Total deftests + helpers + sub-helpers
   - Any HolonLRU-specific layers added (eviction, multi-client)
   - LOC delta vs prior 731
3. **Proof file updates:**
   - step-A: changes made (substrate shape adaptation)
   - step-B: changes made + :should-panic retirement
   - Both passing post-update

4. **Workspace verification:** `cargo test --release --workspace`
   final result. EXPECT 0-failed.

5. **Path:** Mode A clean (rebuild ships; workspace 0-failed) /
   Mode B (substrate gap surfaces at Layer N) / Mode C (complectens
   violation in test file shape).

6. **Honest deltas:** any HolonLRU-specific divergence from the
   wat-lru template; sub-helper Level-3-taste extractions; any
   layer-plan refinements.

## What success looks like

**Mode A clean ship:** All three files updated; workspace = 0
failed; the 9 LRU :should-panic annotations retired; the
substrate-as-teacher cascade closes another link (arc 130 slice
1 + sweep 2a + sweep 2b together prove the substrate works
end-to-end on both LRU services).

**Mode B:** A layer fails; substrate has a bug surfaced cleanly;
open follow-on arc.

**Mode C:** complectens violations in the test file shape;
reland with sharper brief.

## Why this brief matters for the cooperation

User direction 2026-05-06: "lets get holon-lru cleaned up." The
substrate-side ships in sweep 2a; this brief is the test side.
Together they prove arc 130's premise: when the substrate's
pair-by-index discipline propagates correctly, the deadlock-
pattern tests retire because the deadlock pattern itself is gone.

The mutual-agreement chain:
- User → Orchestrator: cleanup direction
- Orchestrator → Sonnet (this brief): rebuild tests against new
  substrate; retire :should-panic; mirror wat-lru's pattern
- Sonnet → Reality: workspace 0-failed; :should-panic crutches gone

If sonnet ships Mode A clean, sweep 2a + sweep 2b commit
atomically as one ship. The HolonLRU cleanup completes.
