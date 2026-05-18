# SCORE — Arc 208 Slice 1: substrate audit + Result flip

**Date:** 2026-05-17  
**Executor:** sonnet (claude-sonnet-4-6)  
**Commit:** PENDING (orchestrator commits atomically)

---

## Row A — Verification gate passed (5 checks)

**YES**

1. **Baseline git status:** `?? .claude/worktrees/` only. Clean.

2. **Substrate state confirmed (file:line):**
   - `src/check.rs:13402-13452` — `Process/readln` registered as `[ProcessPeer<I,O>] -> :I` (bare); `Process/println` registered as `[ProcessPeer<I,O>, :O] -> :()` (bare). Both confirmed pre-flip.
   - `src/runtime.rs:4782-4787` — dispatch arms `":wat::kernel::Process/readln"` → `eval_kernel_process_readln` and `":wat::kernel::Process/println"` → `eval_kernel_process_println`.
   - `src/runtime.rs:17989-18037` — `eval_kernel_process_readln`: `RecvOutcome::Value(v) => Ok(v)` (bare, not Result-wrapped); `Disconnected|Shutdown => Err(RuntimeError::ChannelDisconnected)` (panics).
   - `src/runtime.rs:18049-18091` — `eval_kernel_process_println`: `SendOutcome::Ok => Ok(Value::Unit)` (bare); `SendOutcome::Disconnected => Err(RuntimeError::ChannelDisconnected)` (panics).
   - `src/runtime.rs:18093-18108` — `eval_process_peer_struct` shared unwrap helper (unchanged; not in scope).

3. **ProcessDiedError grep:** Confirmed at `src/types.rs:632` (struct mint), `src/check.rs:13174-13178` (Vector<ProcessDiedError> chain type used by drain-and-join + join-result), `src/runtime.rs:18574-18688` (builder functions: `process_died_error_channel_disconnected`, `process_died_error_panic`, `process_died_error_runtime`, etc.). `single_died_chain` at `src/runtime.rs:18541` is the chain builder used by Process/drain-and-join Err arms — confirmed reusable for readln/println.

4. **Arc 111 precedent grep:** Thread-tier shape at `src/check.rs:13613-13650`:
   - `Receiver/recv` → `Result<Option<T>, Vector<ThreadDiedError>>` (using `comm_ok_option_t()`)
   - `Sender/send` → `Result<(), Vector<ThreadDiedError>>` (using `comm_send_ret()`)
   - Runtime: `RecvOutcome::Disconnected → Ok(Value::Result(Arc::new(Ok(Value::Option(Arc::new(None))))))` (clean close = Ok(None)); `SendOutcome::Disconnected → Ok(Value::Result(Arc::new(Err(single_died_chain(thread_died_error_channel_disconnected())))))`.

5. **Consumer grep:**
   - `wat-tests/counter-service-process-N3.wat` — 7 call pairs (14 verb calls total)
   - `wat-tests/counter-actor-proof-process.wat` — 4 call pairs (8 verb calls total)
   - `tests/wat_process_peer_ipc_round_trip.rs` — 1 call pair embedded in wat string
   - `tests/probe_counter_actor_process_diag.rs` — 2+2 calls embedded in wat string
   - `src/types.rs:1033-1049` — comments only (no dispatch)
   Total: 4 consumer files with actual call sites.

---

## Row B — Process/readln return-type sub-decision settled

**YES — Decision: `Result<:I, :Vector<ProcessDiedError>>` (plain Result, no Option wrapper)**

**Audit evidence:**

At the PipeFd transport (`src/typed_channel.rs:324-386`): `reader.read_line()` returns `Ok(None)` on clean EOF → `RecvOutcome::Disconnected`. The pre-arc-208 `eval_kernel_process_readln` maps BOTH `Disconnected` and `Shutdown` to `Err(RuntimeError::ChannelDisconnected)`. There is NO path that returns `Ok(None)` at the process tier.

The thread-tier `Receiver/recv` uses `Option` because clean-sender-drop (all Senders drop, channel drains cleanly) is a valid lifecycle distinct from thread panic. The Sender can drop without the thread dying. At the process tier, the subprocess's stdout EOF is produced when: (1) subprocess exits (all cases: clean, panic, kill), or (2) subprocess explicitly closes stdout. Case (2) is unusual and from the parent's perspective semantically equivalent to "subprocess is done communicating" = "no more data = subprocess death for communication purposes."

