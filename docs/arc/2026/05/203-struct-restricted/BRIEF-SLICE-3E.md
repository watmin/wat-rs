# BRIEF — Arc 203 Slice 3e: server-id validation wiring (secret-witness goes live)

**Phase:** Fifth stepping stone. Wires `server-id` from dead-data (currently stored in capability structs but never validated) to live validation (embedded in every Wire payload; checked by server; rejected on mismatch). Makes the capability via secret-witness pattern honest.

**Predecessor:** Slice 3d shipped at `45a1727` — process variant with multiplexed Wire over stdio. Both 3c (thread) and 3d (process) ship working servers but server-id is stored unused.

**Successor:** Slice 3f closure paperwork (INSCRIPTION + 058 row + USER-GUIDE).

## Motivation

User direction 2026-05-17: *"we are not done until its done — we continue the arc... these are user-facing documentation now... show them (and me) how to use our forms."*

Current state (per honest review of artifacts):
- `Client.server-id` + `Admin.server-id` stored, per-field-restricted to `[:counter::]` (privacy ✓)
- Server NEVER validates incoming messages carry matching server-id
- At thread tier: channel ownership prevents forge (structural)
- At process tier: shared ProcessPeer means wrappers MUST embed correct identity, but server doesn't double-check
- Capability-via-mint-restriction works; capability-via-secret-witness is designed but not wired

Both files are USER-FACING DOCUMENTATION. They must show the honest pattern.

## Goal

Update BOTH existing slice artifacts in-place:
- `wat-tests/counter-service-capability-N3.wat` (slice 3c, thread tier)
- `wat-tests/counter-service-process-N3.wat` (slice 3d, process tier)

Each file demonstrates the validated pattern; happy-path tests continue to pass.

## Required code changes (BOTH files)

### Wire enum — server-id as first field on both variants

```scheme
(:wat::core::enum :counter::Wire
  (Admin (server-id :wat::core::String) (req :counter::AdminReq))
  (User  (server-id :wat::core::String) (id :wat::core::String) (req :counter::UserReq)))
```

### Response enums — AccessDenied variant

```scheme
(:wat::core::enum :counter::AdminResp
  (Provisioned   ...)
  (Deprovisioned ...)
  (Stopped       ...)
  (AccessDenied))                          ;; server refused — server-id mismatch

(:wat::core::enum :counter::UserResp
  (Value v)
  (Ok    v)
  (AccessDenied))                          ;; server refused — server-id mismatch
```

### Wrappers — embed server-id from capability on every send

Every `:counter::*` wrapper that constructs a Wire variant reads `Admin.server-id` or `Client.server-id` and includes it. E.g.:

```scheme
(:wat::core::defn :counter::increment
  [client! <- :counter::Client, n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [sid (:counter::Client/server-id client!)
     cid (:counter::Client/client-id client!)
     tx  (:counter::Client/user-tx client!)
     rx  (:counter::Client/user-rx client!)
     _   (:wat::kernel::send tx (:counter::Wire::User sid cid (:counter::UserReq::Increment n)))
     resp (option::expect (recv rx) "increment: peer died")]
    (:wat::core::match resp -> :wat::core::i64
      ((:counter::UserResp::Ok v)         v)
      ((:counter::UserResp::Value v)      v)
      ((:counter::UserResp::AccessDenied) (:wat::kernel::assertion-failed!
        "increment: server refused — server-id mismatch" None None)))))
```

### Server dispatch — validate server-id on receipt

The server holds its own `server-id` (constant string at spawn for the demo; production would use uuid::v4). Each dispatched message extracts the wire's server-id; if mismatch with self.server-id, emit AccessDenied; ELSE process normally.

For thread tier (3c): in the dispatch loop, when a message arrives on admin-rx or any user-rx, check the wire's server-id first; on mismatch, send AccessDenied on the corresponding response channel.

For process tier (3d): in the subprocess's `:user::main` loop, every readln/decoded Wire is checked for server-id match against the subprocess's own server-id (passed in via program args or hardcoded at spawn time); on mismatch, write WireResp tagged AccessDenied.

### Server-id minting

