# Arc 170 Stone A SCORE — `Thread/drain-and-join` + `Process/drain-and-join`

**BRIEF:** `BRIEF-STONE-A-DRAIN-AND-JOIN.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-A-DRAIN-AND-JOIN.md`

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `:wat::kernel::Thread/drain-and-join` substrate primitive defined + registered | **YES** | `src/runtime.rs:16949` defines `fn eval_kernel_thread_drain_and_join`; dispatch arm at `src/runtime.rs:4329` (`":wat::kernel::Thread/drain-and-join" => eval_kernel_thread_drain_and_join(...)`). |
| B | `:wat::kernel::Process/drain-and-join` substrate primitive defined + registered | **YES** | `src/runtime.rs:16445` defines `fn eval_kernel_process_drain_and_join`; dispatch arm at `src/runtime.rs:4288`. |
| C | Type signatures registered (Thread/drain-and-join + Process/drain-and-join both return `Result<(), Vec<*DiedError>>`) | **YES** | `src/check.rs:12482` registers `:wat::kernel::Process/drain-and-join` with `ret = Result<Tuple([]), Vec<ProcessDiedError>>`; `src/check.rs:12619` registers `:wat::kernel::Thread/drain-and-join` with `ret = Result<Tuple([]), Vec<ThreadDiedError>>`. |
| D | Tests pass — Thread happy + Process happy + Thread panic + Process panic | **YES** | `tests/wat_arc170_stone_a_drain_and_join.rs` defines all 4 tests; `cargo test --release -p wat --test wat_arc170_stone_a_drain_and_join` → `test result: ok. 4 passed; 0 failed`. Test names: `stone_a_thread_drain_and_join_clean_exit_returns_ok`, `stone_a_process_drain_and_join_clean_exit_returns_ok`, `stone_a_thread_drain_and_join_panic_returns_err`, `stone_a_process_drain_and_join_panic_returns_err`. |
| E | `cargo build --release --workspace --tests` clean | **YES** | Final `cargo build --release --workspace --tests`: `Finished release profile [optimized] target(s)`; no errors. (5 pre-existing dead-code warnings on retired internal fns — not introduced by this stone.) |
| F | Workspace test failure count ≤ baseline | **YES** | Baseline (pre-Stone-A) and post-Stone-A both end with `error: 4 targets failed: wat::probe_lifeline_pipe_proof, wat::test, wat::wat_arc170_program_contracts, wat-cli::wat_cli`. Re-runs per target post-implementation: `wat_arc170_program_contracts` 23 pass / 1 fail (`t6_spawn_process_factory_with_capture_round_trips`, pre-existing — unrelated `:wat::core::unquote` runtime error inside child program); `wat --test test` 176 pass / 1 fail (`deftest_wat_tests_tmp_totally_bogus - should panic`, pre-existing); `wat-cli --test wat_cli` 14 pass / 1 fail (`startup_error_bubbles_up_as_exit_3`, pre-existing); `probe_lifeline_pipe_proof` flaky (1/100 trial-failure baseline; re-ran clean post-implementation). NO new failures introduced; new target `wat_arc170_stone_a_drain_and_join` adds 4 passes. |

**6/6 PASS.**

## Honest deltas

### Shared internal helper extraction

**Yes — partial.** Two small internal helpers were extracted, both private to `src/runtime.rs`:

- `fn drain_thread_output_channel(thread_struct: &Arc<StructValue>, subject_ast: &WatAST, op: &str) -> Result<(), RuntimeError>` (`src/runtime.rs:17027`) — drains the typed Receiver at field 1 by calling `typed_recv` in a loop until `Disconnected` / `Shutdown` / `DecodeError`. `DecodeError` is treated like a disconnect for drain purposes (we discard drained values either way; the downstream join surfaces the real outcome).
- `fn drain_process_reader_field(proc_struct: &Arc<StructValue>, field_index: usize, subject_ast: &WatAST, op: &str) -> Result<(), RuntimeError>` (`src/runtime.rs:16526`) — drains an IOReader at field 1 (stdout) or 2 (stderr) via `read_line` until `None`. Kernel-level read errors are treated as EOF for drain purposes.

**No** modifications to the existing `eval_kernel_thread_join_result` / `eval_kernel_process_join_result` (per BRIEF constraint). The join logic inside `eval_kernel_thread_drain_and_join` / `eval_kernel_process_drain_and_join` is a near-duplicate of the existing eval-fn bodies (ProgramHandle dispatch + SpawnOutcome match). Stone B can re-factor the shared join body into one helper when it hides the older verbs; doing it in this stone would require touching the explicitly-frozen call sites.

### Type signature shape that worked

Both helpers register with the **identical shape** the existing `*_join-result` signatures use:

```rust
TypeScheme {
    type_params: vec!["I".into(), "O".into()],
    params: vec![thread_ty()],   // or process_ty()
    ret: TypeExpr::Parametric {
        head: "wat::core::Result".into(),
        args: vec![
            TypeExpr::Tuple(vec![]),                       // == :wat::core::nil
            thread_died_chain_ty(),                        // Vec<ThreadDiedError>
            // (process side: process_died_chain_ty())
        ],
    },
    rest_param_type: None,
}
```

