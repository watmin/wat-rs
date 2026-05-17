# SCORE — Arc 206 Slice 1: substrate UUID promotion

**Status:** CLOSED 2026-05-17.

## Scorecard

| Row | Criterion | Result | Evidence |
|---|---|---|---|
| A | `:wat::core::uuid::v4` mints + returns canonical-shaped String | YES | `uuid_v4_returns_36_char_string` passes; output is 36-char quoted string |
| B | entropy (two calls differ) | YES | `uuid_v4_two_calls_differ` passes |
| C | canonical hyphen positions (len=36 ∧ 5 parts when split on `-`) | YES | `uuid_v4_canonical_hyphen_positions` passes |
| D | callable without telemetry dep | YES | test file imports only `wat`; no `wat_telemetry`, no `wat_measure`; 4/4 pass |

## Namespace choice

**`:wat::core::uuid::v4`** — not `:wat::kernel::uuid::v4`.

Rationale: UUID minting is a runtime utility that returns `:wat::core::String`. It belongs in the same category as `:wat::core::string::*` and `:wat::core::regex::*` (string-returning substrate utilities, no opaque type). `:wat::kernel::` is for primitives that interact with the OS scheduling model (spawn-thread, spawn-process, println, signals). UUID generation has no kernel interaction — it is pure RNG → string.

## Implementation touchpoints

| File | Change |
|---|---|
| `src/string_ops.rs` | Added `eval_uuid_v4` handler; module doc updated to include `uuid::*` |
| `src/runtime.rs` | Added `:wat::core::uuid::v4` dispatch arm (before regex block) |
| `src/check.rs` | Registered type scheme `[] -> :wat::core::String` (after regex block) |
| `Cargo.toml` | Enabled `mint` feature on `wat-edn` dep (required for `wat_edn::new_uuid_v4`) |
| `tests/wat_arc206_uuid_substrate.rs` | New test file; 4 tests; 4/4 pass |

## Alias mechanism

**Separate independent impls** — not a wat-level alias.

`:wat::telemetry::uuid::v4` continues to use its own `RustSymbol` shim (`:rust::telemetry::uuid::v4` → `wat_edn::new_uuid_v4`). `:wat::core::uuid::v4` is a new substrate dispatch arm calling the same `wat_edn::new_uuid_v4` via `crate::string_ops::eval_uuid_v4`. Both paths call the same underlying function. This is intentional — no fragile alias mechanism needed; both implementations are trivial single-function wrappers over `wat_edn::new_uuid_v4`.

Backward compat confirmed: `cargo test --release -p wat-telemetry` — 36/36 pass, including `deftest_wat_telemetry_uuid_test_distinct_pair` and `deftest_wat_telemetry_uuid_test_many_distinct`.

## Feature gate delta

**Honest delta:** The `wat` crate now enables `wat-edn`'s `mint` feature directly (`wat-edn = { ..., features = ["mint"] }`). Previously only `wat-telemetry` pulled this feature. This is correct — the substrate now owns UUID minting capability, so the substrate crate must declare the dep. No new external crate pins; `uuid` is already transitively present via `wat-edn`'s existing lockfile entry.

## Workspace baseline

Pre-existing failures (4, unchanged before and after this arc):
- `lifeline_pipe_zero_orphans_across_100_trials` (timing-sensitive lifeline probe)
- `deftest_wat_tests_tmp_totally_bogus` (intentional panic probe)
- `t6_spawn_process_factory_with_capture_round_trips` (process fixture)
- `startup_error_bubbles_up_as_exit_3` (CLI exit-code probe)

Note: BRIEF stated "3 pre-existing failures" but baseline measurement shows 4. All 4 existed on the branch before arc 206 changes; none introduced by this arc.

## Next

Slice 2 (INSCRIPTION + 058 row + USER-GUIDE) is now unblocked.
Arc 203 slice 3f-uuid (counter-service demos → `:wat::core::uuid::v4`) is unblocked.
