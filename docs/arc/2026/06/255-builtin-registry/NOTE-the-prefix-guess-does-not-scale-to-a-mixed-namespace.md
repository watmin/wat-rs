# NOTE — `effectful_by_prefix` cannot express `:wat::core::`, and that blocks the HOFs

> Measured 2026-08-29 while sizing P6-c-W6. **No row, nothing drawn.** This records a wall found by
> sizing, so the wave that hits it inherits the measurement.

## The mechanism, and where it runs out

`declared_purity_vs_effectful_by_prefix_census` (`src/intrinsic/mod.rs:987`) keeps **one** real
assertion: **`Effectful ⇒ effectful_by_prefix`**. The prefix list (`src/runtime.rs`,
`pub(crate) fn effectful_by_prefix`) is now:

```
:wat::kernel::  ·  :wat::io::  ·  :wat::holon::  ·  :wat::config::  ·  :wat::stream::  ·  :wat::rete::
```

Every entry is a namespace that is **mostly or wholly effectful**. W2 added `:wat::stream::` for
`next`; W5b added `:wat::rete::` for six session mutators. Both were honest: the namespace really is
effectful-leaning, and the cost was a handful of pure verbs showing up as counted disagreements
(108 → 117 → 120 across those two waves).

**`:wat::core::` is not that kind of namespace.** It holds ~82 unhomed verbs of which the great
majority are pure — `length`, `empty?`, `not`, `nth`, `reverse`. Adding it to the list to admit a
handful of effectful ones would mark the whole namespace effectful, make the prefix guess
**vacuous for the largest namespace in the language**, and inflate the disagreement census by
roughly the size of core itself.

## What that blocks, concretely

The higher-order verbs — `map` · `mapv` · `filter` · `foldl` — and the stream forcers —
`stream->vec` · `stream->pvec` · `seqable->stream`. Each **runs code it did not write**: a caller's
fn, or a thunk. That is the exact mechanism three verbs have already been ruled Effectful on:

```
:wat::stream::next          W2   forcing a thunk runs a captured wat closure
:wat::rete::eval-test/-insert  W5b  eval_inner on caller-supplied expressions
:wat::rete::collect-rules   W5c  shape-only filter, then invokes every match
```

So the honest ruling for the HOFs is very likely `Effectful` — **and making it would fail the
assertion**, with no honest widening available.

## The three ways out, none of them drawn

1. **Widen `:wat::core::`.** Cheapest, and it destroys the guess for the biggest namespace. The
   prefix stops carrying information exactly where the most verbs are.
2. **Make the fallback finer than a prefix** — a named exception list beside the prefix list, or a
   per-verb declaration the census consults. This is the "freeze names, never a count" shape the
   arc already uses twice (`FROZEN_CHECKER_DEBT_LEDGER`, `KNOWN_UNREVIEWED`), and it is the only
   option that keeps the guess meaningful.
3. **Retire the prefix fallback where the registry is authoritative.** Once a verb is homed its
   `@Purity` IS the declaration; the prefix exists to guess about verbs the registry does not know
   yet. As the campaign shrinks that population, the fallback's job shrinks with it — so the
   question may be *when does this mechanism retire*, not *how does it grow*.

⚠ **Option 3 is the one worth measuring first**, because the campaign is what changes its answer.
Nobody should widen `:wat::core::` before asking whether the prefix guess should still exist for a
namespace whose verbs are being homed one wave at a time.

## What is NOT blocked

Core verbs that run no caller code — the collection readers, the predicates, the converters — can be
homed today with `Pure` and no widening. **P6-c-W6 takes exactly those**, and treats a required
widening as a STOP rather than a chore.
