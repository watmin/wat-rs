# Arc 213 stone χ-1 — SCORE

## Classification: Mode A

All 17 scorecard criteria satisfied. First-attempt compile clean. 3/3 probe tests pass. Baseline unchanged.

## Actual vs predicted runtime

- **Predicted:** 30-45 min Mode A
- **Actual:** ~10 min (read BRIEF + EXPECTATIONS, read typed_channel.rs, check SHUTDOWN_RX type in runtime.rs, check lib.rs pub mod, add wrapper, write probe, build, test)
- **Delta:** Faster than predicted. Pattern was a direct copy of typed_recv's existing Crossbeam arm; no surprises.

## Honest-delta report (Risk 1-5)

### Risk 1 — SHUTDOWN_RX accessor signature

No delta. `crate::runtime::SHUTDOWN_RX.get()` returns `Option<&crossbeam_channel::Receiver<()>>`.
Confirmed at `src/runtime.rs:179`:
```rust
pub static SHUTDOWN_RX: OnceLock<crossbeam_channel::Receiver<()>> = OnceLock::new();
```
The `select!` recv arm receives `&Receiver<()>` — identical pattern to `typed_recv`'s Crossbeam arm (lines 304-313). Zero dereference dance needed.

### Risk 2 — `crossbeam_channel::select!` macro in generic context

No delta. `select!` composed cleanly with `impl<T> Receiver<T>`. The generated code resolves T at each instantiation. Compile clean on first attempt. No fallback needed.

### Risk 3 — `Sender<T>: Clone` + `Receiver<T>: Clone`

No delta. Manual `impl<T> Clone` for both wrappers compiles without issues.

### Risk 4 — Module path in the probe

No delta. `src/lib.rs:103` has `pub mod typed_channel;`. The probe's `use wat::typed_channel::{unbounded, RecvError, TryRecvError};` resolves correctly.

### Risk 5 — Derived traits

Chose conservative path: no `#[derive(Debug)]` on `Sender<T>` or `Receiver<T>`. The 35 migration target sites don't require Debug on the channel handle itself. This matches EXPECTATIONS prediction.

## LOC

| File | Predicted | Actual |
|---|---|---|
| `src/typed_channel.rs` | 60-80 | 82 |
| `tests/probe_channel_primitive.rs` | 25-35 | 26 |

The +2 over the upper bound of `src/typed_channel.rs` is the section separator comment block (7 lines) — the API struct+impl code is 75 lines matching prediction.

## Build output

```
cargo build --release
   Compiling wat v0.1.0 (/home/watmin/work/holon/wat-rs)
   [5 pre-existing dead_code warnings, unchanged from baseline]
    Finished `release` profile [optimized] target(s) in 18.36s
```

No new warnings. No errors.

## Probe test output

```
cargo test --release --test probe_channel_primitive
     Running tests/probe_channel_primitive.rs (target/release/deps/probe_channel_primitive-52bb0a842933aae6)

running 3 tests
test probe_chi1_try_recv_empty_returns_empty ... ok
test probe_chi1_sender_drop_triggers_recv_err ... ok
test probe_chi1_unbounded_round_trip ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Baseline regression output

```
cargo test --release --test wat_arc170_program_contracts
     Running tests/wat_arc170_program_contracts.rs

