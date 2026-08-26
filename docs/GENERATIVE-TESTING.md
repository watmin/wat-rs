# Generative testing in wat — `:wat::gen::`

> The design record for `wat/gen.wat`: what a generator IS here, how to build one, what is
> deliberately absent, and **what wat does not need that Clojure does**.
>
> Numbers verified 2026-08-26 against the tree. The library is `wat/gen.wat` (stdlib, loads after
> `wat/seq.wat`); its laws are `wat-tests/gen.wat` (**27 deftests**, discovered by `wat::test! {}`
> so there is no total to hand-maintain); its first consumer is
> `wat-tests/rete/differential-fuzz.wat`.

## The core idea

A generator is an **indexed set**, not a seeded random source:

```
Gen<T> = defstruct { card : i64,  at : i64 -> T }
```

Because `at` is a total function of an index, three things that are separate machinery in the
QuickCheck / `test.check` lineage collapse into one:

| operation | here |
|---|---|
| enumerate | iterate `0..card` — exhaustive whenever the space fits |
| sample | pick any `i < card` — reproducible by construction |
| shrink | walk down toward 0 — index arithmetic |

And a failing case gets a **permanent name**. A `test.check` seed is meaningless the moment the
generator changes; a coordinate like `[3 1 0 2]` dials in the same case a year later.

The cost, stated plainly: **every dimension must be bounded.** You cannot generate an unbounded
structure. For differential testing against a superlinear reference that is a feature — it is what
keeps the reference affordable.

## Using it

You declare **one space per argument**, and the library forms the **product** and walks all of it.

```wat
(:wat::gen::record :user::Args
  (:wat::gen::ints 0 10)                                            ;; 10 points
  (:wat::gen::elements (:wat::core::PersistentVector "a" "b" "c"))  ;;  3 points
  (:wat::gen::bools))                                               ;;  2 points
;; => card 60
```

`check` over that with the property `n < 7` reports **points 60, violations 18** — and 18 is
exactly 3 failing `n` values × 3 strings × 2 bools. Every combination, visited once.

**Your job is to declare each axis's bounds, not to pick values.** That is the whole difference
from a drawing generator: `card` tells you the size before you run, and the outcome tells you the
denominator afterward, so "0 violations" can never be reported without what it was 0 out of.

### The surface

**Types.** `Gen :- [T]` · `Coord` and `Bases` (both `PV<i64>` — see *Aliases are transparent*
below) · `CheckOutcome`.

| role | verbs |
|---|---|
| **construct** | `gen card at` (the only constructor — floors `card` at 0) · `ints lo hi` · `elements vs` · `bools` |
| **the workhorse** | `coords bases` — mixed-radix coordinates; `card-of bases` |
| **reshape** | `fmap f g` · `such-that pred g` (exact filter) |
| **combine** | `one-of gs` (sum, range dispatch) · `bind ga f` (dependent) · `lift2`/`lift3` over a constructor value · `record T g…` (macro, N-ary) |
| **collections** | `vector-of g n` (fixed length) · `vector-upto g lo hi` (variable) |
| **drive** | `check g prop` → `CheckOutcome` |
| **sample** | `coords-scattered bases` (digit-reversed order) · `take n g` (prefix) |
| **shrink** | `shrink-index g k fails?` (any `Gen`) · `shrink c fails?` (coords-shaped, sharper) · `shrink-dim` · `descend` |
| **index arithmetic** | `digit` · `shift` · `nth` · `with` · `reverse-index` |

**`bools` is the one total generator, and the only one the library can derive with certainty.**
`ints` and `elements` make the caller choose a range or a pool, which is right for an unbounded
domain. `bool` has exactly two points: no range to invent, nothing the author knows that the
library does not. It is **exhaustive, not a sample** — `check` over it has seen both, always.

The same argument covers any all-unit enum (card = variant count) and `u8` (card 256). Neither is
built: no caller. A verb with no caller is a claim, not a capability.

## What wat does NOT need that Clojure does

This is the judgement the design rests on. Each item is machinery `clojure.spec` / `test.check`
must carry that wat can skip.

