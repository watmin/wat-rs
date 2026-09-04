# DESIGN — a minted accessor the resolver cannot see makes 66 names unwritable

## Why

Work-list **C15**: *"a synthesized record accessor can never be attested, so any `wat-scripts/` file
touching one is unavoidably RED."*

**Driven at HEAD `b5c068ebd`, not read.** A four-line probe under `wat-scripts/scratch-pad/`:

```wat
(:wat::core::defn :zz::via-of [n <- :wat::rete::DerivationNode]
    -> (:wat::core::PersistentVector :- [:wat::rete::DerivationStep])
  (:wat::rete::DerivationNode/via n))
```

It type-checks. It runs. And `every_rete_name_in_wat_scripts_code_resolves` goes **RED**:

```
wat-scripts/scratch-pad/zz-c15-probe.wat:4  :wat::rete::DerivationNode/via
```

`via` is declared at `wat/rete.wat:377`, inside a `defrecord` at `:374`. The accessor is **minted and
live** — the resolver simply cannot see it, because accessors are generated at freeze from the
`defrecord` and never appear textually under `src/` or `wat/`, which is the only place `attested()`
looks.

## The population, counted by balanced parse

**19 `defrecord`s in `wat/rete.wat`, 66 synthesized accessors** — every one of them unwritable in
`wat-scripts/` today:

| record | fields |
|---|---|
| `Export` | 11 · `v abi classes fields nodes conds drivers progs folds rhs deps` |
| `Session` | 8 · `network rules alpha-memory beta-memory production-memory facts next-id query-memory` |
| `AccumulateNode` | 5 · `id result-var acc-form from-alpha-id children` |
| `DerivationStep` | 4 · `supporting pattern bindings constraints` |
| `Rule` `Query` `AlphaNode` `TestNode` `NegationNode` `ExistsNode` `QueryNode` `DerivationNode` | 3 each |
| `Token` `Element` `RootJoinNode` `HashJoinNode` `ProductionNode` `Support` `Explained` | 2 each |

⚠ **A first count said 46 and was wrong** — the regex stopped at the first `])`, so every record whose
last field carries a nested type (`(:wat::core::PersistentVector :- [...])`) lost it. It reported
`DerivationNode` as 2 fields, missing `via` — **the exact accessor the probe drove.** Anchor a count
on a known-positive before quoting it.

**This is not a lint being strict. It is the rete data model being unaddressable from the corpus the
lint guards.**

## The contract decision, pinned

**Resolve `:wat::rete::<Type>/<field>` against the DECLARATION, and check the field.**

A name of that shape resolves when `<Type>` is an aggregate declared in `wat/rete.wat` **and**
`<field>` is one of its declared fields. Nothing weaker:

- **Not "any token with a slash"** — that admits `DerivationNode/nonexistent`, and a resolver that
  cannot refuse a misspelled accessor is a resolver that has stopped checking. The field half is
  what keeps this a gate.
- **Not a rune per accessor.** `rune:lint(rete-name-unminted)` would be **a lie about a minted
  name** — the row says so and it is right. 66 runes asserting "deliberately unminted" about names
  that are minted would be the worst outcome available.
- **Not an allowlist.** Same defect one layer along: it goes stale the moment a field is added, and
  the whole reason this gate exists is that lists cannot notice what was never added to them.

The declaration is the authority, the same way `RETE_OPS` is the authority for the operator
namespace. This extends the existing three-source universe (`rows ∪ attested ∪ known-forms`) with a
fourth whose source of truth is a file already in the tree.

## Out of scope = REJECTED

- **`:wat::core::` and every other namespace's accessors.** The gate only governs `:wat::rete::`;
  widening it is a different, larger question and must be measured before it is drawn.
- **Making the 66 accessors *used* anywhere.** This strike makes them writable, not written.
- **Any change to how accessors are synthesized.** The generation is correct; the resolver's model
  of what exists is what is incomplete.
