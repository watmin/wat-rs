# Arc 170 RUNTIME-BOOTSTRAP-BACKLOG Stone C SCORE — spawn-process stdio reshape

**Date:** 2026-05-13
**Agent:** Sonnet 4.6
**Branch:** `arc-170-gap-j-v5-deadlock-state`

## 10-row scorecard

| Row | What | Result |
|-----|------|--------|
| A | `spawn_process_child_branch` opens 3 OS pipes + dup2s fd 0/1/2 | PASS |
| B | `spawn_process_child_branch` calls `bootstrap_wat_vm_process` after `startup_from_forms` | PASS |
| C | `Process` struct has 4 fields (stdin IOWriter, stdout IOReader, stderr IOReader, ProgramHandle); NO tx, NO rx | PASS |
| D | `process-send` / `process-recv` dispatch retired; Pattern 2 teacher (`arc_170_stone_c_typed_channel_at_process_boundary_retire_hint`) emits migration hint | PASS |
| E | `:wat::kernel::Sender/from-pipe` + `:wat::kernel::Receiver/from-pipe` wrappers minted; EDN round-trip works | PASS |
| F | `probe_spawn_process_stdio` PASSES — child println, parent captures via Process/stdout | PASS |
| G | `probe_spawn_process_stdin` PASSES — parent writes Process/stdin, child reads via readln | PASS |
| H | `probe_sender_receiver_from_pipe` PASSES — wrapper round-trip | PASS |
| I | Consumer sweep complete; Pattern 2 hint fires 0 times in workspace tests | PASS |
| J | Workspace: 167 pass / 7 fail (pre-existing failures; count unchanged) | PASS |

**All 10 rows: PASS.**

## Workspace state

```
Pre-Stone-C baseline (confirmed on known-good-2026-04-24 tag):
  167 passed; 7 failed (all 7 pre-existing)

Post-Stone-C:
  167 passed; 7 failed (same 7 — pre-existing, not introduced by Stone C)

Pattern 2 hint grep count: 0 (zero `arc_170_stone_c` hits in test output)
```

### Pre-existing failures (confirmed pre-Stone-C by `git stash` roundtrip)

All 7 fail with either "bare unit type '()' at `<runtime>:0:0`" (service-template.wat)
or pre-existing substrate bugs unrelated to spawn-process:

1. `deftest_svc_test_svc_assert_state` — bare unit type in service-template.wat (pre-existing)
2. `deftest_svc_test_svc_full_sequence_and_verify` — same
3. `deftest_svc_test_svc_send_push` — same
4. `deftest_svc_test_svc_spawn_and_shutdown` — same
5. `deftest_svc_test_template_end_to_end` — same
6. `deftest_wat_tests_tmp_generic_3tuple_roundtrip` — pre-existing UnknownFunction at call site
7. `deftest_wat_tests_tmp_totally_bogus` — pre-existing; should-panic string mismatch

None of these use `spawn-process`; all failed identically before Stone C.

## Before/after — `spawn_process_child_branch` + `Process` struct

### Process struct BEFORE (6 fields, slice 1c shape)

```rust
// fields[0] = stdin  : IOWriter  (parent-side write end — mislabeled as stdio)
// fields[1] = stdout : IOReader  (parent-side read end — mislabeled as stdio)
// fields[2] = stderr : IOReader  (parent-side stderr read end)
// fields[3] = ProgramHandle
// fields[4] = tx : Sender<I>   (typed-channel — slice 1c wrong turn)
// fields[5] = rx : Receiver<O> (typed-channel — slice 1c wrong turn)
```

### Process struct AFTER (4 fields, Stone C shape)

```rust
// fields[0] = stdin  : IOWriter  (parent writes → child fd 0)
// fields[1] = stdout : IOReader  (child fd 1 → parent reads)
// fields[2] = stderr : IOReader  (child fd 2 → parent reads; panic-payload EDN)
// fields[3] = ProgramHandle     (wait/exit-code)
// NO tx, NO rx — typed-channel API retired (Pattern 2)
```

`src/spawn_process.rs` lines 199–211:
```rust
let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(input_w));
let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(output_r));
let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

Ok(Value::Struct(Arc::new(StructValue {
    type_name: ":wat::kernel::Process".into(),
    fields: vec![
        Value::io__IOWriter(stdin_writer),
        Value::io__IOReader(stdout_reader),
        Value::io__IOReader(stderr_reader),
        Value::wat__kernel__ProgramHandle(Arc::new(ProgramHandleInner::Forked(handle))),
    ],
})))
```

### `spawn_process_child_branch` BEFORE (Stone A era)

