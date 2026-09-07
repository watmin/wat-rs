# NOTE — match/cond: clause-as-bracket + drop the non-return `-> :T`; the conditional form needs a new name (NOT `cond`)

> **Deferred design decision (builder, 2026-07-21, arc 278).** Surfaced reviewing the caller.1
> `probe_arc278_call_site.wat` match form. **rete (arc 278) is the stepping stone** to the broader
> `-> :T`-in-non-return-positions annihilation that this rides on; the swap lands after that. Recorded here
> per the arc-109 `NOTE-*.md` convention (sibling of `NOTE-generic-bracket-syntax-edn.md`).

## The principle (the "why", general beyond match)
Scheme uses **one delimiter for two meanings**: `(...)` is both *apply this* and *group these*. Clojure split
them — `(...)` = application, `[...]` = ordered structure, `{...}` = associative structure — which is what makes
a Clojure dialect read cleanly. wat is mid-migration from the first to the second. **The rule: anywhere a form
is grouping rather than calling, it is a bracket — no exception.** A `match`/`cond` clause is *structure* (an
ordered pattern/test paired with a body), not a call, so it is a bracket. Then the only parens left inside a
clause are the genuine calls (the pattern-constructor `(:Some f)`, the body-call `(contains? …)`), and the eye
separates structure from application instantly. This is the same move `let`/`fn`/`defn`/`defrecord` already made
(bindings/params/fields are `[...]`); applying it to clauses makes `match`/`cond` *rhyme* with the rest of wat.

## The swap (match)
```clojure
;; TODAY (Scheme heritage — list-clauses + a NON-RETURN `-> :T` annotation):
(:wat::core::match file -> :wat::core::bool
  ((:wat::core::Some f) (:wat::core::string::contains? f "…"))
  (:wat::core::None     false))

;; TARGET — bracket-clauses, and the inline `-> :T` GONE (type inferred by per-arm unification):
(:wat::core::match file
  [(:wat::core::Some f) (:wat::core::string::contains? f "…")]
  [:wat::core::None     false])
```
Two changes, both ratified:
1. **Clause `(pattern body)` → `[pattern body]`** — the bracket principle above. Kills the Scheme double-paren
   `((`. Single body form (use `(do …)` for multiple, as Clojure).
2. **Drop the inline `-> :T`** — it is a `-> :T` in a **non-return position** (a match-arm-type annotation, not a
   fn return), and those are being annihilated (builder, 2026-07-21). The arm type is *inferred* (per-arm bodies
   unified to one type by the checker), not annotated at the form. rete is the stepping stone to the general
   non-return-`-> :T` removal; this swap lands with / after it. **(UPDATE 2026-07-22: the non-return `-> :T`
   annihilation is COMPLETE — `match`/`if`/`apply`/`readln'` all reject it; `-> :T` is legal only at a fn/defn
   argspec return. So change #2 has LANDED corpus-wide; only the clause-bracket flip (#1) + the LHS refinement
   below remain deferred.)**

## The LHS refinement — a match arm is a FLAT variant clause `[Variant [binds] body]` that RHYMES WITH `fn` (builder, 2026-07-22)
The bracket-clause swap above settles the *clause* delimiter (`[…body]`). This settles the **arm shape inside it**:
a variant arm is the **flat three-element clause** `[<Variant> [<destructure>] <body>]` — the variant, its body's
destructure vec, then the body — exactly the `[binds] body` shape `fn`/`let`/`defn` already use. The
vector-bodied variant encoding (arc 258 R45 `LVCEM TENEBRASQVE FERO` / R46 `IN LVCE PVRGATI` — *every* variant is
`[]` unit or `[items]` N, one uniform rule) is what makes the destructure vec exact: it destructures the variant's
vector-encoded body, per-arity uniform.

```clojure
;; TODAY — HETEROGENEOUS LHS: a bare field-binding for Some, a bare keyword for None (two shapes to learn):
(:wat::core::match opt
  [(:wat::core::Option/Some f) f]
  [:wat::core::Option/None     nil])

;; TARGET (builder, 2026-07-22) — the FLAT variant clause `[Variant [destructure] body]`, rhyming with `fn`:
(wat.core/match opt
  [wat.core.Option/Some [n] n]      ;; Some's 1-field body → destructure [n], then body `n`
  [wat.core.Option/None []  nil])   ;; None's EMPTY body   → destructure [],  then body `nil`
;; (rust-scheme surface, same shape: [:wat::core::Option/Some [n] n] / [:wat::core::Option/None [] nil])
```