Arc 170's lifeline-pipe + FD-multiplex mechanism detects subprocess death deterministically. There is no "subprocess closed stdout cleanly while continuing to run and doing useful work" pattern in the current substrate. The clean EOF == subprocess death equivalence holds.

**Four-questions verdict:**
- **Obvious?** YES — `Result<:I, ...>` maps to thread-tier `Sender/send` shape (one output variant); no Option level of indirection.
- **Simple?** YES — 1 less wrapper level than `Result<Option<:I>, ...>`; no `Option/expect` gymnastics for callers.
- **Honest?** YES — the substrate genuinely does not distinguish "clean EOF, subprocess still alive" from "subprocess exited" via the current transport.
- **Good UX?** YES — callers pattern-match on `(Ok v)` / `(Err chain)` rather than `(Ok (Some v))` / `(Ok None)` / `(Err chain)`.

---

## Row C — `Process/readln` flipped to Result-returning

**YES**

**Type scheme change (`src/check.rs:13435-13444`):**
```rust
// BEFORE: ret: TypeExpr::Path(":I".into())
// AFTER:
ret: TypeExpr::Parametric {
    head: "wat::core::Result".into(),
    args: vec![TypeExpr::Path(":I".into()), proc_io_err_chain_ty()],
},
```
Where `proc_io_err_chain_ty()` = `Vector<wat::kernel::ProcessDiedError>`.

**Eval handler change (`src/runtime.rs:17989-18051`, rewritten):**
```rust
// BEFORE:
RecvOutcome::Value(v) => Ok(v),
Disconnected | Shutdown => Err(RuntimeError::ChannelDisconnected { ... })

// AFTER:
RecvOutcome::Value(v) => Ok(Value::Result(Arc::new(Ok(v)))),
Disconnected | Shutdown => Ok(Value::Result(Arc::new(Err(single_died_chain(
    process_died_error_channel_disconnected(),
)))))
```
EDN decode errors remain as `RuntimeError::MalformedForm` (parse failure, not transport failure; no recovery path for caller).

---

## Row D — `Process/println` flipped to Result<:nil, :Vector<ProcessDiedError>>

**YES**

**Type scheme change (`src/check.rs:13444-13462`):**
```rust
// BEFORE: ret: TypeExpr::Tuple(vec![])
// AFTER:
ret: TypeExpr::Parametric {
    head: "wat::core::Result".into(),
    args: vec![TypeExpr::Tuple(vec![]), proc_io_err_chain_ty()],
},
```

**Eval handler change (`src/runtime.rs:18049-18099`, rewritten):**
```rust
// BEFORE:
SendOutcome::Ok => Ok(Value::Unit),
SendOutcome::Disconnected => Err(RuntimeError::ChannelDisconnected { ... })

// AFTER:
SendOutcome::Ok => Ok(Value::Result(Arc::new(Ok(Value::Unit)))),
SendOutcome::Disconnected => Ok(Value::Result(Arc::new(Err(single_died_chain(
    process_died_error_channel_disconnected(),
)))))
```

---

## Row E — New tests `tests/wat_arc208_process_io_result.rs`

**YES**

7 test cases, all pass:

| Test | Covers |
|---|---|
| `arc208_t1_process_readln_println_registered_as_result_returning` | Type-scheme registration via `CheckEnv::with_builtins()` — both verbs mention Result + ProcessDiedError |
| `arc208_t2_process_println_and_readln_return_ok_on_live_peer` | Happy path: echo server round-trip; println returns `Ok(nil)`, readln returns `Ok(String)` |
| `arc208_t3_process_println_returns_err_on_dead_peer` | Err path for println: dead peer returns `Err(chain)` not RuntimeError panic |
| `arc208_t4_process_readln_returns_err_on_dead_peer` | Err path for readln: dead peer returns `Err(chain)` not RuntimeError panic |
| `arc208_t5_err_chain_head_is_channel_disconnected` | Chain content: head is `ProcessDiedError::ChannelDisconnected` variant |
| `arc208_t6_walker_rejects_process_println_in_body_position` | Walker fires `CommCallOutOfPosition` for Process/println in `do`-body (forbidden position) |
| `arc208_t7_walker_rejects_process_readln_in_body_position` | Walker fires `CommCallOutOfPosition` for Process/readln in `do`-body (forbidden position) |

