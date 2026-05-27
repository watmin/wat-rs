# Arc 239 — span-arity test-rot sweep (the hidden phenomenal-error tail)

**Surfaced 2026-05-28** scoring Stone 237.7a: a full `cargo build --tests --workspace` revealed **21
compile errors across 15 integration-test targets** — invisible because the tracked green-metric is
`cargo test --lib` (834/0), which compiles ONLY `src/lib.rs`. Every `tests/*.rs` and `crates/*/tests`
is a separate compile unit the lib build never touches. Span-coordinate signature drift (arc 138 /
233) piled up behind the metric. `feedback_no_pre_existing_excuse`: fix the rot AND the visibility gap.

**All known fixes. None are the fork-mechanism outlier** (that's arc 213 runtime concern — it would
not appear as a compile error).

## The ledger (honest count — fix all of these)

**Class A — span-arity (20× `E0061`, `expected Span, found Value`):** a method gained a trailing
`Span` arg; stale callers pass `(name, value)`. Fix: append `Span::unknown()` (the test-scaffolding
idiom — 59 test files already use it).
- `crates/wat-cli/tests/wat_cli.rs` (2)
- `tests/probe_arc214_slice4_stone2_env_get_trio.rs` (3) · `..._stone3_env_dig_trio.rs` (1)
- `tests/probe_arc216_stone2_vector_roundtrip.rs` (1) · `..._stone5b_hashset_native_storage.rs` (2)
- `tests/probe_arc234_stone4_match_hash_destructure.rs` (1)
- `tests/probe_arc237_stone1_typeunion_substrate.rs` (1)
- `tests/probe_counter_actor_process_diag.rs` (1) · `tests/probe_sender_receiver_from_pipe.rs` (1)
- `tests/wat_arc170_stone_a_drain_and_join.rs` (2) · `..._stone_c1_threadpeer.rs` (6)
- `tests/wat_arc170_typed_channel_pipes.rs` (5) · `tests/wat_arc208_process_io_result.rs` (5)
- `tests/wat_process_peer_ipc_round_trip.rs` (1)

**Class B — records-split (1× `E0026`):** `tests/probe_arc234_stone2b_defrecord_macro.rs:65` —
destructures `wat__Record` expecting `holon_form`, but the BASE `wat__Record` variant has no
`holon_form` (only `wat__holon__Record` does — arc 237 S-C.2c split). Fix the destructure to the base
shape (match `{ class_fqdn, struct_form }`).

## The work (substrate-as-teacher cascade)

1. `cargo build --release --tests --workspace --keep-going` — read the errors.
2. Class A: append `Span::unknown()` as the trailing arg at each `expected Span, found Value` site.
   (Pure arg-threading — NO behavior change. Use a real in-scope span only if one is obviously
   present; otherwise `Span::unknown()`.)
3. Class B: fix the one `holon_form` destructure to the base `wat__Record` shape.
4. Iterate until `cargo build --release --tests --workspace --keep-going` → **0 errors**, then
   `cargo test --release --workspace --no-fail-fast` → **0 FAILED**.

## STOP triggers (REJECTION — surface, do not force-fix)

- Any error that is **not** Class A (span-arg append) or Class B (the one records destructure). A
  structural error tied to an in-flight arc (170/212/213/214 reshape) is the "arcs closing in on
  themselves" outlier the user flagged — STOP and report it; do NOT hack the test to compile.
- If appending `Span::unknown()` would change a test's ASSERTION (not just make it compile) — STOP.

## Constraints

- Edits in `tests/` + `crates/*/tests/` only (+ `src/` ONLY if a shared test-helper lives there).
- NO behavior changes — this is compile-rot repair, not test-logic changes. NO holon-rs. NO renames.
- Do NOT commit (orchestrator scores + commits). Report: each file touched, the count fixed per class,
  any STOP triggered, and the final `cargo build --tests` + `cargo test --workspace` results.

## Closure follow-up (NOT this stone — tag it)

Add `cargo build --release --tests --workspace` (the *test build*) to the green-gate so signature
drift can't silently re-rot behind the `--lib` metric. This is the failure-engineering class-fix.
