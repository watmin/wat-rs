# BRIEF — STONE 255.28: purity sees through a generic type

**Drawn 2026-09-25 against `main` @ `034eeea82`.** Floor 6094/6094, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 211. **Executor tier: Opus.**

## Ruled 2026-09-25 — only data crosses a comm

Builder: *"all things are pointing to only data may cross over a comm … we have ephemeral and durable on
services for this exact thing … resources may only be declared on ephemeral structs."* It was already
the design:

- the defservice header: `:durable` *"crosses the wire"*, `:ephemeral` *"resources + peer clients; never
  crosses"*;
- `Nature::is_pure` (`src/types.rs` ~:489): `Struct`/`Peer` are impure, and a peer *"crosses no comms;
  only its address does"*.

The thread escape hatch (`ThreadSelfPeer`) violates it. The cure is three stones in order:

1. **This stone:** purity sees through generics.
2. A thread address is data (B3 without auth; the builder: *"auth on threads is nonsense"*).
3. The hatch closes.

**This one comes first because the wall must see before it can refuse.**

## Why — measured (`FINDING-what-relies-on-the-thread-escape-hatch.md`)

`is_pure_type` (`src/check.rs` ~:14966) judges a parametric head by its **type arguments only**
(`_ => args.iter().all(|a| is_pure_type(a, types))`). It never substitutes them into the head's declared
fields or variants. So:

- `(ThreadSelfPeer :- [(Address :- [i64 i64 Transport.Shared]) i64])`: impure, and correctly refused
  once the hatch is closed;
- **the same address inside a Pure generic enum `(E :- [Transport.Shared])`, whose variant field is
  `(Address :- [… T])`: judged PURE.** That is exactly defservice's `Status` shape, and why the lineage's
  2461 Shared-address sends were invisible to the static wall.

Probe (in the measurement worktree): `wat-scripts/scratch-pad/probe-tsp-hatch-generic-enum-blind.wat`.
**Recreate it** under `tests/types/` as this stone's rows.

## The work

1. **A parametric type's purity is its declaration's purity, instantiated.**
   - For `(Head :- [A…])` where `Head` names a user/stdlib `Aggregate` or `Enum`:
     - the declaration's nature/purity marker still decides first. A `Struct` is impure, whatever its
       arguments;
     - then **substitute the arguments into every field and variant field type** and require each to be
       pure.
   - A builtin container (`Vector`, `Option`, …) keeps "pure iff its arguments are pure": its fields are
     its arguments.
   - Guard recursion (a self-referential generic).
   - Keep the type-variable guard's behaviour for an **unbound** variable: purity is decided at
     instantiation.
2. **Census the consumers of `is_pure_type`**: the record-field wall (`ImpureFieldInPureAggregate`), the
   §7 peer wall, the hatch-closed checks, and whatever else. Each now sees through generics. Report every
   site.
3. **Rows** (`tests/types/probe_arc255_28_*`):
   - the probe's control A (refused) and subject B, a Shared address through a Pure generic enum, **now
     refused** where a pure type is required (a pure record field, and a `Peer` payload through a
     producer);
   - the `Wire` twin accepted;
   - a generic `Struct` instantiated with pure args is still impure;
   - a Pure generic with an unbound type variable is still decided at instantiation.

   Show each row's pre-stone rc.

## STOP triggers

1. **Fallout** (a program the stdlib or corpus relies on that becomes impure once seen through):
   classify every site as (a) a genuine resource-in-data the old blindness hid, or (b) a checker case the
   change broke. The escape-hatch finding predicts the lineage `Status` (thread tier) as class (a).
   ⛔ **Do not fix class (a) here.** It is stones 2–3. **If (a) blocks the floor, STOP and report the
   sites verbatim**: the three stones may then have to land together, and that is the builder's ruling.
   Any (b) → STOP, report.
2. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| subject B (Shared through a generic) | refused; pre-stone accepted |
| the Wire twin and the unbound-var row | accepted |
| a generic Struct | impure |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` (or classified) · NEW 2 / RECOVERY 0 · ≤ 211 |

Runtime prediction: 2–3 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name the
  arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words on a pre-stone build from this brief's commit. Capture `rc=$?`
  on the next statement.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents. If you stop without committing, revert your own edits and
  save the patch to the scratchpad `s28/`.
