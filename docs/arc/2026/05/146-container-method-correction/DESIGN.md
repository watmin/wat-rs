# Arc 146 — Multimethod entity + per-primitive correction

**Status:** drafted 2026-05-03. Refreshed after multimethod consensus
emerged from the arc 144 → arc 146 discovery cascade.

**Predecessor framings (rejected; see arc 144 REALIZATIONS):**
- Bridging via per-handler defer arms (slice 3b option A — too leaf-local)
- Bridging via wrapper at dispatch seam (slice 3b option D — hides
  the substrate violation rather than removing it)
- Type/verb retirement (forced sweep, removes the polymorphic surface
  the user actually wants)

**Current framing:** add multimethod as a first-class substrate
entity; declare each polymorphic-name as a multimethod backed by
clean per-Type impls; retire the hardcoded `infer_*` handlers as
each migration completes.

## Context (read this first; arc resumption depends on it)

The substrate today runs two parallel type-checking models:
1. **Scheme-based** primitives (foldl, map) — registered TypeScheme
   in CheckEnv; clean rank-1 parametric polymorphism.
2. **Handler-based** primitives (length, empty?, contains?, get,
   assoc, dissoc, keys, values, conj, concat — 10 of them) —
   bypass scheme instantiation; do ad-hoc dispatch on concrete
   container shapes via hardcoded `infer_*` functions.

The two models DISAGREE on polymorphic input. When a free type-var
`:T` reaches a handler (e.g., from a macro-synthesized alias body),
the handler rejects it ("expected Vec<T> | HashMap<K,V> | HashSet<T>,
got :T") even though the registered scheme says any T is fine.

The handlers exist because polymorphic-name primitives like `:length`
violate the substrate's design constraint (one rank-1 scheme per
name; no overloading). `length` claims to be ONE polymorphic
operation across three container types — that breaks rank-1; the
handler compensates.

**The correction:** add a new entity kind — multimethod — that
honestly represents "this name dispatches over input type to one
of N per-Type impls." The polymorphic name stays; the handler
retires; the substrate has one model (scheme-based) plus one new
entity kind (multimethod) that delegates to clean rank-1 schemes.

Per arc 144 REALIZATION 6 + COMPACTION-AMNESIA-RECOVERY § FM 10:
the substrate gains an ENTITY KIND, not a type-system feature.
This is the smaller architectural change.

## What ships

### A new entity kind: Multimethod

```rust
pub struct Multimethod {
    pub name: String,
    pub arms: Vec<MultimethodArm>,
}

pub struct MultimethodArm {
    pub pattern: Vec<TypeExpr>,    // input-type pattern (one per arg)
    pub impl_name: String,          // keyword path of the per-Type impl to call
}
```

The multimethod's CONTRACT is the arms table. There is no "overall
return type" — each arm's return type is its impl's return type;
each call site resolves to a specific arm and gets that specific
return type. No union-type machinery anywhere.

### A wat-level declaration form

```scheme
(:wat::core::defmultimethod :wat::core::length
  ((:wat::core::Vector<T>)    :wat::core::Vector/length)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
```

Each arm is `((type-pattern...) impl-keyword)`. Pass-through
semantics: all args at the call site flow unchanged to the matched
impl. Constraint: every arm's impl must have the same arity as the
multimethod's surface form.

**Why pass-through (not bound-capture + body):** multimethod is
dispatch; not transformation. Pure routing keeps each arm atomic.
If transformation is needed for a specific case, write a wrapper
function and route to it.

### Check-time dispatch

When the type checker encounters `(:length c)`:
1. Look up `:length` — found as a Multimethod (via lookup_form's
   new 6th branch).
2. Infer types for each arg.
3. Match the inferred arg types against each arm's pattern.
4. Pick the matching arm.
5. The call's return type is THAT arm's impl's return type
   (instantiated via standard scheme machinery).
6. Type-check the call as the impl call.

If no arm matches: clean `TypeMismatch` listing all arm patterns
("expected one of: Vec<T> | HashMap<K,V> | HashSet<T>, got :String").

### Runtime dispatch

When the runtime encounters `(:length c)`:
1. Look up `:length` — found as a Multimethod.
2. Inspect each arg's value tag.
3. Match the tags against each arm's pattern (Vec → Vector arm;
   HashMap → HashMap arm; etc.).
4. Call the matched impl with the same args.
5. Return the impl's result.

If no arm matches at runtime: this should be impossible if
check-time worked, but emit `RuntimeError::MultimethodNoMatch`
with the actual value type for diagnosis.

### Reflection

Arc 144's `Binding` enum gains a 6th variant:

```rust
Binding::Multimethod {
    name: String,
    arms: Vec<MultimethodArm>,
    doc_string: Option<String>,
}
```

Arc 144's `lookup_form` gains a 6th branch (consults the
multimethod registry).