### 1. No `spec` layer, and no `Arbitrary`-style derivation

Clojure needs a separate shape-description language because it is dynamically typed, and spec's
chronic failure mode is the spec drifting from the code it describes. wat's `defrecord` declares
field types and they are readable at runtime:

```
(:wat::runtime::field-names-of :user::Point)  ->  [:x :y :label]
(:wat::runtime::field-types-of :user::Point)  ->  [wat.type/i64 wat.type/i64 wat.type/String]
```

The obvious conclusion — derive the generator from the type — was written down as the highest-value
open item. It is **not built, and the reason is not a missing intrinsic.**

**Construction from a type value is already available.** A constructor is a first-class function
value in wat: `(:user::apply2 :user::Point' 3 4)` → `#user/Point {:x 3 :y 4}`. `lift2`/`lift3` are
built directly on it. What is unavailable is construction from a type *keyword* — and that is not a
gap to fill, because the result type could not be known statically; it would be a hole in the
checker, for the same reason Rust cannot build a struct from a runtime `TypeId`.

**And derivation would not be worth much anyway.** `field-types-of` yields `wat.type/i64` — a type,
carrying **no bounds**. A finite generator is nothing *but* bounds, so deriving one from `i64` must
invent a range, which is exactly the decision the author must make. `spec`'s auto-gen earns its
keep in a language with no types to lean on; here the types are known and what is missing is the
interesting **subset**, which no reflection knows.

*(A Generic-style middle path is reachable — reflect a record's fields and assemble from a table of
per-type leaf generators. It still needs the author to supply a bound per leaf type, so the table
carries the same information `record` takes positionally. Worth building when a second consumer
wants it; not before.)*

**Macro-time reflection is not a gap either.** `wat/telemetry.wat:286`, in its own words:
*"compile-time/macro-expand reflection of a baked record is DEAD, proven; runtime resolves for both
stdlib and user records."* Macro expansion runs before user types exist, so making types visible
there would be a phase inversion, not a fix. This library needs none of it: the lifts take a
constructor value (arity carried, checker-verified) and `record` takes its arity from the caller's
argument count.

### 2. No retry-based `such-that` — filtering is EXACT

`test.check`'s `such-that` filters an opaque random source by retry-and-discard. It can fail
outright ("couldn't satisfy predicate after 100 tries") and it silently biases the distribution.

A finite indexed generator has no such problem: enumerate `0..card` once, keep the passing indices,
and the survivors ARE the new generator with an exact new cardinality. No retries, no failure mode,
no bias. **Strictly better, and only possible because the space is finite.**

### 3. No per-generator shrink trees — shrinking is generator-independent

In Clojure every generator carries its own shrinker, because a generated value is opaque and only
its producer knows how to make it smaller. Here the structure lives in the **index**, so one
implementation shrinks every generator. There are two:

- **`shrink-index g k fails?`** — general. Walks down for the smallest index that still fails,
  meaningful because enumeration order IS a simplicity order: `coords` yields all-zero first,
  `one-of`/`bind` place earlier branches first, `vector-upto` puts short vectors before long ones.
- **`shrink c fails?`** — coordinate descent on digits. Sharper (O(sum of bases) vs O(k)) but only
  for a `coords`-shaped space.

Both run **one** search, `descend`: walk `0..start`, keep the first candidate that still fails,
stop. They differ only in what a candidate index *means*, which is a parameter.

> ⚠ **This section's claim was false for as long as it existed.** "Shrinking is
> generator-independent" was written the day `shrink` was written — and `shrink` took a
> *coordinate*, so it composed with none of `bind`, `such-that`, `one-of` or `record`, the
> combinators that make the library worth having. The claim described the design's potential; the
> code delivered it for one shape. `shrink-index` is the general form, added when the seam was
> finally checked instead of assumed.

### 4. Heterogeneous `tuple` is not needed

`gen/tuple` is a necessity in Clojure. Here the applicative lift over a constructor value already
produces heterogeneous products, fully typed —
`lift3 :user::Tri' (ints 0 2) (ints 5 7) (elements ["p" "q"])` yields `#user/Tri {:a 1 :b 6 :c "q"}`
at card 8. There is no anonymous-tuple shape to add, because the constructor names the result type.

