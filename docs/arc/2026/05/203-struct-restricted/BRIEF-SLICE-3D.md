# BRIEF — Arc 203 Slice 3d: process variant (Wire enum over stdio)

**Phase:** Fourth stepping stone. Same architecture as slice 3c (admin/user capability separation, dynamic Provision/Deprovision, per-user state) but over a SUBPROCESS communicating via stdio instead of in-process crossbeam channels.

**Predecessor:** Slice 3c shipped at `e7aa671` — capability structs (Admin + Client) wrap 3b's multi-user lifecycle; struct-restricted enforces ctor + field access; behavior enforces Wire variant routing.

**Successors:** Slice 3e closure paperwork.

## Goal

ONE wat-tests deftest proving the same observable shape as 3c but over the **process tier**:
- Server runs as a subprocess (spawn-process); communicates via ambient stdio
- Admin holds a `:counter::AdminProc` capability wrapping the parent-side ProcessPeer (parent → subprocess)
- Each user holds a `:counter::ClientProc` capability — references the SAME ProcessPeer (Arc-clone) + per-client identity
- All messages multiplex on the single ProcessPeer stream via `:counter::Wire` (request) + `:counter::WireResp` (response) — Admin/User variants tagged
- Sequential request-response (test body single-threaded; no concurrent demux needed)
- Same body observations as 3c (provision 3 clients; each independent ops; deprovision one; admin Stop)

## Required artifacts

### `wat-tests/counter-service-process-N3.wat`

ONE deftest. Parent-side declarations + inline subprocess program + capability wrappers + body.

**Parent-side prelude — protocol + capabilities:**

```scheme
;; Wire enum (parent → subprocess) — UNCHANGED conceptually from 3a/3b/3c
;; but at process tier serialized as line-delimited EDN over stdio
(:wat::core::enum :counter::Wire
  (Admin (req :counter::AdminReq))
  (User  (id :wat::core::String) (req :counter::UserReq)))    ;; client-id in User wire — server needs to route

;; WireResp enum (subprocess → parent) — NEW for process tier
;; (in 3c each channel had its own response type; at process tier all responses
;; share one wire so we tag the response variant by category)
(:wat::core::enum :counter::WireResp
  (Admin (resp :counter::AdminResp))
  (User  (resp :counter::UserResp)))

;; AdminReq + AdminResp + UserReq + UserResp UNCHANGED from 3c
;; (Provision needs initial state; Deprovision needs client-id; Stop;
;;  AdminResp::Provisioned now returns ONLY the id since there are no
;;  per-user channels at process tier; client constructed parent-side
;;  via :counter::provision wrapping the SHARED ProcessPeer)
(:wat::core::enum :counter::AdminReq
  (Provision   (initial :wat::core::i64))
  (Deprovision (id :wat::core::String))
  (Stop))

(:wat::core::enum :counter::AdminResp
  (Provisioned (id :wat::core::String))                        ;; just the id; no channels at process tier
  (Deprovisioned (id :wat::core::String))
  (Stopped))

;; User enums verbatim from 3c
(:wat::core::enum :counter::UserReq (Get) (Increment (n :wat::core::i64)) (Reset))
(:wat::core::enum :counter::UserResp (Value (v :wat::core::i64)) (Ok (v :wat::core::i64)))

;; Capability structs — wrap shared ProcessPeer + identity
(:wat::core::struct-restricted :counter::AdminProc
  [:counter::]
  ([:counter::] server-id <- :wat::core::String
   [:counter::] peer!     <- :wat::kernel::ProcessPeer<counter::WireResp, counter::Wire>
   [:counter::] proc!     <- :wat::kernel::Process<counter::Wire, counter::WireResp>)  ;; needed for drain-and-join in stop
  ())

(:wat::core::struct-restricted :counter::ClientProc
  [:counter::]
  ([:counter::] server-id <- :wat::core::String
   [:counter::] client-id <- :wat::core::String
   [:counter::] peer!     <- :wat::kernel::ProcessPeer<counter::WireResp, counter::Wire>)  ;; shared peer; Arc-clone of admin's
  ())
```

**Subprocess program (constructed inline in `:counter::spawn-proc`):**