`(:help :length)` returns the multimethod's declaration form as
EDN — the user reads the arms table directly.

`signature-of`, `lookup-define`, `body-of` for a multimethod each
return appropriate views (declaration form, declaration form,
arms table).

### Multimethod registry

A new field on `SymbolTable`:
```rust
pub multimethod_registry: Option<Arc<MultimethodRegistry>>,
```

Mirrors the existing `macro_registry: Option<Arc<MacroRegistry>>`
shape. Per arc 109's "capability carrier" pattern (memory:
`feedback_capability_carrier`).

## What gets migrated (the audit)

Not all 10 hardcoded primitives are GENUINELY polymorphic. Audit
each before assuming multimethod:

| Primitive | Genuine multimethod? | Notes |
|---|---|---|
| `length` | YES — Vector/HashMap/HashSet | 3 arms |
| `empty?` | YES — Vector/HashMap/HashSet | 3 arms |
| `contains?` | YES — but verbs may differ per type | Vector/contains? (T), HashMap/contains-key? (K), HashSet/contains? (T) — open question whether one multimethod or two distinct names |
| `get` | YES — Vector/HashMap | Vector/get (i64 → Option<T>), HashMap/get (K → Option<V>); HashSet's "get-by-equality" is just contains? |
| `assoc` | NO — HashMap-only (and Vector/set is a different verb) | Mint :HashMap/assoc; Vector/set is its own primitive; no multimethod needed |
| `dissoc` | NO — HashMap-only | Mint :HashMap/dissoc; rename arc 146 |
| `keys` | NO — HashMap-only | Mint :HashMap/keys; rename arc 146 |
| `values` | NO — HashMap-only | Mint :HashMap/values; rename arc 146 |
| `conj` | YES — Vector/HashSet | 2 arms; HashMap doesn't conj (uses assoc) |
| `concat` | NO — Vector-only (string::concat already namespaced) | Mint :Vector/concat; rename arc 146 |

Multimethod migrations: length, empty?, contains?, get, conj (5).
Pure rename to Type/verb: assoc, dissoc, keys, values, concat (5).

Both groups need per-Type impls minted as clean rank-1 schemes.
Multimethod group additionally gets a `defmultimethod` declaration
in wat.

## Slice plan

### Slice 1 — Substrate multimethod mechanism

NO migration yet. Just the substrate machinery.

- `Multimethod` + `MultimethodArm` structs
- `MultimethodRegistry` (HashMap<String, Multimethod>)
- `SymbolTable.multimethod_registry: Option<Arc<MultimethodRegistry>>`
- `:wat::core::defmultimethod` substrate form parsing in freeze.rs
- Check-time dispatch at the `infer_list` head-keyword switch:
  if head matches a registered multimethod, route to multimethod
  arm-matching instead of the normal scheme path
- Runtime dispatch at the eval list-call site: same shape
- Arc 144 `Binding::Multimethod` variant + lookup_form 6th branch
- Arc 144 reflection: signature-of / lookup-define / body-of
  per-multimethod behavior
- Test: declare a tiny test multimethod (over two test types) +
  verify check + runtime + reflection end-to-end

~400-700 LOC Rust + ~150 LOC tests. SUBSTANTIAL slice; the
foundation for the migration.

### Slice 2 — Migrate `length` (canonical first migration)

The proof that the mechanism works for a real primitive.

- Mint `:wat::core::Vector/length`, `:wat::core::HashMap/length`,
  `:wat::core::HashSet/length` as TypeSchemes in register_builtins
- Each impl delegates to existing length runtime logic (zero
  behavior change at the runtime level; just naming)
- Declare `:wat::core::length` as multimethod in `wat/core.wat`
  (or wherever core declarations live)
- Retire `infer_length` + dispatch arm at check.rs:3080
- Verify slice 6 length canary (arc 143 slice 6 test) turns GREEN
- Verify all existing call sites still work

~150-250 LOC + tests. Smaller than slice 1; the mechanism does
the heavy lifting.

### Slice 3 — Migrate `empty?` family

Same shape as slice 2. Vector/empty?, HashMap/empty?, HashSet/empty?.

### Slice 4 — Migrate `contains?` family

OPEN QUESTION (per audit Q): is `contains?` ONE multimethod or TWO
distinct names? HashMap/contains-key? takes a KEY; Vector/contains?
+ HashSet/contains? take an ELEMENT. The verbs DIFFER. Decision in
slice 4 brief.

### Slice 5 — Migrate `get` family

Vector/get (i64 → Option<T>), HashMap/get (K → Option<V>). Two
arms. HashSet's get-by-equality is just contains?.

### Slice 6 — Migrate `conj` family

Vector/conj, HashSet/conj. Two arms.

### Slice 7 — Pure rename family (no multimethod needed)