running 24 tests
test t1_legacy_3arg_main_fires_walker ... ok
test t11_legacy_main_signature_fires_walker_diagnostic ... ok
test t14_spawn_process_wait_handle_is_idempotent ... ok
test t10_spawn_thread_unchanged_positive_control ... ok
test t2_canonical_main_returns_nil_value ... ok
test t13_spawn_process_child_exits_clean_on_parent_tx_drop ... ok
test t18_run_hermetic_with_io_layer2_echo_doubled ... ok
test t17b_run_hermetic_layer1_failing_assertion_surfaces_failure ... ok
test t17_run_hermetic_layer1_passing_assertion ... ok
test t1_canonical_nil_main_freezes ... ok
test t3_argv_reachable_via_ambient ... ok
test t18b_run_hermetic_with_io_layer2_failing_assertion_surfaces_failure ... ok
test t7_spawn_process_non_portable_capture_fires_diagnostic ... ok
test t2_canonical_main_with_let_body_returns_nil ... ok
test t9_spawn_program_callsite_fires_walker ... ok
test t12_spawn_process_child_emits_without_recv ... ok
test t8b_fork_program_ast_callsite_fires_walker ... ok
test t8_fork_program_callsite_fires_walker ... ok
test t9b_spawn_program_ast_callsite_fires_walker ... ok
test t16_spawn_process_sequential_spawns_no_fd_zombie_leak ... ok
test t4_spawn_process_keyword_fn_round_trips_typed_value ... ok
test t6_spawn_process_factory_with_capture_round_trips ... ok
test t5_spawn_process_inline_lambda_round_trips ... ok
test t15_spawn_process_child_panic_disconnects_recv_and_exits_nonzero ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

24/24 unchanged.

## Scorecard

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | `wat::typed_channel::Sender<T>` newtype minted with `send(t) -> Result<(), SendError<T>>` | YES | YES |
| 2 | `wat::typed_channel::Receiver<T>` newtype minted with `recv() -> Result<T, RecvError>` cascade-aware | YES | YES |
| 3 | `Receiver<T>::try_recv() -> Result<T, TryRecvError>` | YES | YES |
| 4 | `unbounded<T>()` + `bounded<T>(n)` factory functions | YES | YES |
| 5 | `Sender<T>` + `Receiver<T>` both implement `Clone` | YES | YES |
| 6 | `RecvError`, `SendError`, `TryRecvError` re-exported from crossbeam_channel | YES | YES |
| 7 | recv() routes through `SHUTDOWN_RX` cascade-aware `select!` (pattern parity with typed_recv) | YES | YES |
| 8 | recv() bootstrap fallback to bare `.inner.recv()` when SHUTDOWN_RX uninitialized | YES | YES |
| 9 | `tests/probe_channel_primitive.rs` minted with 3 tests | YES | YES |
| 10 | Probe test `probe_chi1_unbounded_round_trip` PASS | YES | YES |
| 11 | Probe test `probe_chi1_sender_drop_triggers_recv_err` PASS | YES | YES |
| 12 | Probe test `probe_chi1_try_recv_empty_returns_empty` PASS | YES | YES |
| 13 | cargo build --release clean | YES | YES |
| 14 | No modifications to existing typed_send/typed_recv/SenderInner/ReceiverInner | YES | YES |
| 15 | Zero modifications outside src/typed_channel.rs + tests/probe_channel_primitive.rs + SCORE doc | YES | YES |
| 16 | Dirty tree intact (src/fork.rs + src/spawn_process.rs untouched) | YES | YES |
| 17 | cargo test --release --test wat_arc170_program_contracts result unchanged from pre-χ-1 baseline | YES | YES |

## Placement decision

Code placed at line 528 of `src/typed_channel.rs` — immediately before the `make_thread_peer_pair_for_test()` function, after `make_pipe_channel_pair()`. This follows existing file flow: constructors → send/recv/try_recv functions → χ-1 wrapper → test helpers. The BRIEF said "after typed_try_recv at line 407 OR at end of file — sonnet's choice based on local convention." Chose placement before the test helper rather than after it, to keep test-scope functions at the very end. The section separator comment makes the boundary obvious.

## Files modified

- `/home/watmin/work/holon/wat-rs/src/typed_channel.rs` — 82 lines added (χ-1 wrapper section)
- `/home/watmin/work/holon/wat-rs/tests/probe_channel_primitive.rs` — new file, 26 lines
- `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md` — this file

## Not committed

Per BRIEF constraint: orchestrator commits after independent SCORE verification.
