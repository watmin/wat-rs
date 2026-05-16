# Arc 198 Slice 2 Stone 3 EXPECTATIONS

**BRIEF:** `BRIEF-STONE-3-APPLY-TO-JOIN-RESULT.md`

## Independent prediction

**Runtime band:** 30 minutes sonnet.

Reasoning:
- 2 attribute annotations: trivial (2 lines added above each fn)
- 1 verification test: ~50-80 LOC
- Possibly 1 `use` statement addition if path resolution requires it
- Possibly 1 Cargo.toml line if wat-macros isn't already a non-dev dep of wat (it likely is for #[wat_dispatch])

**Time-box:** 60 min hard stop.

## SCORE methodology

5 rows YES/NO per BRIEF:

- **Row A** (Thread attribute applied): grep shows attribute above the fn
- **Row B** (Process attribute applied): same for Process
- **Row C** (new test passes): `cargo test --release -p wat --test wat_arc198_slice2_stone_3_apply` green
- **Row D** (predecessors still pass): targeted tests all green
- **Row E** (workspace baseline): cargo test summed failed ≤ baseline + flake variance

## Honest deltas to watch for

- **Import path resolution.** The attribute lives in `wat-macros` crate. The wat crate likely already has wat-macros as a dep for `#[wat_dispatch]`. Verify the `use wat_macros::restricted_to;` (or however Stone 2 exported it) works in `src/runtime.rs`. May need re-export or path adjustment.

- **Both walkers firing — error format combination.** When user-namespace code calls `Thread/join-result`:
  - Stone B's `validate_join_result_user_namespace` produces error mentioning "drain-and-join"
  - Arc 198's `walk_for_def_restricted_call` produces `DefRestrictedCallerNotAllowed` error mentioning the whitelist
  - Both should appear in the diagnostic output (additive errors)
  - Stone B's test assertions grep for `Thread/join-result` AND `drain-and-join` substrings — both present in Stone B's error → still passes
  - Arc 198 slice 1's tests grep for `DefRestrictedCallerNotAllowed` shape — present in arc 198's error → still passes

- **Cargo.toml dependency direction.** wat-macros is a proc-macro crate; wat depends on it. Verify the dep is present in `[dependencies]` (non-dev) so the attribute can be applied to production fns, not just test code.

- **`pub(crate)` visibility on the substrate fns.** Stone 2's attribute generates `inventory::submit!` at module scope adjacent to the fn. `pub(crate)` should be fine for both. Verify no visibility surprises.

## Workspace baseline (commit `6775510`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures + lifeline flake variance

Post-Stone-3 target:
- ≥ baseline + 1 passed (new application test)
- ≤ baseline failures (additive — should not change behavior visibly; Stone B's rule still fires + arc 198's walker now fires)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30 min | TBD |
| Scorecard rows | 5/5 PASS | TBD |
| Workspace fail count | ≤ baseline | TBD |
| New test count | 1 | TBD |
| Import path used | `use wat_macros::restricted_to;` OR absolute | TBD |
| Both walkers fire | YES (expected) | TBD |
| Substrate-discovery surprises | 0-1 | TBD |
| Mode | Additive (2 attribute applications + 1 test) | TBD |
