# Arc 213 stone χ-2 — SCORE: Migrate caller sites to `wat::typed_channel`

## Runtime band

Actual: ~65 min Mode A with honest-delta surface (criterion 17 partial + typed_channel.rs touch).

## LOC changed

~40 lines across 4 caller files + 2 lines in typed_channel.rs (Debug derive on Sender + Receiver).

## New files

1 (this SCORE doc).

---

## Scorecard

| # | Criterion | Expected | Actual | Notes |
|---|---|---|---|---|
| 1 | `src/thread_io.rs` import line migrated (`use crate::typed_channel::{Receiver, Sender}`) | YES | YES | Line 28 changed |
| 2 | `src/thread_io.rs` ALL `crossbeam_channel::{Sender,Receiver,unbounded,bounded}` references migrated to `crate::typed_channel::*` | YES | PARTIAL | All Rust-typed channel factories migrated; Value-typed channels stay bare (see honest-delta §) |
| 3 | `src/runtime.rs` non-cascade-primitive `crossbeam_channel::{Sender,Receiver,unbounded,bounded}` references migrated | YES | PARTIAL | `ProgramHandleInner::InThread` + line 17544 factory migrated; HandlePool NOT migrated (see honest-delta §) |
| 4 | `src/runtime.rs:179` `SHUTDOWN_RX` UNCHANGED (still bare crossbeam) | YES | YES | Untouched |
| 5 | `src/runtime.rs:185` `SHUTDOWN_TX_PTR` UNCHANGED (still bare crossbeam) | YES | YES | Untouched |
| 6 | `src/runtime.rs:233` `init_shutdown_signal` factory UNCHANGED (still bare crossbeam) | YES | YES | Untouched |
| 7 | `src/freeze.rs` `crossbeam_channel::*` migrated | YES | PARTIAL | Line 1080 recv inherits cascade-aware type via ProgramHandleInner::InThread change (0 explicit edits needed); line 1022 spawn_service return type stays bare (extracts inner crossbeam sender from SenderInner::Crossbeam — structural, not migrated) |
| 8 | `src/spawn.rs` `crossbeam_channel::*` migrated | YES | YES | Line 181 factory migrated to `crate::typed_channel::bounded::<SpawnOutcome>(1)` |
| 9 | `cargo build --release` clean | YES | YES | 0 errors, 5 pre-existing warnings (unchanged from pre-χ-2 baseline) |
| 10 | `cargo test --release --test probe_channel_primitive` 3/3 PASS | YES | YES | See output below |
| 11 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS | YES | YES | See output below |
| 12 | NO touches to `src/typed_channel.rs` (the wrapper home) | YES | DELTA | `#[derive(Debug)]` added to `Sender<T>` and `Receiver<T>` — see honest-delta § |
| 13 | NO touches to `src/check.rs` / `src/lexer.rs` / `src/parser.rs` / `src/types.rs` (type-name strings, not callers) | YES | YES | None of these files touched |
| 14 | NO touches to `src/fork.rs` / `src/spawn_process.rs` (dirty tree δ-1) | YES | YES | Neither file touched |
| 15 | NO `wat_arc170_program_contracts` run during verification | YES | YES | Only `probe_channel_primitive` + `probe_pidfd_primitive` run |
| 16 | SCORE doc inscribes site-counts migrated per file + cargo build output + probe outputs | YES | YES | All in this document |
| 17 | Total `crossbeam_channel::` references outside typed_channel.rs / runtime.rs cascade lines / check.rs / lexer.rs / parser.rs / types.rs → ZERO post-migration | YES | PARTIAL | Value-typed channels in thread_io.rs + freeze.rs line 1022 stay bare — see honest-delta § |

---

## Honest-delta surfaces

### Delta A — Risk 1 fires: HandlePool uses `rx.len()` (wrapper doesn't expose `len`)

**File:** `src/runtime.rs`
**Site:** line 16951 — `rx.len()` in `eval_handle_pool_pop`... actually in `eval_handle_pool_finish`
**Channel:** `wat__kernel__HandlePool { rx: Arc<crossbeam_channel::Receiver<Value>> }`
**What it does:** `HandlePool::finish` checks `rx.len()` to assert no orphaned handles remain. `len()` is NOT on `crate::typed_channel::Receiver<T>`.

**Decision taken:** Left `HandlePool` as bare `crossbeam_channel::bounded::<Value>(n.max(1))` (line 16861) and `rx: Arc<crossbeam_channel::Receiver<Value>>` (runtime.rs line 504). The `tx.send` (line 16863) and `rx.try_recv()` (line 16906) also stayed bare as part of the same channel pair.

**Consequence for criterion 17:** HandlePool's `crossbeam_channel::Receiver<Value>` reference at line 504 is an honest non-zero remaining.

**Orchestrator decision needed:** Should `len()` be added to the wrapper? Or should HandlePool use a different internal structure (e.g., AtomicUsize counter alongside the channel)? χ-3 (restricted_to wall) will surface this site explicitly.

