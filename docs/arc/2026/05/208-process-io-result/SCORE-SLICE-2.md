# SCORE — Arc 208 Slice 2: consumer ripple to honest match-on-Err

**Date:** 2026-05-17
**Executor:** sonnet (claude-sonnet-4-6)
**Commit:** PENDING (orchestrator commits atomically)

---

## Row A — Verification gate passed (baseline + grep + crash-test-proc shape understood)

**YES**

1. **Baseline git status:** `?? .claude/worktrees/` only. Clean.

2. **Baseline cargo test (pre-slice-2):** 4 pre-existing failures matching expected pool:
   - `lifeline_pipe_zero_orphans_across_100_trials` (pre-existing flake)
   - `deftest_wat_tests_tmp_totally_bogus` (intentional canary)
   - `t6_spawn_process_factory_with_capture_round_trips` (Stone D2 honest delta)
   - `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)

3. **Consumer grep result:** No Process/readln+println consumers outside the 4 known targets.
   - `src/types.rs:1033-1049` — comments only; no dispatch. Confirmed.
   - `tests/wat_arc208_process_io_result.rs` — already uses Result properly (slice 1 tests).
   - All 4 known consumer files confirmed.

4. **crash-test-proc shape understood:** `counter-service-process-N3.wat` lines 711-730 (pre-conversion).
   The helper spawns a fresh subprocess that panics immediately, then calls `Process/drain-and-join`
   to detect the abnormal exit. This tests the `drain-and-join` error path ONLY — it has no
   `Process/println` or `Process/readln` calls. Its purpose (per SCORE-SLICE-3F.md:42-44) was to work
   around the inability of main wrappers to catch transport errors. After slice 2, main wrappers CAN
   catch transport errors via match-on-Err. The `drain-and-join` path is orthogonal and retained (see Row F).

---

## Row B — `counter-service-process-N3.wat` wrappers propagate ServerDied via match-on-Err; Result/expect retired from main service wrappers

**YES**

**Files converted:** `wat-tests/counter-service-process-N3.wat`

**Pattern applied (per BRIEF § "Pattern to apply per Process I/O callsite"):**

Each wrapper that previously used `Result/expect` now uses nested `match`:
```scheme
(:wat::core::match (:wat::kernel::Process/println pr msg)
  -> :wat::core::Result<T,counter::ServiceError>
  ((:wat::core::Ok _)
    (:wat::core::match (:wat::kernel::Process/readln pr)
      -> :wat::core::Result<T,counter::ServiceError>
      ((:wat::core::Ok wire-resp)
        ;; existing wire-resp match body
        ...)
      ((:wat::core::Err chain)
        (:wat::core::Err (:counter::ServiceError::ServerDied chain)))))
  ((:wat::core::Err chain)
    (:wat::core::Err (:counter::ServiceError::ServerDied chain))))
