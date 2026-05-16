# Arc 170 Stone C2 REVISION SCORE — substrate-composition proof via real-spawn round-trip

**BRIEF:** `BRIEF-STONE-C2-REVISION.md`
**Predecessor SCORE (PARTIAL):** `SCORE-STONE-C2-PROCESSPEER.md` — preserved as historical record per `feedback_inscription_immutable`.

## Status: SHIPPED — 5/5 PASS

Mock retired; real subprocess round-trip composing existing primitives passes. Zero substrate additions (no new verbs, types, or structs). The user-facing IPC surface remains Stone D's `run-processes` bracket; this test is substrate-composition proof, not the user-facing pattern.

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `make_process_peer_for_test` retired from `src/typed_channel.rs`; no remaining callers | **YES** | `grep -rn "make_process_peer_for_test\|make_pipe_for_test" /home/watmin/work/holon/wat-rs/ --include='*.rs'` → 0 live hits (only matches are in `docs/arc/2026/05/170-program-entry-points/` historical artifacts + one stale `src/types.rs:1002` doc comment — see Honest deltas). The function body + the inner `make_pipe_for_test` helper are both removed from `src/typed_channel.rs`; file shrinks from 659 → 572 lines (-87 LOC, matching the LOC accounting in the predecessor SCORE). |
| B | Test file renamed to `tests/wat_process_peer_ipc_round_trip.rs`; concept-anchored | **YES** | `ls tests/wat_process_peer_ipc_round_trip.rs tests/wat_arc170_stone_c2_processpeer.rs` → new path exists, old path is absent. `git status --short` reports `D tests/wat_arc170_stone_c2_processpeer.rs` and `?? tests/wat_process_peer_ipc_round_trip.rs` (the latter shows as ?? because the original `AM` slot was filled by the deleted file). |
| C | T2 rewritten as real-spawn round-trip; T1 + T3 unchanged in behavior | **YES** | `tests/wat_process_peer_ipc_round_trip.rs` contains: `build_spawn_process_call` (line 70-83), `process_peer_round_trips_string_via_real_subprocess` (T2) invoking `spawn-process` + `Process/stdin` + `Process/stdout` + `Sender/from-pipe` + `Receiver/from-pipe` + `ProcessPeer/new` + `Process/println` + `Process/readln` + `Process/drain-and-join`. T1 (`process_peer_type_mints_in_both_parametric_orientations`) and T3 (`process_peer_is_client_side_only_no_server_variant_emitted`) keep the original assertions verbatim; only the function names dropped the `stone_c2_` prefix per the BRIEF's concept-naming guidance. |
| D | 3 new tests pass | **YES** | `cargo test --release -p wat --test wat_process_peer_ipc_round_trip` → `3 passed; 0 failed; 0 ignored`. Re-run for stability: same result. Per-test wall time: 0.02s total. |
| E | Workspace test failure count ≤ baseline | **YES** | `cargo test --release --workspace --no-fail-fast` → 4 failing tests in 4 targets: `lifeline_pipe_zero_orphans_across_100_trials` (flake, variance-sensitive — saw 0 fails on the immediately-following re-run), `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`. Identical set to the pre-revision baseline captured in `SCORE-STONE-C2-PROCESSPEER.md` § Workspace test count. Zero new failures attributable to this revision. New target `wat_process_peer_ipc_round_trip` adds +3 passes; the +3 passes from the deleted `wat_arc170_stone_c2_processpeer` target are conserved (same 3 concept-level tests). |

**5/5 PASS.**

## Honest deltas

### `ProcessPeer/new` auto-gen worked first try

