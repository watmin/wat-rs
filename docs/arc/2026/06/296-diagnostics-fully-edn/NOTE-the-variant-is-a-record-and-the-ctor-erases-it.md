# NOTE — a variant IS a tagged record, and the constructor ERASES that

**Found 2026-09-07**, while drawing Stone M. Not designed — each step was a measurement that
refused to be what the orchestrator had just claimed. Recorded because the conclusion is worth
more than the stone that produced it, and because three queued stones turn out to be one finding.

## The chain, in the order the disk gave it up

**1. A variant value already carries its field names.** `src/value/value.rs:1150`:

```rust
pub struct EnumValue {
    pub type_path: String,
    pub variant_name: String,
    /// Field names in declaration order. **Same length as `fields`, always.**
    /// Arc 296 G′: carried, never looked up — the enum mirror of `AggregateValue.names`.
    pub names: Arc<Vec<String>>,
    pub fields: Vec<Value>,
}
```

★ *"the enum mirror of `AggregateValue.names`"* — the substrate's own comment. **At the VALUE
level a variant is already a record plus a tag.** Nothing needs adding there.

**2. The constructor throws that away.** `register_enum_methods` (`src/declare/register.rs:1296`)
gives every variant ctor the return type `parametric_decl_type(enum_name, params)` (`:833`) — the
ENUM type. Confirmed independently by the checker refusing a destructure:

```
:wat::core::let: parameter struct-destructure (x y) expects a struct type; got :usr::Shape
```

```
today       Circle ──erased──▶ Shape      information DESTROYED at construction
proposed    Circle ──<:──────▶ Shape      information KEPT, widened on demand
```

Erasure buys the safe direction for free and makes the precise direction UNEXPRESSIBLE.

**3. wat already HAS subtyping — with the operator written out.** `src/types.rs`:

```rust
subtype_edges: HashMap<String, Vec<String>>,     // :542 — name → parent FQDNs, GENERAL
Nature::Struct => -1, Record => 0, HolonRecord => 1,   // :239 — a rank FLOOR, not exact kind
/// Every parsed aggregate registers `:Name <: root_keyword()`.
/// Non-nature-root parents are REJECTED AT PARSE TIME.   // :304
```

Arc 293 **annihilated inheritance while keeping subtyping** — the correct split. The edge map is
general; a parse-time wall admits only `:Name <: <nature root>`.

## ⛔ THE QUESTION THAT STONE ACTUALLY ASKS

Not *"add subtyping"* — wat has it. **`Variant <: Enum` is ONE ENTRY in an existing map, behind a
wall built for a different relation.** So: **is `Variant <: Enum` inheritance?**

It is not. Inheritance is *"Circle inherits Shape's fields and behaviour"* — a class hierarchy,
which is what 293 killed. `Variant <: Enum` is **tagged-union membership**: Circle does not inherit
from Shape, it IS one of Shape's cases. Sum type, not hierarchy. Two different relations sharing an
arrow — and a wall that cannot tell them apart over-refuses.
`[[feedback_a_predicate_can_be_wrong_in_both_directions]]`

## ★ THREE QUEUED STONES ARE ONE FINDING

Queued separately, for unrelated reasons, before any of this was visible:

```
Stone M            ctor becomes a map           construction goes record-shaped
:keys on defrecord destructuring goes uniform   across ONE-SHAPE aggregates
variant <: enum    a variant IS one of those    inherits BOTH, for free
```

The blocked destructure error was *"expects a **struct type**"* — so once a variant HAS a
struct-shaped type, `{:keys [d]}` on it is not a new feature; it is the `defrecord` stone's feature
arriving somewhere new. Likewise `defclause` (72 live sites, already the open-surface router)
dispatches per variant with no new dispatch mechanism — the entity-kind answer, not a type-system
reach.

**One mechanism, three symptoms:** `process-rect` unwritable · `defclause` unable to route per
variant · `{:keys}` with nothing to bind against. All three are constructor erasure.

## ⚠ THE ONE THING TO MEASURE BEFORE DRAWING IT

Can `Shape.Circle` be a type WITHOUT becoming a second way to spell a record? If a variant type is
fully general, someone will return `Shape.Circle` where they meant a record, and the language has
two spellings for one-shape data — the exception-shaped thing this arc keeps deleting. The likely
honest answer is that a variant type is a NARROWING usable in parameter and dispatch position, not
a general-purpose type to build signatures around. **That is a measurement, not an opinion, and it
is the first act of that stone.**

## Provenance

The builder derived the substitution ladder unprompted and got the direction right — *"a thing who
accepts a struct can also accept either record"* is exactly `candidate.rank() >= required.rank()`,
and the arrow's direction is the half that is usually inverted. Barbara Liskov, 1987; formalized
with Jeannette Wing, 1994. The ladder was in the substrate before it had the name.
