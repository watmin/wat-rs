# SCORE — Arc 206 Slice 1.5: substrate UUIDv5 promotion

**Status:** CLOSED 2026-05-17.

## Scorecard

| Row | Criterion | Result | Evidence |
|---|---|---|---|
| A | `:wat::core::uuid::v5` mints + returns canonical String | YES | `uuid_v5_returns_36_char_string` passes; output is 36-char, 5-part hyphenated string |
| B | deterministic (same inputs → same UUID) | YES | `uuid_v5_deterministic_same_inputs_produce_same_uuid` passes |
| C | namespace + name independently affect output | YES | `uuid_v5_different_namespace_produces_different_uuid` + `uuid_v5_different_name_produces_different_uuid` both pass |
| D | workspace baseline preserved | YES | 4 pre-existing failures unchanged; `:wat::telemetry::uuid::v4` + `:wat::core::uuid::v4` (slice 1) both still work |

## Implementation touchpoints

| File | Change |
|---|---|
| `crates/wat-edn/Cargo.toml` | Added `"uuid/v5"` to `mint` feature (was `["uuid/v4"]`, now `["uuid/v4", "uuid/v5"]`) |
| `crates/wat-edn/src/lib.rs` | Minted `pub fn new_uuid_v5(namespace: uuid::Uuid, name: &str) -> uuid::Uuid` — thin wrapper around `uuid::Uuid::new_v5(&namespace, name.as_bytes())` |
| `src/string_ops.rs` | Added `eval_uuid_v5` handler; parses namespace string via `uuid::Uuid::parse_str`, panics with `assertion-failed!` diagnostic on invalid namespace, calls `wat_edn::new_uuid_v5`, returns canonical String |
| `src/runtime.rs` | Added `:wat::core::uuid::v5` dispatch arm next to `:wat::core::uuid::v4` |
| `src/check.rs` | Registered type scheme `[namespace :wat::core::String, name :wat::core::String] -> :wat::core::String` next to v4 scheme |
| `Cargo.toml` | Added `uuid = "1"` as direct dep to `wat` crate (required for `uuid::Uuid::parse_str` in `eval_uuid_v5`) |
| `tests/wat_arc206_uuid_v5.rs` | New test file; 4 tests; 4/4 pass |

## Feature gate delta

**Honest delta:** The `mint` feature in `crates/wat-edn/Cargo.toml` now enables both `uuid/v4` and `uuid/v5`. The `uuid/v5` feature pulls `sha1_smol v1.0.1` (a new transitive dep locked into `Cargo.lock`). No other external crate pins changed.

**Direct uuid dep added:** The `wat` crate's root `Cargo.toml` now declares `uuid = "1"` as a direct dependency. This was required because `eval_uuid_v5` in `src/string_ops.rs` calls `uuid::Uuid::parse_str` — a Rust type from the `uuid` crate. Relying solely on the transitive path through `wat-edn` would be unsound. Adding it explicitly is the standard Rust approach and matches the ecosystem norm (same pattern as the `libc` dep the `wat` crate already carries).

## Namespace parsing

`uuid::Uuid::parse_str` accepts any valid UUID format. The eval handler panics with a clear `assertion-failed!` diagnostic on parse failure, matching the substrate input-validation panic pattern (`feedback_shim_panic_vs_option`): construction/input-validation panics; lookup/query returns `Option<T>`.

## Workspace baseline

Pre-existing failures (4, identical to slice 1 baseline — unchanged):
- `lifeline_pipe_zero_orphans_across_100_trials` (timing-sensitive lifeline probe)
- `deftest_wat_tests_tmp_totally_bogus` (intentional panic probe)
- `t6_spawn_process_factory_with_capture_round_trips` (process fixture)
- `startup_error_bubbles_up_as_exit_3` (CLI exit-code probe)

Backward compat confirmed: slice 1's `wat_arc206_uuid_substrate` tests (4/4) pass unchanged.

## Next

Slice 2 (INSCRIPTION + 058 row + USER-GUIDE) is now unblocked for both `:wat::core::uuid::v4` and `:wat::core::uuid::v5`.
Arc 203 slice 3f-uuid (counter-service demos → `:wat::core::uuid::v4`) is unblocked.