### 5. No seeds, and no `frequency`

A coordinate is the case name, stable across generator changes in a way a seed can never be.

`frequency` is **deliberately absent and stays absent.** In `test.check` it biases a random draw.
Here every point is visited exactly once, so a weight cannot change what enumeration sees — and
where it *would* matter, in a prefix of a sampled order, the bias is already expressible because
**cardinality IS the weight**: a branch with a 3-point space occupies three times the indices.
`one-of [a a b]` is a 2:1 mix. The combinator adds no expressive power, only a second way to say
the same thing.

### 6. No `card` overflow guard — the substrate already refuses

This was written down as the *top* work item, on the reasoning that a wrapped `card` is the worst
kind of defect: a silent under-count that every law still passes, since each coordinate decodes
correctly. A checked multiply was written and then **deleted unreached**, because the multiply
inside it raised first:

```
#wat.runtime/IntegerOverflow {:message "i64 overflow: 4000000000 :wat::core::i64::* 4000000000
                                        does not fit in 64 bits" ...}
```

Clojure would promote to BigInt (changing the type under you); C-family arithmetic would wrap in
silence. **wat is the only one of the three where this cannot become a quiet under-count.** One
fewer thing to build, found by trying to build it.

## Sampling is a different ORDER, not randomness

That reframing is the design, and everything good follows:

- **No seeds** — a case stays a coordinate.
- **Resumable and monotone** — run `0..K` today and `K..2K` tomorrow; coverage only grows and
  nothing is re-tested. A seeded PRNG gives neither.
- **One proof obligation** — the traversal must be a **bijection** on `0..card`, or sampling
  silently revisits points and misses others while reporting a clean count.

`coords-scattered` is digit-reversal (van der Corput / Halton, mixed radix): take `k`'s digits in
bases `(b0..bn-1)`, reverse the sequence, re-read against the reversed bases. The product is
unchanged and digit ranges match position-wise, so it is a bijection — **checked in wat against the
real library** by `wat-tests/gen.wat`'s L26, which runs on every floor.

**The spread, measured — and it is not "well-spread everywhere".** Over `[3 3 3 3 4]`, card 324,
dimensions covered by the first K:

| K | order | dim0 | dim1 | dim2 | dim3 | **dim4** |
|---|---|---|---|---|---|---|
| 16 | sequential | 3/3 | 3/3 | 2/3 | 1/3 | **1/4** |
| 16 | scattered | 1/3 | 2/3 | 3/3 | 3/3 | **4/4** |
| 64 | sequential | 3/3 | 3/3 | 3/3 | 3/3 | **1/4** |
| 64 | scattered | 3/3 | 3/3 | 3/3 | 3/3 | **4/4** |

The two orders are mirror images: sequential covers the fastest-varying dimensions and **starves
the slowest**; scattered covers the slowest immediately and under-covers the fastest at small K.

That asymmetry is decisive rather than academic. In the rete fuzzer's space `dim4` is **chain
depth** — the dial that exposed the leading-filter defect class. A sequential prefix has still not
varied it after 64 of 324 points. Scattered hits all four depths within 16. **"Sample the first K
sequentially" would have tested depth 0 and nothing else.**

**API consequence:** digit reversal needs the BASES, not an opaque `Gen`. A `Gen` from
`fmap`/`such-that`/`lift` has lost its radix shape, and `such-that` in particular *renumbers* the
space — so compose the scatter BEFORE it.

**Honest limit:** against a superlinear reference, K is bounded by TIME, not by design. Sampling
enlarges the reachable space; it does not make the reference affordable.

## The failure surface is a matchable VALUE

```
CheckOutcome  =  :Checked [points violations first-failure]   |   :EmptySpace
```

An early version **raised** on an empty generator — the same defect the no-hidden-failures law
forbids, written hours after reading it. A nicer raise was never the fix. The hazard is that
`violations = 0` reads as success whether the property held at ten thousand points or was never
applied at all.