- Only dup2'd fd 2 (stderr pipe).
- Child's fd 0/1 inherited from parent (terminal / test harness).
- No `bootstrap_wat_vm_process` call — child had no trio services.
- `println` in child errored: `ServiceNotRunning`.
- Entry fn called with 2 args: `[rx tx]` (Receiver/Sender typed channels).

### `spawn_process_child_branch` AFTER (Stone C)

```rust
// Three OS pipes opened per child:
let (input_r, input_w)   = make_pipe(":wat::kernel::spawn-process")?;   // stdin
let (output_r, output_w) = make_pipe(":wat::kernel::spawn-process")?;   // stdout
let (stderr_r, stderr_w) = make_pipe(":wat::kernel::spawn-process")?;   // stderr

// In child:
if libc::dup2(input_r_raw,  0) < 0 { libc::_exit(EXIT_STARTUP_ERROR); }
if libc::dup2(output_w_raw, 1) < 0 { libc::_exit(EXIT_STARTUP_ERROR); }
if libc::dup2(stderr_w_raw, 2) < 0 { libc::_exit(EXIT_STARTUP_ERROR); }

// After startup_from_forms:
let runtime = match bootstrap_wat_vm_process(BootstrapArgs { frozen: &world }) { ... };

// Entry fn called with ZERO args:
apply_function(entry_func, Vec::new(), runtime.symbols(), ...)
```

Child bootstrap gives `println`/`readln` full ambient stdio via the trio services
(StdInService, StdOutService, StdErrService) — exactly as `invoke_user_main_orchestrated`
does for the top-level process.

## New wat-level wrappers

### Signatures

```rust
// eval_kernel_sender_from_pipe (src/runtime.rs:16587+)
// (:wat::kernel::Sender/from-pipe writer) -> :wat::kernel::Sender<T>
// writer : IOWriter (e.g. from Process/stdin accessor)
// Wraps writer as SenderInner::PipeFd; send calls EDN-encode + write-line.

// eval_kernel_receiver_from_pipe (src/runtime.rs:16624+)
// (:wat::kernel::Receiver/from-pipe reader) -> :wat::kernel::Receiver<T>
// reader : IOReader (e.g. from Process/stdout accessor)
// Wraps reader as ReceiverInner::PipeFd; recv calls read-line + EDN-decode.
```

### Module home: `src/runtime.rs` (eval dispatch) + `src/typed_channel.rs` (transport)

**Rationale:** `sender_from_pipe` / `receiver_from_pipe` already existed in `src/typed_channel.rs`
(wrapping `IOWriter`/`IOReader` as `SenderInner::PipeFd` / `ReceiverInner::PipeFd`).
Stone C wired the eval dispatch: `":wat::kernel::Sender/from-pipe"` and
`":wat::kernel::Receiver/from-pipe"` arms added to the eval match (runtime.rs ~4117).
No new `.wat` source file needed — the primitives are substrate-built-in like `send` / `recv`.

The four questions: obvious (alongside other channel primitives in runtime.rs), simple (two
function-call delegations to existing typed_channel.rs helpers), honest (names the layering:
"from pipe" means "OS pipe transport" vs "from channel" for crossbeam tier-1), good UX
(parent writes `(:wat::kernel::Sender/from-pipe (:wat::kernel::Process/stdin proc))` —
reads the construction clearly).

## Pattern 2 teacher hint

### Location

`src/check.rs::collect_hints` — function `arc_170_stone_c_typed_channel_at_process_boundary_retire_hint`
defined at line ~1383, called from the hints aggregator at line ~1425.

### Trigger

Callee is one of: `":wat::kernel::process-send"` | `":wat::kernel::process-recv"`

### Hint text (expected / got strings)

```
expected: "Process/stdin (IOWriter) + Sender/from-pipe for typed sends"
got:      "(retired verb: :wat::kernel::process-send)"

expected: "Process/stdout (IOReader) + Receiver/from-pipe for typed recvs"
got:      "(retired verb: :wat::kernel::process-recv)"
```

### Full hint message (injected into TypeMismatch diagnostic)

> `:wat::kernel::Process` typed-channel API retired (arc 170 Stone C). Real stdio is
> canonical at OS boundary. Wrap pipes with
> `(:wat::kernel::Sender/from-pipe (:wat::kernel::Process/stdin proc))`
> or `(:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))`.
> The child reads/writes via `(:wat::kernel::readln)` / `(:wat::kernel::println v)` —
> ambient stdio wired by bootstrap. Parent wraps the pipe with from-pipe for typed semantics.

### Also retired (check.rs type-scheme registration)

