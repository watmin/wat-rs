# SCORE — Arc 203 Slice 3f-naming: rename Client → User

**Commit:** pending (orchestrator commits atomically)
**Date:** 2026-05-17

## Methodology

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | All three files compile + tests pass after rename | YES | `cargo test --release -p wat --test test counter_service` → 4/4; `deftest_counter_client_capability_proof` → 1/1 pass |
| B | Naming consistent: `:counter::User` everywhere; `user!` parameter naming; `user-id` field naming throughout including Wire/UserProc variants | YES | `grep -n "Client\|client-id\|client!"` across all three files returns zero matches |
| C | Workspace baseline preserved | YES | `cargo test --release --workspace --no-fail-fast` shows 3 failures — same 3 pre-existing failures as before (deftest_wat_tests_tmp_totally_bogus, t6_spawn_process_factory_with_capture_round_trips, startup_error_bubbles_up_as_exit_3) |

## Files touched

1. `wat-tests/counter-client-capability-proof.wat` — `:counter::Client` → `:counter::User`; `client-id` field → `user-id`; `Client/new`, `Client/server-id`, `Client/client-id`, `Client/peer!` → `User/new`, `User/server-id`, `User/user-id`, `User/peer!`; `client!` → `user!` in wrappers + test body; comment references updated
2. `wat-tests/counter-service-capability-N3.wat` — same struct rename; Wire::User `id` field → `user-id`; all `Client/...` accessor calls → `User/...`; `client!` parameters → `user!`; `client-a!`, `client-b!`, `client-c!` → `user-a!`, `user-b!`, `user-c!`; comment references updated
3. `wat-tests/counter-service-process-N3.wat` — `:counter::ClientProc` → `:counter::UserProc`; `client-id` field → `user-id`; Wire::User `id` field → `user-id` (both parent and subprocess copies); all `ClientProc/...` → `UserProc/...`; `client!` → `user!`; `client-a!`/`client-b!`/`client-c!` → `user-a!`/`user-b!`/`user-c!`; comment references updated

## Honest deltas

None. Purely mechanical rename as specified. No semantic changes. No implementation-detail renames (server-internal `id-str` generation prefix `"client-"` left intact — that is a string value the server generates, not a type/field name).
