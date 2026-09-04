# DESIGN — STONE: the registry can be enumerated

> **Builder, 2026-09-04:** *"should.... we make this now?.. is now an appropriate time to
> implement such tooling?"* → *"then... we build such that we can see"*
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]` item **7** (reflection). This stone
> closes a hole in that item: the registry answers **"what about X?"** and cannot answer
> **"which X satisfy P?"**

## The gap, measured

From wat, the registry answers per-name — `metadata-of`, `lookup-define`, `signature-of-defn`,
`return-type-of`, `extract-arg-types`. Every one takes a name you already have.

There is exactly **one** enumeration surface: `(:wat::intrinsic::examples)` walks `all_entries()`
and returns 591 elements — **measured by running it** — but projects only examples.

```
answerable from wat     what are :wat::core::quote's properties?
NOT answerable from wat which rows are Kind::SpecialForm?
                        which rows carry no @syntax?
                        which rows are @Totality Partial?
```

`all_entries()` is Rust-only, and every caller is a `#[test]` body inside `src/intrinsic/mod.rs`.

## ★★★ THE CONSUMER IS NAMED BY THE SUBSTRATE ITSELF

`wat/runtime-meta.wat:241`, in `Totality`'s own enum declaration, verbatim:

> *"★ THIS VARIANT IS THE WORK LIST: the totality endgame's census is
> `all_entries().filter(|e| e.totality == Partial)`."*

**The substrate names a census as the work list for the final phase of the builder's roadmap**
(*"registry → crates → clojure syntax → then we force everything to totality"*) — and that census
cannot be run from a wat program. This stone is that phase's instrument, built before the phase
needs it rather than after.

That makes this a step-1 stone with a **named** step-2, which is the campaign's own bar. It is not
tooling for the orchestrator's convenience.

## ⛔ WHAT THIS DOES **NOT** FIX — stated because I claimed otherwise first

My first framing was that this would let the censuses I got wrong today be re-derived by query.
**That is false and the acceptance rows must not rest on it.** Those failures were:

```
special_forms.rs's insert() count   30 vs 35   SOURCE TEXT — the registry has no opinion
the #holon spelling census          10 vs 32   CORPUS TEXT — likewise
the arity tokenizer                 broken     CORPUS TEXT
```

This stone answers **registry** set questions only. A census of Rust source or of the `.wat` corpus
stays a grep, and stays as fallible as it was. Claiming otherwise would be selling the stone on a
capability it does not have.

## ⛔ AND IT MUST NOT BE USED TO DERIVE THE LEDGERS

The obvious next thought — *"the four absence ledgers can compute themselves now"* — is **wrong
and this stone must say so.** The ledgers are frozen ON PURPOSE: a ratchet freezes NAMES so it can
say *"this one is new, that one went stale."* A ledger that computes both sides always agrees with
itself and proves nothing. `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`,
`[[feedback_two_instruments_agreeing_is_not_corroboration]]`.

**Enumeration is what a frozen list is compared AGAINST, never what generates it.**

## The shape — it composes, it does not invent

`metadata-of` already returns a complete row per name (measured live):

```
{:name :kind :arity :purity :determinism :totality :expand-time :category
 :defined-in :layer :added :ret :doc}
```

And `(:wat::intrinsic::examples)` already returns `(Vector :- [:wat::intrinsic::Example])`, a
wat-side `defrecord` (`wat/doctest.wat:13`). **The four-site pattern is shipped once already** and
is the template:

```
1  the record        (:wat::core::defrecord :wat::intrinsic::Example [...])   wat/doctest.wat:13
2  the checker scheme  () -> (Vector :- [Example])                            src/check.rs:17089
3  the load order      the record must load before the verb                   src/load/stdlib.rs:295
4  the verb            #[wat_intrinsic(":wat::intrinsic::examples")]           src/intrinsic/reflect.rs:67
```

### The Row's fields — chosen, with the exclusions reasoned

```
INCLUDE (census-relevant, all scalar/enum)
  name          :wat::core::keyword
  kind          :wat::runtime::Kind
  arity         :wat::core::i64            (-1 for Variadic — metadata-of's existing convention)
  purity determinism totality expand-time category    the five closed-domain axes
  syntax        :wat::core::String         "" when none  ⬅ NOT in metadata-of's map today; this
                                           week proved it load-bearing (the special-form tables)
  ret-type      :wat::core::String
  alias-of      (Option :- [String])       ⬅ the alias-vs-RESTRICTION fork needs this
  has-handler   :wat::core::bool

EXCLUDE, and why
  doc / prose / ret(description)   552 rows x ~3KB of prose = megabytes in one value.
                                   `metadata-of` serves these per-name and already does.
  source                           the Rust handler's full text. Same reason, worse.
  examples                         `(:wat::intrinsic::examples)` already returns them.
  args / yields / see / deprecated  structured sub-lists; `extract-arg-types` serves args
                                   per-name. Out of v1 scope, named not deferred-in-prose.
```

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **enumerate the registry** | YES | YES | YES | YES |

- **Obvious? YES** — a sibling of `examples`, same namespace, same zero-arg shape, returning the
  rows instead of one projection of them.
- **Simple? YES**, and only because it was measured: the per-row shape already exists in
  `metadata-of`, the Vector-of-records shape already exists in `examples`, and the four-site
  wiring is a template that shipped once. No new mechanism, no new design.
- **Honest? YES** — and it closes a real hole in the RULING's item 7 rather than papering it.
- **Good UX? YES** — every future census in this campaign becomes a query against the authority
  instead of a regex against source.

## Scope

**In:** the `Row` record · the `(:wat::intrinsic::rows)` verb · its checker scheme · its load
position · a `.wat` probe that runs the three censuses in the acceptance table.

**Out, affirmatively:**
- **Deriving any of the four absence ledgers.** See the ⛔ above; this would destroy them.
- **`args`/`yields`/`see`/`deprecated` projections.** v1 is scalars and enums.
- **Any census of Rust source or of the `.wat` corpus.** Out of reach by construction, stated
  above so no later reader mistakes this stone's scope.
- **Adding `:syntax` to `metadata-of`'s per-name map.** The Row carries it; whether the per-name
  map should too is a separate question with its own consumers and its own tests.
