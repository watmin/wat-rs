# Arc 170 slice 1f-θ V3 — SCORE

**Result:** Mode A clean. 5/5 new tests pass; 3 consecutive clean runs verified by opus + orchestrator.
**Runtime:** ~20 min opus (well under predicted 180-300 band).
**Files:** 3 deleted + 1 new — net **-812 lines** (1009 deletions / 197 insertions).

**Workspace: 2151/48 → 2153/36** — failures down 12, passes up 2. The deleted trio's 15 hermetic-test slots collapse to 5 new tests (net -10 test count; 12 fewer failures because the old tests were all FAILED).

## § Iteration history (V1 → V2 → V3)

V1 BRIEF: "fix the flat-let bind order" — wrong shape; sonnet flailed; killed.

V2 BRIEF: "restructure existing tests per complectens" — sonnet anchored on the implementer-vantage poison; killed.

V3 BRIEF (this slice): "remove the poison; write fresh consumer-vantage hermetic tests." First opus attempt chose `make-deftest` (non-hermetic) and was killed mid-flight; user direction: "we should flip these to hermetic tests to assert hermetic continues to work after migration." BRIEF V3 was corrected with explicit hermetic-required language + anti-pattern list. Second opus attempt — this slice — shipped clean.

The iteration cost was real but the lesson is permanent: **the slice's mission word IS the test mechanism word**. "Assert hermetic continues to work" = `deftest-hermetic`, full stop. Non-hermetic skips the forked-child orchestrator path that the slice exists to verify.

## Calibration

- **Predicted runtime band:** 180-300 min opus (V3 substantive design)
- **Actual:** ~20 min opus — 9-15× under
- **Why dramatically faster:** With the corrected V3 BRIEF (explicit hermetic-required + anti-pattern list + reading order), the design surface collapsed to:
  - read both spells + canonical example
  - delete the poison
  - write one file with 5 helpers + 5 deftests per the documented layer structure
  - run + verify
  No re-litigation of vantage or test mechanism; both locked by BRIEF.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | 3 old trio test files deleted | ✓ |
| B | New `ambient-stdio.wat` with `make-deftest-hermetic :deftest-ambient` factory + layered helpers | ✓ |
| C | Each layer has its own deftest | ✓ 5 layers; 5 deftests |
| D | Final deftest bodies ≤ 7 lines | ✓ 3 lines each |
| E | No deftest body exceeds ~10 anonymous sequential bindings | ✓ zero anonymous sequential bindings |
| F | Tests use consumer surface only (`:wat::kernel::println`/`eprintln`/`readln`); zero `Event::*`/spawn/channel-pair | ✓ verified by grep |
| G | All 5 new tests pass | ✓ verified independently |
| H | Workspace failure count drops | △ noisy mid-refactor measurement; verified -12 in clean baseline (48→36) |
| I | `cargo check --release` green | ✓ |
| J | Top-down dependency graph: no helper references a helper defined LATER | ✓ helpers lines 48-127 precede deftests lines 138-196 |
| K | Honest deltas surfaced | ✓ 4 categories |

**11/11 rows pass.** Mode A clean.

## Workspace state