`Checked` carries **both numbers**, so a caller cannot extract a violation count without the point
count arriving in the same match arm. `first-failure` carries the witness — without it a caller
learns "3 violations" and cannot reach one of them, and inside a `deftest`, which cannot print,
that is the difference between a tool that finds bugs and one that reports a number.

**And the two numbers are ordered by construction: `0 <= violations <= points`.** `prop` returns
`bool`, not `i64`. It was `[T :-> i64]` and the driver *summed* it, which permitted two states that
cannot be true of any real check — `violations > points` (a prop returning 5), and a witness beside
a zero count (negative returns cancelling). Every prop already returned only 0 or 1, so the width
bought nothing and cost exactly those readings. A weighted prop is now a type error:

```
:wat::gen::check: parameter #2 expects [i64 :-> bool]; got [i64 :-> i64]
```

Not caught — **unrepresentable.**

**`EmptySpace` is nullary, and that is a decision.** `check` cannot honestly know *why* a space is
empty: a `Gen` is `{card, at}` and carries no provenance, which is the thesis that lets every
combinator return one type. Threading a reason through all construction sites, to populate a field
whose only consumer discards it (for a law, an empty space is always a failure), is not worth it.
The arm says THAT the space is empty and never WHY.

## Two things that will bite you

**Aliases are transparent.** `Coord` and `Bases` are both `PV<i64>` and the checker canonicalizes
aliases away, so handing a coordinate where bases belong type-checks clean. The names buy
legibility, not distinctness. Making the swap unrepresentable needs a wrapper record whose unwrap
lands on the hot enumeration path — not taken, for a confusion with no recorded instance.

**A `Gen` must not enter a `defrecord`.** It carries a function, so it cannot survive an EDN
round-trip. The containment rule is supposed to refuse this and **currently does not** for the
parametric spelling: `(defrecord Wrap [g <- (Gen :- [i64])])` loads clean and crosses the wire with
`:at #wat.core/fn nil` — `card` honest, `at` dead, nothing in the value saying so. Diagnosed with a
proven patch in
`docs/arc/2026/06/293-struct-record-symmetry/NOTE-a-parametric-struct-passes-the-purity-gate.md`.
**Until that lands, nothing stops it.**

## Patterns — the corpus you should copy from

`wat-tests/gen-patterns.wat` is **documentation that runs**: six recognizable shapes of generative
test, each against real substrate, each with data that is not a toy (multi-token strings, enums
carrying payloads, generated command sequences). Find the shape that matches your problem, copy it,
swap the domain.

| | pattern | reach for it when | worked example |
|---|---|---|---|
| **P0** | **DIFFERENTIAL** | you have a second implementation | `wat-tests/rete/differential-fuzz.wat` |
| P1 | ROUND-TRIP | you have an inverse pair | `split ∘ join == id` |
| P2 | METAMORPHIC | you have **no oracle** | joined length == Σ parts + separators |
| P3 | MODEL-BASED | the thing is **stateful** | a command program vs a simpler model |
| P4 | ALGEBRAIC | the thing is an **operation** | `HashSet/conj` idempotent + commutative |
| P5 | DEPENDENT | valid inputs are a **relation** | an index that is in range *by construction* |

**P0 is first for a reason, and the ranking is empirical rather than aesthetic.** Differential is
the only one of the six that has actually found a defect here: **three live rete defects, all
silent, none reachable by the 57-query hand-written corpus.** The other five, run against the
substrate, are green — which is evidence the substrate is sound on those paths and *not* evidence
that the patterns are powerful. If you have a reference implementation, a slow-but-correct oracle,
or an old version, use it. The oracle you did not have to invent is the one that cannot be wrong
in the same way as the code.

**The reason differential pays is worth naming**: every other pattern requires you to *state* the
property, and a property you state is a property you already thought of. A differential test
asserts only *"these two agree"* — so it finds the disagreements you would never have predicted.
That is exactly how family C was found (`:not` over a derived class), which nobody would have
written a law for.

## ⛔ When generative testing is the WRONG tool

