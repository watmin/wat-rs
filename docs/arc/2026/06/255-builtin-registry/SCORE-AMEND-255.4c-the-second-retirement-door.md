# SCORE-AMEND — STONE 255.4c: the teach-fix added a SECOND retirement door

Folded into `37b8d7b43` (not a repair commit). Parent: `AMEND-255.4c-the-second-retirement-door.md`.
Arc 241 suite green again. The 17 still teach at check time. Do not start 255.5.

## What was wrong

The 4b teach-fix put a retirement consult **first** in `infer_list`, before the match
that already owns teaching. That arm emitted a plainer `MalformedForm` and **preempted**
the dedicated 241 arms (`struct` → Stone 241.8 reason + `did you mean: … [replaces a
retired form]`). 32 tests went red. A second door.

## What changed — no new door

Deleted the early `infer_list` arm.

Retired heads still **reach** check (`is_resolvable_call_head` / rust `covers` pass-through
stays — that is not a second door). Intercepting arms now **stand aside** so the existing
door can fire, same exclusion HOME-9 already used for `:wat::std::stat::` / `list`:

- kernel/std prefix silent-accept: `&& !is_retired(k)`
- rust-scheme dispatch: `&& !is_retired(k)`

They then fall through `_ => {}` to **door 1** (`check.rs` ~5971): `MalformedForm` with
`remedies_for` — `did you mean: {replacement} [replaces a retired form]`.

`:wat::kernel::HandlePool::new` and `:wat::core::struct` are the same kind: check-time
`MalformedForm` carrying a retirement `Remedy`. Struct still uses its dedicated 241.8
arm (richer reason). HandlePool uses door 1 (the generic consult). Neither is a lesser
citizen: both attach the structured remedy. Arc 241's path **can** serve the 17 once
resolve lets them through and the interceptors do not swallow them.

## Walls I ran

- arc 241 suite + `wat_arc143_define_alias` + `probe_def_not_special` — **116 passed**
- `every_discovering_gate_declares_how_it_knows_it_reached_something` — pass
- `every_recorded_migration_is_fixtured_or_runed` — pass
- `retirement_table_is_fully_reachable` — pass
- crate clippy `-p wat --all-targets -D warnings` — **0**

Floor / workspace clippy / census / delta: orchestrator. Do not push. Do not start 255.5.