The subprocess declares its OWN copies of the enums (Wire, WireResp, AdminReq, AdminResp, UserReq, UserResp) at top-level. EDN round-trip works because both universes use same enum shape.

The subprocess's `:user::main`:
- Maintains registry `Vector<(client-id, state)>` (per-user state only; no channels at process tier — all clients share stdio)
- Maintains next-id counter (monotonic)
- Loop: `:wat::kernel::readln` (ambient) → decode `:counter::Wire` → dispatch by variant
  - `Wire/Admin Provision initial` → mint id, add `(id, initial)` to registry, `:wat::kernel::println (WireResp/Admin (Provisioned id))`
  - `Wire/Admin Deprovision id` → drop registry entry, println `(WireResp/Admin (Deprovisioned id))`
  - `Wire/Admin Stop` → println `(WireResp/Admin Stopped)`, return nil (program exits → subprocess exits)
  - `Wire/User id req` → look up registry entry for id, handle UserReq, mutate state, println `(WireResp/User (UserResp variant))`
  - Tail-call self with updated registry

**Parent-side wrappers under `:counter::*`:**

```scheme
;; Spawner — spawns subprocess with the program forms; returns AdminProc
(:wat::core::defn :counter::spawn-proc
  []
  -> :counter::AdminProc
  ;; (:wat::core::let
  ;;   [program-forms (:wat::core::forms <subprocess enum decls + :user::main>)
  ;;    proc (:wat::kernel::spawn-process program-forms)
  ;;    peer (:wat::kernel::ProcessPeer/new
  ;;           (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
  ;;           (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin proc)))]
  ;;   (:counter::AdminProc/new "server-counter-proc-0" peer proc))
  )

(:wat::core::defn :counter::provision-proc
  [admin! <- :counter::AdminProc
   initial <- :wat::core::i64]
  -> :counter::ClientProc
  ;; Process/println admin.peer (Wire/Admin (Provision initial))
  ;; Process/readln admin.peer → WireResp/Admin (Provisioned id)
  ;; (:counter::ClientProc/new admin.server-id id admin.peer)
  )

(:wat::core::defn :counter::deprovision-proc
  [admin! <- :counter::AdminProc
   client! <- :counter::ClientProc]
  -> :wat::core::nil
  ;; similar
  )

(:wat::core::defn :counter::stop-proc
  [admin! <- :counter::AdminProc]
  -> :wat::core::nil
  ;; send Wire/Admin Stop; recv WireResp/Admin Stopped; Process/drain-and-join admin.proc
  ;; uses inner/outer let to absorb drain-and-join lockstep per slice 3c pattern
  )

(:wat::core::defn :counter::get-proc
  [client! <- :counter::ClientProc]
  -> :wat::core::i64
  ;; send (Wire/User client.client-id Get); recv WireResp/User (Value v); return v
  )

(:wat::core::defn :counter::increment-proc
  [client! <- :counter::ClientProc, n <- :wat::core::i64]
  -> :wat::core::i64)

(:wat::core::defn :counter::reset-proc
  [client! <- :counter::ClientProc]
  -> :wat::core::i64)
```

**Body — same observable shape as 3c:**

```scheme
(:wat::core::let
  [admin!     (:counter::spawn-proc)
   client-a!  (:counter::provision-proc admin! 10)
   client-b!  (:counter::provision-proc admin! 100)
   client-c!  (:counter::provision-proc admin! 0)
   _a1   (:counter::increment-proc client-a! 5)
   _b1   (:counter::increment-proc client-b! 50)
   _c1   (:counter::get-proc client-c!)
   _d    (:counter::deprovision-proc admin! client-b!)
   _a2   (:counter::get-proc client-a!)
   _c2   (:counter::reset-proc client-c!)
   _s    (:counter::stop-proc admin!)]
  ;; assertions matching 3c
  ...)
```

## Scope (no substrate changes)

Pure consumer slice. Zero edits to src/.