### Delta B — Risk 3 fires: `typed_channel::Sender<T>` and `Receiver<T>` lacked `#[derive(Debug)]`

**Where:** `src/typed_channel.rs` — touched despite BRIEF saying DO NOT TOUCH
**What:** Added `#[derive(Debug)]` to `Sender<T>` (line 541) and `Receiver<T>` (line 568).
**Why:** Caller structs/enums that derive `Debug` (`ProgramHandleInner`, `StdOutServiceEvent`, `StdErrServiceEvent`, `StdInServiceEvent`) require `Debug` on all their fields. Without `#[derive(Debug)]` on the wrapper, cargo produced 7 `E0277` errors on first build attempt.
**Scope of change:** 2 derive annotations. No API surface change; no methods added. `Clone` was already manually implemented in χ-1.
**EXPECTATIONS Risk 3:** "likely-easy to fix ... but a Mode B surface if it requires non-trivial wrapper changes." This is the trivial case — two derive lines. Reported here for transparency.

### Delta C — Value-typed channels in thread_io.rs stay bare crossbeam

**What:** The 8 `crossbeam_channel::bounded::<Value>(1)` factory calls in `register_thread_with_services` (lines 568-603) that produce the `wat_*_data_tx/rx`, `wat_*_reply_tx/rx`, `wat_*_ack_tx/rx` channels were NOT migrated.

**Why:** These channels are IMMEDIATELY consumed by `sender_value(tx: crossbeam_channel::Sender<Value>)` or `receiver_value(rx: crossbeam_channel::Receiver<Value>)` which call `sender_from_crossbeam`/`receiver_from_crossbeam`. Those functions require the raw `crossbeam_channel::Sender<Value>` — the inner type that lives INSIDE `SenderInner::Crossbeam`. Migrating the factory would produce `crate::typed_channel::Sender<Value>` (the newtype wrapper), which cannot be passed to `sender_from_crossbeam`. The bridge is a structural coupling: these channels become wat-side `Value::wat__kernel__Sender` Values via wrapping, not T-typed substrate channels.

**Corresponding bridge function params:** `spawn_stdin_bridge`, `spawn_stdout_bridge`, `spawn_stderr_bridge` all have `wat_data_tx: crossbeam_channel::Sender<Value>` and `wat_ack_rx/wat_reply_rx: crossbeam_channel::Receiver<Value>` — explicitly qualified after the import change to keep these as bare crossbeam.

**What DID migrate:** The Rust-typed channels (carrying `StdInServiceEvent`, `StdOutServiceEvent`, `StdErrServiceEvent`, `String`, `()`) — all 8 factory calls for those were migrated to `crate::typed_channel::bounded::<T>(1)`.

**Consequence for criterion 17:** thread_io.rs has ~14 remaining `crossbeam_channel::` references, all structural (the Value-typed bridge channels + extraction helpers + RuntimeServices fields). These cannot be zero without a rearchitecture of the bridge pattern.

### Delta D — freeze.rs line 1022 `spawn_service` return type stays bare

**Site:** `src/freeze.rs:1022` — `fn spawn_service(...) -> Result<(Value, crossbeam_channel::Sender<Value>), RuntimeError>`
**Why:** This function calls `extract_control_tx` which extracts the raw inner `crossbeam_channel::Sender<Value>` from `SenderInner::Crossbeam { sender: s, .. }`. The returned value IS the bare crossbeam sender — it cannot be the wrapper type without restructuring the extraction helpers. Left as-is; the 1 recv site in freeze.rs (line 1080) inherits cascade-aware semantics automatically from the `ProgramHandleInner::InThread` type change — no additional edit needed.

---

## Site counts migrated per file

### `src/thread_io.rs`
- Import: 1 site (line 28: `crossbeam_channel::{Receiver,Sender}` → `crate::typed_channel::{Receiver,Sender}`)
- Factory calls migrated: 6 sites (bounded::<StdInServiceEvent>, bounded::<String>, bounded::<StdOutServiceEvent>, bounded::<()> ×2, bounded::<StdErrServiceEvent>)
- Struct field types auto-migrated via import: `StdOutServiceEvent::Add.data_rx`, `StdOutServiceEvent::Add.ack_tx`, `StdErrServiceEvent::Add.data_rx`, `StdErrServiceEvent::Add.ack_tx`, `StdInServiceEvent::Add.data_rx`, `StdInServiceEvent::Add.reply_tx`, all 6 `ThreadIO` fields (6 fields)
- Explicitly qualified Group B (stayed bare crossbeam): `RuntimeServices` 3 fields, `unwrap_value_sender` return, `unwrap_value_receiver` return, `sender_value` param, `receiver_value` param, `extract_control_tx` return, `unwrap_receiver_for_orchestrator` return, bridge function 3×2 params, 8 factory calls for `bounded::<Value>`