Why the flat form is right:
- **It rhymes with `fn`/`let`/`defn`.** `[binds] body` — a destructure vec then a body — is *the* shape across all
  of wat. A match arm IS destructure-and-bind (like fn params), so a variant clause `[Variant [binds] body]` reads
  as "for `Variant`, destructure `[binds]`, then evaluate `body`" — a named-arity clause. No new shape to learn.
- **The wire still reads this way (R45/R46).** A variant value *is* `[tag …body]`; the `Variant [binds]` head+vec
  destructures that body — the WRITER of a variant and the READER of an arm see the **same body-shape** discriminator.
- **Flat, no pattern-call wrapper.** The variant and its destructure sit as two flat arm elements, not nested in a
  `(Variant [binds])` call — fewer parens, the eye reads left-to-right (which variant · what it binds · what it does).
- **The empty `[]` is visible proof of a unit variant** ("carries nothing"), never an absence you infer.

### The grammar assertion (the invariant the parser/checker holds) — the HEAD disambiguates arm-arity
A match arm is **exactly one of**:
1. `[_ body]` — the non-binding **wildcard** clause (2-element).
2. `[<bare-symbol> body]` — a **binding** clause: binds the whole scrutinee as that bare name (2-element).
3. `[<Variant> [d0 d1 …] body]` — a **variant** clause (3-element): a namespaced variant head, its destructuring
   vec (arity **equals** the variant's field count — `[]` for a unit variant; a wrong-arity vec is a located error),
   then the body.

The **head's form is the discriminator, context-free**: a *namespaced* head (`Type/Variant`, or a `::`-keyword in
rust-scheme) is a variant clause (3-element, expects a destructure vec then body); a *bare* symbol or `_` is a
wildcard/binding clause (2-element, second element is the body). So `[x [1 2 3]]` is unambiguously a binding `x`
with body `[1 2 3]` (bare head → 2-element), while `[Foo/Bar [n] body]` is a variant clause (namespaced head →
3-element) — no type info needed to parse the shape.

Rejected: the old **bare-field** form `(<Variant> field)`, the old **bare-keyword** unit `:Variant`, AND the
**nested pattern-call** `[(<Variant> [d]) body]` (the considered 2-element alternative — see below).

### The considered alternative (nested 2-element `[(Variant [binds]) body]`) — and why the flat form wins
An earlier draft made every arm a uniform 2-element `[pattern body]` where the variant pattern was the call
`(Variant [binds])`. It had two virtues: a uniform 2-element arm, and a shape *shared* with `cond`'s `[test body]`.
The flat form trades those for the `fn`-rhyme and flatness — and the trade is net-positive, because the shared
2-element arm was the *weaker* rhyme (a match is destructure-and-bind, like `fn`, not a flat test/expr pair). The
four questions on the flat form: **Obvious** YES (fn-clause shape), **Simple** YES (flat; the namespaced-head
discriminator is context-free), **Honest** YES (the destructure vec shows arity), **Good UX** YES (a shape every
wat user already knows from `fn`). Losing the cond-shared arm is not a loss — it **sharpens** the match≠cond
distinction (next section): a 3-element variant clause is visibly a *different form* from a 2-element cond clause.

## `cond` — appealing but the NAME is suspect; NEEDS A NEW TERM (intueri — OWED, not yet cast)
`cond` (a defmacro, `wat/core.wat:1204`) today is `(cond (test body) … (:else body))` — head = a truthy *test*.
Under the flat variant-clause refinement above, the two forms **no longer share an arm shape**: a `cond` clause is
2-element `[test body]`, a `match` variant clause is 3-element `[Variant [binds] body]`. That divergence *reinforces*
keeping them distinct — a unified bracket-clause was the old appeal, and it is gone. Even setting arity aside:
**do not name the bracket-claused conditional form `cond`.** A Clojure user reads `cond` and expects **flat** `test expr test expr` pairs
(Clojure `cond`/`case`/`core.match` are all flat, no per-clause delimiter) — a bracket-claused `cond` would
*induce confusion* (builder). So the unified/bracketed conditional-dispatch form **needs a new term**, distinct
from Clojure's `cond`.
- **OWED: cast intueri** for that term when the swap is drawn (intueri owns all names — do not narrate one here).
  Constraint for the cast: a name for a *bracket-claused, ordered, first-match dispatch* form (test-dispatch
  and/or pattern-dispatch), that does **not** collide with a Clojure user's `cond` expectation.