The most-feared substrate-discovery risk per the BRIEF (STOP trigger #1) did not surface. The auto-synthesized constructor at `src/runtime.rs:1906` produced a `Value::Struct` tagged `:wat::kernel::ProcessPeer` with `[rx, tx]` fields exactly as the `Process/readln` + `Process/println` eval handlers expect at `src/runtime.rs:17427` + `:17486`. No constructor-verb mint pressure surfaced; the rejected reflex stayed rejected.

### `Sender/from-pipe` / `Receiver/from-pipe` typing flowed without surprise

Both verbs have no explicit type scheme registered in `src/check.rs` — they dispatch by keyword match at runtime in `src/runtime.rs:4489` + `:4492`. Eval composition is type-erased at the Value layer (both pipe-backed and crossbeam-backed receivers surface as `Value::wat__kernel__Receiver`), so the wat-source `let` that composes them feeds straight into `ProcessPeer/new` with no unification pressure. The arc112 slice 2b probe at `tests/arc112_slice2b_process_send_recv.rs:62-63` already established the wat-level type-check passes for the same composition shape inside a top-level `:user::main`; this revision exercises the same composition through `eval` against an embedded `let` form, which sidesteps freeze-time inference altogether (eval doesn't re-type-check ad-hoc let bodies built from `parse_one!`). STOP trigger #2 also did not fire.

### client/server variable naming was clean

The BRIEF mandated client/server (conversation roles) over child/parent (OS-tree). The test reads top-to-bottom with `server = spawn-process(...)`, `peer = ProcessPeer/new(rx-over-server-stdout, tx-over-server-stdin)`, `reply = (Process/readln peer)`. The role framing read naturally and made the data-flow direction obvious without resorting to explicit role-types — see T2's wat source block. No client/server-as-explicit-types surface pressure arose.

### Subprocess lifecycle: no gotcha — drain-and-join in the let does it

The server's `:user::main` does exactly one `readln` + one `println` and returns `:nil`. Once the client writes "hello" via `Process/println`, the server's `readln` unblocks; it `println`s the reply; the client `Process/readln` recovers it; then `Process/drain-and-join server` runs synchronously and returns `Ok(:())` because the child has already exited cleanly. No ordering bug surfaced — the natural let-binding order matched the protocol order. (Stone A's drain-and-join semantics — drain stdout + stderr to EOF before joining — are the safety net here; without it the client might race the child's pipe-close.)

### Test-infrastructure quirks: stderr-drain helper exists, never fired

The BRIEF prescribed mirroring the `wat_arc170_program_contracts.rs:308-323` stderr-drain pattern for the failure path. `drain_server_stderr` is implemented and wired into the `match eval(...)` panic arm. It was never invoked across the build + multiple test runs; the round-trip succeeded cleanly every time. Kept in place as a regression diagnostic — if a future change breaks the subprocess wiring, the panic message will carry the child's `#wat.kernel/ProcessPanics` EDN instead of an opaque `RuntimeError`.

### Verbose-is-honest verified

The three-step build at T2's wat source —
```
[rx       (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
 tx       (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
 peer     (:wat::kernel::ProcessPeer/new rx tx)]
```
— is verbose. It is also the entire point: the composition tree is visible. A `ProcessPeer/from-process` constructor verb would have compressed this to one call; the cost would be hiding precisely what this test exists to prove (that the substrate composes). The bracket macro (Stone D) will eventually compress it for user code, but for the substrate-composition proof the verbose form IS the proof.

### Stale doc comment in `src/types.rs:1002`

A doc-comment line at `src/types.rs:1002` references the now-retired helper:

> `// goes through the substrate-internal make_process_peer_for_test`

The BRIEF explicitly bars modifying the substrate Stone C2 implementation (`src/types.rs`, `src/check.rs`, `src/runtime.rs`); the in-scope change set is the helper retirement in `src/typed_channel.rs` plus the test-file rewrite. Surfacing the stale comment here per `feedback_assertion_demands_evidence` — orchestrator-side patch in the atomic commit is the right channel (one-line doc fix, no behavior change). NOT a STOP trigger: it's a stale comment, not a substrate gap.

### Workspace test count vs baseline

| Target | Pre-revision baseline | Post-revision | Delta |
|---|---|---|---|
| `wat_arc170_stone_c2_processpeer` (mock) | 3 passed / 0 failed | (file deleted) | -3 mock passes |
| `wat_process_peer_ipc_round_trip` (real-spawn) | (did not exist) | 3 passed / 0 failed | +3 real-spawn passes |
| `wat_arc170_stone_c1_threadpeer` (regression) | 3 / 0 | 3 / 0 | unchanged |
| Workspace failing targets | 4 (lifeline flake + 3 stable: totally_bogus + t6_spawn_process + startup_error) | 4 (same set; one re-run saw 3 — lifeline flake variance) | unchanged at baseline |

Net: **0 new failures; 0 new substrate additions; substrate net -87 LOC (helper retired); test file gains real-spawn proof + concept-anchored name + Stone D framing.**

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60-90 min | ~30 min (helper retirement + rename + T2 rewrite + build + test cycles) |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ baseline (4) | = baseline (4 — identical set; flake-variance gives 3 on some runs) |
| New test count | 3 | 3 |
| `ProcessPeer/new` auto-gen surprises | 0-2 | 0 |
| `Sender/from-pipe` typing surprises | 0-2 | 0 |
| Constructor-verb pressure surfaced | 0 (rejected by BRIEF) | 0 (no pressure arose) |
| Stale doc comments not in BRIEF scope | 0-1 | 1 (`src/types.rs:1002`, surfaced for orchestrator patch) |

## What's ready for Stone D

The substrate composes cleanly through pure wat source:
```
ProcessPeer/new
  (Receiver/from-pipe (Process/stdout server))
  (Sender/from-pipe   (Process/stdin  server))
```
Stone D's `run-processes` bracket macro emits exactly this pattern (plus N-fold tuple aggregation + drain-and-join cleanup). The substrate has every primitive it needs; the macro is pure expansion. ZERO additional substrate work is owed before Stone D begins.

## Lesson — first-reflex constructor pressure was 100% theoretical

The BRIEF's STOP trigger #4 ("Any urge to mint a constructor verb") was framed as the rejected-reflex compass. During the actual implementation the urge never surfaced. The composition reads naturally because the primitives compose: `Receiver/from-pipe` + `Sender/from-pipe` + `ProcessPeer/new` are three keyword calls in a `let`, each producing the exact Value shape the next consumes. The "ergonomic friction" that would have justified a constructor verb is invisible at this size; if it shows up at Stone D's bracket-macro level, the macro is the right home, not a new substrate verb. `feedback_no_new_types` held without strain.
