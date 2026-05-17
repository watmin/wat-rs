# BRIEF — Arc 203 Slice 3f-naming: rename Client → User (Admin + User pattern)

**Phase:** Small naming refactor surfaced post-3f. The emergent (Admin, Client) pattern reframes more honestly as **(Admin, User)** — both are clients to the server; one is privileged (Admin), one isn't (User).

**Predecessor:** Slice 3f shipped at `dfd5897` — error propagation Result-bearing wrappers; current naming uses `:counter::Client`.

## Goal

Mechanical rename across three files:
1. `wat-tests/counter-client-capability-proof.wat` — slice 2 single-user proof
2. `wat-tests/counter-service-capability-N3.wat` — slice 3c+3e+3f thread tier
3. `wat-tests/counter-service-process-N3.wat` — slice 3d+3e+3f process tier

## What renames

| Before | After | Notes |
|---|---|---|
| `:counter::Client` | `:counter::User` | struct type rename |
| `:counter::ClientProc` | `:counter::UserProc` | process-tier variant struct |
| `Client/new` | `User/new` | auto-synthesized constructor; renames with struct |
| `Client/server-id` | `User/server-id` | accessor rename |
| `Client/client-id` | `User/user-id` | accessor rename AND field rename (per Admin+User pattern: User holds a user-id, not a "client-id") |
| `Client/user-tx` | `User/user-tx` | accessor (field name already `user-tx` — keep) |
| `Client/user-rx` | `User/user-rx` | accessor (field name already `user-rx` — keep) |
| `client!` (parameter) | `user!` | wrapper parameter name |
| `client-id` (value bindings) | `user-id` | local variable rename |
| `client-a!`, `client-b!`, `client-c!` (test bodies) | `user-a!`, `user-b!`, `user-c!` | test body variable rename |
| Wire/User payload `id` field | `user-id` | Wire variant field rename |
| AdminResp::Provisioned payload `id` field | `user-id` | response variant field rename |
| AdminResp::Deprovisioned payload `id` field | `user-id` | response variant field rename |
| AdminReq::Deprovision payload `id` field | `user-id` | request variant field rename |
| Server's registry entry `(client-id, ...)` | `(user-id, ...)` | internal registry field rename |
| Comment text: "client" → "user" where it refers to the User type | (case-by-case) | preserve domain-specific uses of "client" for the abstract "both are clients" sense |
| File names | UNCHANGED | `counter-client-capability-proof.wat` stays (historical record per `feedback_inscription_immutable`); future demos may use `counter-user-*.wat` if revisited |

## What does NOT rename

- `:counter::Admin` + `:counter::AdminProc` — Admin keeps its name
- `:counter::Wire` enum + `Wire/Admin` + `Wire/User` variants — Wire/User already correctly named
- `:counter::AdminReq` + `:counter::AdminResp` + `:counter::UserReq` + `:counter::UserResp` — protocol enums correctly named already
- `:counter::ServiceError` + its variants — domain-agnostic; no naming change
- `wat::kernel::*` substrate types (ThreadPeer, Process, ProcessPeer, Sender, Receiver, etc.) — substrate-owned
- Stop/Provision/Deprovision admin op names — already correctly named
- Get/Increment/Reset user op names — already correctly named
- Comments that use "client" abstractly (e.g., "both Admin and User are clients to the server") — keep "client" where it's the abstract role, not the type name

## Field-rename detail: `client-id` → `user-id`

This is the most invasive rename. Touches:
- Struct field declaration: `[:counter::] client-id <- :wat::core::String` → `[:counter::] user-id <- :wat::core::String`
- Accessor calls: `(:counter::Client/client-id client!)` → `(:counter::User/user-id user!)`
- Wire variant field: `(User (server-id :String) (id :String) (req :UserReq))` → `(User (server-id :String) (user-id :String) (req :UserReq))` — Wire enum field rename
- AdminResp variants carrying ids (Provisioned, Deprovisioned)
- AdminReq::Deprovision (carries user-id)
- Server-side dispatch when handling Wire/User: extracts user-id payload
- Server-side registry entries

Rationale: User holds a user-id (consistent with "User" type name). A "client-id" inside a struct named "User" is residual from the prior pattern; rename completes the naming-coherence.

## Run cargo test verification

After rename, all four counter-service tests should pass:
- `cargo test --release -p wat --test test counter_service` should show 4/4
- `cargo test --release -p wat --test test deftest_counter_client_capability_proof` should pass (slice 2 single-user variant)

Workspace baseline: same 3 pre-existing failures preserved.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically.
- DO NOT touch src/. Pure rename refactor.
- DO NOT change semantics — same protocol, same behavior, only naming
- DO NOT use `--no-verify`
- DO NOT operate in `.claude/worktrees/`
- `:counter::Admin` does NOT rename

## SCORE methodology

3 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | All three files compile + tests pass after rename | `cargo test --release -p wat --test test counter_service` shows 4/4; `deftest_counter_client_capability_proof` still passes |
| B | Naming consistent: `:counter::User` everywhere; `user!` parameter naming; `user-id` field naming throughout including Wire/AdminResp variants | grep confirms no remaining `:counter::Client`, `client!`, or `client-id` usages |
| C | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 30-45 min sonnet (mechanical rename; high predictability).

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3F-NAMING.md`
2. Return summary: rows passed/failed, files touched, any honest deltas

You are launching now. T-minus 0.