Substrate primitives required (all confirmed shipped):
- struct-restricted (slice 1)
- :wat::kernel::spawn-process (arc 170 slice 6 — accepts program forms)
- :wat::kernel::ProcessPeer + Process/readln + Process/println + Process/drain-and-join (Stone C2 + Stone C3)
- :wat::kernel::Sender/from-pipe + Receiver/from-pipe (Stone C revision era)
- :wat::core::forms for constructing subprocess program (arc 091 slice 8)
- :wat::core::Vector + foldl + struct + enum + match (substrate baseline)

## Design choices (fixed in BRIEF — no relitigation)

1. **Multiplexed single-stream architecture** — admin + all users share the one ProcessPeer. Wire/WireResp enums tag Admin vs User-with-client-id. Per user direction "a process communicating over a crossbeam is a divide by zero statement"; process tier MUST use stdio; multiplexing on single stream is the only sensible shape with current substrate
2. **Sequential request-response** — test body single-threaded; for each user op, send → read response → return. Server's responses come back in same order; no demux required for sequential pattern
3. **Server registry = Vector<(id, state)>** — per-user state only; no per-user channels (all share stdio)
4. **Wrapper naming `-proc` suffix** — `:counter::spawn-proc`, `:counter::provision-proc`, etc. — distinguishes from slice 3c's thread-tier wrappers. Both demos can coexist
5. **AdminProc carries Process (not just ProcessPeer)** — so `:counter::stop-proc` can absorb the drain-and-join lockstep in inner/outer let pattern (per slice 3c precedent)
6. **ClientProc.peer is a clone of AdminProc.peer** — ProcessPeer is wat-value; struct accessor returns Arc-clone (per slice 3c finding); multiple Client capabilities reference same underlying PipeFd channels safely

## STOP triggers

1. **Subprocess EDN round-trip rejects enum carrying enum** (Wire wrapping AdminReq/UserReq) — verify; both universes declare same enums; should round-trip per slice 2 + Counter actor process proof patterns
2. **ProcessPeer in struct-restricted field rejected** — slice 3c proved Sender/Receiver in struct-restricted; ProcessPeer is a struct itself; verify
3. **`:wat::core::forms` inline-construction of subprocess program issue** — slice 6 of arc 170 + Counter actor process proof use this pattern; verify
4. **Workspace baseline regresses** beyond 3 pre-existing failures
5. **Proc macro cache** — `touch crates/wat-macros/src/lib.rs` after creating new .wat file (per slice 3b lesson)

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- cwd anchor: `/home/watmin/work/holon/wat-rs/`; never in `.claude/worktrees/`.
- DO NOT touch src/. Pure consumer.
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- DO NOT mint new substrate primitives.
- DO NOT operate in `.claude/worktrees/`.

## Decay disclosure (orchestrator)

The subprocess program construction pattern works per `wat-tests/counter-actor-proof-process.wat` (shipped at `9b0c517`) — that file's :counter/spawn-process inline-declares enums + :user::main via `(:wat::core::forms ...)`. Slice 3d extends this with multi-user state.

EDN encoding for Wire/WireResp is automatic per arc 091 slice 8 — wat-edn's enum serialization handles tagged variants with payloads.

Sequential request-response constraint is honest for the test body's single-threaded nature; if a future consumer needs CONCURRENT multi-user at process tier, that's a separate slice with response-demux requirements (not in 3d).

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_process_N3` builds clean |
| B | Process variant lifecycle works end-to-end (provision 3, exercise each, deprovision one, stop) | Test passes |
| C | EDN round-trip Wire + WireResp across subprocess boundary | implicit via test passing |
| D | Capability enforcement at process tier (test body outside `:counter::*` cannot mint/read restricted fields) | Code review confirms |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 90-120 min sonnet (more complex than 3c — subprocess program + bidirectional Wire/WireResp + EDN encoding for multi-variant enums). Hard stop: 150 min.

## Workspace baseline (post-slice-3c commit `e7aa671`)

3 pre-existing failures. Post-3d target: +1 passing deftest; 3 failures preserved.

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3D.md` per § SCORE methodology
2. Return final summary: rows passed/failed, workspace delta, file paths touched, honest deltas (especially around EDN encoding, subprocess program forms, ProcessPeer in struct-restricted), suggested corrections for slice 3e closure

You are launching now. T-minus 0.
