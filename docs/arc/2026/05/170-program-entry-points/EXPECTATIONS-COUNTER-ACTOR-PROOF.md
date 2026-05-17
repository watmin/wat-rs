# EXPECTATIONS — Counter actor pattern proofs (thread + process tiers)

**BRIEF:** `BRIEF-COUNTER-ACTOR-PROOF.md`
**Drafted:** 2026-05-16, pre-spawn.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- 2 new wat-tests files (~80-150 lines each)
- Pattern code: enum decls + dispatch + 4 client wrappers + spawn — straightforward composition
- Test body: spawn + exercise + 6 asserts + drain-and-join
- First consumer of the inscribed Counter pattern — substrate-actual gaps will surface
- Investigation time for any inscription↔substrate gap: ~10-20 min per gap
- Process-tier program-forms construction is novel pattern in this test — may need iteration

Larger than mechanical sweeps because it's PROVING the inscription. Each substrate primitive used must agree with the inscription's syntax; mismatches surface as build/test failures and need correction.

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — thread-tier deftest passes | YES | medium-high (substrate primitives all exist; pattern is straightforward) |
| B — process-tier deftest passes | YES | medium (spawn-process program-forms construction less practiced; ProcessPeer/new argument order may need verification) |
| C — body shape identical across tiers (only verbs differ) | YES | high (intentional symmetry; sonnet writes them in parallel) |
| D — workspace baseline preserved | YES | high (purely additive tests; no substrate change) |
| E — inscribed pattern claims verified | YES (with honest deltas) | medium (some details may differ; surfacing them IS the value) |

**5/5 PASS predicted; ~70% confidence overall.** Lower than recent slices because this is INTEGRATION-proving (validating the inscription against substrate-actual).

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **`recv` / `send` Result wrapping.** Per arc 110/111, recv returns `Result<I, ThreadDiedError>`. The Counter dispatch's `(match (recv server-rx!) ((Counter/Request/Get) ...))` may need to handle the Result first — either via `(match (recv server-rx!) ((Ok req) (match req ...)) ((Err _) :nil))` or `(match (option::expect (recv server-rx!) "...") ...)`. The inscription's example is simplified; sonnet finds the actual shape.

2. **`(:wat::kernel::readln)` vs alternative spelling.** Process-tier server uses ambient readln — verify the exact verb (might be `(readln)` if symbol-resolved, or might need full FQDN `:wat::kernel::readln`).

3. **`ProcessPeer/new` argument order.** Could be `(rx tx)` or `(tx rx)` — verify from Stone C2's SCORE or check.rs registration. Inscription suggests `(Process/stdout, Process/stdin)` which is `(rx, tx)` perspective — confirm.

4. **`spawn-process` program-forms construction.** The Vector<WatAST> arg per arc 170 Slice 6 — exact idiom may be `(:wat::core::Vector :wat::WatAST 'form1 'form2)` or via quasiquote-list or via `(:wat::core::forms ...)`. Sonnet picks correct form.

5. **Server-side spawn-thread fn signature.** spawn-thread takes `:Fn(Receiver<I>, Sender<O>) -> :nil` per arc 114 — the type-keyword construction for `:Receiver<Counter/Request>` may need the same expand-time machinery the run-threads macro uses (per arc 199 / D1 SCORE). For a direct-spawn (non-macro), the user writes the full keyword literally — should work but verify.

6. **deftest prelude form.** Per `wat-tests/run-thread.wat`, the deftest signature is `(deftest :name () body)` where `()` is the prelude. Counter's enums + defns go in the prelude. Verify the splice machinery handles all 6+ defns + 2 enums cleanly.

### Less likely surprises

7. **Enum variant declaration syntax differs.** Per arc 113 / 098 / similar, enum variants declared as `Variant` (no-payload) or `(Variant :Type)` (with payload). Inscription matches this; should work.

8. **Type checker rejects the recursive dispatch fn's tail call.** Recursive defns should typecheck per ITERATION-PATTERNS.md Pattern 6 + arc 166 (defn); verify.

9. **Process child's `:user::main` form differs from expected.** Per arc 170 Slice 6 + IPC contract (recovery doc Section 13), main returns `:nil`; verify the Counter wrapper fits.

## Workspace baseline (commit `6231dae`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2328 passed / 4 failed (3 stable + lifeline flake variance ±1)
- Pre-existing failures (DO NOT BLOCK):
  - `deftest_wat_tests_tmp_totally_bogus` (unresolved reference)
  - `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)
  - `t6_spawn_process_factory_with_capture_round_trips` (arc 170 slice 6 documented gap)
  - `lifeline_pipe_zero_orphans_across_100_trials` may flap

Post-proof target:
- Pass count: ≥ 2328 + 2 (both new deftests pass; one each per tier)
- Fail count: ≤ 4 (no regressions)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | ≤ 4 | TBD | TBD |
| New test count | 2 (thread + process) | TBD | TBD |
| Inscription↔substrate gaps surfaced | 1-3 (recv-Result, ambient verb spelling, ProcessPeer/new arg order all likely) | TBD | TBD |
| INTERSTITIAL corrections suggested | 1-3 | TBD | TBD |
| STOP-triggers fired | 0-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