`assoc` → `HashMap/assoc`; `dissoc` → `HashMap/dissoc`;
`keys` → `HashMap/keys`; `values` → `HashMap/values`;
`concat` → `Vector/concat`. Each is a clean Type/verb rename per
arc 109's existing convention. No multimethod involved.

These are HashMap-only or Vector-only operations that were
mistakenly grouped with the polymorphic primitives.

### Slice 8 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry + ZERO-MUTEX cross-ref
(if the multimethod registry uses OnceLock or atomics in any
non-obvious way) + end-of-work-ritual review.

## Open questions

### Q1 — Type-pattern matching

How does the check-time arm-pattern-matching handle parametric
heads (e.g., Vec<T>)? Need to use the existing `unify` machinery —
each arm pattern is itself a TypeExpr; matching = unification with
the arg's inferred type.

For polymorphic patterns (`Vec<T>` where T is the arm's own type
variable), the unification produces a fresh T per arm. The arm's
impl is then instantiated with that T.

Slice 1 brief verifies this works mechanically; if the existing
unify needs extension for arm-pattern matching, surface as a
sub-slice.

### Q2 — Aliases of multimethods

Arc 143's `:wat::runtime::define-alias` aliases callables.
Aliasing a multimethod could mean:
- Alias the NAME — new name points at the same arm table. The new
  name is also a multimethod.
- Alias one ARM — `(define-alias :my-vlen :Vector/length)` aliases
  the per-Type impl directly. Clean rank-1 alias as today.

Slice 1 brief decides: probably both work because the alias
machinery just looks up the target via lookup_form and gets a
Binding; if the target is a multimethod, the alias points at the
multimethod. Test in slice 2.

### Q3 — Multimethod arity must match arm impl arity

Constraint: every arm's impl has the same arity as the multimethod's
surface form. Slice 1 enforces this at `defmultimethod` parse time
(grep each impl's signature; compare arity; error if mismatch).

### Q4 — `contains?` verb consistency

Per audit: HashMap's contains-key? takes K; Vector + HashSet take
T (element). If we make ONE multimethod, the arm patterns are
distinct (HashMap×K vs Vec×T vs HashSet×T) and the verbs in the
impl names differ (contains-key? vs contains?).

Decision deferred to slice 4 brief. Two viable paths:
- ONE multimethod `:contains?`; arm patterns differ; impl names
  differ (HashMap/contains-key? vs Vector/contains? vs HashSet/contains?).
  Caller writes `(:contains? c x)` and dispatch picks correctly.
- TWO names: `:contains?` (Vector + HashSet only) and
  `:contains-key?` (HashMap-only). Caller picks the right surface.

Probably ONE multimethod is the right call — the user writes
`(:contains? thing element-or-key)` regardless. But verify in
slice 4.

### Q5 — Where does `defmultimethod` live?

Substrate provides the form. User wat code (or substrate stdlib)
USES it. The substrate's own multimethod declarations probably
live in a new file `wat/core.wat` (or similar — might need a
`wat/multimethods.wat`). Decision in slice 1 brief.

## Why this is foundation work (not velocity work)

Per COMPACTION-AMNESIA-RECOVERY § 12: arc 109's wind-down friction
IS the foundation auditing itself. Multimethod is the substrate's
honest answer to a class of polymorphism it currently lies about
via lying schemes + handlers. Adding the mechanism + correcting
each primitive compounds into the impeccable foundation the user's
next-leg work requires.

The slow path is the right path. Each migration slice can spawn
independently; sonnet executes mechanically against substrate-
informed briefs; the foundation strengthens with each cycle.

## Cross-references

- `docs/arc/2026/05/144-uniform-reflection-foundation/REALIZATIONS.md`
  — six realizations including the multimethod consensus + the
  discipline lesson + cascade reordering
- `docs/arc/2026/05/144-uniform-reflection-foundation/SCORE-SLICE-3.md`
  — the diagnostic that triggered the cascade
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 10 + § 12 — the
  discipline + strategic context this arc embodies
- `docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`
  + `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  Type/verb + Pattern 2 poison precedents this arc draws from
- `src/check.rs:3036-3082` — the 10 dispatch arms being retired
- `src/check.rs:7243-8122` — the 10 hardcoded handlers being retired
- arc 144 `Binding` enum (src/runtime.rs:6267) — gets a 6th variant
- arc 144 `lookup_form` (src/runtime.rs:6315) — gets a 6th branch

## Status notes

- DESIGN refreshed against multimethod consensus.
- Implementation deferred until arc 144 closes through slice 4
  (verification — which becomes simpler post-slice-2 of this arc).
- Arc 144 slice 3b CANCELLED (per arc 144 REALIZATION 4).
- Arc 109 v1 closure now blocks on arc 144 + arc 130 + arc 145 +
  this arc.
