# BRIEF — Arc 203 Slice 3b: dynamic Provision/Deprovision (registry mutation; N grows/shrinks)

**Phase:** Second stepping stone. Builds on slice 3a's foundation (Wire enum, select-based dispatch, admin/user routing) — adds dynamic registry: admin can Provision new users, Deprovision existing ones, server's select set grows/shrinks.

**Predecessor:** Slice 3a shipped at `d4d76b4` — Wire enum (`(Admin AdminReq) | (User UserReq)`), server selects on `Vec<Receiver<Wire>>`, admin/user protocols structurally separate.

**Successors:** Slice 3c wraps in capability structs (struct-restricted Admin + Client); 3d ships process variant over stdio; 3e closure.

## Goal

ONE wat-tests deftest proving:
- Admin can Provision N users dynamically (server mints client-id, creates user channel pair, registers in HashMap)
- Each user has its OWN per-user state (server tracks per-client-id state)
- Admin can Deprovision a specific user (server drops registry entry; user's recv sees Disconnect on next op)
- Server's select set rebuilds each iteration from `[admin-rx, *user-rxs-from-registry]`
- Auto-cleanup: when a user-rx returns Disconnected (user dropped their Sender), server drops that registry entry on its own
- Admin Stop returns final state of all live users (or last-active, depending on what's simplest)

## Required artifacts

### `wat-tests/counter-service-thread-N3.wat`

ONE deftest. Prelude declares + body exercises multi-user lifecycle.

**Prelude — extends 3a's protocol:**

```scheme
;; AdminReq grows from 3a's (Stop) to add Provision + Deprovision
(:wat::core::enum :counter::AdminReq
  (Provision   (initial :wat::core::i64))                      ;; admin requests new user; sends initial state
  (Deprovision (id :wat::core::String))                        ;; admin requests removal by client-id
  (Stop))

;; AdminResp grows: Provisioned carries user-side channel ends + client-id
(:wat::core::enum :counter::AdminResp
  (Provisioned (id  :wat::core::String)
               (tx  :wat::kernel::Sender<counter::Wire>)
               (rx  :wat::kernel::Receiver<counter::Wire>))    ;; user-side ends; admin hands them to the user
  (Deprovisioned (id :wat::core::String))
  (Stopped))

;; UserReq + UserResp unchanged from 3a
(:wat::core::enum :counter::UserReq
  (Get)
  (Increment (n :wat::core::i64))
  (Reset))

(:wat::core::enum :counter::UserResp
  (Value (v :wat::core::i64))
  (Ok    (v :wat::core::i64)))

;; Wire enum unchanged from 3a
(:wat::core::enum :counter::Wire
  (Admin (req :counter::AdminReq))
  (User  (req :counter::UserReq)))
```

**Server-side dispatch — extends 3a with registry:**

```scheme
;; Registry shape: HashMap<:String, (server-side-rx, server-side-tx, state)>
;; Each entry: client-id → (rx for incoming from this user, tx for outgoing
;; to this user, current state for this user's counter).

;; Server-side dispatch — recursive; takes admin channels + registry + next-id counter.
(:wat::core::defn :counter::dispatch
  [admin-rx <- :wat::kernel::Receiver<counter::Wire>
   admin-tx <- :wat::kernel::Sender<counter::AdminResp>
   registry <- :wat::core::HashMap<...>                        ;; sonnet picks the exact HashMap value-type
   next-id  <- :wat::core::i64]
  -> :wat::core::nil
  ;; Build select set: [admin-rx, *(map (.rx) (.values registry))]
  ;; Select; match on (idx, value)
  ;;   idx==0 admin: handle AdminReq variant; mutate registry; recur
  ;;   idx>0 user: look up which client-id via N-1 into registry keys; handle UserReq;
  ;;          send response on that user's tx; mutate state; recur
  ;;   value==Disconnected on user-idx: drop that registry entry; recur
  ;; Stop: send Stopped; return nil (thread exits)
  ...)
```

**Body — exercises multi-user lifecycle:**

```scheme
(:wat::core::let
  [(server-admin-tx, server-admin-rx, server-thread) (:counter::spawn)        ;; spawn server with empty registry
   
   ;; Provision 3 users with different initial states
   _p1   (:wat::kernel::send server-admin-tx (Admin (Provision 10)))
   resp1 (:wat::kernel::recv server-admin-rx)                                  ;; expect Provisioned id1 tx1 rx1
   ...
   _p2   (... Provision 100 ...)
   ...
   _p3   (... Provision 0 ...)
   ...
   
   ;; Each user does ops independently
   _u1   (send tx1 (User (Increment 5)))   resp-u1 (recv rx1)                  ;; expect Ok 15
   _u2   (send tx2 (User (Increment 50)))  resp-u2 (recv rx2)                  ;; expect Ok 150
   _u3   (send tx3 (User Get))             resp-u3 (recv rx3)                  ;; expect Value 0
   
   ;; Deprovision user 2
   _d2   (send server-admin-tx (Admin (Deprovision id2)))
   resp-d2 (recv server-admin-rx)                                              ;; expect Deprovisioned id2
   
   ;; User 1 + 3 still work
   _u1b  (send tx1 (User Get))   resp-u1b (recv rx1)                           ;; expect Value 15
   _u3b  (send tx3 (User (Reset))) resp-u3b (recv rx3)                         ;; expect Ok 0
   
   ;; Admin Stop
   _s    (send server-admin-tx (Admin Stop))
   resp-s (recv server-admin-rx)                                               ;; expect Stopped
   
   _join (Thread/drain-and-join server-thread)]
  ...)
```

## Scope (no substrate changes)

Pure consumer slice. Zero edits to src/. All artifacts in wat-tests/ + docs/.

Substrate primitives required (all confirmed shipped):
- `:wat::kernel::spawn-thread`, `select`, `send`, `recv`, `Sender<T>`, `Receiver<T>` (post-Stone-C3 honest naming)
- `:wat::core::HashMap` for registry
- Enums + match
- Per slice 3a: Wire enum pattern proven; scope-deadlock inner-let pattern proven; Thread/drain-and-join in outer scope

## Design choices (orchestrator-fixed; sonnet executes)

1. **Per-user state**: each user's counter is INDEPENDENT (registry holds per-client state). Demonstrates the architectural point (server tracks per-client state) more clearly than shared state.

2. **client-id generation**: use a monotonic counter in the server (no telemetry dep needed). Format: `:wat::core::String` like `"client-1"`, `"client-2"`. Server increments next-id on each Provision.

3. **Auto-cleanup on user Disconnect**: when select returns Disconnected on a user-rx, server drops that registry entry silently and continues. NO error to admin (admin can't tell user dropped vs Deprovision in this slice; that distinction comes later if needed).

4. **Stop semantics**: server returns Stopped (no final-state aggregation); per-user states are lost on Stop. Simple. Final-state aggregation can be a later concern if needed.

## STOP triggers

1. **HashMap value type with embedded Receiver<Wire>** — substrate may have issues storing channel-end values inside HashMap. If HashMap rejects Receiver as a value type, surface; we may need to use a Vector of structs instead, or restructure
2. **Building select Vec from HashMap values** — substrate may need explicit type annotation on the resulting Vec<Receiver<Wire>>; surface any issue
3. **AdminResp carrying Sender + Receiver values** — verify enum variants can hold channel-end values; surface if substrate rejects
4. **Workspace baseline regresses** beyond 3 pre-existing failures
5. **Scope-deadlock checker complains about the new registry's Senders** — registry holds N user-side-tx Senders (server-side for sending responses); the checker may flag these. Use inner-let or factored-fn pattern per slice 3a's experience

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- cwd anchor: `/home/watmin/work/holon/wat-rs/`; never in `.claude/worktrees/`.
- DO NOT touch src/ — pure consumer.
- DO NOT introduce struct-restricted Admin/Client yet (that's 3c).
- DO NOT introduce process variant yet (that's 3d).
- DO NOT use `--no-verify`.
- DO NOT mint new substrate primitives.

## Decay disclosure (orchestrator)

Per slice 3a SCORE, the Wire-enum approach is established and works. Slice 3b extends it with registry mutation; the select-on-Vec<Receiver<Wire>> shape continues. HashMap<String, ...> for registry is conventional but not exercised in prior counter proofs — slice 3b is the first to use registry-as-state in this pattern; honest deltas may surface there.

If sonnet finds HashMap-carrying-channel-values is fundamentally incompatible with the substrate, the fallback is Vector-of-records (each record a struct holding id + rx + tx + state). Surface and pivot if needed.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_thread_N3` builds clean |
| B | Provision returns user-side channel ends + client-id; multiple provisions yield distinct IDs | Test asserts ids differ + channels work independently |
| C | Per-user state independent (each user's increments affect only their own counter) | Test exercises 3 users with different operations; asserts each sees its own state |
| D | Deprovision drops a specific user; others continue | Test deprovisions user 2; users 1+3 still work; user 2's channel may be dropped or return Disconnected on its next op |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 60-90 min sonnet. Hard stop: 120 min.

## Workspace baseline (post-slice-3a commit `d4d76b4`)

3 pre-existing failures. Post-3b target: +1 passing deftest; 3 failures preserved.

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3B.md` per § SCORE methodology
2. Return final summary: rows passed/failed, workspace delta, file paths touched, honest deltas surfaced (especially registry-shape decisions), suggested BRIEF corrections for stones 3c-3d

You are launching now. T-minus 0.
