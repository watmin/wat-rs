# Generative testing in wat — `wat/gen.wat` (`:wat::gen::`)

> Status 2026-08-25. **PROMOTED TO THE STDLIB** as `wat/gen.wat`, namespace `:wat::gen::`, on the
> `wat/grep.wat` precedent — a move of proven code, with the numbers that earned it:
>
> | evidence | number |
> |---|---|
> | laws, all mutation-proven, driven through the library's own driver | **18 over 319 points** |
> | live rete defects found by its first consumer | **3** |
> | scale, measured | linear to **500k points at ~23us/point** |
> | cost relative to the oracle it drives | **~300x cheaper** — never the bottleneck |
>
> The `gen-` name prefix dissolved into the namespace on promotion, as `:user::wat-grep` became
> `:wat::grep::`: it is `(:wat::gen::ints 0 3)`, not `gen-ints`. This doc is the design record —
> what is built, what is deliberately absent, and **what wat does not need that Clojure does**.

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

### FOUND — two live rete defects, 2026-08-25

The fuzzer's first widened run found **22 divergences of 504 shapes**, every one at the
newly-added accumulate shape, decomposing into exactly two families. Both reproduce minimally,
both are SILENT, and both are preserved in
`tests/rete/probe_arc278_accumulate_divergences.{rs,wat}`.

| family | shape | native | oracle |
|---|---|---|---|
| **A** | LEADING accumulate | **depth+1** (rows == rounds: 2→2, 3→3) | 1 |
| **B** | fact cond + accumulate + a SECOND `where` | **0** | 1 |

Family A is the same class as the leading `:not`/`:exists` defect fixed on 2026-08-24
(`71d0e700e`) — **and that fix did not reach accumulate.** Family B is independent of depth: it
reproduces at depth 0, so it is not a fixpoint issue. `qB1` and `qB2` differ by exactly one
trailing, trivially-true `where`.

**Why the existing corpus could not see either:** the accumulate axes (`accum`, `min-finding`)
compare DERIVED FACTS, and `production_delta` dedups those by value — a rule deriving one distinct
fact reads identically whether its token passed once or four times. That masking is the reason
this fuzzer compares beta rows, and it is now the reason it found something.

The gate is a RATCHET pinned at 22, not a zero: asserting zero would redden the floor and block
unrelated work, and deleting the accumulate shape to keep a gate green is the trade this codebase
refuses. Movement in either direction is a red test demanding an explanation.

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

### Open — ONE ordered list

> This list had forked into two disagreeing orderings within a day of being written (an Open list
> and a second, differently-ordered list under *Maturity*), with a skipped number and an item
> present in only one of them. Same failure as a forked breadcrumb, at work-list scale. **There is
> one list. It is this one.**

1. **Widen the rete target's grammar** — accumulators, stratified negation, intra-condition
   `:or`/`:not`, multi-rule interaction. Two payoffs at once: it hardens rete, and it is what
   makes the unused combinators *needed* (choosing between rule shapes is `gen-one-of`; picking a
   class name is `gen-elements`; assembling a rule from generated parts is the lifts). Today's
   space is uniform i64 coordinates, which is precisely why only `gen-coords` is pulled.
2. **A second target** — the promotion criterion. One consumer has tested one path, not
   genericity.
3. **Sampling driver** — the CEILING. See the design below; it is the only item here that
   enlarges what the library can ever reach.
4. **Shrinking** — coordinate descent. Unnecessary while spaces enumerate (reading failures in
   coordinate order hands you the minimal case for free), essential the moment (3) lands.
5. **Bounded collections** — `gen-vector g n`.

## The path to a sampling driver

**Sampling here is not randomness. It is a different ORDER over the same total enumeration.**
That reframing is the whole design, and everything good follows from it:

- **No seeds.** A case stays a coordinate, so a finding survives generator changes (§5).
- **Resumable and monotone.** Run `0..K` today and `K..2K` tomorrow: coverage only grows and
  nothing is re-tested. A seeded PRNG gives neither.
- **One new proof obligation, already familiar.** The traversal must be a BIJECTION on `0..card`
  — the same property as L4, provable by the same instrument. If it is not, sampling silently
  revisits points and misses others while reporting a clean count.

Two candidate orders, both deterministic:

**(a) Stride.** `i_k = (k * s) mod card`, with `gcd(s, card) = 1` so the walk is a full cycle.
Cheap; needs `gcd` and a stride choice. Spread is even but structured — a stride can align with a
dimension's base and starve it.

