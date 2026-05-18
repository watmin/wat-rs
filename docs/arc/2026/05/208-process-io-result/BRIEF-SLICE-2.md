# BRIEF — Arc 208 Slice 2: consumer ripple to honest match-on-Err

**Predecessors:** Slice 1 SHIPPED at `44cde7b`. Process/readln + Process/println return `Result<_, Vector<ProcessDiedError>>`; walker enforces silent Process I/O illegal; 4 consumer files patched with `Result/expect` as option (a) preservation (panic-on-Err equivalent to pre-208).

**Scope: convert the 4 `Result/expect` sites to honest match-on-Err.** Closes the arc 203 slice 3f honest delta: process-tier wrappers will surface `ServerDied` via `ServiceError` directly through main service code paths; the `crash-test-proc` helper workaround retires.

## Targets

1. **`wat-tests/counter-service-process-N3.wat`** — THE primary target. Slice 3f of arc 203 named this honest delta:
   > *"process-tier user wrappers (get-proc, increment-proc, reset-proc, deprovision-proc) can only surface AccessDenied via Result; transport failure still panics."* (SCORE-SLICE-3F.md:36)
   
   Slice 2 closes that: each wrapper that calls `Process/println` or `Process/readln` matches the new `Result` and propagates `(:counter::ServiceError::ServerDied (chain ...))` on Err. The thread-tier wrappers already do this for `PeerDied`; process-tier mirrors the pattern.
   
   The `crash-test-proc` helper (used to demonstrate ServerDied independently in slice 3f) retires — main service wrappers now demonstrate ServerDied directly when subprocess dies.

2. **`wat-tests/counter-actor-proof-process.wat`** — arc 170-era counter actor proof. Convert Result/expect to honest handling matching the same shape as counter-service-process-N3.

3. **`tests/wat_process_peer_ipc_round_trip.rs`** — Rust-side IPC test. Likely simpler conversion (Rust pattern matching on Value::Result).

4. **`tests/probe_counter_actor_process_diag.rs`** — Rust diagnostic probe. Same shape as wat_process_peer_ipc_round_trip.

## Verification gate (sonnet's first action)

1. **Baseline.** `git status --short` clean. `cargo test --release --workspace --no-fail-fast 2>&1 | grep FAILED` records baseline. Expected: ~3-4 flaky failures from the pool (tmp_totally_bogus canary, t6, startup_error_exit_3, and one of {lifeline_pipe, ambient_stdio_println_string}).
2. **Grep for any Process/readln+println consumer NOT in the 4 known targets.** `grep -rn "Process/readln\|Process/println" --include="*.wat" --include="*.rs" . 2>/dev/null | grep -v "/target/" | grep -v ".claude/" | grep -v "src/runtime.rs\|src/check.rs"` — surface ANY hit outside the 4 known files + arc 208's own test file (`tests/wat_arc208_process_io_result.rs` — already uses Result properly).
3. **Confirm `crash-test-proc` shape in counter-service-process-N3.** Read the helper + its callers to understand what slice 2 retires and what replaces it.

## Pattern to apply per Process I/O callsite

