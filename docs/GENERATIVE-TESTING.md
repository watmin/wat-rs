# Generative testing in wat — `wat-scripts/lib/gen.wat`

> Status 2026-08-25. The library exists, proves its own laws, and has ONE consumer
> (`wat-scripts/fuzz/rete-differential.wat`). It is **not mature**. This doc is the tracking
> record: what is built, what is missing, and — the part worth reading — **what wat does not need
> that Clojure does**, with the reason for each.

## The core idea

A generator is an **indexed set**, not a seeded random source:

```
Gen<T> = defstruct { card : i64,  at : i64 -> T }
```

Because `at` is a total function of an index, three things that are separate machinery in the
QuickCheck / `clojure.test.check` lineage collapse into one operation:

| operation | here |
|---|---|
| enumerate | iterate `0..card` — exhaustive whenever the space fits |
| sample | pick any `i < card` — uniform, reproducible by construction |
| shrink | walk a coordinate's digits down — index arithmetic |

And a failing case gets a **permanent name**. A `test.check` seed is meaningless the moment the
generator changes; a coordinate like `[3 1 0 2]` still dials in the same case a year later.

The cost, stated plainly: **every dimension must be bounded.** You cannot generate an unbounded
structure. For differential testing against a superlinear oracle that is a feature — it is what
keeps the reference affordable.

## What wat does NOT need that Clojure does

This is the judgement the design rests on. Each row is a thing `clojure.spec` / `test.check` must
carry that wat can skip, and the reason is wat's type system and reflection.

### 1. No `clojure.spec` layer at all — **the types ARE the spec**

Clojure needs a separate shape-description language because it is dynamically typed, and spec's
chronic failure mode is the spec drifting away from the code it describes. wat's `defrecord`
already declares field types, and they are readable at runtime. Verified 2026-08-25:

```
(:wat::runtime::field-names-of :user::Point)  ->  [:x :y :label]
(:wat::runtime::field-types-of :user::Point)  ->  [wat.type/i64 wat.type/i64 wat.type/String]
```

So a generator can be **derived from the type itself**. There is no second artifact, so there is
nothing to keep in sync. This is the single biggest reduction against the Clojure design, and it
is why `gen-of-type` (below) is the highest-value item on the list rather than a nicety.

### 2. No retry-based `such-that` — filtering is EXACT

`test.check`'s `such-that` filters an opaque random source by retry-and-discard. It can fail
outright ("couldn't satisfy predicate after 100 tries") and it silently biases the distribution.

A finite indexed generator has no such problem: enumerate `0..card` once, keep the indices whose
values pass, and the survivors ARE the new generator, with an exact new cardinality. No retries,
no failure mode, no bias. **Strictly better, and only possible because the space is finite.**

### 3. No per-generator shrink trees — shrinking is generator-INDEPENDENT

In Clojure every generator must carry its own shrinker, because a generated value is opaque and
only its producer knows how to make it smaller.

Here the structure lives in the **index**, not the value. Shrinking is coordinate descent on the
digits of `i`, so **one implementation shrinks every generator** built from `gen-coords`. Nothing
per-generator to write, and nothing to get wrong per-generator.

### 4. Heterogeneous `tuple` is sugar, not a primitive

`gen/tuple` is a necessity in Clojure. Here `gen-coords` + `gen-fmap` already expresses any
product: decode the coordinate, construct whatever shape you want. A typed `tuple` combinator is
convenience we may add; nothing is blocked without it.

### 5. No seeds anywhere

See above — a coordinate is the case name, and it is stable across generator changes in a way a
seed can never be.

### 6. No `card` overflow guard — the substrate already refuses

This was written down as the **top** item on the work list, on the reasoning that a wrapped `card`
is the worst kind of defect: a silent under-count that every law still passes, since each
individual coordinate decodes correctly. A checked multiply was then written — and **deleted
unreached**, because the multiply inside it raised first.

`wat`'s `i64::*` is CHECKED. Verified 2026-08-25 with bases `[4000000000 4000000000]`:

```
#wat.runtime/IntegerOverflow {:message "i64 overflow: 4000000000 :wat::core::i64::* 4000000000
                                        does not fit in 64 bits" :op ":wat::core::i64::*" ...}
```

So the hazard is real but not ours to handle, and for a reason with nothing to do with
generators: Clojure would promote to BigInt (changing the type under you) and C-family arithmetic
would wrap in silence. **wat is the only one of the three where this cannot become a quiet
under-count.** One fewer thing to build, found by trying to build it.

## Built

| verb | what |
|---|---|
| `Gen :- [T]` | `defstruct` — carries a function, so not a `defrecord` (arc 293.W containment) |
| `gen-ints lo hi` | `card = hi-lo`, `at i = lo+i` |
| `gen-fmap f g` | reshape the yield, preserve cardinality |
| `gen-coords bases` | mixed-radix coordinates — the workhorse |
| `gen-check g prop` | the driver: enumerate, apply, tally |
| `gen-digit` / `gen-shift` | index arithmetic |
| `gen-elements vs` | pick from a value vector |
| `gen-such-that pred g` | exact filter — survivors ARE the new generator |
| `gen-one-of gs` | sum of cardinalities, range dispatch |

**Eight laws**, 272 points, proven by `wat-scripts/fuzz/gen-selftest.wat`, driven through `gen-check` itself, gated by
`tests/lint/gen_lib_laws.rs`. L4 (the bijection) is load-bearing: without it, enumeration can
visit tuples twice and miss others while reporting a clean case count.

## The work list

### Done 2026-08-25

- ~~`card` overflow guard~~ — **NOT NEEDED**; the substrate raises `IntegerOverflow`. See §6.
- **`gen-elements v`** — pick from a value vector. Law L6.
- **`gen-such-that pred g`** — the exact filter, no retries. Law L7; mutation-proven by inverting
  the predicate (5 violations, the whole filtered space).
- **`gen-one-of [g...]`** — `card` is the SUM, `at` dispatches by range, branches occupy
  contiguous index blocks so a coordinate still localizes a failure. Law L8.

### Open, ordered

1. **`gen-of-type T`** — derive a generator from a declared record type via `field-types-of`.
   This is the item that makes the library generic across the whole language rather than
   per-target, and it is the one wat can have and Clojure cannot.
2. **sampling driver** — for spaces past enumeration; still index-addressed, never seeded.
3. **shrinking** — coordinate descent. Unnecessary while spaces enumerate (reading failures in
   coordinate order already hands you the minimal case), essential once they do not.
4. **bounded collections** — `gen-vector g n`.
5. **widen the rete target's grammar** — accumulators, stratified negation, intra-condition
   `:or`/`:not`, multi-rule interaction. The current space covers depth x leading-filter x prefix
   x where-position x duplicates and nothing else.

## Promotion

`gen.wat` lives in `wat-scripts/lib/` and moves to the stdlib once a **second target** proves it
generic — the `wat-grep` precedent, which sat in scripts for months before earning promotion. One
consumer has not tested genericity; it has only tested one path.
