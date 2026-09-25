# WEIGH — STONE 255.28: purity sees through a generic type — ACCEPTED

**Executor commit `f2bcb26d0`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured

| row | measured | result |
|---|---|---|
| tree | `git status` | clean |
| floor | `.floor/2026-09-25T04-03-26Z` → `04-12-07Z` | 1 red (the stone's own: its first draft renamed the function holding 3 ledgered compares; fixed at the cause, not by touching the ledger) → **`6106 tests run: 6106 passed`** |
| ⭐ Shared through a generic, as a field | pre (`79febbaba`) vs new | **0 → 1** |
| ⭐ Shared through a generic, as a self-peer payload | pre vs new | **0 → 1** |
| the Wire twin | pre vs new | 0 → 0 |
| the IDE's "4 arguments" error | `src/check.rs:15040/15175` | **stale**: a 3-argument call matches a 3-argument fn, and `src == HEAD` |

Taken from the report without re-running:

- clippy 0;
- census byte-identical (215, `no STOP-8`);
- delta NEW 2 / RECOVERY 0;
- ledger 211;
- 12 exact-assert rows, every accepted row with a refused twin, and every refused row except the two
  controls accepted pre-stone.

## What landed

`is_pure_type`'s parametric fallback also asks `declared_instance_is_pure`:

- for a declared `Aggregate`/`Enum` head, the declared nature decides first (a `Struct` is impure, an
  `Impure` enum is impure);
- then the arguments are substituted into every field and variant field, and each must be pure;
- the arguments are still asked too, so a verdict **only ever gets stricter**;
- an unbound `:T` is decided at instantiation;
- a thread-local guard stops recursion.

All five consumers now see through generics: the §7 peer wall, the aggregate/enum containment walls, the
holon EDN widening, `derived_nature`, and the surface-protocol gate.

## ⭐ No fallout, and why that is honest

The census is byte-identical. The lineage `Status` and bracket `PoolMsg` class (a) the brief predicted
**did not appear**, because **no wall asks their purity yet**: the thread-spawn producer, `send`/`recv`
and the `ThreadSelfPeer` hatch call no purity check. They become visible when stones 2–3 put a wall there.
That is the executor's prediction, not a measurement. **The wall can now see; nothing yet asks it where
those channels are.**

## Carried

- The refusal text says *"impure (struct) type"* even for a generic enum. The wording predates this stone.
- Parametric `Newtype`/`Surface` heads are unchanged; none exist in `wat/`.
