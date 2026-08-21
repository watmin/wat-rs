# DESIGN — arc 109 step ②a: THE UNWRITABLE ONES. Bare parametric heads, fixed BEFORE any wall.

`DESIGN-STONE-the-four-walls.md` establishes why this goes first: Walls 1/3/4 emit ~4,354 errors that
each carry their own remedy; **Wall 2 emits errors that carry nothing.** A bare head has no args to
recover — the information was never written down, and the checker cannot infer K,V at an *annotation*
site. Raise all four together and the sites needing real judgment are buried inside the ones needing
none, indistinguishable.

> **Do the unwritable work first, alone, where it cannot hide.**

## The measurement — 88% of it is one subsystem

242 bare parametric heads in type position (after `<-` / `->`), across `wat/` + `tests/**/*.wat`:

```
wat/rete.wat                                  166   ← 69% of the whole population
tests/rete/probe_arc278_4a_production_fire     20
tests/rete/probe_arc278_4b_cascade             16
tests/rete/probe_arc278_4c_retraction           8
tests/rete/…native_insert / …insert_all         4
─── rete subtotal                             214   ← 88%
tests/collection/…transform_dispatch_parity     3
tests/collection/…hashmap_roundtrip             2
wat/query.wat                                   1
tests/types/…primed_generic_head_primed         1
─── the tail                                   28
```

⚠ The four-walls stone says **238**; this pattern says **242**. The difference is pattern shape, not
corpus drift — neither number is authoritative and the STRIKE must not be scored against either.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]` The gate is *zero bare heads
remaining*, measured after, not a count matched up front.

## ★★ IT IS NOT 166 JUDGMENTS. IT IS ~10, EACH REPEATED.

`rete.wat`'s bare heads cluster on a small set of recurring semantic roles:

```
alpha-mem 10 · beta-mem 6 · network 4 · bindings 4 · prod-mem 3 · net 2 · ext 2
derived 2 · acc 2 · support · production-memory · nb · m · elements …
```

Determine the type ONCE per role; apply it mechanically per site. *IVDICIVM SEMEL, MACHINA SAEPE* —
judgment spent once, machine applied many times.

## ★★ AND THE TYPES ARE ALREADY WRITTEN DOWN — IN THE PROSE ABOVE THEM

The evidence is inside a single record (`rete.wat:28`):

```wat
;; bindings: {?var → value} — variable bindings accumulated left-to-right.
(:wat::core::defrecord :wat::rete::Token
  [matches  <- :wat::core::PersistentVector<(wat::core::Record,wat::core::i64)>   ; PARAMETRIC
   bindings <- :wat::core::PersistentMap])                                        ; BARE
```

Same record, same author, **one line apart** — one honestly typed, one not. And three lines above,
the author's own comment: *"the pair is heterogeneous: a Record + an i64, **which a bare PV cannot
honestly type**."* The discipline was understood and applied to `matches`; `bindings` simply never got
it, and its intended shape sits in the comment (`{?var → value}`) instead of in the type.

**That is the method: read the doc comment above the role, write the type it already describes.**
This is recovery of documented intent, not invention.

## The one contract decision

> **When the prose does not say, the site is a FINDING — not a guess.**

`:wat::core::Value` type-checks and would silence every site. It is also the weakest type that
compiles, and writing it where a truer type exists cements the heresy in a form that now looks
compliant. **A site whose true K/V cannot be determined from its comment, its constructors, and its
consumers is reported, not filled.** Better 12 unresolved sites named than 12 lies that pass the wall.

## The strikes

| # | scope | sites | shape |
|---|---|---|---|
| **A** | `wat/rete.wat` | 166 | ~10 roles · read the prose · judgment once each |
| **B** | `tests/rete/**` | 48 | mostly mirrors of A's roles — apply A's answers |
| **C** | the tail (4 files) | 28 | individually read; no shared roles |

A first. B inherits A's role table and should be near-mechanical. C is small and independent.

## Out of scope — affirmatively cut

- **Every wall.** Nothing becomes illegal in this stone; it is preparation. The corpus must stay green
  throughout, because a bare head is still legal.
- **Constructor call sites** (the 977). Value position, a different population, Wall 3's business.
- **`Session.network`'s heterogeneity.** `rete.wat:171`'s comment records that `network` is declared
  id→Node but stores raw heterogeneous node records. If a role's true type needs a supertype that does
  not exist, that is a FINDING under the contract decision above — do not mint a type here.

## The four questions

- **Obvious?** YES — each site gains the type its own comment already describes.
- **Simple?** YES — ~10 determinations, then mechanical application.
- **Honest?** YES, and *only* because of the contract decision: a stone that wrote `Value` everywhere
  would show the same green floor while making the heresy permanent and invisible.
- **Good UX?** YES — `bindings <- (:wat::core::PersistentMap [K V])` tells a reader what the engine's
  hottest structure actually holds, which today only a comment does.