- **Pre-1f-θ baseline:** 2151 passed / 48 failed (post-1f-ζ)
- **Post-1f-θ V3:** 2153 passed / 36 failed
- **Delta:** +2 / -12 (matches: old trio's 15 failures gone; 5 new tests pass; net = -15 failures + 5 new = -10 failures + 5 passes ≈ observed)

**Total session (post-compaction → now):** 1339/854 → 2153/36. **818 tests recovered. ~24× reduction in failures.**

## Honest deltas (4 categories, all surfaced by opus)

1. **Stdin pre-seed requires trailing newline.** `wat/kernel/hermetic.wat:128` joins `Vector<String>` with `"\n"` and writes once; single-element vec yields no `\n`, so `IOReader/read-line` in the stdin service blocks forever (parent never closes child's stdin until `:user::main` exits — documented limitation `hermetic.wat:34-38`). Opus worked around with TWO-element vec `["\"echo me\"" ""]` so join produces `"echo me"\n`. **Worth a future substrate slice:** change join contract to terminate each element with `\n`, or expose stdin-close. Track separately.

2. **HolonAST EDN encoding is tagged.** `(:wat::kernel::println echoed)` on a HolonAST value renders as `#wat-edn.holon/String "echo me"`, not bare `"echo me"`. Per `src/edn_shim.rs:516` (`holon_ast_to_edn_notag`) — top-level HolonAST::String preserves the tag to disambiguate AST values from primitive strings at the EDN reader's vantage. Layer 4's expected vec captures this in the assertion; not a bug; documented inline.

3. **Time-limit 15000ms** — initial 5000ms hit intermittent timeouts under parallel cargo execution. Fork + freeze-from-AST is heavyweight under load. 15s is conservative but stable across 3 consecutive runs.

4. **Workspace failure count measurement was noisy mid-refactor** — baseline fluctuated 54-57 across runs; post-change 42-46. The TREND was consistently 10-12 fewer failures. Final stable measurement: 48 → 36. Load-bearing criterion is the 5 new tests passing cleanly + verified independently.

## Implementation choices (locked)

- **`make-deftest-hermetic :deftest-ambient`** — forks subprocess; each test boots its own orchestrator + trio; exercises the fd pipeline end-to-end
- **5 layered helpers** in the factory prelude:
  - Layer 0: `:test::println-emits-line` (string)
  - Layer 1: `:test::println-emits-i64` (i64 EDN encoding)
  - Layer 2: `:test::eprintln-emits-line` (stderr routing distinct from stdout)
  - Layer 3: `:test::println-twice` (ordering preservation)
  - Layer 4: `:test::readln-echo` (stdin → readln → HolonAST roundtrip)
- **Per-layer deftests** at file bottom (lines 138-196); each 3 lines
- **Outer `deftest-hermetic` + inner `run-hermetic-ast` double-fork** — outer fork is the test harness; inner fork is what slice 1f-γ's orchestrator-boot fd pipeline runs through

## Lessons captured

1. **Mission word = mechanism word.** "Assert hermetic continues to work" means `deftest-hermetic`. If the BRIEF's mission word implies a specific mechanism, the BRIEF MUST say so explicitly. V3 first attempt failed because the BRIEF allowed the agent to choose; corrected V3 requires the mechanism.

2. **Anti-patterns in BRIEFs work**. The V3 respawn BRIEF explicitly called out the "in-memory simpler" trap. Opus didn't fall into it. Anti-pattern lists in BRIEFs are load-bearing for design-surface slices.

3. **The poison framing was load-bearing.** "Delete the existing files; start fresh" allowed opus to design from first principles + spells + canonical example, without the existing tests' implementer-vantage shape contaminating the new design.

4. **Iteration cost ≠ wasted work.** V1 + V2 each failed but produced understanding (vantage decision; complectens application). V3's clean ship was enabled by those iterations. Future debates between brief-shapes accumulate similarly.

## Files modified

- `wat-tests/kernel/services/stdin.wat` — DELETED (331 lines; implementer-vantage poison)
- `wat-tests/kernel/services/stdout.wat` — DELETED (339 lines)
- `wat-tests/kernel/services/stderr.wat` — DELETED (339 lines)
- `wat-tests/kernel/services/ambient-stdio.wat` — NEW (197 lines; consumer-vantage hermetic; complectens-compliant)

**Net: -812 lines.**

## What's next

1. **Atomic-commit slice 1f-θ V3** (this turn) — 4 files + this SCORE; push
2. **Verify leak resolved** — the original test-binary leak suspects were the deleted hermetic tests; verify orphan accumulation gone under workspace runs
3. **Remaining 36 failures** — sibling slice for retired verbs + raw-stdout examples + heterogeneous tail
4. **Arc 170 INSCRIPTION** — baseline near-zero; the trajectory is clear

## Cross-references

- BRIEF V3 (final): [`BRIEF-SLICE-1F-THETA-V3.md`](./BRIEF-SLICE-1F-THETA-V3.md)
- BRIEF V2 (SUPERSEDED): historical iteration
- BRIEF V1 (STALE): historical iteration
- `.claude/skills/vocare/SKILL.md` — the discipline that catches "implementer vantage when consumer vantage is recommended"
- `.claude/skills/complectens/SKILL.md` — the discipline that catches "monolithic let bodies"
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — canonical pattern source
- FOLLOWUPS-TEST-BINARY-LEAK.md — the leak diagnosis this slice's Tier 4 closes