- **OPEN:** whether `match` (pattern-dispatch, typed, exhaustive) and the test-dispatch form stay two forms or
  unify under the one new term. Not sold on unifying under `cond`. Decide at draw time.

## Grounded facts (verified 2026-07-21)
- `match` is a runtime special form — `eval_match` (`src/runtime.rs:13088`), tail twin `eval_match_tail`
  (`:3578`); current grammar requires `-> :T` between scrutinee and arms (`:3596`), each arm a list `(pat body)`.
- match **catch-all is `_`** (non-binding wildcard, `runtime.rs:13073`) or a **bare identifier** (binds the
  scrutinee as that name, `:13072`) — **not** `:else`. (So `:else` is cond's terminal; a `match` over a closed
  enum closes by **exhaustiveness**, needing no catch-all — e.g. `[:None false]` completes `Option`.)
- `cond` is a defmacro (`wat/core.wat:1204`), clauses `(test body)`, terminal `:else`.

## Execution (when drawn)
A **corpus-wide wat-fix codemod** (`wat-scripts/fixes/`, never hand-edits) — a semantics-preserving delimiter
flip (`(clause)` → `[clause]`) + the inline-`-> :T` drop; hard-flip (parser requires the bracket, no bad-form
education). Fold with any concurrent match-touching sweep to touch the tree once.

## Status
**DEFERRED.** rete (arc 278) is the stepping stone; lands with/after the non-return-`-> :T` annihilation.
The new term is an **owed intueri cast** at draw time.

---

## ⛔ AMENDED 2026-08-25 — THE FORM SURVIVES `296/STONE-H`; TWO OF ITS FOUR REASONS DO NOT

`294/SEAM.md` carried this as an open contradiction — *"H makes a variant body a MAP; the match note
destructures it as a VECTOR and cites the vector encoding as its warrant. Neither knows."* Both docs
were read in full, 2026-08-25. **The seam was right about the citation and wrong about the conflict**,
and the difference is what makes this cheap.

### The form does not collide

`[Variant [binds] body]` works unchanged under H. Positional destructuring of a variant needs the
variant's **fields to be ORDERED**, not its wire body to be a vector — and they are ordered at the
declaration: `EnumVariant::Tagged { fields: Vec<(String, TypeExpr)> }`. Clojure destructures a map
positionally-by-declaration in exactly this way. Nothing in the grammar assertion above needs to move.

### But the WARRANT was pinned to a wire format that H replaces

Two sentences in this note are load-bearing and go **false** under H — not wrong when written, and
not edited by anyone; left behind when their subject moved:

1. *"The vector-bodied variant encoding (arc 258 R45/R46 — every variant is `[]` unit or `[items]` N,
   one uniform rule) is what makes the destructure vec exact: it destructures the variant's
   vector-encoded body, per-arity uniform."*
   Under H the body is `{:value 42}`. **The stated foundation of the vec's exactness stops existing.**
   The vec is still exact — but because the declaration's field list is ordered, not because the wire
   is a vector.
2. *"**The wire still reads this way (R45/R46).** A variant value *is* `[tag …body]`; … the WRITER of
   a variant and the READER of an arm see the **same body-shape** discriminator."*
   H deletes this in both halves. The wire is `#wat.core/Option.Some {:value …}`, and H says outright
   that once the dot in the tag discriminates, *"body shape stops carrying any burden at all."* There
   is no shared body-shape discriminator left to see.

### The fix is a DERIVATION, not a patch

The warrant is re-grounded on the thing that does not move:

> The destructure vec is exact because a variant's fields are **declared in order** and that order is
> what the arm binds — `[]` for a unit variant, `[d0 d1 …]` matching the declared arity. The vec
> mirrors the DECLARATION, not the wire. Whatever the wire carries — arc 258's `[items]` vector or
> `296/STONE-H`'s `{:key …}` map — the arm reads the same, because the declaration is the same.

That version survives H, survives 251's keyword→symbol flip, and survives whatever replaces the wire
after those. The old warrant could not survive its own arc.

★ **This is R9's class, still alive.** Written 2026-07-22 and true then; H drawn 2026-08-15; the note
went false with nobody's hand on it, and neither document knew. A claim that recomputes itself from
its subject — *fields are declared in order* — cannot rot this way.
`[[R9 DERIVAMVS NE MENTIAMVR]]`

### What is NOT settled here

Whether H ships at all. This amendment says only that **if** it ships, this note's form stands and its
warrant is the above. H remains DRAWN, NOT BUILT.

---

## ★ AMENDMENT 2026-09-06 — THE ARM'S MIDDLE ELEMENT IS A MAP PATTERN, NOT A DESTRUCTURE VEC