```

**Wrappers converted (6 total):**

| Wrapper | Return type | println Err arm | readln Err arm |
|---|---|---|---|
| `provision-proc` | `Result<UserProc,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |
| `deprovision-proc` | `Result<nil,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |
| `stop-proc` | `Result<nil,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |
| `get-proc` | `Result<i64,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |
| `increment-proc` | `Result<i64,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |
| `reset-proc` | `Result<i64,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |
| `test-forge-proc-rejection` | `Result<nil,ServiceError>` | `Err(ServerDied chain)` | `Err(ServerDied chain)` |

**stop-proc structural note:** Preserves the inner/outer let pattern (SERVICE-PROGRAMS lockstep)
required to drop `pr` (ProcessPeer) before calling `Process/drain-and-join`. Inner-let now returns
`Result<Process<Wire,WireResp>,ServiceError>` instead of bare `Process`; outer matches on that
result before calling drain-and-join. Template from thread-tier `counter-service-capability-N3.wat`
`stop` function (lines 555-596).

**Test result:** `deftest_counter_service_process_N3` → PASS

**Diff summary:** ~130 lines changed (removals of Result/expect wrappers + additions of match arms;
net positive due to added Err arms + updated comments).

---

## Row C — `counter-actor-proof-process.wat` same conversion

**YES**

**Files converted:** `wat-tests/counter-actor-proof-process.wat`

**Pattern applied:** Wrappers return bare `i64` (no `ServiceError` type in this proof-of-concept).
Honest conversion: match-on-Err with `assertion-failed!` on Err arm. Semantically identical to
`Result/expect` (panic on transport failure) but structurally honest — Process I/O in match-value
position satisfies the walker.

**Honest delta from BRIEF:** BRIEF says "matching the same shape as counter-service-process-N3."
counter-service-process-N3 propagates `ServiceError::ServerDied`. counter-actor-proof-process has no
`ServiceError` type — wrappers return bare `i64`. The honest shape for bare-return wrappers:
`assertion-failed!` on Err (structurally honest; same panic semantics). Full `ServiceError`
propagation would require changing wrapper signatures and all callers — outside scope for this proof.

**Wrappers converted (4 total):** `counter-proc/get`, `counter-proc/increment`, `counter-proc/reset`,
`counter-proc/shutdown`. Each: match on println result → match on readln result → match on Response.

**Test result:** `deftest_counter_actor_process_proof` → PASS

---

## Row D — `wat_process_peer_ipc_round_trip.rs` Result/expect → honest match

**YES**

**File converted:** `tests/wat_process_peer_ipc_round_trip.rs`

The embedded wat string (T2 round-trip test) replaced `Result/expect` bindings with nested match:
```scheme
;; Before: let [_written (Result/expect (println peer "hello") "...")
;;              reply    (Result/expect (readln peer) "...")]
;; After:  match (println peer "hello") ->
;;           ((Ok _) match (readln peer) ->
;;             ((Ok reply) (let [_drained (drain-and-join server)] reply))
;;             ((Err _) assertion-failed! ...))
;;           ((Err _) assertion-failed! ...)
```

Rust-side match on `reply` value unchanged — still matches `Value::String`.

**Test result:** `process_peer_round_trips_string_via_real_subprocess` → PASS (all 3 T1/T2/T3 pass)

---

## Row E — `probe_counter_actor_process_diag.rs` same

**YES**

**File converted:** `tests/probe_counter_actor_process_diag.rs`

The embedded wat string (probe 3 — `probe_counter_subprocess_full_process_peer`) replaced 4
`Result/expect` calls (2 println + 2 readln for Increment and Shutdown round-trips) with nested
match structure. The final expression still returns `resp` (the Increment response) from the deepest
Ok arm. Rust-side match on `Value::Enum` unchanged.

**Test result:** All 3 probes pass (`probe_counter_subprocess_minimal`, `probe_counter_subprocess_with_defn`,
`probe_counter_subprocess_full_process_peer`)

---

## Row F — `crash-test-proc` helper retained; main service wrappers now demonstrate ServerDied via transport I/O path

**YES — RETAINED with rationale**

**Retention decision:** crash-test-proc is RETAINED because it tests a distinct failure mode.

**Rationale:**
- `crash-test-proc` spawns a fresh subprocess that panics, then calls `Process/drain-and-join` to
  detect the abnormal exit. It has NO `Process/println` or `Process/readln` calls.
- Its slice 3f purpose (SCORE-SLICE-3F.md:42-44): worked around inability of main wrappers to catch
  transport errors by using `drain-and-join` on a separate crashed subprocess.
- After slice 2: the main wrappers NOW demonstrate ServerDied via the transport I/O path
  (Process/println or Process/readln Err when subprocess dies mid-communication).
- `crash-test-proc` covers the DRAIN-AND-JOIN path: subprocess exits abnormally AFTER communication
  completes, detected via `Process/drain-and-join`. This is orthogonal to the transport I/O path.
- These are two distinct failure modes with distinct substrate paths; both have demonstration value.

**Comment updated:** `crash-test-proc` function block and test body step 11 updated to reflect
retained-for-drain-and-join-path rationale (not "workaround for missing transport error").

**ServiceError comment block updated:** Stale "panics on transport failure" note removed; three
ServerDied paths now documented (transport I/O, drain-and-join, crash-test-proc).

---

## Row G — Workspace baseline preserved (flaky pool only; NO new failures)

**YES**

Post-slice-2 `cargo test --release --workspace --no-fail-fast` results across 3 runs:

**Run 1 (immediately after conversion):**
- `lifeline_pipe_zero_orphans_across_100_trials` — FAILED (pre-existing)
- `deftest_wat_tests_tmp_totally_bogus` — FAILED (pre-existing canary)
- `t6_spawn_process_factory_with_capture_round_trips` — FAILED (pre-existing)
- `startup_error_bubbles_up_as_exit_3` — FAILED (pre-existing)
- `deftest_wat_tests_holon_lru_*` (8 failures) + `deftest_wat_lru_*` (4 failures) — appeared

**Investigation:** lru failures pass in isolation (`cargo test -p wat-lru` → 12/12 pass;
`cargo test -p wat-holon-lru` → 19/19 pass). These files contain NO `Process/readln` or
`Process/println` calls. These are pre-existing flakes under parallel load (noted in slice 1
SCORE row F: "2 additional flakes under full parallel load pass in isolation").

**Runs 2 and 3:** lru failures absent. Only the expected 4-failure pool.

**Verdict:** lru failures are pre-existing parallel-load flakes, NOT caused by slice 2 changes.
Baseline preserved.

---

## Row H — Walker rule does NOT fire on new code (all Process I/O in match arms)

**YES**

All converted Process/println and Process/readln calls are in `match` value-position (the subject
of a `match` form), which is the accepted position for `validate_comm_positions`. The walker fires
only on Process I/O in `do`-body or similar List contexts (T6/T7 prove this).

**Evidence:**
- `cargo test -p wat --test wat_arc208_process_io_result` → 7/7 pass including T6/T7 (walker
  fires correctly on forbidden positions; converted code does not trigger these)
- Full wat deftest suite: 183/183 pass (no `CommCallOutOfPosition` errors from converted code)
- Walker acceptance of match-value position confirmed by T2/T5 which exercise Process I/O in match
  arms and pass at check time

---

## Row I — Arc 203 slice 3f honest delta CLOSED

**YES**

**The delta (SCORE-SLICE-3F.md:32-44):**
> "process-tier user wrappers (get-proc, increment-proc, reset-proc, deprovision-proc) can only
> surface AccessDenied via Result; transport failure still panics."

**Status after slice 2:** CLOSED.

Every process-tier wrapper that calls `Process/println` or `Process/readln` now has explicit
`(Err chain)` arms that propagate `Err(ServiceError::ServerDied chain)`. Transport failure no longer
panics in the wrapper body — it surfaces as a typed `ServiceError::ServerDied` through the same
`Result<T, ServiceError>` return type that already carries `AccessDenied`.

**Cross-reference:** SCORE-SLICE-3F.md:32-44 named the gap. Arc 208 slice 1 flipped the substrate.
Arc 208 slice 2 closed the consumer side. The honest delta is now history.

**Demand 2 of arc 203 (DESIGN § "What arc 203 demands from upstream"):** SATISFIED. Arc 203 demand 2
was process-tier Result-bearing I/O. With slice 2 shipped, all process-tier service wrappers are
honest about transport failure. Arc 203 closure still awaits demand 1 (protocols arc — defservice
meta-form); demand 2 is closed.

---

## Honest deltas from BRIEF

1. **counter-actor-proof-process.wat uses assertion-failed! not ServerDied:** BRIEF says "matching
   the same shape as counter-service-process-N3." counter-actor-proof-process has no ServiceError
   type and wrappers return bare `i64`. Honest conversion uses `assertion-failed!` on Err — same
   panic behavior, structurally honest (match-value position). Full ServiceError propagation would
   require wrapper signature changes + caller updates, outside slice 2 scope for a proof-of-concept.

2. **lru flakes on first parallel run:** Not caused by slice 2. Pre-existing parallel-load flakes
   (documented in slice 1 SCORE row F). Pass in isolation.

3. **crash-test-proc retained (Mode B predicted):** EXPECTATIONS § "Mode B — crash-test-proc has
   secondary purpose (~15%)" predicted this. Drain-and-join isolation is the secondary value.

## Files touched (line diffs)

| File | Lines added | Lines removed | Net |
|---|---|---|---|
| `wat-tests/counter-service-process-N3.wat` | ~120 | ~80 | +40 |
| `wat-tests/counter-actor-proof-process.wat` | ~50 | ~35 | +15 |
| `tests/wat_process_peer_ipc_round_trip.rs` | ~18 | ~15 | +3 |
| `tests/probe_counter_actor_process_diag.rs` | ~25 | ~15 | +10 |
| `docs/arc/2026/05/208-process-io-result/SCORE-SLICE-2.md` | NEW | — | — |
