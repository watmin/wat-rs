# BRIEF — Arc 206 Slice 1.5: substrate UUIDv5 promotion

**Predecessor:** Slice 1 shipped at `4ff2b72` — `:wat::core::uuid::v4` available at substrate level; pattern + namespace choice (`:wat::core::uuid::*`) established.

**Goal:** Mint `:wat::core::uuid::v5` at substrate level. Same pattern as v4; deterministic SHA-1-based UUID for content addressing + hierarchical-derivation use cases.

## Substrate work

1. **Add `v5` feature flag** to `uuid` dep in `crates/wat-edn/Cargo.toml` (currently `uuid = "1"` — needs `features = ["v5"]` added alongside whatever's already enabled).

2. **Mint `wat_edn::new_uuid_v5(namespace, name)`** in `crates/wat-edn/src/lib.rs`. Signature mirrors v4 pattern:
   - `pub fn new_uuid_v5(namespace: uuid::Uuid, name: &str) -> uuid::Uuid`
   - Thin wrapper around `uuid::Uuid::new_v5(&namespace, name.as_bytes())`

3. **Register `:wat::core::uuid::v5`** at substrate level (mirror v4 from slice 1):
   - `src/string_ops.rs` — `eval_uuid_v5` handler; parses namespace string via `uuid::Uuid::parse_str`; panics with assertion-failed! diagnostic on invalid namespace; calls `wat_edn::new_uuid_v5`; returns canonical hyphenated String
   - `src/runtime.rs` — dispatch arm for `:wat::core::uuid::v5`
   - `src/check.rs` — type scheme `[namespace :wat::core::String, name :wat::core::String] -> :wat::core::String`

## Tests in NEW `tests/wat_arc206_uuid_v5.rs`

4 cases:
1. **basic** — `(:wat::core::uuid::v5 some-namespace-uuid "test-name")` returns canonical 36-char hyphenated string
2. **deterministic** — same `(namespace, name)` → same UUID (call twice, assert equality)
3. **different namespace** — different namespace, same name → different UUID
4. **different name** — same namespace, different name → different UUID

For (2), (3), (4): can use a fixed namespace string like `"6ba7b810-9dad-11d1-80b4-00c04fd430c8"` (the standard DNS namespace per RFC 4122) or mint via `:wat::core::uuid::v4` for the namespace (couples slices but proves composition).

## Invalid-input handling

If `namespace` string is not a valid UUID, `uuid::Uuid::parse_str` returns Err. The eval handler should `assertion-failed!` with a clear diagnostic ("uuid::v5: namespace must be a canonical UUID string, got: <value>"). Don't return Result — input validation panics are standard substrate pattern (per `feedback_shim_panic_vs_option`).

## HARD constraints

- DO NOT commit. Orchestrator commits.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`.
- DO NOT retire `:wat::telemetry::uuid::v4` or other backward-compat paths.
- DO NOT mint typed `:wat::core::Uuid` (out of scope; future arc).
- DO NOT use `--no-verify`.

## STOP triggers

1. **`uuid` crate's v5 feature requires another feature dependency** (e.g., `v5` may need `std` or `sha1`); read uuid crate docs + surface
2. **Workspace baseline regresses** beyond 4 pre-existing failures
3. **`Uuid::parse_str` error type doesn't compose cleanly with the eval handler's panic path** — surface

## Time-box

Predicted 30-45 min sonnet. Hard stop 60 min. Very mechanical mirror of slice 1.

## SCORE methodology

4 rows YES/NO:

| Row | Evidence |
|---|---|
| A — `:wat::core::uuid::v5` mints + returns canonical String | basic test passes |
| B — deterministic (same inputs → same UUID) | determinism test passes |
| C — namespace + name independently affect output | both "different namespace" + "different name" tests pass |
| D — workspace baseline preserved | 4 pre-existing failures unchanged; backward-compat (:wat::telemetry::uuid::v4 + :wat::core::uuid::v4 from slice 1) both still work |

## On completion

Write `SCORE-SLICE-1-5.md`; return summary including any honest deltas (especially around uuid crate's feature-flag requirements, parse_str error handling pattern).

You are launching now. T-minus 0.