**The flat positional clause `[<Variant> [d0 d1 …] <body>]` ruled above is RETIRED.** The arm is now:

```clojure
;; the wire (arc 296 stone H)
#wat.core/Option.Some {:val 42}

;; the arm — NEARLY THE WIRE ITSELF
(wat.core/match option
  [wat.core/Option.Some {:val val} (wat.core/+ 0 val)]
  [wat.core/Option.None {}         -1])
```

Builder, 2026-09-06: *"seeing them in the match form as nearly identical as their wire form… is quite
compelling"* — and, on the migration cost: *"it will be a rough migration.. but i do not care - i want
my language to be coherent."*

### Why the positional vec had to go — its warrant failed TWICE

1. **Original warrant (2026-07-22, in this NOTE above):** *"the wire still reads this way (R45/R46) …
   the WRITER of a variant and the READER of an arm see the same body-shape discriminator."* Stone H
   deletes the vector wire; body shape "stops carrying any burden at all."
2. **Re-warrant (`dd35875d8`, 2026-08-25):** *"the vec mirrors the DECLARATION, not the wire"*, resting
   on *"Clojure destructures a map positionally-by-declaration in exactly this way."* **That clause is
   not true of Clojure** — map destructuring there is by key; positional destructuring is for
   sequences, and records destructure as maps. The re-warrant re-grounded onto a false premise.

A form whose justification has been rebuilt once and then rests on a wrong claim about the language
we are becoming was never a settled ruling.

### The deciding property — POSITION vs NAME

The positional vec binds **by position** against data that is **named**. Reorder a declaration's
fields and every arm of arity ≥ 2 silently rebinds, with nothing to catch it. That is exactly the
`field-N` fragility arc 296 G/G′ removed from the wire, surviving one layer up on the READ side. H's
own thesis — *"it declares binders … then throws them away into a positional vector. This stone
removes the exception. One rule for named data, both kinds, both directions"* — is unfinished while
the arm stays positional.

### The order is KEY-FIRST, and the reason is which operation this is

Measured 2026-09-06 against a live record:

```
{a :x  b :y}   binding-first   EXIT=0   ← Clojure's LET-destructure order; what wat accepts today
{:x a  :y b}   key-first       EXIT=3   "let binder must be a bare symbol, a vector of…"
```

Both orders exist in Clojure and they belong to **different operations**: `let` destructuring is
binding-first (`{a :x}`); **pattern matching is key-first** (`core.match`'s `{:x x}` — "has key `:x`,
bind it as `x`"). A match arm is a PATTERN, so it takes the pattern order. The first draft of this
amendment proposed `{val :val}` by reaching for the `let` relative — the identical "wrong relative"
error that made the `fn`-rhyme argument attractive for the positional vec.

### The unit variant is `{}`

Empty map, mirroring `#wat.core/Option.None {}` — visible proof the variant carries nothing, the same
role the empty `[]` played in the retired form.

### The grammar assertion is UNCHANGED

An arm is still exactly one of `[_ body]` · `[<bare-symbol> body]` · `[<Variant> <pattern> body]`, and
the **head is still the context-free discriminator** (namespaced head → 3-element). Only the middle
element's shape changes, from a destructure vec to a map pattern.

### COST, measured — this does not exist yet

```
((usr/Container.X x)      body)   today's arm: pattern-call, BARE positional binding   EXIT=0
((usr/Container.X {:x v}) body)   a map pattern in an arm                              EXIT=3
                                  "malformed match form: map/set literal is not a valid match sub…"
```

One map-pattern reader in match position, plus per-arm name/arity validation against the declaration.
Distinct from `let`'s binder-first destructure (arc 257.2) — same delimiter, different operation,
different order; do NOT route the arm through the `let` path.

### OPEN — the constructor form, deliberately NOT decided here

A variant constructor today takes **kwargs** `(usr/Container.X :x 42)` or **positional**
`(usr/Container.X 42)`. The builder's example writes `(usr/Container.X {:x 42})` — a map literal,
which would complete the symmetry writer → wire → reader as ONE shape. Whether that map form
*replaces* kwargs or joins it is undecided. If it replaces them, it deletes the five-way duplicated
kwargs-vs-positional predicate catalogued in
`[[NOTE-kwargs-or-positional-is-decided-in-five-places]]` — which has ALREADY drifted (two sites
spell the parity check `is_multiple_of(2)`, three spell it `% 2 == 0`). That would make this a far
larger win than a spelling change, and it is the builder's call.