`cargo test --release -p wat --test wat_arc208_process_io_result` → 7/7 pass.

---

## Row F — Workspace baseline preserved (≤4 pre-existing failures)

**YES**

`cargo test --release --workspace --no-fail-fast` result:
- `lifeline_pipe_zero_orphans_across_100_trials` — FAILED (pre-existing)
- `deftest_wat_tests_tmp_totally_bogus` — FAILED (pre-existing, intentional "should panic" test)
- `t6_spawn_process_factory_with_capture_round_trips` — FAILED (pre-existing)
- `startup_error_bubbles_up_as_exit_3` — FAILED (pre-existing)

Zero new failures. 183 wat-tests pass (deftest_counter_service_process_N3 + deftest_counter_actor_process_proof both pass after consumer patches).

---

## Row G — Walker rule decision

**YES — Walker rule included in slice 1 (not deferred to slice 2)**

**Decision rationale:** Adding `":wat::kernel::Process/readln"` and `":wat::kernel::Process/println"` to the `matches!` list in `validate_comm_positions` (`src/check.rs:2152-2168`) was a 2-line change. The walker covers WatAST::List nodes; call sites in `do`-body, function arguments, and direct expression positions are covered. Let-binding RHS inside WatAST::Vector nodes are NOT reached by the walker (same limitation as thread-tier send/recv — this is the existing design contract, not a gap to fix here).

The walker fires atomically with the flip — after arc 208, Process/readln or Process/println in a `do`-body (or similar List context) without match/Result-expect is a compile-time error. This is the exact discipline arc 110 established for thread-tier send/recv.

T6 and T7 verify the walker fires for the `do`-body forbidden position.

---

## Row H — Consumer count from grep (for slice 2 BRIEF planning)

**YES — 4 consumer files documented**

| File | Call sites | Nature | Slice 1 patch |
|---|---|---|---|
| `wat-tests/counter-service-process-N3.wat` | 7 println + 7 readln = 14 total | Arc 203 process-tier demo | `Result/expect` wrapping (option a: panic-on-Err, semantically equivalent to pre-208) |
| `wat-tests/counter-actor-proof-process.wat` | 4 println + 4 readln = 8 total | Arc 170 process-tier proof | `Result/expect` wrapping (same) |
| `tests/wat_process_peer_ipc_round_trip.rs` | 1 println + 1 readln (embedded wat) | Arc 170 Stone C2 test | `Result/expect` wrapping |
| `tests/probe_counter_actor_process_diag.rs` | 2 println + 2 readln (embedded wat) | Diagnostic probe | `Result/expect` wrapping |

**Slice 2 targets:** Replace `Result/expect` wrappers with proper `(match ... ((:Ok v) ...) ((:Err chain) (:Err (ServiceError::ServerDied chain))))` arms — this is the full honest-Result consumer ripple that unlocks the "transport failure surface as ServiceError::ServerDied without crash-test-proc workaround" value.

---

## Honest deltas from BRIEF

1. **Walker in slice 1 (not deferred):** BRIEF listed this as "conditional — if trivial." It was trivial (2-line addition to the `matches!` list). Absorbed in slice 1 for atomic substrate honesty per the BRIEF's own guidance.

2. **Consumer patches (option a, minimal):** BRIEF hard constraint said "DO NOT touch arc 203 demos"; EXPECTATIONS explicitly overrode this with option (a) as acceptable to keep workspace green. All 4 consumer files patched with `Result/expect` (panic-on-Err) — semantically equivalent to pre-208 behavior. Slice 2 will convert to honest error propagation.

3. **T6/T7 test shape:** Initial attempt used let-binding position. Walker doesn't reach into `WatAST::Vector` binding nodes (documented design). Fixed to `do`-body position which is a WatAST::List child → correctly caught. Honest delta: walker has the same let-binding coverage gap for Process/readln/println as it does for thread-tier send/recv.
