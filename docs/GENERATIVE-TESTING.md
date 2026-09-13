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

### 1. No `clojure.spec` layer at all — but NOT for the reason first assumed

Clojure needs a separate shape-description language because it is dynamically typed, and spec's
chronic failure mode is the spec drifting away from the code it describes. wat's `defrecord`
already declares field types, and they are readable at runtime. Verified 2026-08-25:

```
(:wat::runtime::field-names-of :user::Point)  ->  [:x :y :label]
(:wat::runtime::field-types-of :user::Point)  ->  [wat.type/i64 wat.type/i64 wat.type/String]
```

The obvious conclusion was: derive the generator from the type, `s/gen`-style, and call it
`gen-of-type`. It was written down as the **highest-value open item**. Building it produced two
findings, and the second is the one that matters.

**(a) It is not expressible — but the reflection subsystem is neither lacking nor defective, and
my first account of this was wrong.** The correction is worth more than the original claim.

I reported "reflection reads shapes but cannot build them" as a substrate gap. The builder's
reply was one line — *a type value IS its ctor* — and it is right. **A constructor is a
first-class function value in wat:**

```
(:user::apply2 :user::Point' 3 4)   ->   #user/Point {:x 3 :y 4}
```

So construction is available, idiomatic, and fully typed; `gen-lift2` / `gen-lift3` are built
directly on it. What is genuinely unavailable is construction from a type **keyword**, and that
is *not a gap to fill*: the result type could not be known statically, so it would be a hole in
the checker rather than a missing intrinsic. Static typing is the reason, and it is the same
reason Rust cannot build a struct from a runtime `TypeId`.

**And a second thing I reported as a finding was not one.** I noted that `field-names-of` /
`field-types-of` fail at macro-expansion time even for stdlib types, called the allowlist entry
at `src/macros/eval.rs:717` "unreachable in practice", and framed it as a latent inconsistency
that would cost the next person an hour.

It is documented behaviour, and the codebase already routes around it. `wat/telemetry.wat:286`,
in its own words: *"compile-time/macro-expand reflection of a baked record is **DEAD, proven**;
runtime resolves for both stdlib and user records."* A prior self proved exactly this and wrote
down the resolution — reflect at RUNTIME — which is what `framing-floor-of` does directly below
that comment.

It cost an hour because I theorised before grepping for prior art, not because the substrate is
inconsistent. **There is no work item here.** Macro expansion runs before user types exist;
making types visible at macro time would be a phase inversion, not a fix. And this library needs
none of it: `gen-lift2`/`gen-lift3` take a constructor value (arity carried, checker-verified),
and `gen-record` takes its arity from the caller's argument count. Neither ever called a
reflection verb.

**(b) It would not be worth much anyway.** `field-types-of` yields `wat.type/i64` — a type,
carrying **no bounds**. A finite generator is nothing *but* bounds. Deriving one from `i64` would
have to invent a range, which is precisely the decision the author must make. `spec`'s auto-gen
earns its keep in a language with no types to lean on; here the types are already known, and what
is missing is the interesting SUBSET — which no amount of reflection knows.

So the answer is `gen-record` (below): field generators given explicitly, and the emitted
constructor call is **ordinary checked wat**. The checker performs exactly the verification
reflection would have, at compile time. Proven: three generators for a two-field record is an
`ArityMismatch`; a `String` generator for an `i64` field is `expects :wat::core::i64; got
:wat::core::String`.

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

### 4. Heterogeneous `tuple` is not needed — `gen-lift2`/`gen-lift3` subsume it

`gen/tuple` is a necessity in Clojure. Here the applicative lift over a constructor value already
produces heterogeneous products, fully typed:
`gen-lift3 :user::Tri' (ints 0 2) (ints 5 7) (elements ["p" "q"])` yields
`#user/Tri {:a 1 :b 6 :c "q"}` with card 8. There is no anonymous-tuple shape to add, because the
constructor names the result type.

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
| `gen-lift2 f ga gb` | applicative lift over a CONSTRUCTOR VALUE — the idiomatic builder |
| `gen-lift3 f ga gb gc` | ditto, ternary; gives heterogeneous products for free |
| `gen-record T g...` | **macro** — N-ary sugar for 4+ fields, emitting a checked prime constructor |
| `gen-nth c i` | read one digit of a coordinate |

**Ten laws**, 284 points, proven by `wat-scripts/fuzz/gen-selftest.wat`, driven through `gen-check` itself, gated by
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

- **`gen-record T g...`** — the macro above. Law L9, mutation-proven by reversing the field
  wiring (4 violations). Two lessons the substrate taught while building it: a literal
  binder in a macro template is REFUSED (hygiene bound gate E, arc 249) and must come from
  `fresh-symbol`; and bare-positional construction is retired, so the emitted constructor is the
  PRIME name (`:user::Point'`), built the same way `kwargs-lower` builds it.
- ~~`gen-of-type T`~~ — **NOT BUILT, deliberately.** See §1: not expressible (reflection is
  read-only; macros run before user types register), and not valuable (types carry no bounds).

### Open, ordered

1. **sampling driver** — for spaces past enumeration; still index-addressed, never seeded.
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
