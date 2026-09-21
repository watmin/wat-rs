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

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5939/5939 passed**, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| ⭐ **THE DELTA — my tree, like-for-like** | **80 → 77** |

**Arc so far: 104 → 97 → 80 → 77.**

**Classification (77):** `unresolved reference` **44** (was 47) · `ReteCheckErrors` 16 ·
`defsurface` 8 · `ProgramBodyEvalFailed` 3 · other 1.

## ⭐ THE TEACHING IS REAL, AND PARITY IS PROVEN — not asserted

The 4b fix claimed the 17 rows now teach; 4c claimed they teach *equally*. **Both verified on the
built binary:**

```
:wat::kernel::HandlePool::new  →  "retired; use ':wat::kernel::HandlePool/new' instead"
   :remedies [#wat.kernel/Remedy {:form ":wat::kernel::HandlePool/new" :kind :retirement …}]

:wat::core::struct (arc-241 control) →
   :remedies [#wat.kernel/Remedy {:form ":wat::core::defstruct" :kind :retirement …}]
```

⭐ **Same structured `Remedy`, same `:kind :retirement`, both at CHECK time.** The brief's demand —
*"one is not a lesser citizen because it arrived through a different resolution path"* — is met and
**measurable**, not a claim in a SCORE.

## ⭐ THE CURE'S SHAPE IS THE RIGHT ONE — it deleted a door rather than adding one

> *"Deleted the early `infer_list` arm… Intercepting arms now **stand aside** so the existing door
> can fire: kernel/std prefix silent-accept `&& !is_retired(k)`; rust-scheme dispatch
> `&& !is_retired(k)`."*

The interceptors were **swallowing** retired heads before the teaching door could see them. The fix
is two guard clauses that say *"not mine"* — ⛔ **not a second consult.** And the pass-through in
`is_resolvable_call_head` / `covers` correctly **stays**: letting a retired head survive resolution
*so check can teach* is the opposite of a second door.

⚠ It also names the precedent it followed — *"same exclusion HOME-9 already used for
`:wat::std::stat::` / `list`"* — so the shape is the house's, not invented for this stone.

## ⛔ THE WHOLE 255.4 SEQUENCE — four amends, and every red was earned

`28 → 3 → 32 → 0`. Recorded because the middle number matters:

| round | reds | what it taught |
|---|---|---|
| strike | 28 | the rename stopped halfway: two **consumers** still required `::` |
| 4b | 3 | three walls demanded the NEW artifacts prove themselves (non-vacuity, replay fixture, reachability) |
| 4c | **32** | ⛔ **the teach-fix added a SECOND DOOR** — the arc's own defect, committed by its cure |
| final | **0** | delete the door, make the interceptors stand aside |

⭐ **The 32 was the most valuable red in the arc.** A plainer message that fires first looks like a
fix and is a regression in teaching. Only the arc-241 suite could tell the difference, and it did.

**VERDICT: ACCEPTED.**