Today (slice 1's option-a preservation):
```scheme
(:wat::core::Result/expect (:wat::kernel::Process/println peer msg))
```

After slice 2 (honest match-on-Err):
```scheme
(:wat::core::match (:wat::kernel::Process/println peer msg) -> :ResultType
  ((:wat::core::Ok _)
    ;; continue normal flow
    ...)
  ((:wat::core::Err chain)
    ;; surface ServerDied
    (:wat::core::Err (:counter::ServiceError/ServerDied chain))))
```

For the wat-tests counter service: each wrapper that returns `Result<T, :counter::ServiceError>` adds an `(Err (ServerDied chain))` propagation arm for the Process/println + Process/readln callsites. The thread-tier pattern (slice 3f's PeerDied propagation) is the template — mirror it at process tier with ServerDied.

For the Rust-side tests: pattern-match on `Value::wat__core__Result` to extract Ok/Err; on Err, assert or propagate the structured chain.

## ServerDied semantics

Per arc 203 slice 3f SCORE: `(ServerDied (chain :wat::core::Vector<wat::kernel::ProcessDiedError>))`. Field name `chain`, NOT `cause` (matches arc 113's widened error type).

`ServiceError::ServerDied` already exists in counter-service-process-N3 (slice 3f minted it). Slice 2 just adds the propagation arms; no new error type.

## Walker rule sanity check

Slice 1 added Process/readln + Process/println to `validate_comm_positions`. After slice 2's conversion, every Process I/O callsite SHOULD be in a `match` arm (the walker's accepted position). If any callsite ends up in an invalid position (e.g., a bare expression in a `do` body), the walker will fire at check time — that's the substrate teaching the migration is incomplete. Sonnet handles cleanly OR surfaces.

## Crash-test-proc retirement

Per arc 203 slice 3f SCORE: *"Since process-tier wrappers can't catch transport errors, the ServerDied Err path is demonstrated via a standalone helper `crash-test-proc` that spawns a fresh subprocess that panics, then calls `Process/drain-and-join` to detect the failure."* (SCORE-SLICE-3F.md:42-44)

After slice 2: the MAIN service wrappers can surface ServerDied via the natural error path. The crash-test-proc helper becomes redundant — slice 2 retires it. Tests that previously relied on crash-test-proc for the ServerDied path use the main service wrappers instead (e.g., spawn server, force-crash it via SIGKILL or by sending invalid input that triggers panic, observe ServerDied propagating through the next Process/println call).

If the standalone crash-test-proc test has value beyond ServerDied demonstration (e.g., testing drain-and-join in isolation), keep it; if its ONLY purpose was the ServerDied workaround per slice 3f, retire it. Sonnet's call.

## HARD constraints

- DO NOT touch `src/` (substrate is complete from slice 1)
- DO NOT touch `crates/wat-edn`, `crates/wat-telemetry`
- DO NOT amend arc 110/111/112 INSCRIPTIONs or arc 203 slice 3f SCORE
- DO NOT commit; orchestrator commits atomically
- DO NOT use `--no-verify` / `--no-gpg-sign`
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`

## STOP triggers

1. **Grep surfaces Process/readln+println consumer outside the 4 known targets** — surface; orchestrator decides extend-scope vs out-of-arc-208.
2. **Workspace baseline regresses** with NEW non-flake failures beyond the known pool.
3. **`ServiceError::ServerDied` doesn't compose** at some callsite (e.g., wrapper signature doesn't return `Result<_, ServiceError>`) — surface; may indicate a slice 1 patch missed something.
4. **Walker rule fires on slice 2 code** indicating the match-on-Err pattern doesn't satisfy `validate_comm_positions` — surface; orchestrator + sonnet refine the conversion pattern.
5. **crash-test-proc has tests/value beyond ServerDied workaround** that would lose coverage on retirement — surface; orchestrator decides keep vs retire.

## SCORE methodology

`docs/arc/2026/05/208-process-io-result/SCORE-SLICE-2.md` with these rows (atomic YES/NO):

| Row | Evidence |
|---|---|
| A — Verification gate passed (baseline + grep + crash-test-proc shape understood) | Each check + result inscribed |
| B — `counter-service-process-N3.wat` wrappers propagate ServerDied via match-on-Err; Result/expect retired from main service wrappers | Diff inscribed; test still passes |
| C — `counter-actor-proof-process.wat` same conversion | Diff inscribed; test still passes |
| D — `wat_process_peer_ipc_round_trip.rs` Result/expect → honest match | Diff inscribed; test passes |
| E — `probe_counter_actor_process_diag.rs` same | Diff inscribed; test passes |
| F — `crash-test-proc` helper retired (if applicable per slice 3f workaround framing); main service wrappers now demonstrate ServerDied | Code retirement or retention rationale inscribed |
| G — Workspace baseline preserved (flaky pool only; NO new failures) | cargo test output |
| H — Walker rule does NOT fire on new code (all Process I/O in match arms) | cargo test passes; no ProcessJoinBeforeOutputDrain or comm-position errors |
| I — Arc 203 slice 3f honest delta CLOSED: process-tier wrappers surface ServerDied directly through main code paths | Inscribed; cross-ref to SCORE-SLICE-3F.md:32-44 |

## Time-box

Predicted 60-90 min sonnet. Hard stop 105 min. Substantive consumer migration with semantic-preserving honest-Err handling. Smaller than slice 1 because substrate is settled; mechanical conversion guided by the thread-tier slice 3f template.

## On completion

Return summary: rows passed/failed, files touched (line diffs), crash-test-proc retention decision + rationale, any consumers found outside the 4 targets, walker firings observed (should be zero).

T-minus 0. Begin.