A generator here is **finite and total over a product**. That is its power and it is the whole
source of its limits. Reach for a plain `deftest` when any of these holds — and a generative test
written anyway is theatre, which is worse than no test because it looks like coverage.

**1 · You cannot bound the input.** `Gen<T> = {card, at}`. An arbitrary-length stream, a real file,
an unbounded recursion — none has a `card`. You can bound a *proxy*, but then the proxy is what you
tested, and the gap between it and the real input is exactly where the bug will live.

**2 · The interesting case is SPARSE, not SMALL.** Enumeration reaches everything in a space that
fits; it reaches almost nothing in a space that does not. A hash collision, one overflow boundary,
a specific timestamp — those are one point in 2⁶⁴, and neither enumeration nor a scattered prefix
will find them. **Generative testing finds bugs that are DENSE in a small space. Example tests find
bugs you can NAME.** If you can name it, name it.

**3 · The bug is a SCHEDULE, not a value.** Races, deadlocks, lock ordering, timeouts — none is a
point in a product space. P3 generates a *sequence* of operations, which is sequential interleaving;
it says nothing about concurrent ones. A generator cannot produce a thread schedule.

**4 · Deciding pass/fail needs state the value does not carry.** `prop` is `[T :-> bool]`: pure and
total. If the property needs a clock, a peer, a file, or the outcome of an earlier test, `check` is
the wrong harness — it is an enumeration, not a fixture runner.

**5 · The oracle costs more than the coverage is worth.** The rete fuzzer pays ~13.6 ms per case
for 1260 cases because its oracle is a second engine in-process. If yours is a network call, a
product space is the wrong shape and you want a handful of chosen cases.

**6 · One example states the contract better.** The one people get wrong. A generative test says
"this holds across the space"; an example says "THIS must be true". When the contract *is* the
specific thing — an exact error message, a documented special case, a boundary that is interesting
precisely because it is that value — the example is clearer and the generator buries it. **A
generative test that restates a single example is theatre.**

**7 · You would have to write the implementation twice.** If the only way to state the expected
answer is to recompute what the code computes, you have a tautology, not a test — break the code
and the "expected" value breaks identically. This is not hypothetical: four laws in this library's
own suite did exactly that and could not fail (GEN-VIGILIA L2). If round-trip, metamorphic and
model-based all fail to fit, that is the signal the generator is not the tool here.

> **The test to apply before writing one:** *what would this catch that a hand-written case would
> not?* If the answer is "a case I did not think of", proceed. If it is "the same case, with more
> ceremony", write the case.

## What "mature" would mean, and what is actually proven

Stated so it can be argued with rather than felt:

| claim | evidence | status |
|---|---|---|
| the machinery is self-consistent | 27 laws, every one mutation-proven | **proven** |
| the laws can fail for the reason they exist | each fix reverted, gate must go red | **proven** — and 4 laws once could not |
| it finds real defects | 3 live rete defects, all silent | **proven**, via P0 only |
| a real problem can be *expressed* in it | the pattern corpus: sequences, payload enums, multi-token strings | **proven** |
| it is useful to someone who did not write it | — | **NOT proven.** One consumer, one author. |

**The last row is the honest gap, and it is not closable from inside.** Nineteen laws written by
the hand that added the combinators is a closed loop, and this library has been in one before: a
verb once shipped with zero laws *and* zero consumers, caught only by counting call sites. The
promotion criterion was never "more features". It is **a second consumer that someone else reached
for** — and the corpus above exists to make that reach cheap.

## How this library is kept honest

Three disciplines, each bought with an incident:

**A law per JOIN, not only per component.** Nineteen laws, every one mutation-proven and green,
did not see six defects that could compute a wrong answer — because each proved one verb in
isolation and every defect lived at a **seam** between two separately-built pieces. When a fix is
mutation-tested now, the recorded result is which laws go red: repeatedly it has been *"N passed, 1
failed"*, and the 1 is always the seam law. A law per component proves the components and says
nothing about the paths between them.

**Mutate to the DO-NOTHING implementation.** An identity passes far more gates than a scramble.
`test-shrink-index` was passed by replacing `shrink-index`'s entire body with `k`, because its
space had been chosen where the correct answer equals the do-nothing answer.

