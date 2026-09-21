# SCORE-AMEND — STONE 255.4b: 28 → 3. Three walls asking the new artifacts to prove themselves.

Folded into `82575be23` (not a repair commit). Parent: `AMEND-255.4b-prove-the-new-artifacts.md`.
The 3 remaining floor reds are gone. Delta not re-run here (orchestrator). Wat-side work not undone.

## 1. `one_member_join.rs` non-vacuity

`git ls-files` of `tests/`, `wat/`, `wat-scripts/`, `wat-tests/` — **2459** paths driven 2026-09-21.
Floor `> 1500` with a `NON-VACUITY` marker introducing the assert. A glob that matches
nothing cannot pass over an empty set. Same shape as `one_param_spec.rs`.

## 2. Replay fixture for `type-member-colon-to-slash`

`wat-scripts/fixes/replay/type-member-colon-to-slash/{before.pre,after.post,ORACLE}`.
Synthetic pre-image (the live corpus is already rewritten). Spec quotes the header
(`Type::member → Type/member`). Near-misses: already-slash `Bytes/to-hex` and nested
type name `Cache::GetRequest` stay. Codemod-generated `after.post`, idempotent.

## 3. Door 1 teaches the 17 silent rows

The rows existed; resolve rejected them as `UnresolvedReference` before check, so
door 1 never ran. For `:wat::kernel::HandlePool::new` the kernel-prefix silent-accept
would also have waved them through if they had arrived.

One table, two consults of it — not a second `is_known_type`:

- `is_resolvable_call_head` / rust `covers`: a retired head is allowed through so
  check can teach, instead of dying as "unknown" / "not covered by use!".
- `infer_list`: door 1 **first**, before the kernel-prefix arm and before rust-scheme
  `UnknownCallee`. Exact table hit → `MalformedForm` `'{old}' is retired; use '{new}'`.

## Walls I ran

- `every_discovering_gate_declares_how_it_knows_it_reached_something` — pass
- `every_recorded_migration_is_fixtured_or_runed` — pass
- `retirement_table_is_fully_reachable` — pass
- `one_member_join` / `one_variant_separator` — pass
- crate clippy `-p wat --all-targets -D warnings` — **0**

Floor / workspace clippy / census / delta: orchestrator. Do not push. Do not start 255.5.
