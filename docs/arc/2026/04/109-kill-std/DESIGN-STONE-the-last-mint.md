# DESIGN — the last mint, and then the wall

> *"annihilation is our greatest pleasure - we return with exactly one way to do this"*
> — the builder, 2026-08-23

`defservice` stopped minting the `launch` turbofish in `c6c614fe2`. The wall was then applied to
measure what remains, and the answer is **two classes, one of them tiny**.

## The census — imposed, in one pass

The parked minting wall was applied and flipped from REFUSE to LOG-AND-CONTINUE, so a single floor run
yields the whole population rather than waterfalling one site at a time:

```
CLASS 1 — a client-fn DECLARATION name          wat/service.wat:1996
  wat::cache::lru-svc/put<K,V>        4502
  wat::cache::lru-svc/get<K,V>        4482
  + wat-tests::{pcache,barebox,pair,box}-svc/{get,put}<…>

CLASS 2 — keyword-node fed an angle string      tests/resolve/ (two fixtures)
  :wat::core::Vector<…> · :wat::core::HashMap<…> · :T<…> · :Stream<…>
  :(wat::core::Vector<T>,wat::core::i64)                     ~8 total
```

Nothing else. `keyword/of` is retired, `launch` is fixed, `proto-tp` is gone.

## Class 1 is a DECLARATION, not a call head

The code says so itself, at the mint:

> *"the client fn's SIGNATURE names the surface's parametric messages, so the **DECLARATION** carries
> the service's own binders (`{b}/{op}{p}`), exactly as `/start`, `/stop`, `/grant` do."*

```clojure
method-name (:wat::core::keyword/from-string
              (:wat::core::string::interpolate "{b}/{op-str}{p}" :b fqdn-base :op-str op-str :p fqdn-tp))
```

`{p}` is `fqdn-tp` — the `<K,V>` suffix, built at `service.wat:303` from `fqdn-tp-syms`. This is
**position 1**, the declaration binder, which has worked since γ-i at the very start of the campaign.

★ **And the fix already has a worked precedent in the same file, shipped one stone ago.** `proto-tp`
was exactly this shape and was killed by emitting the bare name with `:- [syms]` as siblings.
`fqdn-tp` is its twin and dies the same way.

## Class 2 is testing a converter whose input is being retired

`tests/resolve/probe_arc251_keyword_to_type_form.wat` and `probe_arc251_type_namespace_fix.wat` feed
`keyword-node` angle-bearing strings on purpose, to exercise `keyword/to-type-form` — the converter
that turns `:Vector<i64>` into `(Vector :- [i64])`.

⚠ **`to-type-form` itself is NOT dead and must not be retired here.** `wat/service.wat:434` calls
`to-type-form-colon`, and its own comment records why:

> *"(`:S<K,V>`) or already a form (`(S :- [K V])`); `keyword/to-type-form-colon` takes a …"*

It is a transition shim accepting **either** spelling. Once nothing mints the angle form its angle half
becomes unreachable — but proving that needs the wall up and a green floor, so it belongs to the purge,
not here. What this stone owes is the two fixtures: with the wall up, `keyword-node` refuses their
input, so they become negative controls proving the refusal, or they move to the form spelling and keep
testing the half that survives.

## Then the wall

Both minting doors refuse an angle type-head, using the lexer's own predicate. **And the third door
goes up with them** — `symbol-node` (`src/edn_shim.rs`), found by an earlier rider: genuinely unwalled,
and harmless today only because the checker's surface arm keys on `Keyword` rather than `Symbol`. An
unwalled door is an unwalled door.

The acceptance criterion the builder set stands: *unless those callers fail on illegal syntax, we've
failed.* With classes 1 and 2 closed, nothing legitimate calls it — so the proof becomes a **negative
control**: a deliberate `keyword/from-string "my::Map<K,V>"` must be refused, while
`"wat::core::i64::<"` and `"foo/bar"` must still mint.

## The four questions

- **Obvious?** YES. After this, `<` opens a type head nowhere — not written, not minted, not rendered.
  One spelling, one operator, four positions.
- **Simple?** YES. Class 1 is `proto-tp`'s twin with a precedent one stone old; class 2 is two fixtures;
  the wall is a predicate already written and already measured.
- **Honest?** YES, and it is the last honesty gap in the campaign: the language has refused this
  spelling at the reader since `86e1b105a` while its own stdlib minted it ~9000 times per floor run.
- **Good UX?** YES. A macro author who concatenates a type name learns at expand time, at their own
  call site, with the form spelled out.

## Out of scope, affirmatively cut

- **The purge** of `split_type_params`, `canonical_callable_name`, `check.rs`'s explicit-suffix arm,
  and `to-type-form`'s angle half. All are dead once nothing mints — and the wall plus a green floor is
  what proves it, so the purge is the stone AFTER this one.
- **`keyword/from-string` → `(:wat::core::keyword "x")`.** Its own NOTE; decided with the
  verb-equals-type family.
