# Arc 201 Slice 4 EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4.md`
**Drafted:** 2026-05-16, pre-spawn (before sonnet dispatched).

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- ~150 mechanical edits across ~18 files
- Substrate Rust: handful of sites in runtime.rs + check.rs + freeze.rs + stdlib.rs (~15-25 edits)
- wat/runtime.wat: 1-3 edits (define-alias macro)
- 13 test files: ~70-100 edits (mix of literal calls + Rust identifier renames)
- 3 active docs: ~10-15 prose edits
- Build cycle + cargo test (full workspace) ~5 min
- Per-file substring-preservation paranoia (must not corrupt `signature-of-fn`) — adds ~10 min careful-grep time
- No design judgment beyond mechanical sweep + STOP-trigger 6 (alias temptation)

Compared to slice 1 (signature-builder + 5 emission sites + tests — substantive shape change): slice 4 is wider but shallower. Compared to slice 3 (one new verb + 8 tests): slice 4 is more sites but no implementation logic.

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — `signature-of` GONE; `signature-of-defn` REGISTERED | YES | high (mechanical; STOP-trigger 1 covers stragglers) |
| B — Internal Rust identifiers renamed; slice 3 sibling preserved | YES | high (STOP-trigger 3 explicitly catches substring corruption) |
| C — wat/runtime.wat define-alias uses new name; tests pass | YES | high (single macro consumer; well-bounded) |
| D — 13 test files swept; all pass | YES | medium-high (mechanical, but cross-test substring discipline matters) |
| E — Workspace failure count ≤ baseline (3) | YES | high (purely additive surface change; behavior identical to pre-rename) |

**5/5 PASS predicted; ~80% confidence overall.**

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **Hidden consumer.** Orchestrator's grep missed a site (probably in a comment or doc). STOP-trigger 1 catches; sonnet reports site + sweep verdict. Most likely candidate: a USER-GUIDE.md prose section or a comment in a file orchestrator didn't grep deeply.

2. **Substring near-misses.** Tests for `signature-of-fn` may have docstrings referencing `signature-of` as the named-callable counterpart — sonnet must update where the reference IS to THIS primitive but PRESERVE where the discussion is about the slice-3 sibling's contrast.

3. **`signature_of_*` test fixture names.** Files like `tests/wat_arc146_dispatch_mechanism.rs` have test functions named like `signature_of_dispatch_returns_declaration`. Rename to `signature_of_defn_dispatch_returns_declaration` — but this also affects any `cargo test <name>` invocations in docs/comments. Sonnet captures rename count + collateral.

4. **MODULARIZATION-NOTES.md identifier list.** Line 75: `eval_lookup_define, eval_signature_of, eval_body_of` — the identifier `eval_signature_of` updates to `eval_signature_of_defn`. Verify the surrounding context still reads true after rename.

5. **Sliced run time spread.** Prediction is 60-90; if substring discipline forces per-file careful sweep mode, could hit 90+. If sonnet uses a global-rename tool (Edit replace_all per file with care), faster. Range honest.

### Less likely surprises

6. **A test BEHAVIOR depends on the verb spelling.** Unlikely (the verb's behavior is independent of its name) but possible if a test calls some test infrastructure that strings the verb name. STOP-trigger 2 escalates.

7. **A check.rs comment reference is ambiguous.** Some comments at the cited lines may reference `signature-of` as a CLASS of primitive (the reflection family) where the rename is awkward. Sonnet judges — update if it's the spelling, preserve if it's the family concept (rare case; flag in SCORE).

8. **Site count drift.** Estimated ~150 edits — actual could be 100-200. Not a quality signal; sonnet reports honest count.

## Workspace baseline (commit `9105e17`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: **1679 passed / 3 failed**

**Pre-existing failures (DO NOT BLOCK slice 4):**
1. `lifeline_pipe_zero_orphans_across_100_trials` — FD-multiplex flake variance (slice 3 SCORE noted)
2. `deftest_wat_tests_tmp_totally_bogus` — unrelated wat-test fixture w/ unresolved reference
3. `t6_spawn_process_factory_with_capture_round_trips` — arc 170 Slice 6 documented preservation (closure-capture-across-fork; substrate-equivalent path doesn't exist yet)

Post-slice-4 target:
- Pass count ≥ 1679 (purely additive rename; nothing should regress)
- Fail count ≤ 3 (no new failures introduced)
- Lifeline flake variance acceptable (the test may flap to passing)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | ≤ 3 | TBD | TBD |
| Site count | ~150 edits / ~18 files | TBD | TBD |
| STOP-triggers fired | 0-1 (likely 1: hidden consumer) | TBD | TBD |
| Substring near-misses | 0 (STOP-trigger 3 catches) | TBD | TBD |
| Active-doc prose edits beyond mechanical replace | 0-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
