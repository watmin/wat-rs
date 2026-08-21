# NOTE (arc 255, #110) — THE INVENTORY. 32 verbs the checker cannot see, out of 520.

**Filed 2026-08-20. MEASURED, with four controls.** The 294 seam records `peer-pid` as the
blanket-accept's *"first named victim"*. Nobody had ever counted the rest. This is the count.

## The census

Every verb the runtime dispatches — `runtime.rs` literal arms **∪** `#[wat_intrinsic]` registrations,
deduped: **520**. Each handed NINE arguments. A verb with any scheme or bespoke arm refuses; a
blanket-accepted verb passes.

```
520  probed
487  CHECKED     — a scheme or an infer arm refuses 9 args
 33  UNCHECKED   — blanket-accepted (32 real; 1 is my extractor's `<fqdn>` doc placeholder)
```

**94% covered.** #110's remaining surface is 32 verbs, not hundreds — materially smaller than the
seam's framing implied, and that is good news worth having before scoping 255.1b-iv.

### Controls (the probe discriminates before it is trusted)

```
:wat::core::HashMap/get        REJECT   known scheme
:wat::core::PersistentMap/get  REJECT   scheme added today at 9c82f157
:wat::kernel::peer-pid         ACCEPT   the seam's known victim
:wat::core::totally-bogus      ACCEPT   ← see below; this is a FINDING, not a failed control
```

## ★★ THE BIGGER FINDING — the blanket is the RESERVED PREFIX, not a missing scheme

The fourth control was supposed to reject. It does not, and the namespace is why:

```
(:wat::core::totally-bogus 1 2)   exit 0   invented verb, ACCEPTED
(:wat::io::totally-bogus 1 2)     exit 0   ACCEPTED
(:wat::rete::totally-bogus 1 2)   exit 0   ACCEPTED
(:user::totally-bogus 1 2)        exit 1   UnresolvedReference
(:my::totally-bogus 1 2)          exit 1   UnresolvedReference
```

At **any** arity — 0, 1, 2, 9. This is `255/DESIGN.md:10` in its own words: *"Rust builtins … **nowhere**
— a 454-arm compile-time `match` … **(can't)** → reserved-prefix blanket-accept."*

**So #110 is not "32 verbs lack schemes." It is: any `:wat::`-prefixed name is accepted at call
position whether or not it exists.** The 32 are the verbs that DO exist and are unchecked; the hole
also swallows every name that does not.

### The consequence, demonstrated

```wat
(:wat::core::HashMap/gett m "k")      ; typo for /get
```
`--check` → **exit 0**, silent. Runtime → `UnknownFunction: :wat::core::HashMap/gett`.

**`--check` certifies a program that calls a function which does not exist.** Caught at runtime, so
not silent corruption — but a real honesty gap in the one instrument that claims a program is sound
before it runs. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## The inventory — 32 verbs

Grouped by why they are probably unchecked. ⚠ **The groupings are my reading, not measured** — each
needs its own look before anyone acts.

**Special forms / macro machinery (11)** — plausibly checked elsewhere, by a bespoke path this probe
cannot see. Verify before treating as gaps.
`fn` · `do` · `apply` · `quasiquote` · `forms` · `macro-error` · `fresh-symbol` · `defalias` ·
`use!` · `str` · `variant`

**Constructors / aggregate machinery (5)**
`aggregate-new` · `kwargs-construct` · `struct-new` · `struct-field` · `None`

**The `List` family (6)** — the std twin of a family that IS checked (`Vector/*`, `HashMap/*`), so this
is the closest analogue to the 13 persistent verbs fixed today at `9c82f157`.
`List/of` · `List/get` · `List/length` · `List/empty?` · `List/contains?` · `List?`

**Container heads used as constructors (2)** — `PersistentVector` · `Tuple`. ⚠ Note both are among the
six whose BRACKET form landed today; their bare constructor path is what this row measures.

**Domain verbs (8)** — the ones most likely to be genuine gaps:
`:wat::kernel::peer-pid` (the seam's named victim, on the capability path) ·
`:wat::runtime::metadata-of` (**arc 255's own reflection verb**) ·
`:wat::holon::CosineOutcome` · `:wat::holon::DotOutcome` ·
`:wat::verify::file-path` · `:wat::verify::s3-path` · `:wat::verify::string` ·
`:wat::intrinsic::variadic-args-measurement`

★ `:wat::runtime::metadata-of` being unchecked is worth its own look — it is the verb arc 255 built to
answer *"what are the properties of this form?"*, and the checker cannot see its call sites.

## What this changes for 255.1b-iv

The seam says #110 must precede the flip because *"after the flip `(f HashMap<K,V>)` reads as VALID
EDN and silently changes arity 2→3."* That reasoning is unaffected. What changes is the **shape** of
the fix:

- Registering schemes for the 32 does **not** close the hole. The reserved-prefix blanket accepts
  unknown names, so a typo still passes. **The resolver must verify `:wat::` names against the
  registry** — which is exactly what arc 255 built the registry to make possible.
- The 32 are then the population that must have schemes *before* the blanket is removed, or their call
  sites go red.

⚠ **UNMEASURED, and load-bearing for that plan:** whether removing the blanket makes the existing
corpus red, and by how much. Nobody has run that. It is one `git stash`-free experiment away and it
should be measured before 255.1b-iv is scoped, not after.

## Reproducing

The probe is nine args to every verb, comparing against the four controls above. It is small enough to
re-derive, and deliberately not committed as a script — the census's value is the number and the
inventory, both recorded here. Re-run it after any scheme work to watch 32 fall.
