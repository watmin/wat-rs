# BRIEF — Arc 203 Slice 3a: server dispatch loop (thread, hardcoded N=1 user)

**Phase:** First stepping stone toward the server-pattern proofs (admin/user channel separation, multi-user provisioning, both transports). This stone proves the foundation: `:wat::kernel::select` fan-in works across admin + user receivers, server routes by index, admin/user protocols are distinct.

**Predecessor:** Stone C3 substrate honesty fix shipped at `cfdf3b9` — ProcessPeer/ThreadPeer field types now honestly named `:wat::kernel::Sender/Receiver`.

**Successors:** Stone 3b adds dynamic registry (Provision/Deprovision); Stone 3c wraps in capability structs; Stone 3d ships process variant; Stone 3e closure.

## Goal

ONE wat-tests deftest proving:
- Server actor selects across admin-rx + ONE user-rx
- Admin can Stop the server (returns Final state via the admin channel)
- User can Get/Increment/Reset (full counter protocol via the user channel)
- Admin and user channels carry STRUCTURALLY-DIFFERENT enum types (admin can't send UserReq; user can't send AdminReq — type system enforces)

No capability structs yet (bare channels). No provisioning yet (N=1 hardcoded). No process variant yet. Pure thread-tier foundation.

## Required artifacts

### `wat-tests/counter-service-thread-N1.wat`

ONE deftest. Prelude declares + body exercises.

**Prelude — protocol enums + server dispatch + admin + user:**

```scheme
;; Admin protocol (privileged operations)
(:wat::core::enum :counter::AdminReq
  (Stop))                                                     ;; for 3a: only Stop; 3b adds Provision/Deprovision

(:wat::core::enum :counter::AdminResp
  (Stopped (final :wat::core::i64)))                          ;; carries server's final state

;; User protocol (RPC operations)
(:wat::core::enum :counter::UserReq
  (Get)
  (Increment (n :wat::core::i64))
  (Reset))

(:wat::core::enum :counter::UserResp
  (Value (v :wat::core::i64))
  (Ok    (v :wat::core::i64)))

;; Server-side dispatch loop
;; Takes: admin-rx (Receiver<AdminReq>), admin-tx (Sender<AdminResp>),
;;        user-rx (Receiver<UserReq>), user-tx (Sender<UserResp>),
;;        state (i64).
;; Builds select set [admin-rx, user-rx]; idx==0 admin, idx==1 user.
;; Stop → send Final + return nil (thread exits). UserReq → respond + recur.
(:wat::core::defn :counter::dispatch
  [admin-rx <- :wat::kernel::Receiver<counter::AdminReq>
   admin-tx <- :wat::kernel::Sender<counter::AdminResp>
   user-rx  <- :wat::kernel::Receiver<counter::UserReq>
   user-tx  <- :wat::kernel::Sender<counter::UserResp>
   state    <- :wat::core::i64]
  -> :wat::core::nil
  ;; ... select + match idx + match value + tail-call self with new state ...
  )

;; Server spawner — creates both channel pairs, spawns thread, returns
;; (Tuple admin-tx admin-rx-for-caller user-tx-for-caller user-rx)
;; (or simpler: returns a 4-tuple of the parent-side ends; substrate detail)
(:wat::core::defn :counter::spawn
  [initial <- :wat::core::i64]
  -> :wat::core::Tuple<
       :wat::kernel::Sender<counter::AdminReq>,
       :wat::kernel::Receiver<counter::AdminResp>,
       :wat::kernel::Sender<counter::UserReq>,
       :wat::kernel::Receiver<counter::UserResp>>
  ;; ... create channel pairs, spawn-thread with server-side ends + initial,
  ;; return client-side ends ...
  )
```

**Body — exercises the proof:**

```scheme
(:wat::core::let
  [chans            (:counter::spawn 10)
   admin-tx         (:wat::core::Tuple/0 chans)
   admin-rx         (:wat::core::Tuple/1 chans)
   user-tx          (:wat::core::Tuple/2 chans)
   user-rx          (:wat::core::Tuple/3 chans)
   ;; User exercises domain
   _i               (:wat::kernel::send user-tx (:counter::UserReq::Increment 5))
   resp-inc         (:wat::kernel::recv user-rx)   ;; expect Ok 15
   ;; ... assertions ...
   _g               (:wat::kernel::send user-tx (:counter::UserReq::Get))
   resp-get         (:wat::kernel::recv user-rx)   ;; expect Value 15
   ;; Admin stops
   _s               (:wat::kernel::send admin-tx (:counter::AdminReq::Stop))
   resp-stop        (:wat::kernel::recv admin-rx)] ;; expect Stopped 15
  ;; assertions on the responses
  ...)
```

