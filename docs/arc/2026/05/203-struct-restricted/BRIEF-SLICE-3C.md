# BRIEF — Arc 203 Slice 3c: capability structs (struct-restricted Admin + Client)

**Phase:** Third stepping stone. Wraps slice 3b's bare-channel multi-user lifecycle in struct-restricted capability values — `:counter::Admin` (privileged) + `:counter::Client` (per-user). Behavior still enforces protocol separation (server matches Wire variant); struct-restricted enforces that only `:counter::*` defns can MINT these capabilities (users hold but cannot forge).

**Predecessor:** Slice 3b shipped at `15cf7a8` — Vector-based registry, Provision/Deprovision lifecycle, per-user state, auto-cleanup on Disconnect. Wire enum proven; bare channel ends returned to caller.

**Successors:** Slice 3d ships process variant (Wire enum over stdio); 3e closure.

## Goal

ONE wat-tests deftest extending 3b with capability wrappers:
- `:counter::Admin` struct-restricted (only `:counter::*` can mint or read fields) — holds server-id + admin channels
- `:counter::Client` struct-restricted (only `:counter::*` can mint or read fields) — holds server-id + client-id + user channels
- `:counter::spawn` returns Admin (privileged handle) instead of bare admin channels
- `:counter::provision admin!` returns Client (capability) instead of bare AdminResp variant
- Admin-side wrappers under `:counter::*`: `provision`, `deprovision`, `stop`
- User-side wrappers under `:counter::*`: `get`, `increment`, `reset`
- Test body OUTSIDE `:counter::*` exercises via wrappers only (cannot forge Admin/Client; cannot read their restricted fields)

## Design choice (fixed in BRIEF — DO NOT relitigate)

