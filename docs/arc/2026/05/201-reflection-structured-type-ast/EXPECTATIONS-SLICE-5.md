# Arc 201 Slice 5 EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-5.md`
**Drafted:** 2026-05-16, pre-spawn.

## Independent prediction

**Runtime band:** 30-60 min sonnet.

Reasoning:
- Mirror of an existing handler (`eval_extract_arg_names`); same walker, one-character difference (pair[0] → pair[1])
- New eval handler: ~50-80 LOC (full mirror including docstring)
- Dispatch arm + type-scheme registration: ~10 LOC
- 4-5 unit tests: ~120-180 LOC
- Build cycle + targeted + workspace verify: ~5 min
- Smaller than slice 3 (which was a structurally novel verb with the fn-value vs fn-AST decision); comparable to slice 4 minus the consumer sweep

**Time-box:** 90 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — verb minted | YES | high (mechanical sibling of extract-arg-names) |
| B — Atom extraction for monomorphic args | YES | high (slice 1's emission rules deliver Atoms; the walker just reads them) |
| C — Bundle extraction for parametric args | YES | high (slice 1's emission rules deliver Bundles; the walker reads them) |
| D — Composes with Bundle/children for D2 chain | YES | high (composition is uniform; slice 3's signature-of-fn test already proved similar) |
| E — Workspace failure count ≤ baseline | YES | high (purely additive verb; no existing tests touched) |

**5/5 PASS predicted; ~88% confidence overall.** High confidence because the work is structurally mirror-an-existing-handler.

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **Walker abstraction decision.** Sonnet picks between: (a) factor out a shared walker fn that takes a "slot index" parameter (extract-arg-names uses index 0; extract-arg-types uses index 1); (b) keep two near-identical handlers per `feedback_simple_is_uniform_composition`. Either is honest; capture which + why.

2. **Variadic-rest handling.** If `eval_extract_arg_names` emits a special slot for `&` rest-binders (per `function_to_signature_ast`'s variadic emission logic at `src/runtime.rs:~9128-9145`), extract-arg-types should mirror — emit the rest type AST in the corresponding position. Sonnet checks + reports.

3. **Return-type lifting.** Wrapping `Vec<HolonAST>` as `Value::vec(...)` needs specific helpers. Likely mirrors how `Bundle/children` (slice 2) builds its Vector return. Honest delta on the exact helper used.

4. **`/gaze` on the name.** Working name `extract-arg-types`. Possible refinements: `extract-arg-types` reads cleanest (sibling of extract-arg-names); alternatives like `extract-param-types`, `arg-types`, `type-of-args` were not floated and probably don't add clarity. Sonnet either confirms or surfaces `/gaze` rationale.

5. **arc 057/143 surface check.** Per the recurring lesson (slices 2 + arc 199 reject), sonnet should grep first. SCORE should reference what was checked even if "nothing relevant" (proves the check happened). Specifically: does any existing primitive already do this? Search `extract-arg`, `arg-types`, `signature-types`, `param-types`. Expected: zero matches before this slice; this is genuinely additive.

### Less likely surprises

6. **Bundle pair structure differs from BRIEF assumption.** BRIEF assumes pair-Bundles are `[name, type]` (2 children). If they're actually 3 children (e.g., `[name, separator, type]`), adjust pair[1] → pair[2]. STOP-trigger 2 catches.

7. **The walker has hidden complexity around `Var` types or fn-typed args.** If a fn type slot needs special unwrapping vs other parametric types, that's edge case work. Likely already handled by slice 1's uniform Bundle emission.

8. **Sibling factoring exposes a refactor opportunity for arc 143's extract-arg-names.** If the factored walker is significantly cleaner, sonnet might tempted to update arc 143's caller too. STOP — leave arc 143 untouched; only ADD the new handler. Refactor opportunity captured as honest delta + queued for separate consideration.

## Workspace baseline (commit `c47c601`, captured 2026-05-16)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2323 passed / 3 failed (last verified post-arc-202; lifeline flake passed that round)
- Pre-existing failures (DO NOT BLOCK slice 5):
  - `deftest_wat_tests_tmp_totally_bogus` (unresolved reference in test fixture)
  - `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)
  - `t6_spawn_process_factory_with_capture_round_trips` (arc 170 slice 6 documented gap)
  - `lifeline_pipe_zero_orphans_across_100_trials` may flap; not a regression either way

Post-slice-5 target:
- Pass count ≥ 2323 + 4-5 (new tests)
- Fail count ≤ 4 (no regressions; lifeline variance acceptable)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 30-60 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | ≤ 4 | TBD | TBD |
| New test count | 4-5 | TBD | TBD |
| Walker factoring decision | likely parallel (keep duplicate); possibly factored | TBD | TBD |
| Variadic handling | mirrors extract-arg-names behavior (whatever that is — sonnet checks) | TBD | TBD |
| `/gaze` exchanges | 0 | TBD | TBD |
| arc 057/143 surface check | "nothing relevant" confirmed | TBD | TBD |
| STOP-triggers fired | 0-1 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