(Above is rough sketch — sonnet finalizes the exact recv/send wrapping per arc 110/111 `Result<Option<T>>` discipline. Use `option::expect` / `result::expect` per established patterns.)

## Scope (no substrate changes)

Pure consumer slice. Zero edits to src/. All artifacts in wat-tests/ + docs/.

Substrate primitives required (all confirmed shipped):
- `:wat::kernel::spawn-thread` (arc 114)
- `:wat::kernel::select` (uniform-T fan-in returning `(idx, Result<Option<T>, ThreadDiedError>)`)
- `:wat::kernel::send` / `:wat::kernel::recv` (arc 110/111 — return Result<Option<T>>)
- `:wat::core::enum` declarations + match
- `:wat::core::Tuple` for the spawn return shape
- `:wat::kernel::Sender<T>` / `:wat::kernel::Receiver<T>` (post-Stone-C3 honest naming)

## STOP triggers

1. **`select` doesn't accept different Receiver T at the same call** — slice 2 SCORE Delta 1 (`select` is uniform-T per src/check.rs:13810). The admin and user channels are DIFFERENT types (Receiver<AdminReq> vs Receiver<UserReq>). If select rejects mixed types, this stone needs to use a UNIFIED enum at the wire (top-level `(Wire (Admin AdminReq) | (User UserReq))`) and the prelude shape changes — surface this as the first STOP and we re-spec
2. **Workspace baseline regresses** beyond 3 pre-existing failures
3. **Recursive defn type-check fails on the dispatch signature** — unlikely (Counter actor proofs already use this pattern); surface if it fires

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- cwd anchor: `/home/watmin/work/holon/wat-rs/`; never in `.claude/worktrees/`.
- DO NOT touch src/ — pure consumer.
- DO NOT introduce struct-restricted, Client/Admin capabilities, or Provisioning yet — those are stones 3b/3c.
- DO NOT add the process variant — that's stone 3d.
- DO NOT mint new substrate primitives.
- DO NOT use `--no-verify`.

## Decay disclosure (orchestrator)

`:wat::kernel::select`'s signature per src/check.rs:13802-13823: `∀T. Vec<Receiver<T>> -> :(i64, Result<Option<T>, ThreadDiedError>)`. Uniform T per the type scheme. If this stone needs heterogeneous select (admin and user different types), STOP 1 fires and we pivot to the unified-Wire-enum shape.

The honest pattern if STOP 1 fires:
```scheme
(:wat::core::enum :counter::Wire
  (Admin (req :counter::AdminReq))
  (User  (req :counter::UserReq)))
```
Server selects on `Vec<Receiver<Wire>>`; matches Wire variants; routes to admin or user dispatch. Admin and user clients each get their own `Sender<Wire>` but the type system doesn't enforce which variants they can construct — protocol enforces at receipt. That's honest with arc 198/203's "behavior enforces, not type system" lesson.

If STOP 1 fires, sonnet should surface the choice (heterogeneous → BRIEF retry; unified → adopt and proceed).

## SCORE methodology

4 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_thread_n1` builds clean |
| B | User round-trip succeeds (Increment, Get, Reset all assert correctly) | Test passes |
| C | Admin Stop succeeds with Final state | Test passes; received `Stopped <state>` matches expected |
| D | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 45-75 min sonnet. Hard stop: 90 min.

## Workspace baseline (post-Stone-C3 commit `cfdf3b9`)

3 pre-existing failures (deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips). Post-3a target: +1 passing deftest; 3 failures preserved.

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3A.md` per § SCORE methodology
2. Return final summary: rows passed/failed, workspace delta, file paths touched, honest deltas (especially if STOP 1 fired — unified Wire enum chosen instead of heterogeneous select), suggested BRIEF corrections for stones 3b-3d

You are launching now. T-minus 0.
