# NOTE — the three nature roots are registered as STRUCTS, and one of them has no patch

**Found 2026-08-15, by asking what three table rows *said*** — not by hunting a bug. The builder,
reading the 296 stone's inventory: *"hrm..... these are strange..... what do these 3 /say/?"*

## THE MODEL (builder, verbatim — this is correct and nothing below disputes it)

> *"structs are allowed to hold anything.... including impure values.... records may not hold any
> impure values... they are always pure data ..... and `:wat::kernel::Peer` isn't a data holder,
> its a declaration of the thing... a Peer is a thing you can do request/reply with.... holon
> records are just core records who bear a constructed hologram ready to be queried.. the
> hologram is ready at all times with a holonic record."*

`Nature`'s four variants encode exactly that, and `is_pure()` — `!matches!(self, Struct | Peer)` —
is the wall. **The enum is right. The three ROOT REGISTRATIONS are not.**

## WHAT THE THREE ROOTS ARE

Not data records — the **top of the is-a lattice**, the three flavors a user's aggregate subs
from. `src/types.rs`, verbatim:

- `:wat::core::Struct` — *"the nature-root for all struct types. Registered FIRST so every
  subsequent parsed struct finds it in the registry. User structs register
  `:Name <: :wat::core::Struct` via `nature.root_keyword()`."*
- `:wat::core::Record` — *"Opaque umbrella type for the wat-record hologram… Registered as opaque
  zero-field struct so the TypeEnv contains the path."*
- `:wat::holon::Record` — *"the 'holonic record' flavor — a record that carries a HolonAST
  alongside its struct-form… mirrors `:wat::core::Record` exactly."*

Zero fields because they are lattice roots, registered so the subtype edge has a target that
resolves. `root_keyword()` names a fourth — `:wat::kernel::Peer` — deliberately **never
registered**: `Nature::Peer` is an off-ladder surface marker, and `AggregateValue` asserts it
never reaches a value (`value.rs:1302`, `unreachable!`). That one is correct as-is.

## WHAT IS **NOT** BROKEN — measured, so the finding below is not over-claimed

| claim | probe | result |
|---|---|---|
| a struct may hold impure values | `Nature::Struct.is_pure() == false` (permissive floor) | ✓ |
| a record may hold only pure values | `[r <- :wat::holon::Record]` in a `defrecord` | ✓ **the containment rule FIRES** |
| a struct is REJECTED from the `:wat::core::Record` umbrella | `takes-record(:t::S)` | ✓ `TypeMismatch` |
| a record satisfies `:wat::core::Record` | `takes-record(:t::Pt)` | ✓ clean |
| a record satisfies `:wat::core::Struct` | `takes-struct(:t::Pt)` | ✓ clean (rank ladder: `:Struct` is the permissive floor) |

The lattice works. The wall works. This is a narrow defect, not a rotten foundation.

## WHAT **IS** BROKEN

### 1 — both Record umbrellas are registered `nature: Nature::Struct`

Verbatim from the registrations. Under the model that reads *"a record may hold impure values"* —
the exact inverse of what a record is. The cause is legible in the comments: someone wanted an
**opaque** placeholder and reached for `Struct` as the generic choice. **`opaque` got conflated
with `Struct`.**

### 2 — `is_pure_type` carries a hardcoded patch to undo #1

`src/check.rs:13042`, and its comment names the failure it prevents:

> *"the `:wat::core::Record` umbrella means 'any record' — pure… Must short-circuit BEFORE the
> `types.get(p)` aggregate arm, which sees Record registered as `Nature::Struct` (opaque umbrella)
> and **would return a FALSE POSITIVE impure verdict**."*

So the substrate already knew, and papered over it at the consumer instead of fixing the
declaration. Classic stem-not-root.

### 3 — ⛔ THE PATCH WAS NEVER GIVEN TO THE SIBLING

`wat::holon::Record` appears **nowhere in `is_pure_type`**. Measured differential — same position,
two sibling umbrellas, opposite verdicts:

```
[r <- :wat::core::Record]    →  clean
[r <- :wat::holon::Record]   →  ImpureFieldInPureAggregate
   "field \"r\" has impure (struct) type \":wat::holon::Record\""
```

**No pure aggregate in wat can hold a field typed "any holon record"** — while "any record" is
fine. The holon flavor is the one that is *most* certainly pure data (a core record plus a
hologram), and it is the one the type system calls impure.

**Honest bound on severity:** this is a **latent capability gap, not a wrong answer being
produced today.** Nothing in the corpus currently types a field as the holon umbrella, which is
precisely why it survived — the twin was never exercised. It produces no bad output; it refuses
a legal program. That is the mirror lens again ([[feedback_the_mirror_is_an_instrument_not_a_fix]]):
enumerate one side of a pair, demand a twin for each state, and the missing one is the finding.

## THE FIX, AND WHY IT IS THE ROOT NOT THE STEM

Register the two Record umbrellas with the nature they ARE — `Nature::Record` and
`Nature::HolonRecord` — and **delete the `is_pure_type` special case**, which then has nothing to
paper over. The differential probe above is the gate: red today, green when it is right.

⚠ **Check before striking:** `register_builtin` derives the subtype edge from
`nature.root_keyword()`, so changing a root's own nature changes the edge it registers — and for
a root, `child == root`, which the guard already skips. Verify that skip still fires, and that
the seeded `:wat::holon::Record <: :wat::core::Record` edge survives, before believing a green.

## WHY THIS BLOCKS THE 296 MIGRATION STONE

It converts **STOP-1 from a precaution into a prediction.** When these three are transcribed to
wat, they will be written as what they *are* — a `defrecord` root carrying `Nature::Record` — which
DIVERGES from the Rust literal and raises `DuplicateType`.

Two paths, and the ruling matters:

- **(a) fix the natures first**, delete the patch, then migrate onto correct ground.
- **(b) migrate as-is**, transcribing `nature: Struct` to keep `==`.

**(b) writes the lie into the source of truth**, which is the one thing this whole chain exists to
stop. (a) is small — three natures, one deleted special case, one differential gate.