**Option U — unified Wire enum (slice 3a/3b's proven shape).** Admin + users send `Sender<Wire>`; server selects on `Vec<Receiver<Wire>>`; matches variant. Behavior enforces protocol separation; struct-restricted enforces capability minting.

**NOT Option S (split AdminWire/UserWire):** `:wat::kernel::select` is uniform-T; splitting wire types would force two server loops or polling; doesn't compose with the substrate. Slice 3a's pivot to Wire enum holds.

## Required artifacts

### `wat-tests/counter-service-capability-N3.wat`

ONE deftest. Capability-wrapped multi-user lifecycle.

**Prelude — extends 3b's protocol with capability structs:**

```scheme
;; Wire + AdminReq + AdminResp + UserReq + UserResp UNCHANGED from 3b.
;; (Copy the four enums + Wire from counter-service-thread-N3.wat verbatim.)

;; Capability struct — Admin (privileged handle, server-issued).
;; Only :counter::* defns can mint (ctor whitelist) or read fields (per-field whitelist).
(:wat::core::struct-restricted :counter::Admin
  [:counter::]                                                     ;; only :counter::* can call Admin/new
  ([:counter::] server-id  <- :wat::core::String                    ;; server's secret-witness; never exposed
   [:counter::] admin-tx   <- :wat::kernel::Sender<counter::Wire>   ;; admin->server channel
   [:counter::] admin-rx   <- :wat::kernel::Receiver<counter::AdminResp>)
  ())                                                              ;; no public fields

;; Capability struct — Client (per-user handle, server-issued via Provision).
(:wat::core::struct-restricted :counter::Client
  [:counter::]
  ([:counter::] server-id  <- :wat::core::String
   [:counter::] client-id  <- :wat::core::String
   [:counter::] user-tx    <- :wat::kernel::Sender<counter::Wire>   ;; user->server channel (sends Wire/User variants)
   [:counter::] user-rx    <- :wat::kernel::Receiver<counter::UserResp>)
  ())
```

**Privileged wrappers (under `:counter::*` — they read restricted fields):**

```scheme
;; Server spawner — creates channels, generates server-id (monotonic counter
;; or simple constant), spawns thread with dispatch loop from 3b, returns Admin.
(:wat::core::defn :counter::spawn
  []                                                                ;; no args; server-id is server-internal
  -> :counter::Admin
  ;; create admin channel pair; spawn thread running dispatch; return Admin
  )

;; Admin operations
(:wat::core::defn :counter::provision
  [admin! <- :counter::Admin
   initial <- :wat::core::i64]
  -> :counter::Client
  ;; send (Admin (Provision initial)) on admin.admin-tx;
  ;; recv Provisioned id tx rx from admin.admin-rx;
  ;; construct Client {server-id: admin.server-id, client-id: id, user-tx: tx, user-rx: rx}
  )

(:wat::core::defn :counter::deprovision
  [admin! <- :counter::Admin
   client! <- :counter::Client]
  -> :wat::core::nil
  ;; send (Admin (Deprovision client.client-id)) on admin.admin-tx;
  ;; recv Deprovisioned ack
  )

(:wat::core::defn :counter::stop
  [admin! <- :counter::Admin]
  -> :wat::core::nil
  ;; send (Admin Stop); recv Stopped
  )

;; User operations
(:wat::core::defn :counter::get
  [client! <- :counter::Client]
  -> :wat::core::i64
  ;; send (User Get) on client.user-tx; recv UserResp from client.user-rx; extract value
  )

(:wat::core::defn :counter::increment
  [client! <- :counter::Client
   n <- :wat::core::i64]
  -> :wat::core::i64
  ;; same shape with Increment n
  )

(:wat::core::defn :counter::reset
  [client! <- :counter::Client]
  -> :wat::core::i64
  ;; same shape with Reset
  )
```

**Body — exercises via capability wrappers (test body is outside `:counter::*`):**

```scheme
(:wat::core::let
  [admin!     (:counter::spawn)
   client-a!  (:counter::provision admin! 10)
   client-b!  (:counter::provision admin! 100)
   client-c!  (:counter::provision admin! 0)
   
   ;; Each user independent
   _a1   (:counter::increment client-a! 5)        ;; → 15
   _b1   (:counter::increment client-b! 50)       ;; → 150
   _c1   (:counter::get client-c!)                ;; → 0
   
   ;; Deprovision client-b mid-flight
   _d    (:counter::deprovision admin! client-b!)
   
   ;; Users a + c still work
   _a2   (:counter::get client-a!)                ;; → 15
   _c2   (:counter::reset client-c!)              ;; → 0
   
   ;; Admin Stop
   _s    (:counter::stop admin!)]
  
  ;; Assertions on the values bound above
  ...)
```

## Scope (no substrate changes)

Pure consumer slice. Zero edits to src/.

Substrate primitives required (all confirmed shipped):
- struct-restricted (slice 1)
- Wire enum + select + send + recv (slice 3a)
- Vector registry + dispatch + Provision/Deprovision (slice 3b)
- `:wat::kernel::Sender<Wire>` + `:wat::kernel::Receiver<...>` (post-Stone-C3 honest naming)

## STOP triggers

1. **struct-restricted field carrying Sender<Wire> rejected** — slice 2 used ThreadPeer (single field bundling channels); 3c uses separate Sender + Receiver fields. If substrate rejects channel-values as struct-restricted fields, surface; we may need to wrap channels in ThreadPeer first then put ThreadPeer in the capability
2. **Whitelist `[:counter::]` not matching `:counter::*` callers** — should work per slice 2's proven pattern; surface if it doesn't
3. **Workspace baseline regresses** beyond 3 pre-existing failures
4. **Proc macro cache** — per slice 3b honest delta 4; remember to `touch crates/wat-macros/src/lib.rs` to force discovery of new wat-tests file
5. **`:counter::spawn` server-id generation** — use a simple constant `"server-0"` or monotonic counter (per slice 3b precedent for client-id); no telemetry dep needed

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- cwd anchor: `/home/watmin/work/holon/wat-rs/`; never in `.claude/worktrees/`.
- DO NOT touch src/ — pure consumer.
- DO NOT introduce process variant yet (that's 3d).
- DO NOT use `--no-verify`.
- DO NOT mint new substrate primitives.

## Decay disclosure (orchestrator)

The capability shape mirrors slice 2's `:counter::Client` proven pattern but adds Admin + carries channel pairs as separate Sender/Receiver fields (slice 2 used ThreadPeer as single field). If substrate rejects this, fall back to ThreadPeer-wrapping per slice 2.

Per slice 3b SCORE corrections inherited:
- Registry is always Vector (never HashMap) — but slice 3c doesn't touch the registry, just wraps the existing dispatch
- `foldl` not `reduce`
- Inner type-alias in `:(...)` must be bare
- AdminResp::Provisioned.rx is `Receiver<UserResp>` (not Wire)
- Proc macro cache invalidation: `touch crates/wat-macros/src/lib.rs` after creating new .wat file

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_capability_N3` builds clean |
| B | Capability wrappers work end-to-end (provision returns Client; each client independent ops; deprovision + stop work) | Test passes |
| C | Test body (outside `:counter::*`) CANNOT mint Admin or Client (verify by inspection — body only uses wrappers) | Code review: test body's prefix vs `[:counter::]` whitelist; no `(:counter::Admin/new ...)` or `(:counter::Client/new ...)` calls outside `:counter::*` |
| D | Test body CANNOT read restricted fields (server-id, client-id, channels) — would fire DefRestrictedCallerNotAllowed at check time | Negative would be assertion via Rust-side test if needed; for slice 3c, code review of body access suffices (body never reads .server-id etc.) |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 60-90 min sonnet. Hard stop: 120 min.

## Workspace baseline (post-slice-3b commit `15cf7a8`)

3 pre-existing failures. Post-3c target: +1 passing deftest; 3 failures preserved.

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3C.md` per § SCORE methodology
2. Return final summary: rows passed/failed, workspace delta, file paths touched, honest deltas surfaced (especially struct-restricted-with-channel-fields if it needs ThreadPeer fallback), suggested BRIEF corrections for slice 3d

You are launching now. T-minus 0.