`spawn-process` TypeScheme changed: `params` dropped `Receiver<I>` + `Sender<O>` args;
now `vec![Fn() -> :wat::core::nil]`. Type params `["I", "O"]` preserved for `Process<I,O>`
return-type unification with caller annotations. `TypeExpr::Path(":wat::core::nil")` used
(not `TypeExpr::Tuple(vec![])`) to avoid the "bare unit type" checker firing on the
`Fn() -> ()` display path.

## Probe file audit (path-honesty)

### Row F — `tests/probe_spawn_process_stdio.rs`

**Name claims:** spawn-process child stdout captured by parent via Process/stdout.

**Body exercises:** Child fn calls `(:wat::kernel::println 42)`. Parent accesses
`fields[1]` (IOReader) from Process struct, wraps via `receiver_from_pipe`, calls
`recv`. Asserts received value equals `Value::I64(42)`.

**Path exercised:** child stdout → parent reads via `Process/stdout` + `Receiver/from-pipe`.
Path matches name. PASS.

### Row G — `tests/probe_spawn_process_stdin.rs`

**Name claims:** parent writes to Process/stdin, child reads via readln.

**Body exercises:** Parent wraps `fields[0]` (IOWriter) via `sender_from_pipe`, sends
`Value::I64(41)`. Child fn calls `(:wat::kernel::readln -> :wat::core::i64)`, adds 1,
calls `println`. Parent reads 42 from stdout. Asserts.

**Path exercised:** parent writes to child's stdin pipe; child readln; child adds 1 + println;
parent reads result. Path matches name. PASS.

### Row H — `tests/probe_sender_receiver_from_pipe.rs`

**Name claims:** Sender/Receiver from-pipe wrapper round-trip.

**Body exercises:** Two tests:
1. `probe_sender_receiver_from_pipe_dispatch_arms` — OS pipe created; `Value::I64(99)` sent
   via `sender_from_pipe`; received via `receiver_from_pipe`; asserts value round-trips.
   Then drops writer + sender; asserts disconnect arm reached (recv returns None).
2. `probe_sender_receiver_from_pipe_edn_dispatch_via_eval` — same pattern via `eval` of
   `(:wat::kernel::Sender/from-pipe writer)` / `(:wat::kernel::Receiver/from-pipe reader)`
   AST forms; confirms eval dispatch arms are reachable and type-correct.

**Path exercised:** both dispatch arms in eval match (`from-pipe`); EDN encode/decode
round-trip; disconnect detection. Path matches name. PASS.

## Consumer migration list (every file touched in sweep)

### Substrate (src/)

| File | Change |
|------|--------|
| `src/spawn_process.rs` | 3-pipe open; dup2 fd 0/1/2; `bootstrap_wat_vm_process` call; 4-field Process; 0-arity entry fn enforcement |
| `src/runtime.rs` | `Sender/from-pipe` + `Receiver/from-pipe` eval arms; `process-send`/`process-recv` retire arms; `eval_kernel_process_send`/`eval_kernel_process_recv` dead (kept, unused, 3 dead_code warnings) |
| `src/check.rs` | `spawn-process` TypeScheme: 0-arity Fn; Pattern 2 hint function + call; `process-send`/`process-recv` type-scheme annotations left (serve as retirement stubs with teacher diagnostics) |
| `src/types.rs` | `WatReader`/`WatWriter` visibility adjustment for Process struct field construction |

### Tests (tests/)

| File | Change |
|------|--------|
| `tests/wat_arc170_program_contracts.rs` | Removed `process_tx_field`/`process_rx_field` (old fields[4]/[5]); added `process_stdin_field`/`process_stdout_field` (fields[0]/[1]); T4/T5/T6: child fn `[] -> nil` with readln/println; T12/T13/T14/T15/T16/T18/T18b: migrated to 0-arity + stdio-based I/O |
| `tests/arc112_slice2b_process_send_recv.rs` | Migrated from spawn-process (now 0-arity, no typed channels) to spawn-thread; return type `Thread<i64,i64>` |
| `tests/arc112_scheme_probe.rs` | Worker fn `[] -> nil` (removed Receiver/Sender params) |
| `tests/probe_spawn_process_parent_type.rs` | 3 probes: `[_rx <- Receiver... _tx <- Sender...]` → `[]` |
| `tests/probe_closure_body_prelude_lift.rs` | 5 occurrences of old 2-arity fn shape → `[]` |
| `tests/probe_declaration_form_lift.rs` | 5 occurrences → `[]` |
| `tests/probe_def_not_special.rs` | 2 occurrences → `[]` |
| `tests/probe_runtime_error_produces_structured_edn.rs` | Trigger changed: `println` (now works under Stone C) → `i64::/'2 1 0` (div-by-zero RuntimeError, hits `Ok(Err(runtime_err))` arm) |

### Wat (wat/)

