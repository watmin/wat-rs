# NOTE (arc 109) — the list rule, and how a parametric reaches an EDN literal

**Filed 2026-08-22. A POINTER carrying MEASUREMENTS, not a decision.** Surfaced while settling what
`'(1 2 3)` means in the `:-` destination syntax. Everything below was measured this session against
`7415f5709`; nothing here is reasoned from the record.

## The builder's rule, stated

```clojure
'(1 2 3)   ≡  (wat.type/List :- [wat.type/i64] 1 2 3)      the CONSTRUCTOR
              (wat.type/List :- [wat.type/i64])            its TYPE
```

The constructor is the type reference plus values — the seam's existing rule for `Vector`, and
`List` joins it. **`'(1 "a")` is not a legal data literal**: a `List` is homogeneous by construction,
so a mixed literal is a `Tuple` (or a `WatAST`). `WatAST` is the bottom rung — everything in wat is
one — and `Value` is the same for values.

## ⛔ MEASURED — `'(1 2 3)` has TWO facets and the checker knows ONE

```
'(1 2 3)  into a  :wat::WatAST  param   →  EXIT 0
'(1 2 3)  into a  List<i64>     param   →  EXIT 3
          ":user::takes: parameter #1 expects :wat::core::List<wat::core::i64>; got :wat::WatAST"
```

Negative control: asking for `List<String>` gives the same `got :wat::WatAST`, so the type genuinely
IS `WatAST` — not an unresolved anything.

The lattice this needs already exists and works. `src/check.rs:15502`, verbatim:

> *"Bottom flows in the ACTUAL position (`Never <: every type`); top flows in the EXPECTED position
> (`every type <: Value`)."*

Measured: `42` into a `Value` param → EXIT 0. So the machinery is there; **what is absent is the rule
that a quoted list literal also inhabits `List<T>`**, with `T` from its elements. One facet taught,
the other not — the same shape as every guard this arc has widened, one layer up.

⚠ I first read this as *"the apostrophe is already taken by quote, so `'(1 2 3)` cannot mean the List
constructor."* **Wrong.** Nothing has to give up the apostrophe; the caller's expected type
discriminates, exactly as the builder said. Recorded because the wrong reading is the tempting one.

## The EDN literal — what the reader is TODAY

```rust
OwnedValue::Tagged(tag, Box<OwnedValue>)     // ONE tag, ONE payload
```

All **81** tags obey it: `#wat.core.Option/Some [v]`, `#ns/Type {:field-0 v0 …}`,
`#wat.core/Tuple[a b]`. **No tag carries a param-spec.** Element types are implicit in the values.

So `#wat.type/Tuple :- [i64 String] [1 "a"]` — a tag consuming THREE forms — is not a spelling
change; it is a reader change against a two-field variant every producer and golden assumes.

**The house precedent for a structured payload is a map with NAMED keys:**

```
#wat.core/Span {:file "src/baz.wat" :line 7 :col 4 :end #wat.core.Option/None []}
```

The builder's candidate keeps `Tagged(tag, one)` and drops the keywords, since we own the reader:

```
#wat.type/Tuple   {[wat.type/i64 wat.type/String] [1 "a"]}
#wat.type/List    {[wat.type/i64] (1 2)}
#wat.type/Vector  {[wat.type/f64] [4.2]}
#wat.type/HashMap {[wat.type/Keyword wat.type/String] {:kw "str"}}
#wat.type/WatAST  (1 "a")            ;; heterogeneous — no param-spec, it is the bottom rung
```

with the rule: **every strongly-tagged collection MUST carry a param-spec**, `WatAST` never does.

## ★ THE FINDING THAT REFRAMES IT — the non-pathological case is UNREACHABLE today

The builder observed that most of those specs are redundant (`4.2` IS an `f64`; `{:kw "str"}` says
itself) and become load-bearing only for narrow numerics: `#wat.type/Vector {[wat.type/u8] [0 1 2]}`.

**`u8` is already in `BARE_PRIMITIVES`. It is not reachable.**

```
(:wat::core::Vector :wat::core::u8 1 2 4)
  → ":wat::core::vec: parameter #2 expects :wat::core::u8; got :wat::core::i64"
```

There is **no narrow-numeric literal syntax** (`u8` appears nowhere in `src/lexer.rs` and nowhere in
the `wat/` stdlib). An integer literal is an `i64` and will not narrow, so no `Vector<u8>` can be
constructed, so none can be serialized, so the case that would make the param-spec necessary cannot
occur.

**Therefore the dependency runs literal → container, not the other way.** How one writes a `u8`
decides whether the container tag must carry anything:

- If the literal carries its own type (`#wat.type/u8 1`, the builder's
  `#wat.type/WatAST (#wat.type/u8 1 "a")`), then `[0 1 2]` is self-describing and the container spec
  is **redundancy chosen for uniformity** — a legitimate choice, made knowingly.
- If narrow numerics are reachable only by coercion in a typed position, the container is the only
  place the type can live and the spec is **required**.

**Settle the literal first.** Deciding the container shape now means deciding it without knowing
whether it has to carry anything.

## The cost of "MUST carry a param-spec", measured

```
352  .edn goldens in tests/
341  contain a bare collection payload
915  bare `[…]` literals inside them (vectors only; sets and maps on top)
```

If the rule is universal on the WRITE side, every one of those becomes `#wat.type/X {[T] […]}` — a
corpus-wide format change landing on the goldens that pin every diagnostic in the tree.

⚠ **Read and write may not want the same answer.** On READ the rule buys totality — the reader never
infers, an under-specified literal is a rejection rather than a silent default, and it is cheap
because it is paid at authoring time. On WRITE it buys round-trip fidelity and costs 915+ sites of
noise on data that already describes itself. Requiring it on read while keeping write minimal costs
`read(write(x)) ≠ x` for empty collections — **which is already true today**: a `Vector<i64>` with no
elements prints as `[]`, and nothing has needed the type back.

## What is NOT settled, and must not be guessed

1. **`WatAST`'s second facet** — where does `'(1 2 3) : List<T>` belong? Beside the `Never`/`Value`
   arms at `check.rs:15502`, or in `assignable`'s Path arm? And what is `T` for `'()`?
2. **Narrow-numeric literal syntax** — the upstream decision this all hangs on.
3. **The tag shape** — single-entry map with a vector key (no keywords), vs. `Tagged` growing a
   param-spec field, vs. tagging only the lossy cases.
4. **Read-side vs write-side** — one rule or two.

Each wants the four questions on every option, in a DESIGN, against these numbers.

## Kin

- `294/SEAM.md` — the constructor-is-the-reference-plus-values rule for `Vector`
- `src/check.rs:15502` — the `Never`/`Value` lattice arms this would extend
- `src/edn_shim.rs` — the `Tagged` contract and the 81-tag vocabulary
- `109/NOTE-the-three-surviving-primes-want-a-sigil.md` — the other naming question filed today
