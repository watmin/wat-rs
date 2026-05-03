# Arc 144 — REALIZATIONS

Discipline named at decision points. Captures the architectural
reasoning that emerged DURING the arc, so future readers (including
future-Claude across compactions) inherit the WHY behind the WHAT.

## Realization 1 — Two parts of the substrate disagree on polymorphic input

**Discovered:** 2026-05-03 mid-arc-144-slice-3, surfaced by the
length-canary diagnostic + user pressure ("are A and D masking
something deeper?").

The substrate runs two parallel type-checking models:

1. **Scheme-based primitives** (foldl, map, etc.) — registered in
   `CheckEnv` via `env.register(name, TypeScheme)`. Rank-1
   parametric polymorphism; `instantiate` freshens type variables
   at every call site. The scheme IS the contract.

2. **Handler-based primitives** (length, empty?, contains?, get,
   assoc, dissoc, keys, values, conj, concat — 10 of them) —
   bypass scheme instantiation and do AD-HOC dispatch on concrete
   container shapes via hardcoded `infer_*` functions. The
   registered scheme is purely a fingerprint (slice 3); the
   handler IS the contract.

When polymorphic input meets a handler, the substrate **disagrees
with itself**: scheme says "any T", handler says "concrete
container shapes only." The TypeMismatch the length canary
surfaces:
```
expected: "Vec<T> | HashMap<K,V> | HashSet<T>"
got:      ":T"
```
reads like a parse error because the substrate's two parts give
contradictory answers on the same call.

## Realization 2 — Why the handlers exist (correct framing)

**My initial drafts of this section were wrong THREE TIMES.**

First draft: "handlers exist because the substrate lacks union types."
Second draft (after user pushback): "handlers exist because two
different categories need different features (constraints + type
classes)." Third draft: "TypeScheme is too narrow."

**All three were wrong in the same way.** Each assumed the substrate
is missing a feature. None checked whether the substrate's existing
model is actually sufficient.

**Correct framing (verified):**

The substrate's type system is INTENTIONALLY rank-1 parametric
polymorphism — `forall vars` at the front of the scheme, no
constraints, no overloading, no return-type dispatch. That's the
design constraint, not a limitation.

After the Type/verb correction (arc 146), every container-method
primitive becomes a clean rank-1 scheme:

- `Vector/length : forall T. Vec<T> -> :i64` — clean rank-1
- `HashMap/length : forall K,V. HashMap<K,V> -> :i64` — clean rank-1
- `HashSet/length : forall T. HashSet<T> -> :i64` — clean rank-1
- `Vector/get : forall T. Vec<T> -> :i64 -> :Option<T>` — clean rank-1
- `HashMap/get : forall K,V. HashMap<K,V> -> :K -> :Option<V>` — clean rank-1
- `Vector/conj : forall T. Vec<T> -> :T -> :Vec<T>` — clean rank-1
- ... and so on for all 10 retired handlers.

**Every single one fits rank-1 cleanly.** No constraints, no
type classes, no union types, no ad-hoc polymorphism needed.

**The handlers do NOT exist because the substrate is missing
features.** They exist because the primitives were named in a way
that VIOLATES the substrate's design constraint. `length` claimed
to be ONE polymorphic operation across three container types —
that broke rank-1 (rank-1 can't express "across three container
types as one name"). Someone compensated with a handler. The
handler is a MANIFESTATION OF THE NAMING VIOLATION, not a
workaround for a missing feature.

The substrate's rank-1 minimality is sustainable IF AND ONLY IF
primitives respect Type/verb naming. Without the convention,
primitives would always need to escape rank-1 via handlers. The
convention is what keeps the type system minimal AND complete.

## Realization 3 — Arc 146 restores the convention

**User direction 2026-05-03**: "i think what we just discovered
is that length is poorly defined and it needs to be corrected."

Arc 146 redefines the 10 violating primitives per arc 109's
existing Type/verb convention (Option/expect, Result/try, etc.):

```
:wat::core::length         →  :wat::core::Vector/length
                              :wat::core::HashMap/length
                              :wat::core::HashSet/length
:wat::core::get            →  :wat::core::Vector/get
                              :wat::core::HashMap/get
:wat::core::keys/values    →  :wat::core::HashMap/keys, /values
:wat::core::dissoc/assoc   →  :wat::core::HashMap/dissoc, /assoc
                              :wat::core::Vector/set
:wat::core::conj           →  :wat::core::Vector/conj
                              :wat::core::HashSet/conj
:wat::core::concat         →  :wat::core::Vector/concat
:wat::core::string::concat ALREADY CORRECT (arc 109 namespace)
```

After arc 146:
- 10 violating primitives → ~20 properly-named Type/verb primitives
- Each becomes a clean rank-1 TypeScheme
- The 10 hardcoded handlers retire (nothing for them to do)
- `lookup_form` finds every primitive via the scheme path uniformly
- Aliases work for every primitive (no handler-vs-scheme
  collision because there's only one model left)
- The substrate's design coherence is restored

**There is no "future fix is open" — arc 146 IS the fix.** Designed
+ slice-planned + tracked. Tasks #240-243 opened.

## Realization 4 — Why slice 3b is CANCELLED

Originally (slice 3 SCORE) the orchestrator enumerated 4 options
to bridge the collision:

- A: per-handler defer arms
- B: constrain alias shapes (rejected — fails obvious)
- C: document as "not aliasable" (rejected — fails honest)
- D: wrapper at dispatch seam

A and D were both viable bridges. The user's framing — "the most
atomic form for something to build atop of" + "if a thing says it
of a generic kind then it is" — initially pointed at D (atomic
foundation; single seam where the deeper fix would land).

**The user's deeper question — "are A and D masking something?" —
revealed that BOTH bridge a substrate inconsistency that
shouldn't exist.** The right fix isn't a bridge; it's removing
the inconsistency by correcting the primitive definitions
(arc 146).

**Slice 3b on Option D is therefore CANCELLED.** Building a
bridge over an inconsistency that arc 146 will REMOVE would be
short-lived scaffolding. Better to ship arc 146 directly.

## Realization 5 — Cascade reordered

```
arc 130 RELAND v1 stepping stone fails: "unknown function: :reduce"
  → arc 143 ships :wat::runtime::define-alias end-to-end
  → arc 144 slice 1: Binding enum + lookup_form refactor
  → arc 144 slice 2: special-form registry (36 forms; +13/-9 deltas
    via audit-first discipline)
  → arc 144 slice 3: TypeScheme fingerprints for 15 hardcoded
    callables; Mode B-canary clean diagnostic surfaces handler-
    vs-scheme collision
  → arc 144 slice 3b CANCELLED — bridges an inconsistency that
    arc 146 removes
  → arc 146: Type/verb container-method correction
  → arc 144 slice 4: verification (length canary green via
    arc 146's Vector/length etc., not via a wrapper)
  → arc 144 slice 5: closure
  → arc 130 RELAND v2 picks up
  → ...
  → arc 109 v1 closes
  → CacheService.wat in-flight resolves
```

## Realization 6 — The discipline lesson

**Three drafts. Three wrong framings.** Each defaulted to "the
substrate needs new features" + "the future fix is open." Both
were assumption-presented-as-truth. Both were deferrals.

The correct discipline (per `feedback_attack_foundation_cracks` +
`feedback_no_known_defect_left_unfixed` + `feedback_absence_is_signal`):

> When a crack surfaces, the fix IS also the diagnostic. Apply,
> use as compass, pivot forward. Don't defer; don't speculate
> about future fixes. NAME the fix and SCOPE it.

The trap I kept falling into:

> Find flaw → assume substrate is missing X → defer "future fix
> open"

The corrective:

> Find flaw → check if primitives respect substrate's existing
> conventions → fix the violation (arc 146); pivot forward.

The substrate is rarely the problem. The convention violations
are. Default to "primitive is wrong"; only conclude "substrate is
wrong" after the convention check rules it out.

Saved as memory `feedback_pivot_not_defer.md`.

## Discipline notes

- The collision was INVISIBLE until polymorphic input met a
  handler. The macro layer — by introducing free type-variables
  through synthesized aliases — was the first caller that exposed
  the disagreement. Substrate-as-teacher pattern at its purest.
- The user's "atomic foundation" tiebreaker (which the four
  questions don't explicitly capture) prompted the "what does this
  mask?" question — which surfaced that bridging was the wrong
  fix at all.
- Knowing what you don't know matters. Each draft of this
  realization claimed knowledge I hadn't verified; user pressure
  forced honest decomposition; the corrected framing emerged.
- The substrate's rank-1 minimality + Type/verb convention
  together form a coherent design. Adding type-system features
  would EXPAND the substrate; restoring the convention is the
  cheaper, more honest move.