Test-side return-type annotations use the form
`:wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>` — note `wat::core::nil` (NOT `:()`) inside the parametric per `feedback_wat_colon_quote` and the substrate's `arc 153` rename (unit → nil). The BRIEF's target line wrote `:Result<:wat::core::nil, :wat::kernel::ThreadDiedError>` (singular Err); the actual existing chain shape is `Vec<*DiedError>` per arc 113 — the wrapper matches the chain shape exactly because it delegates to the same internal mechanism.

### Test names + locations

`tests/wat_arc170_stone_a_drain_and_join.rs`:

- `stone_a_thread_drain_and_join_clean_exit_returns_ok` — spawn-thread sends 3 i64 values, parent doesn't recv; drain-and-join drains the channel + joins clean.
- `stone_a_process_drain_and_join_clean_exit_returns_ok` — spawn-process prints 2 stdout lines + 1 stderr line, parent doesn't read either; drain-and-join drains both pipes + joins clean.
- `stone_a_thread_drain_and_join_panic_returns_err` — spawn-thread panics via `Option/expect None`; drain-and-join returns `Err(chain)` with non-empty `ThreadDiedError` Vec.
- `stone_a_process_drain_and_join_panic_returns_err` — spawn-process panics via `Option/expect None`; drain-and-join returns `Err(chain)` with non-empty `ProcessDiedError` Vec.

### Workspace test count vs baseline

| Target | Baseline | Post-Stone-A | Delta |
|---|---|---|---|
| `wat::wat_arc170_stone_a_drain_and_join` (NEW) | (did not exist) | **4 passed / 0 failed** | +4 passes |
| `wat::wat_arc170_program_contracts` | 23 pass / 1 fail (t6) | 23 pass / 1 fail (t6) | unchanged |
| `wat::test` (wat-rs lib stdlib tests) | 176 pass / 1 fail | 176 pass / 1 fail | unchanged |
| `wat::probe_lifeline_pipe_proof` | 1 fail (flaky 1/100) | 1 pass (this run) | flake-window; not regression |
| `wat-cli::wat_cli` | 14 pass / 1 fail | 14 pass / 1 fail | unchanged |

Net: **+4 new passes; 0 new failures.** Workspace tests still surface `error: 4 targets failed` at the cargo-summary level, identical target list to baseline — all four are pre-existing flakes / unrelated runtime errors.

### Substrate-discovery surprises

Three minor, none blocking:

1. **Thread fields are `Value::wat__kernel__Sender` / `Value::wat__kernel__Receiver`, not raw crossbeam handles.** The type registration at `src/check.rs` labels them `:rust::crossbeam_channel::Sender<:I>` / `:rust::crossbeam_channel::Receiver<:O>`, but the runtime constructs them via `crate::typed_channel::sender_from_crossbeam` / `receiver_from_crossbeam` which produce `Value::wat__kernel__Sender(Arc<SenderInner>)` and `Value::wat__kernel__Receiver(Arc<ReceiverInner>)`. The drain helper matches on the `Value::wat__kernel__Receiver` variant; works for both crossbeam-backed and pipe-fd-backed receivers.
2. **Thread/Process struct field-count differs (3 vs 4).** Thread has `[input, output, join]`; Process has `[stdin, stdout, stderr, join]`. Two different field-index constants needed in the drain helpers; documented inline.
3. **Wat-keyword cheatsheet rule about `<>`-internal `:()`.** Initial test sources used `:Result<:(), :Vec<ThreadDiedError>>`; substrate rejected both the `:()` leading-colon (`feedback_wat_colon_quote`) and the inner `:Vec` leading-colon. Migration: write `:wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>`. The substrate's own diagnostic guided the fix in two iterations.

The drain-and-join wrapper IS just composition (drain step + existing join body); no substrate refactor was required to expose drain machinery for Thread's typed channel. `typed_recv` on a Crossbeam-backed `ReceiverInner` correctly returns `Disconnected` once the worker thread's Sender drops on exit (the spawn driver drops `outcome_tx`'s peer chain after `apply_function`); no deadlock concerns.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90–120 min | ~50 min |
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ baseline | = baseline (no regressions; 4 pre-existing target failures unchanged) |
| New test count | 4+ | 4 |
| Substrate-discovery surprises | 0–2 | 3 (all minor; substrate diagnostics guided the wat-keyword rule iterations) |
| Mode | Additive | Additive (no modifications to existing `eval_kernel_*_join_result`; two new substrate fns + two new internal helpers + two type registrations + two dispatch arms) |

## STOP triggers encountered

**None.** The drain machinery composed cleanly: `typed_recv` for Thread's typed channel, `read_line` for Process's stdout/stderr. No deadlocks. No existing-test breakage. No type-registration-pattern surprises.

## What's ready for Stone B

- `Thread/drain-and-join` + `Process/drain-and-join` substrate-vended and user-callable
- Both helpers wrap the same internal join logic the older `*_join-result` verbs use; the shared-join-helper refactor remains optional (BRIEF explicitly deferred existing-fn modification to Stone B)
- Type signatures registered; user wat code can call them today
- Tests in place that exercise both happy + panic paths; future stones can extend these as their integration baseline

Stone B's walker check can now safely binary-reject `*_join-result` from user namespace — the drain-and-join helpers are the canonical replacements.