**(b) Digit-reversal (van der Corput / Halton, in mixed radix)** — the better fit, because the
machinery already exists. Take `k`'s digits in bases `(b0..bn-1)`, reverse the digit sequence,
re-read against the reversed bases. The product is unchanged and digit ranges match position-wise,
so it is a bijection — **checked in wat, against the real library**, by
`wat-scripts/fuzz/sampling-order-probe.wat`: over `[3 3 3 3 4]`, `card 324`, `distinct-images 324`.
(It was first "checked" in a throwaway Python reimplementation of mixed-radix, which verified a
MODEL of the design rather than `gen-digit`/`gen-shift` themselves — had those carried a bug, the
Python would still have gone green. The probe recomputes it through the library's own verbs and
agrees exactly.)

**And the spread claim, measured rather than hand-waved** — same probe, same library, over the
rete fuzzer's own bases `[3 3 3 3 4]`; dimensions covered by the first K of 324:

| K | order | dim0 (3) | dim1 (3) | dim2 (3) | dim3 (3) | dim4 (4) |
|---|---|---|---|---|---|---|
| 16 | sequential | 3/3 | 3/3 | 2/3 | 1/3 | **1/4** |
| 16 | reversed | 1/3 | 2/3 | 3/3 | **3/3** | **4/4** |
| 64 | sequential | 3/3 | 3/3 | 3/3 | 3/3 | **1/4** |
| 64 | reversed | 3/3 | 3/3 | 3/3 | 3/3 | **4/4** |

So the honest statement is NOT "well-spread everywhere at any K" — the two orders are mirror
images. Sequential covers the fastest-varying dimensions and **starves the slowest**; reversed
covers the slowest immediately and under-covers the fastest at very small K, then evens out.

That asymmetry is decisive here rather than academic: in the rete space `dim4` is **chain depth**
— the round-count dial, the single highest-yield dimension, the one that exposed the leading-filter
class. Sequential sampling still has not varied it after 64 samples (20% of the space). Reversed
hits all four depths within 16.

**The API consequence worth deciding early:** (b) needs the BASES at sample time, not an opaque
`Gen`. So the sampler is built over `gen-coords`' bases — `gen-sample-order bases` — rather than
over a `Gen`. A `Gen` produced by `fmap`/`such-that`/`lift` has lost its radix shape.
`gen-such-that` in particular *renumbers* the space, so a sampler must be composed BEFORE it, or
carry the surviving-index vector.

**Steps.** 1. `gen-reverse-order bases k -> i`, with a bijection law over a small space, mutation-
proven. 2. `gen-check-sampled g order n prop` — apply `prop` to the first `n` of the traversal.
3. Report `sampled=n of card` so the denominator is never mistaken for exhaustive. 4. Only then
widen a space past enumeration, so the first real use has the instrument under it.

**Honest limit:** with a superlinear oracle, `K` is bounded by TIME, not by design. Sampling
enlarges the reachable space; it does not make the oracle affordable.

## Maturity, measured (2026-08-25)

Not mature, and the instrument that says so is a call-site census — the same one that caught the
library having no tests at all:

| verb | real consumers |
|---|---|
| `gen-coords`, `gen-check`, `gen-such-that` | 1 each |
| the other **seven** | **0** |

Fifteen laws over 316 points, every one mutation-proven, is evidence the library is
**self-consistent**. It is not evidence that it is **useful**, and the distinction is the whole
finding. Combinators were added because the QuickCheck tradition has them, then proven against
laws written by the same hand that added them — a closed loop with no consumer pulling. `gen-lift3`
shipped with zero laws AND zero consumers and was only caught by counting.

**What is needed is not features. It is consumers.** The ordered list lives in *Open* above and
is not repeated here — repeating it is how this doc forked in the first place.

`gen-such-that` also has an untested cost: it materializes one `i64` per surviving index and
walks the whole source space at construction. Fine at 828; unmeasured beyond.

### Composition — the half the isolated laws did not cover

L1–L11 prove each verb ALONE, at tiny cardinality, and every one of them at **i64**. A combinator
library's value is in composition, and none of that was tested — which is a different claim from
"no consumer pulls it", and the two were wrongly collapsed. L12–L15 are that missing half, all
mutation-proven:

| law | composition | result |
|---|---|---|
| L12 | `lift2` over `elements` of **String** — the library off i64 at all | `Mix{n:0 s:"a"}`, card 6 |
| L13 | `one-of` over a **filtered** generator | card 7 = 5+2, dispatch correct |
| L14 | `fmap` **after** `such-that` — order of composition | mapped from the SURVIVOR, not the pre-filter index |
| L15 | `one-of` with an **empty** branch | card-0 branch skipped, not swallowing indices |

### `gen-check` REFUSES an empty generator

The composition probe turned up the real hazard: `gen-such-that` with a predicate nothing
satisfies yields `card 0`, `gen-check` then applies the property to **nothing**, and the caller
reads `0 violations` — indistinguishable from passing. The rete gate defends itself with a
`cases > 0` check, but that is a convention every future consumer must remember, and a convention
every caller must remember is the rot this codebase keeps pulling out.

`gen-check` now raises on an empty space, so the trap is closed once in the library rather than
re-defended at each call site. An empty branch inside `gen-one-of` remains legitimate (L15) — it
is *enumerating* nothing that is a caller bug, not *containing* nothing.

## Promotion — done 2026-08-25, and what actually earned it

The bar was never "a second consumer". That reading came from misapplying `CONVENTIONS.md`'s
*"a primitive earns its slot when a concrete caller demands it — not speculatively"*, which
guards against speculative LANGUAGE primitives; testing infrastructure has no prior test
demanding it, by construction.

What `wat/grep.wat` actually records as its own criterion is **a MOVE of proven code, with
shipped numbers**. Those are in the banner above.

**The namespace was also a live defect, not merely a promotion chore.** In scripts the library
defined `:user::Gen`, `:user::ints` … — squatting in its own consumer's namespace, where any
program wanting a record named `Gen` would collide. Scripts cannot define under `:wat::` (the
reserved-prefix gate admits only baked sources), so **promotion was the only available fix**.

Loads after `wat/seq.wat` (uses `into`/`filter`/`foldl`/`mapv`) and needs nothing further — no
holon, no rete, no comms. `:wat::deporder::verify-stdlib` enforces the position.

## Feature completeness — no, and the gap is `bind`

Asked whether the tooling is feature complete, the honest answer is no. The quality gaps are
easier to see than the functional one, and I listed those first while skipping this.

**`bind` — dependent generation — is the real absence.** `test.check`'s `gen/bind` builds a
generator whose SHAPE depends on a previously generated value: *generate n, then generate a
vector of length n*. Nothing here can express that. The rete target fakes it today with fixed
dimension slots and a "none" option per slot, which is why its `filt` dimension is an if-ladder
over 8 fixed shapes rather than "generate a rule, then generate its conditions".

It IS expressible in the finite model: `card(bind ga f) = sum over a of card(f(a))`, with `at`
dispatching like `one-of` over a computed rather than a literal branch list. The cost is real —
constructing `f(a)` for every `a` in the source — but it is bounded and known.

**Bounded collections** (`gen-vector g n`) follow from it: fixed length is expressible today by
repeated `lift`, variable length needs `bind`.

**`frequency` / weighted choice is deliberately absent, and stays absent.** In `test.check` it
biases a random draw. Here every point is visited exactly once, so a weight has no meaning at
all — and to bias a SAMPLE you enlarge a dimension's base, which the coordinate model already
expresses. Another place the finite design needs less.

## The failure surface is a matchable VALUE (2026-08-25)

`check` returns `CheckOutcome`:

```
:Checked [points violations]   ·   :EmptySpace
```

An earlier version RAISED on an empty generator — the same defect the no-hidden-failures LAW
forbids, written hours after reading it. But a nicer raise was never the fix. The hazard is that
`violations = 0` reads as success whether the property held at ten thousand points or was never
applied at all, and a guard that raises still hands back a bare count everywhere else.

`Checked` carries BOTH numbers, so a caller cannot extract a violation count without the point
count arriving in the same match arm. **The wrong reading has no form.** `EmptySpace` is then the
honest name for card 0, and whether that is an error is the CALLER's ruling — the law suite counts
it as a violation, the fuzzer prints `EMPTY-SPACE: this run tested NOTHING`.

Six `Option/expect` sites remain, all internal index invariants rather than caller-facing paths.

### Still open, and honestly

- **Genericity** remains untested by a second consumer. Promotion does not change that; it just
  stops the namespace squatting while the second consumer is found.
- **`gen-vector`** (bounded collections) is the one combinator the tradition has that this does
  not. Not built, deliberately — nothing has asked for it, and building it speculatively is the
  closed loop this library already fell into once.