Use a constant string (e.g., `"server-counter-thread-0"` for 3c, `"server-counter-proc-0"` for 3d) per slice 3c/3d precedent — no telemetry dep. Document in prose: *"In production, mint server-id via `:wat::telemetry::uuid::v4` for unguessability. Constant string demonstrates the validation flow."*

## Prose documentation (both files)

Inscribe explicit comments explaining:
- **Thread tier:** server-id check is DEFENSE IN DEPTH; channel ownership already prevents forge structurally; check is harmless redundancy that mirrors the process-tier pattern uniformly
- **Process tier:** server-id check is LOAD-BEARING; users share the ProcessPeer, so server CANNOT rely on transport-level identity; secret-witness validation IS the auth mechanism

This is user-facing documentation about WHICH tier needs WHICH guarantee. The uniform pattern is easier to teach; the prose tells the reader when each part is necessary vs redundant.

## Forge demonstration (optional; if cleanly doable)

For BOTH files, consider adding ONE additional test case demonstrating cross-server forge rejection:
- Spawn TWO independent servers (different server-ids: "server-A", "server-B")
- Get a Client of server-A
- Try to use that Client's wrappers with server-B's peer (would require an adversarial wrapper — not directly possible because capability fields are restricted)

ALTERNATIVE simpler forge demonstration:
- Within `:counter::*` (the only namespace that can construct Wire variants), construct a Wire with WRONG server-id and send it through admin's tx
- Assert response is AccessDenied
- This is a contrived adversarial test FROM WITHIN the privileged namespace; documents what would happen IF a forge attempt somehow occurred

If cleanly doable, include. If not, the validated wrappers + server dispatch suffice as documentation.

## Scope (no substrate changes)

Pure consumer slice. Zero edits to src/. Updates two existing wat-tests files in-place.

## STOP triggers

1. **Match exhaustiveness on new variants** — adding `AccessDenied` to AdminResp + UserResp grows match arity at every dispatch site. Per slice 3d lesson (match exhaustiveness requires arm per outer variant), all match sites need new arms. Surface any failures
2. **Wire variant arg-count growth** — Wire/Admin grows from 1 arg (req) to 2 (server-id, req); Wire/User from 2 (id, req) to 3 (server-id, id, req). All constructor sites and pattern-match sites need update
3. **Workspace baseline regresses** beyond 3 pre-existing failures
4. **Proc macro cache** — modifying existing files SHOULD trigger normal cargo recompile, but if not, `touch crates/wat-macros/src/lib.rs` per slice 3b lesson

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- cwd anchor: `/home/watmin/work/holon/wat-rs/`; never in `.claude/worktrees/`.
- DO NOT touch src/. Pure consumer.
- DO NOT use `--no-verify`.
- DO NOT mint new substrate primitives.

## Decay disclosure (orchestrator)

Slice 3c + 3d's artifacts are stable; modifying them in-place is the right approach (these are tests, not INSCRIPTIONs; tests evolve). Prior versions live in git history at `e7aa671` (3c) and `45a1727` (3d).

Forge demonstration is OPTIONAL because the capability structure (all-restricted fields, including peer) makes it hard to construct a wat-level forge test cleanly. The validated pattern IS the documentation; forge tests are demonstrating the rejection-path. Sonnet judges what's tractable.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Both files parse + tests compile | `cargo test --release -p wat --test test counter_service` builds clean |
| B | Thread variant happy path passes with server-id wired | `deftest_counter_service_capability_N3 ... ok` |
| C | Process variant happy path passes with server-id wired | `deftest_counter_service_process_N3 ... ok` |
| D | Server validates server-id (visible in dispatch logic; AccessDenied response variant defined) | Code review of both files confirms |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 60-90 min sonnet. Hard stop: 120 min.

## Workspace baseline (post-slice-3d commit `45a1727`)

3 pre-existing failures. Post-3e target: 184 passing wat deftests (unchanged count; same tests updated in-place); 3 failures preserved.

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3E.md`
2. Return summary: rows passed/failed, workspace delta, file paths touched, honest deltas (especially around forge demonstration choice), suggested corrections for slice 3f closure paperwork

You are launching now. T-minus 0.
