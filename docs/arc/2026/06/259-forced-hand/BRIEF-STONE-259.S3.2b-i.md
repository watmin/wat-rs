# BRIEF — Stone 259.S3.2b-i — `(:wat::program::cpu-count)` (the live host-parallelism verb)

## The work (one paragraph)

Mint `(:wat::program::cpu-count)` — a nullary, always-available host verb returning
`std::thread::available_parallelism()` as `:wat::core::i64`. It mirrors `:wat::time::now`:
a host fact answerable in ANY eval context, with NO installed program env required
(unlike the stamped env field `wat.cpu-count`, which is reachable only via
`(:wat::program::env)` behind a seam install). The brackets pool sizes its default
runner count from this verb. While here, DRY the host-parallelism logic: factor the
duplicated `available_parallelism().map(|n| n.get() as i64).unwrap_or(1)` (two sites)
into one shared helper and back the verb + both seam sites with it.

## Reach-stumble origin

The coordinator (S3.2b-ii) needs cpu-count to size its pool. The only existing source
is the env field `wat.cpu-count`, read via `(:wat::program::env)` — which RETURNS A
`MalformedForm` ERROR when no env is installed (runtime.rs:17503). The brackets probe
drives `compute` via bare `eval_in_frozen` (no seam, no install), so the env field is
unreachable for sizing. Resolution mirrors the time precedent: a host fact gets a
stamped field (snapshot, interrogation) AND a live verb (always available, sizing).
cpu-count had only the field. This stone adds the verb.

## Rooms (read in order)

1. `src/runtime.rs:17494-17512` — `eval_program_env`: the nullary-verb shape to mirror
   (arity-0 check → value). Add `eval_program_cpu_count` beside it: arity-0 check (mirror
   the `args.is_empty()` guard + `ArityMismatch { expected: 0, .. }`), then
   `Ok(Value::i64(host_cpu_count()))`.
2. `src/runtime.rs:3804-3809` — the dispatch match arms (`:wat::program::env`,
   `:wat::runtime::argv`). Add: `":wat::program::cpu-count" => eval_program_cpu_count(args, list_span),`.
3. `src/check.rs:14794-14802` — the `:wat::program::env` `TypeScheme` registration.
   Register `:wat::program::cpu-count` with `type_params: vec![]`, `params: vec![]`,
   `ret: TypeExpr::Path(":wat::core::i64".into())`, `rest_param_type: None`.
4. `src/freeze.rs:1093` and `src/kernel/spawn.rs:400` — the two existing inline
   `available_parallelism()...unwrap_or(1)` sites. Replace each with a call to the new
   shared helper.

## The shared helper

Define `pub(crate) fn host_cpu_count() -> i64` (home: `src/runtime.rs`, near the program
verbs) = `std::thread::available_parallelism().map(|n| n.get() as i64).unwrap_or(1)`.
Back the verb (room 1) AND both seam sites (room 4) with it — one source of truth.

## Blast radius

`src/runtime.rs` (helper + eval fn + dispatch arm), `src/check.rs` (one registration),
`src/freeze.rs` + `src/kernel/spawn.rs` (call the helper). No new types. No wat-source
changes. No change to the `wat.cpu-count` env field or its constructor.

## STOP triggers (rejection criteria — ship nothing, surface the gap)

- **STOP-1** — if `TypeExpr::Path(":wat::core::i64".into())` is not how an i64 return is
  spelled in a sibling nullary verb registration, mirror the sibling's exact spelling
  instead; do not guess a variant.

## Gate (run and READ the result before reporting — never chain test+commit)

1. `cargo test --release -p wat --test nursery probe_arc259_program_cpu_count -- --test-threads=1`
   → `cpu_count_is_live_and_install_free` GREEN.
2. `cargo test --release -p wat --test nursery probe_arc259_cpu_count -- --test-threads=1`
   → the env-field cpu-count tests STILL GREEN (the seam re-sourcing didn't break the field).
3. `cargo build --release` clean; `cargo clippy` clean on the touched files.

Report the real command outputs. Do NOT commit — the orchestrator weighs against its own
re-run, then commits.