**Never let a law's oracle be the implementation.** Four laws computed their expected values with
`digit`/`shift` — the verbs under test — so breaking those moved the SUT and the expectation
together and the law stayed green. They compare against literal tables now.

And two rules about numbers, both learned the hard way: **six samples and a stated mean, or no
number** (3-sample medians on a ~700ms benchmark cannot resolve 10%, and a regression was once
chased that did not exist); and **never quote a ratio against a denominator you do not control** —
a ratio whose denominator is expected to shrink decays toward false untouched.

**On adding combinators.** This library once built verbs because the tradition has them, then
proved them against laws written by the same hand — a closed loop with no consumer pulling. One
shipped with zero laws *and* zero consumers and was caught only by counting call sites. The rule
since: **if a consumer writes it twice, it earns its slot.** One-line conveniences that expand to a
single existing call (`pure`, `set-of`) stay out.

## Cost

Per shape, and there is no single number. Measured 2026-08-26, release, mean of six, minus the
~325ms stdlib bootstrap:

| shape | per point |
|---|---|
| `ints` (500k) | ~2.4 µs |
| `coords` bases `[50 100 100]` (500k) | ~33 µs |
| `such-that ∘ bind ∘ record` (card 1260) — *the shape the rete fuzzer ships* | **~287 µs** |

**No ratio against the `$oracle` is quoted**, and that is deliberate rather than an omission. The
`$oracle` is slow-but-correct by design and carries **no perf requirement** — it gets passively
faster as wat stops being interpreted. A ratio whose denominator is expected to shrink is a claim
that decays toward false with nobody touching it, and "never the bottleneck" decays fastest, since
this library's share of a case *grows* as the reference speeds up. Budget from the absolute row
matching the shape you are building.

## Where the tests live

`wat-tests/` is for tests written **in wat, for wat**; `tests/` is for **Rust tooling**.

```
wat/gen.wat    <->  wat-tests/gen.wat                     27 deftests
wat/rete.wat   <->  wat-tests/rete/differential-fuzz.wat   1 deftest (the ratchet)
```

**`deftest` structurally removed a bug the old shape had.** The script version summed its laws by
hand with a `checked=` total that was a hand-maintained literal. Adding three laws, the sum
silently failed to match: three laws fell out of the total while the suite still reported
`laws=21 checked=325 violations=0`. One law per `deftest` removes the shape entirely — there is no
sum, so there is nothing to drop a law from.

The fuzzer's gate is a **ratchet pinned at 120 divergences of 1260 shapes**, in three families, all
reproduced in `tests/rete/probe_arc278_fuzzer_found_divergences.{rs,wat}` and tracked in
`docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md`. Asserting zero would redden the floor and
block unrelated work; deleting the accumulate shape to keep a gate green is the trade this codebase
refuses. Movement either way is a red test demanding an explanation.

It carries `(:wat::test::time-limit "60s")` — and raising it in the wat file was only half. Until
2026-08-26 `scripts/floor.sh` passed no `--profile`, so nextest's `[profile.default]` killed at
**30s**, half the budget the file argues for, and the rete cohort's override missed it because it
filters `binary_id(wat::rete)` while a wat `deftest` compiles into `wat::kernel`.
`.config/nextest.toml` now names it in all three profile mirrors. **If the test is renamed, that
filter must move with it or the 30s kill returns silently.**

## Provenance

Promoted to the stdlib on the `wat/grep.wat` precedent — a move of proven code with shipped
numbers. The namespace was also a live defect: in `wat-scripts/` the library defined `:user::Gen`,
`:user::ints` …, squatting in its own consumer's namespace, and scripts cannot define under
`:wat::`, so promotion was the only available fix.

The 18-ward audit that found the defects this document now describes as fixed, and the full
per-finding record, is
`docs/arc/2026/06/278-rules-engine/GEN-VIGILIA-2026-08-25.md`. Read it before changing `wat/gen.wat`
— it is where the reasoning behind each guard lives, including the ones that look removable.