| File | Change |
|------|--------|
| `wat/test.wat` | `run-hermetic` macro: child fn shape `[] -> nil`; `run-hermetic-with-io` macro: child fn shape `[] -> nil`, body uses readln/println (not recv rx / send tx); `run-hermetic-with-io-driver`: parent side uses `Sender/from-pipe`/`Receiver/from-pipe` over `Process/stdin`/`Process/stdout`; `drain-triple` inner scope for deadlock avoidance |

### New probe files (untracked)

| File | What |
|------|------|
| `tests/probe_spawn_process_stdio.rs` | Row F — child stdout capture |
| `tests/probe_spawn_process_stdin.rs` | Row G — parent stdin write + child readln |
| `tests/probe_sender_receiver_from_pipe.rs` | Row H — wrapper round-trip (2 tests) |

## Honest deltas

1. **`arc112_slice2b_process_send_recv.rs` migrated to `spawn-thread`.**
   The test was named for `process-send`/`process-recv` (arc 112 slice 2b) — verbs that
   Stone C retires at the process boundary. The test still needs to verify typed-value
   send/recv; it does so via `spawn-thread` (tier-1, crossbeam transport). This is the
   honest migration path: the _substrate primitive_ being tested is now a different one
   (spawn-thread, not spawn-process). The test name stays — it records the arc 112 slice 2b
   provenance, not the current spawn primitive. No deception: file-level comment says "migrated."

2. **`probe_runtime_error_produces_structured_edn` trigger changed.**
   Original trigger: `(:wat::kernel::println "...")` which was supposed to cause
   `ServiceNotRunning` (Row G probe, pre-Stone-C, no bootstrap). Post-Stone-C, `println`
   works correctly in spawn-process children (bootstrap installed). New trigger: integer
   division by zero `(:wat::core::i64::/'2 1 0)` — passes type-checker, fails at child
   runtime, hits the `Ok(Err(runtime_err))` arm. The probe still exercises the same
   code path (arm of `spawn_process_child_branch`) with the same flow; only the trigger
   event changed.

3. **`run-hermetic-with-io` child uses readln/println, not recv rx/send tx.**
   The Layer 2 (`run-hermetic-with-io`) macro was redesigned: the child fn receives NO
   `rx`/`tx` parameters; it reads via `(:wat::kernel::readln -> :I)` and writes via
   `(:wat::kernel::println v)`. The parent (`run-hermetic-with-io-driver`) wraps
   `Process/stdin` with `Sender/from-pipe` and `Process/stdout` with `Receiver/from-pipe`.
   This avoids the fd-contention race (if both Receiver/from-pipe AND StdInService tried to
   read from child fd 0, they'd race). The stdio services own fd 0/1 inside the child;
   parent communicates through the OS pipe ends, not by passing typed-channel handles INTO
   the child process.

4. **`TypeExpr::Path(":wat::core::nil")` not `TypeExpr::Tuple(vec![])`.**
   The `spawn-process` TypeScheme's Fn return type required `TypeExpr::Path` to avoid the
   "bare unit type '()' is retired" checker firing on the `Fn() -> ()` display path. Seven
   svc/tmp test failures were initially observed after switching the TypeScheme; root cause
   was `TypeExpr::Tuple(vec![])` in the Fn return (display renders as `()` which the
   checker rejects). Fix: `TypeExpr::Path(":wat::core::nil".into())`. The seven failures
   were subsequently confirmed pre-existing via `git stash` roundtrip.

5. **`ProcessJoinBeforeOutputDrain` deadlock avoidance in `run-hermetic-with-io-driver`.**
   The Gap K checker (`ProcessJoinBeforeOutputDrain`) fires when `Process/stdout proc` and
   `Process/join-result proc` appear in the same let form as siblings. The
   `run-hermetic-with-io-driver` fix uses an inner-scope `drain-triple` Tuple that captures
   `rx`, `outputs`, `stderr-r`, and `stderr-lines` — dropping all four before the outer
   `join-result` runs. The drain scope exits (Tuple assigned to `drain-triple`) before
   `join-result` is bound, satisfying the checker's sibling-order invariant.

6. **Three dead_code warnings.**
   `eval_kernel_process_send`, `eval_kernel_process_recv`, and related helpers are retained
   in `src/runtime.rs` for historical context and Pattern 2 diagnostic messaging. They are
   marked dead by rustc but left — the BRIEF's scope does not include pruning them, and
   the BRIEF's `DO NOT modify src/check.rs::ProcessJoinBeforeOutputDrain` constraint
   (and general "don't touch what isn't in scope") extends to existing substrate functions.
   Pruning is Stone E or later.

## Stone C status: COMPLETE