### `src/runtime.rs`
- Field type migrated: 1 site — `ProgramHandleInner::InThread` field type (line 706)
- Factory call migrated: 1 site — line 17544 `crossbeam_channel::bounded::<SpawnOutcome>(1)` → `crate::typed_channel::bounded::<SpawnOutcome>(1)`
- Recv call sites inherited cascade-aware semantics: 5 sites (lines 17193, 17283, 17703, 17788, 18392) — all `ProgramHandleInner::InThread(rx) => match rx.recv()` patterns
- Send call site inherited cascade-aware semantics: 1 site (line 17633 `outcome_tx.send(outcome)`)
- NOT migrated: HandlePool (lines 504, 16861, 16863, 16906, 16951) — `len()` blocks

### `src/freeze.rs`
- Edits: 0 explicit mechanical edits
- Recv at line 1080 inherits cascade-aware semantics via ProgramHandleInner::InThread type change
- freeze.rs:1022 `spawn_service` return type stayed bare crossbeam (structural — extraction of inner sender)

### `src/spawn.rs`
- Factory call migrated: 1 site — line 181 `crossbeam_channel::bounded::<SpawnOutcome>(1)` → `crate::typed_channel::bounded::<SpawnOutcome>(1)`

### `src/typed_channel.rs` (honest delta — touched minimally)
- `#[derive(Debug)]` added to `Sender<T>` (line 541 region)
- `#[derive(Debug)]` added to `Receiver<T>` (line 568 region)

---

## `cargo build --release` output

```
   Compiling wat v0.1.0 (/home/watmin/work/holon/wat-rs)
warning: function `parse_fn_signature_for_check` is never used
warning: function `eval_kernel_process_send` is never used
warning: function `eval_kernel_process_recv` is never used
warning: function `process_died_error_entry_form_failure` is never used
warning: function `process_died_error_entry_form_failure_value` is never used
warning: `wat` (lib) generated 5 warnings
   Compiling wat-telemetry v0.1.0 ...
   Compiling wat-sqlite v0.1.0 ...
   Compiling wat-lru v0.1.0 ...
   Compiling wat-holon-lru v0.1.0 ...
   Compiling wat-telemetry-sqlite v0.1.0 ...
   Compiling wat-cli v0.1.0 ...
   Compiling with-loader-example v0.1.0 ...
   Compiling with-lru-example v0.1.0 ...
   Compiling interrogate-example v0.1.0 ...
   Compiling console-demo v0.1.0 ...
    Finished `release` profile [optimized] target(s) in 18.71s
```

All 5 warnings are pre-existing (identical to pre-χ-2 baseline). Zero new warnings. Zero errors.

---

## `cargo test --release --test probe_channel_primitive` output

```
running 3 tests
test probe_chi1_try_recv_empty_returns_empty ... ok
test probe_chi1_sender_drop_triggers_recv_err ... ok
test probe_chi1_unbounded_round_trip ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

3/3 PASS. ✓

---

## `cargo test --release --test probe_pidfd_primitive` output

```
running 2 tests
test pidfd_observes_signal_exit ... ok
test pidfd_observes_normal_exit ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

2/2 PASS. ✓

---

## Mode classification

**Mode A with honest deltas B and C.**

- All T-typed substrate channels (SpawnOutcome, StdInServiceEvent, StdOutServiceEvent, StdErrServiceEvent, String, ()) migrated to the cascade-aware wrapper — the primary mission of χ-2.
- Cascade primitives (SHUTDOWN_RX/TX_PTR/init) untouched.
- Dirty tree (fork.rs / spawn_process.rs) untouched.
- No workspace tests run.
- Criterion 12 (NO touch to typed_channel.rs): DELTA — minimal `#[derive(Debug)]` added, documented.
- Criterion 17 (ZERO bare crossbeam outside exclusion list): PARTIAL — Value-typed bridge channels + HandlePool + freeze.rs extraction helper stay bare for structural reasons, documented.

The cascade-completeness gap is structurally closed for:
- All `ProgramHandleInner::InThread` recv sites (5 in runtime.rs + 1 in freeze.rs via type inheritance)
- The SpawnOutcome outcome channel (spawn.rs + runtime.rs spawn-thread)
- All Rust-typed bridge channels (StdInServiceEvent, StdOutServiceEvent, StdErrServiceEvent, String, ())

Remaining bare crossbeam beyond exclusion list:
- HandlePool (needs `len()` — orchestrator decision for separate stone)
- Value-typed bridge channels (structural coupling with sender_from_crossbeam — would require bridge rearchitecture)
- freeze.rs line 1022 extraction helper (structural — returns inner crossbeam from SenderInner)

χ-3 (restricted_to wall) will surface the remaining bare crossbeam sites as explicit compiler-enforced violations, allowing the orchestrator to decide on a per-site basis.

---

## Cross-references

- BRIEF-213-CHI-2-MIGRATE-CALLER-SITES.md — work order
- EXPECTATIONS-213-CHI-2-MIGRATE-CALLER-SITES.md — predictions
- SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md — χ-1 baseline (wrapper minted at commit 0097ee3)
- `src/typed_channel.rs` — `#[derive(Debug)]` added to lines 541 + 568 region
