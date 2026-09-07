# 296 · DESIGN STONE K — records, structs and aliases follow, and the floor becomes NAMED

> Stone J purged hand-written **enums** and walled the door. K is the sibling class — and it is a
> different shape, which is what makes it the precedent the builder is after: *"i'd be stoked to
> have records and structs move into wat… that would be an awesome precedence setter."*

## ⛔ THE DIFFERENCE FROM J, AND IT IS THE WHOLE STONE

J had **zero legitimate exemptions**: every one of the 26 enums could move, so the wall admits
nothing. K does **not**. Measured, `register_builtin` is the one builtin door and it takes exactly
two literal shapes left:

```
5 × TypeDef::Aggregate     3 CATEGORY ROOTS  :wat::core::Struct · :wat::core::Record ·
                                             :wat::holon::Record   — all `fields: vec![]`
                           2 GENERATED       types.rs:2195 and :2442, the latter a LOOP:
                                             format!(":wat::runtime::{}", variant)
4 × TypeDef::Alias         :wat::holon::BundleResult · :wat::holon::Holons ·
                           :wat::core::Bytes · :wat::core::nil
4 × register_builtin_leaf  the scalar primitives — a separate door, separate category
13  records ALREADY wat-sourced via wat_record_from!  (the move is largely DONE for records)
```

**The category roots cannot move, and not for a difficulty reason.** `:wat::core::Record` is what
`defrecord` *produces*. Declaring it with `defrecord` is the concept declaring itself. That is an
impossibility in principle, not an unbuilt capability — and the difference matters, because J taught
that "we haven't got to it" and "it cannot be" look identical from the outside until someone writes
the reason down.

★ **So K's deliverable is not a sweep. It is a NAMED, ARGUED, WALLED FLOOR** — turning "what is
still declared in Rust" from an unexamined residue into a list that defends itself. That is the
precedent: after K, a reader can ask *why is this in Rust?* and get an answer, for every survivor.

## THE ONE CONTRACT DECISION — the wall ASKS, it does not carry a list

An allowlist rots (J's own lesson about hand-lists). The substrate already holds the discriminator:

```rust
// src/types.rs:255 — "Strict inverse of `root_keyword` — the single canonical keyword→nature map."
Nature::from_root_keyword(":wat::core::Record") -> Some(Nature::Record)
Nature::from_root_keyword(":wat::holon::BundleResult") -> None
```

**A `TypeDef::Aggregate` literal is admitted if and only if `Nature::from_root_keyword(name)` is
`Some`** — i.e. it IS a category root, by construction. No permission list, nothing to keep in step;
add a nature and the wall follows, remove one and it follows. Top rung.

The `Alias` half has no equivalent oracle, so its floor IS a list — and therefore must be SHORT and
each entry must carry a reason the lint prints. `:wat::core::nil` is the candidate: it is
`TypeExpr::Tuple(vec![])`, the unit the language is built out of. Whether it can be spelled as a
`typealias` at all is STOP-2, not an assumption.

## THE PREREQUISITE

`wat_alias_register_from!` does not exist — the sibling of `wat_record_from!` (`:389`) and
`wat_enum_register_from!` (`:580`). K is its first consumer, exactly as H-3 was the first consumer
of parametric registration. `typealias` is a live wat form (7 corpus files), so only the Rust half
is missing.

## FOUR QUESTIONS

| | |
|---|---|
| **Obvious?** | YES — after K, every Rust-side type declaration answers "why not wat?" in its own text |
| **Simple?** | YES — one new macro, three alias moves, one wall that ASKS an existing map |
| **Honest?** | YES — and this is where K earns its keep: it refuses to let "cannot" and "haven't" look alike. The three roots are impossible IN PRINCIPLE and the wall says so structurally, not by permission |
| **Good UX?** | YES — a reader meeting a Rust type literal gets told what the admitted form is, and why this one is exempt |

## OUT OF SCOPE, AFFIRMATIVELY

`register_builtin_leaf`'s 4 scalar primitives — a different door, a different category (a primitive
has no wat declaration form at all), and folding them in would make a red ambiguous between the two.
Named here so the next reader knows they were considered, not missed.
