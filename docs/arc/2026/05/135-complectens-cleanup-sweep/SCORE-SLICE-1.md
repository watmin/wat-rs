# Arc 135 Slice 1 — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.

**Agent ID:** `ad6ee237419d03e9e`
**Runtime:** ~16 min (980s).

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Two-file diff | **PASS** | `git status --short`: only `wat-tests/service-template.wat` + `wat-tests/console.wat` modified. |
| 2 | Helpers added | **PASS** | 4 helpers in service-template + 4 helpers in console = 8 total. In predicted 6-20 band. |
| 3 | Each existing deftest body shrinks | **PARTIAL** | service-template: 106→1 ✓ (99% shrink — flagship). Console hello-world: 42→37 (12%). Console multi-writer: 101→81 (20%). Console tests fall short of the 3-7 line target — sonnet's Delta 3 names why: hermetic-program embedded ASTs are inherently irreducible; the OUTER LOGICAL BINDING count shrunk 8→5, but the visual line count is dominated by the embedded program literal. This is a SKILL edge case, not a sonnet failure. |
| 4 | Per-helper deftests added | **PASS** | 9 new per-helper deftests (4 service-template + 5 console). Each helper has exactly one deftest proving it. All clean pass (no channel-pair patterns in either file's prelude, sonnet correctly identified). |
| 5 | No forward references | **PASS** | Both files top-down. Prelude → deftests for helpers → final scenario deftests. No backward jumps. |
| 6 | **Outcomes preserved** | **PASS** | `cargo test --release --workspace` exit=0. 100 result blocks all `ok`. Existing `:svc::test-template-end-to-end`, `Console::test-hello-world`, `Console::test-multi-writer` all pass cleanly (unchanged outcomes). 9 new helper deftests pass cleanly (consistent with helper-pattern). |
| 7 | No commits | **PASS** | `git status` confirms uncommitted modifications. |
| 8 | Honest report | **PASS+** | ~600 words. Three load-bearing deltas surfaced (service Thread/output drain; arc-126-at-call-sites; hermetic-program embedded-AST irreducibility). Each delta is precisely diagnosed with the exact mechanism + reproduction conditions. |

**HARD VERDICT: 7 OF 8 PASS, 1 PARTIAL.** Row 3 is the partial — and the partial is itself the calibration signal. Sonnet's Delta 3 names exactly the SKILL gap that allowed it.

## Soft scorecard (4 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 9 | Helper count | **PASS** | 8 helpers total. In 6-20 band. |
| 10 | Average body shrink | **PARTIAL** | service-template: 99%; console hello-world: 12%; console multi-writer: 20%. Average ~44%; below the 60% target. **However**, sonnet's Delta 3 explains: the visual line count for hermetic-program tests is dominated by the embedded program AST. The outer logical binding count for multi-writer shrunk 8→5 (37%) — itself in band. The metric needs refinement for hermetic-program tests. |
| 11 | Workspace runtime | **PASS** | `cargo test --release --workspace` exit=0; runtime within baseline. |
| 12 | Edge-case usage | **PASS** | Sonnet correctly applied existing SKILL edge cases. Did NOT factor `make-bounded-channel` into helpers. Did NOT use two-prelude split (correctly judged that neither file has mixed-outcome deftests in the SHIPPED form — both files' deftests pass cleanly). Pop-before-finish on lifecycle helper. |

**SOFT VERDICT: 3 OF 4 PASS, 1 PARTIAL.** Same pattern as hard row 3 — body-line shrink doesn't apply cleanly to hermetic-program tests.

## Calibration insights — three new SKILL deltas

### Delta 1 — Service output channel requires recv-before-join (Thread<I, O> with non-unit O)

**Discovery:** the lifecycle helper for `Thread<unit, State>` (where the driver SENDS final state on the output channel before returning) must `recv` from `Thread/output` before calling `Thread/join-result`. If the receiver is dropped before the send, the driver's `expect` panics with "out disconnected." This is specific to services whose driver output is non-unit.

**Why it's a gap:** the existing SKILL pop-before-finish edge case covers HandlePool, but doesn't address the Thread/output case. LRU services use `Thread<unit, unit>` (no payload on output) — the existing examples never hit this.

**Refinement:** add a SKILL note — "When the spawned function's output type is non-unit, the lifecycle helper must drain Thread/output before Thread/join-result."

### Delta 2 — Arc 126 fires on CALL SITES, not just function definitions

**Discovery:** factoring helpers that TAKE both tx and rx of a channel pair as parameters (e.g., `(:test::send-ack-wait req-tx ack-tx ack-rx)`) DOES fire arc 126 at the CALL SITE in the prelude body, even though the helper's body itself only references the parameters. Arc 126 traces from arguments, not just from `make-bounded-channel`.

**Why it's a gap:** the existing edge case warns against factoring `make-bounded-channel` allocations into helpers. It implies extracting the WHOLE workload (allocate + call + drop) into one helper is safe. But CALLING a helper that takes both halves still trips the check at the call site.

**Refinement:** the SKILL's existing edge case note needs extension — "Do not factor any helper-call site that passes both halves of a channel pair into a function. Keep `(:wat::kernel::send req-tx ...)` and `(:wat::kernel::recv ack-rx)` as SEPARATE inline calls; never wrap them in a helper that takes both as parameters."

### Delta 3 — Hermetic-program tests have inherently irreducible bodies

**Discovery:** when a deftest body uses `(:wat::test::run-hermetic-ast (:wat::test::program ...))`, the embedded program AST is a literal that runs in a forked subprocess. The subprocess can't reference the outer prelude's helpers (separate freeze). So the embedded program body must be self-contained — its AST literal is part of the deftest body's visual line count.

**Result:** the `>30 lines = suspect; >50 = likely Level 1` heuristic over-flags hermetic tests. The OUTER LOGICAL BINDINGS shrink with composition; the embedded program stays as-is.

**Refinement:** add a SKILL note — "When a deftest embeds a `(:wat::test::program ...)` literal, count OUTER logical bindings (post-`run-hermetic-ast`-result), not total visual lines. Hermetic program bodies are inherently irreducible."

## Independent prediction calibration

The orchestrator predicted (in `EXPECTATIONS-SLICE-1.md`):

- **65% all-pass** ← partial fire — 7/8 hard + 3/4 soft is close to all-pass but row 3 partial.
- 20% two-prelude split needed — sonnet correctly determined NOT needed.
- **10% new edge case surfaces** ← FIRED. Three edge cases surfaced (Deltas 1, 2, 3). Document refinements queued.
- 3% per-helper deftest gap — didn't fire (all helpers got their deftests).
- 2% outcome regression — didn't fire (all outcomes preserved).

**Actual outcome:** the 65% AND the 10% paths BOTH fired. Sonnet shipped clean (no commits needed for backout) AND surfaced three load-bearing deltas. This is the artifacts-as-teaching record working as designed.

## Ship decision

**SHIP** the slice 1 deliverable. Hard row 3 partial is acknowledged via Delta 3; refining the SKILL is the next move BEFORE slice 2 (so slice 2's sonnet sees the updated edge cases).

The console tests' visual line counts are larger than the 3-7 target, but their OUTER logical structure (post-rewrite) is correct. The body-line metric is the proxy; phase-2 judgment exempts hermetic-program tests from a strict reading of the proxy.

## Next steps

1. Refine the SKILL with Deltas 1, 2, 3 baked into the "Edge cases" section.
2. Commit slice 1 deliverable + this SCORE + SKILL refinements together.
3. Update `arc-130/FOLLOWUPS.md` to mark service-template + console as ✓.
4. Spawn slice 2 with the refined SKILL flowing forward.
