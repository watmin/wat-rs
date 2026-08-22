# DESIGN — `type-equal?`: types are data everywhere except in a macro

> **Builder, 2026-08-22, on `defservice` comparing rendered type names:** *"this feels dumb?… we are
> edn everywhere?… what isn't data to compare?"*

## The gap, stated

`TypeExpr` derives `PartialEq, Eq`. Structural type comparison exists — in Rust, at check time.
**A wat macro cannot reach it.** Grepped: the only type-reflective verb a macro body may call is
`type-params-used-in` (arc 109 β-ii-c). There is no equality.

So `defservice` — which lives entirely in a macro — compares types the only way it can: by rendering
both sides to strings and testing `=`. That is not a design preference. **It is the absence of a door.**

## What that absence has cost, measured

| site | how it broke |
|---|---|
| `:peers` (`service.wat`) | builds `"wat::kernel::Peer<{r},{o}>"` and compares it to the declared `:ephemeral` field type. ②-iii migrated the declaration to a form; the strings stopped matching. **This is the failure that blocked ②-iii.** |
| the `:hibernate` return check | `keyword/to-string` on a node that became a form → raises before any comparison. Six red tests. |
| the other 9 COMPARE sites | same shape, not yet triggered because nothing has fed them a migrated spelling |

Every one is *"two spellings of one type, compared as text."* Fixing them one at a time migrates the
defect. **The door removes the class.**

## The intrinsic

```clojure
(:wat::core::type-equal? [a <- :wat::WatAST  b <- :wat::WatAST] -> :wat::core::bool)
```

```rust
parse_type_node(a)? == parse_type_node(b)?
```

**True iff the two nodes denote the same type, whatever spelling each wears:**

```clojure
(type-equal? :wat::kernel::Peer<A,B>   (:wat::kernel::Peer :- [A B]))       ;; true
(type-equal? :wat::core::Vector<wat::core::HashMap<K,V>>
             (:wat::core::Vector :- [(:wat::core::HashMap :- [K V])]))       ;; true — nested
(type-equal? :wat::kernel::Peer<A,B>   :wat::kernel::Peer<B,A>)             ;; false
```

## ★ The one contract decision — RAISE, do not return false

Given a node that is not a type at all, `type-equal?` **raises**. It does not return `false`.

*"These are not both types"* is a different fact from *"these are different types"*, and collapsing
them into `false` makes a malformed input indistinguishable from a legitimate mismatch — a silent
pass at exactly the sites that exist to catch mistakes. This arc shipped that failure once already
today: a check that could not run returned a green.

## What it is NOT

- **Not subtyping.** `is_subtype` answers that, in Rust, at check time, over the edge table.
- **Not AST equality.** `Peer<A,B>` and `(Peer :- [A B])` are deliberately different ASTs and the
  same type. That difference is the entire point; a structural node comparison would report `false`.
- **Not a checker-time answer.** It compares DECLARED spellings at macro-expansion time. The
  `<T>`-vs-`<?454>` mismatch is a fresh-unification-var problem that exists only after inference, and
  Stone 118.3-B already handles it in `assignable`'s `else` branch. This neither helps nor hurts there.

## Three registrations — the exemplar is `type-params-used-in`

```
src/intrinsic/reflect.rs   #[wat_intrinsic(":wat::core::type-equal?")] + a RUNNABLE @example
src/macros/eval.rs:666     the F5 allow-list — MANDATORY; the verb exists to be called from a macro
src/rete/purity.rs:345     the purity ruling — pure ∧ deterministic ∧ total, RULED not parked
```

⚠ **F5 is not optional garnish.** A macro body may not call a user-defined function at all, and the
admission list is default-deny. An intrinsic missing from it is refused at DEFINITION — the failure
mode that took the stdlib down for 3029 tests.

⚠ The purity gate's own remedy says parking in `KNOWN_UNREVIEWED` *"is the LAST resort and is only
honest for a verb whose ruling is genuinely open."* This one's is not: it reads two nodes, allocates
nothing observable, touches no world state, and returns a bool. Rule it.

## The four questions

- **Obvious?** YES — the substrate's own answer to "are these the same type?" made callable from the
  one place that cannot reach it.
- **Simple?** YES — one comparison, on a value that already derives `Eq`. No new representation, no
  canonical rendering, no second parser.
- **Honest?** YES, and it is the axis that fails today: `defservice` compares renderings and calls the
  result a type judgement. Two spellings of one type answer `false`, and the code cannot tell that
  from a real mismatch.
- **Good UX?** YES — the 11 COMPARE sites become two-line rewrites, and every future spelling change
  costs nothing because nothing compares spellings.

## What this stone does NOT do

It **mints the door only**. Rewriting the 11 COMPARE sites to use it is a separate stone — this one
must land, floor green, and be usable before anything depends on it.
