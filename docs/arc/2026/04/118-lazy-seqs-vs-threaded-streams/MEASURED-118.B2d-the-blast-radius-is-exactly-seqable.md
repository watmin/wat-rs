# MEASURED — 118.B2d's blast radius. Corpus-wide, form-tree, 2026-08-18.

**This is gate item 2 of the B2d stone**, which forbade briefing the fix until the blast radius was
MEASURED rather than guessed: path (1) of the parametric-surface member resolution
(`src/check.rs:4926`) is traversed by **every** parametric surface's method call, not just
`Seqable`'s.

**Instrument:** `wat-scripts/scratch-pad/census-parametric-surface-bindings.wat` — walks the form
tree via `read-string` → `ast->children`, **recursively** (a `defsurface` can sit inside a `do`, so a
top-level-only scan would undercount). Run over all 491 corpus `.wat` files. It prints raw source and
does not classify; classification is below, by reading, with the raw text on the record.

## The counts

```
491 files walked
118 defsurface occurrences  →   13 DISTINCT PARAMETRIC surfaces (name carries <…>)
 27 extend-type occurrences →   25 distinct (type, protocol) pairs
```

The 13 parametric surfaces: `probe::Pair<A,B>` · `probe::Pair2<A,B>` · `probe::PCache<K,V>` ·
`probe::PCtor<K,V>` · `sq::Seqable<T>` · `wat::cache::Cache<K,V>` ·
`wat::capability::Dialable<S,R>` · `wat::capability::TypedCapability<S,R>` ·
`wat::core::Seqable<T>` · `wat-tests::BareBox<T>` · `wat-tests::Box<T>` · `wat-tests::Pair<K,V>` ·
`wat-tests::PCache<K,V>`.

## ★ The classification — every row accounted for

A satisfier is in the **BROKEN class** when its `extend-type` binds the surface's params to a type
**VARIABLE** (`:Seqable<T>`). It is **SAFE** when it binds them to **CONCRETE** types
(`:Pair<wat::core::i64,wat::core::String>`) — that is the case path (1)'s own comment describes, and
it works today.

**BROKEN class — 8 rows, and every one of them is `Seqable`:**

```
wat::core::Vector            AS  :wat::core::Seqable<T>
wat::core::List              AS  :wat::core::Seqable<T>
wat::core::PersistentVector  AS  :wat::core::Seqable<T>
wat::stream::Stream          AS  :wat::core::Seqable<T>
  … and the same four against :sq::Seqable<T>, a probe-fixture copy of the same surface
```

**SAFE / CONCRETE parametric — 4 rows. These are the REGRESSION GUARD:**

```
probe::ISBox   AS  :probe::Pair<wat::core::i64,wat::core::String>
probe::Multi   AS  :probe::Pair2<wat::core::i64,wat::core::String>
<Handle>       AS  wat::capability::Dialable<{Op},{Reply}>        (macro-generated)
<Handle>       AS  wat::capability::TypedCapability<{Op},{Reply}> (macro-generated)
```

The two macro rows were resolved by reading `wat/service.wat:2601-2618`, not assumed: `dialable-ty`
and `typedcap-ty` are built by `string::interpolate` of the **service's own concrete Op/Reply type
names** (`proto-op-ty-str`, `proto-reply-ty-str`), so they expand CONCRETE.
(`[[feedback_an_adjacent_implementation_is_not_the_subject]]` — the census printed the *rendered*
`(:wat::core::unquote …)` form, which a grep for the source text `~dialable-ty` would have missed.)

**MONOMORPHIC surfaces — every remaining row.** They take the `s.type_params.is_empty()` identity
branch and cannot be affected by any change to path (1). This includes both callers of the
`extend-surface` macro (`wat/core.wat:1856`), whose `~surf` is supplied by the caller — checked:
`:k5::HasX` and `:acc::Adder`, **both monomorphic**.

## What this licenses, and what it does NOT

★ **The broken class is EXACTLY `Seqable`.** Nothing else in the corpus binds a parametric surface's
params to a variable. That is far narrower than the stone feared, and it is now measured.

⛔ **But a narrow BROKEN class is not a narrow CHANGE.** The 4 concrete rows traverse the same path
(1) and must not move. So:

- The fix must be **ADDITIVE** — it may only fire where the satisfier's binding is a type variable,
  leaving the concrete-binding path byte-identical. That is the same safety shape the arc-278
  record-top fix used ("only ADDS the supertype, so it can never make a call that dispatches today
  stop dispatching").
- **The 4 concrete rows are the regression guard**, and `probe::ISBox`/`probe::Multi` already have
  fixtures. They must be re-run and stay green.
- ⚠ **`Cache<K,V>`, `Box<T>`, `BareBox<T>`, `PCache<K,V>`, `PCtor<K,V>` have NO extend-type row at
  all** in this census. They are declared and (as far as the corpus shows) unsatisfied, so they
  exercise neither path. That is worth a second look on its own — it is the UNADOPTED class (task
  #48) wearing a surface — but it is **not** B2d's business and it does not gate the fix.

## Gate status for B2d

```
[x] 1. disconfirming probe, committed   tests/types/probe_stone_118_b2d_generic_satisfier{,_neg.wat.bad,_pos.wat}
[x] 2. blast radius MEASURED            this document
[ ] 3. four questions on the fix's shape — extend path (2)'s guard / add a third path / bind at
       registration. NOW POSABLE: (2) is answered, and it says the fix must be additive and that
       exactly one surface is in the broken class.
```
