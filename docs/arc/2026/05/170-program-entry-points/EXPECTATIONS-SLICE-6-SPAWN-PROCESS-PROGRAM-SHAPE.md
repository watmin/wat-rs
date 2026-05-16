# Arc 170 Slice 6 EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-6-SPAWN-PROCESS-PROGRAM-SHAPE.md`
**Task:** #323

## Independent prediction

**Runtime band:** 60–120 minutes.

Reasoning:
- Substrate change: ~50-100 lines in `src/spawn_process.rs` (signature + child-program-construction path)
- 3 canonical macros update in wat/test.wat (~5-10 lines each)
- 1 new macro to mint (`run-hermetic-with-config` ~15 lines + a driver if needed)
- Build cycles + workspace verification: ~10 min
- Honest deltas for substrate discovery: variable; could add significant time if substrate path is more complex than predicted

**Time-box:** 180 min hard stop.

## SCORE methodology

6 rows YES/NO per BRIEF; per-row evidence patterns:

- **Row A** (substrate signature): `grep -nA 30 "fn eval_kernel_spawn_process" src/spawn_process.rs` shows Vec<WatAST> parsing path (look for `Value::Vec` arm or similar).
- **Row B** (canonical macros updated): `grep -A 6 "(:wat::test::run-hermetic$\|run-thread$\|run-hermetic-with-io$" wat/test.wat` shows macro bodies use `(:wat::core::Vector :wat::WatAST ...)` construction.
- **Row C** (new macro): `grep "run-hermetic-with-config\|run-hermetic-with-prelude" wat/test.wat` shows defmacro + driver (if minted).
- **Row D** (build clean): cargo build clean.
- **Row E** (canonical-macro consumers pass): targeted cargo test on the named test files all green.
- **Row F** (workspace baseline maintained): cargo test summed failed ≤ 11 (rotation band); ideally ≤ 2 (post-4c-α-ii baseline).

## Honest deltas to watch for

- **Substrate discovery surprises.** The orchestrator's hypothetical shape (Vec<WatAST> argument; child serializes forms back to source-text) may not match reality. If the substrate's existing child-program-construction path doesn't accept Vec<WatAST> cleanly, surface and propose an alternative.

- **Macro quasiquote edge cases.** The macro expansion needs to construct `(:wat::core::Vector :wat::WatAST '(:wat::core::define ...))` — the inner quote may interact unusually with macro template splicing (`~body` etc.). If splicing produces an unexpected shape, surface.

- **Driver signature changes.** `run-hermetic-driver`, `run-hermetic-with-io-driver`, `run-thread-driver` all take a `Process<...>` returned by spawn-process. The Process shape itself shouldn't change (still has stdin/stdout/stderr fields + ProgramHandle). But verify.

- **fn vs define-vs-fn shape in the child program.** The orchestrator's BRIEF example shows two forms (with-fn-wrapping or define-direct-body). Sonnet — discover which the substrate's child parser accepts; both forms may need wrapping in a fn or not depending on how the child boots :user::main.

- **Existing capability-losing tests** (capacity-mode + scope tests from 4c-α-ii). DO NOT migrate them in this slice — out of scope. Surface them at the end of SCORE as targets for the downstream stone.

- **wat-cli interaction.** wat-cli currently uses `fork_program_from_source` (legacy, slated for retirement in Slice 4b). The pivot makes spawn-process IPC-contract-equivalent to wat-cli. Slice 4b (wat-cli Stone B) now naturally fits — wat-cli can just call spawn-process with the parsed forms. NOT this slice's responsibility, but note in SCORE if the new spawn-process shape clearly unblocks 4b.

## Workspace baseline (commit `ddfb6b5`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2271 passed / 2 failed (tmp_totally_bogus, startup_error_bubbles_up_as_exit_3 — pre-existing rotation members)

Post-slice-6 target:
- ≥ 2271 passed (existing tests preserved; new run-hermetic-with-config deftest adds 1+)
- ≤ 11 failed (variance band; ideally ≤ 2 if no regressions)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60–120 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ 11 (ideally ≤ 2) | TBD |
| Substrate-discovery surprises | 0–3 | TBD |
| Mode | A or B (substrate finding likely) | TBD |
