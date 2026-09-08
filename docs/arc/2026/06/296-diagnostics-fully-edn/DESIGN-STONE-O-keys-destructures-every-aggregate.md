# DESIGN — STONE O: `{:keys …}` destructures EVERY aggregate

## WHY — a probe's coverage became the language's rule

Arc 257.2 minted `{:keys [x y]}`. **Its probe exercised `defstruct` and only `defstruct`**
(`tests/wat_lang/probe_arc257_keys_destructure.wat` — two `defstruct`s, no `defrecord`). The
`defrecord` path was never wired, and because no test asked, nobody found out. Measured on the
green tree:

```
                          {:keys [x y]}      {x :x  y :y}
  defstruct                  check=0            check=0
  defrecord                  check=1  ⛔        check=0
  :wat::holon::defrecord     check=1  ⛔        check=0
  defholon                   check=1  ⛔        check=0
```

`{:keys}` works on **one of four** aggregate kinds. The binder-first form works on all four — so
this is not a missing capability, it is one reader that never learned the others exist.

## ⛔ THE GUARD IS A FOSSIL OF THE CHANGE THAT MADE IT UNNECESSARY

`src/check.rs:12725`:

```rust
// Arc 293.2b — struct-destructure requires Aggregate with kind==Struct.
Some(TypeDef::Aggregate(a)) if a.nature == Nature::Struct => a.clone(),
```

★ **293.2b is the arc that UNIFIED struct and record into one `AggregateDef`.** `StructDef` and
`RecordDef` do not exist (`255/NOTE-2026-08-14`: *"`StructDef`, `RecordDef` and `ProtocolDef` DO NOT
EXIST. Arc 293.2b unified struct+record into `AggregateDef`"*). The predicate **cites that
unification as its authority while encoding the world from before it** — it preserved the old
two-type distinction at the exact moment the two types became one.
`[[feedback_a_walls_paperwork_can_claim_a_door_it_did_not_close]]`

## ★★ AND IT IS BACKWARDS

`Nature::is_pure`, `src/types.rs:224`, verbatim:

> *"The purity wall: `Struct` permits impurity (**holds resources**); `Record`/`HolonRecord`
> **guarantee purity**."*

**The natures differ in PURITY, not in SHAPE.** All four are aggregates with named fields, and
destructuring READS fields — purity is irrelevant to reading. So the guard as written **permits
destructuring the one nature that may hold a live socket, and refuses the three that are guaranteed
pure.** `AGGREGATE-MODEL.md` records no design reason for the restriction; there is no epitaph to
read because nothing died here.

## WHAT IT DELIVERS

```
(let [pt (:probe::Pt :x 1 :y 2)  {:keys [x y]} pt]  …)     for defstruct · defrecord ·
                                                            holon::defrecord · defholon
```

One destructuring vocabulary across every kind that has named fields. `{:keys [x y]}` stays what it
has always been: **shorthand for the binder-first form when binder and field share a name.**

## ⛔ THE ONE CONTRACT DECISION

**The predicate becomes "is this an aggregate?", never a list of natures.** A four-arm match on
`Nature` is the hand-list this arc keeps deleting — and it would go stale the next time a nature is
added, exactly as this one did when a type was removed.

## OUT OF SCOPE — AFFIRMATIVELY CUT

- **`{:keys}` on an ENUM VARIANT.** Ruled: destructuring is for ONE-SHAPE aggregates; a variant is
  reached through `match`, which is what establishes which shape you hold. It becomes available to
  variants only if `variant <: enum` lands and gives a variant its own type — that stone, not this.
- **`{:keys}` on a HashMap.** Different noun. A record is not a map (measured: `:keys` on a
  `hashmap::of` fails today, and this stone does not change that).
- **The `:attrs` rename.** Ruled: `:keys` here means *the declared field names*, and it already
  reads correctly on a struct. Renaming buys nothing and costs 11 existing sites plus Clojure
  familiarity, which is deliberately borrowed.

## THE FOUR QUESTIONS

- **Obvious?** YES. One destructuring form for everything shaped like a record.
- **Simple?** YES. One predicate, widened from a nature-equality to an aggregate test.
- **Honest?** YES. It deletes an exception rather than documenting it, and the exception had no
  stated reason to begin with.
- **Good UX?** YES. The caller stops having to know which of four aggregate kinds they hold in order
  to pick a destructuring spelling.
