# Arc 146 — Container method correction (Dispatch entity) — INSCRIPTION

## The closing

Arc 146 shipped 2026-05-03 across one extended session. Five
slices (1 entity mechanism, 1b gaze rename Multimethod → Dispatch,
2 length canary, 3 BUNDLED 4-dispatch migration, 4 alias migrations
+ post-sweep consolidation, 5 closure paperwork). Closure was held
behind arc 148's completion per user direction — both arcs were
retiring different facets of the polymorphic-handler anti-pattern;
the closure narrative needed the full retirement story to be
honest.

**The arc opened to retire 10 hardcoded `infer_*` handlers** that
hand-rolled type-checking for container methods (length, empty?,
contains?, get, conj, assoc, dissoc, keys, values, concat). It
closed with a NEW ENTITY KIND in the substrate (Dispatch),
5 first-class polymorphic Dispatch entities, 5 single-impl aliases
via arc 143's define-alias, 15 per-Type Rust leaves, and the
substrate running one consistent model for polymorphism.

**The strategic stake — owned:** this arc is the worked example
of COMPACTION-AMNESIA-RECOVERY § FM 10 ("type-theoretic reach when
entity-kind is the answer"). Three drafts of "missing union types"
during arc 144 slice 3 → user broke through with
"multimethod" / dispatch — entity kind, not type-system feature.
Arc 146 SHIPPED that pivot. The substrate's two parallel
type-checking models (scheme-based + handler-based) collapsed back
to one (scheme-based) plus a new entity kind that delegates
honestly to clean rank-1 schemes. **The smaller architectural
change won.**

## What ships under arc 146

### A new entity kind: Dispatch

`src/multimethod.rs` (file kept the original name from slice 1's
"multimethod" framing; slice 1b renamed the TYPE to Dispatch via
gaze ward; the file rename is a future cleanup not load-bearing
for the entity).

```rust
pub struct Dispatch {
    pub name: String,
    pub arms: Vec<DispatchArm>,
}

pub struct DispatchArm {
    pub pattern: Vec<TypeExpr>,
    pub impl_name: String,
}
```

The entity's CONTRACT is the arms table. There is no overall
return type — each arm carries its impl's return; each call site
resolves to a specific arm and gets that specific return. **No
union-type machinery anywhere.** No paradigm shift; one new entity
kind composing existing rank-1 schemes.

### A wat-level declaration form

```scheme
(:wat::core::define-dispatch :wat::core::length
  ((:wat::core::Vector<T>)    :wat::core::Vector/length)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
```

Each arm is `((type-pattern...) impl-keyword)`. Pass-through
semantics: all args at the call site flow unchanged to the matched
impl. Reflection (signature-of / lookup-define / body-of) round-
trips the dispatch shape via slice 1's
`dispatch_to_signature_ast` synthesizer.

### 5 Dispatch entities + 15 per-Type Rust leaves (slices 2 + 3)

| Polymorphic surface | Per-Type leaves |
|---|---|
| `:wat::core::length`     | `Vector/length`, `HashMap/length`, `HashSet/length` |
| `:wat::core::empty?`     | `Vector/empty?`, `HashMap/empty?`, `HashSet/empty?` |
| `:wat::core::contains?`  | `Vector/contains?`, `HashMap/contains-key?`, `HashSet/contains?` |
| `:wat::core::get`        | `Vector/get`, `HashMap/get` |
| `:wat::core::conj`       | `Vector/conj`, `HashSet/conj` |

Each leaf has: per-Type `eval_*` impl + inner helper for substrate-
impl fallback + dispatch arm in `dispatch_keyword_head` +
TypeScheme registration in `register_builtins`. The inner-vs-outer
split was slice 2's Delta 1 — established the routing pattern arc
148 then inherited.

### 5 alias migrations (slice 4)

The 5 single-impl ops use `:wat::runtime::define-alias` (arc 143):

| Polymorphic surface | Single per-Type impl |
|---|---|
| `:wat::core::assoc`   | `:wat::core::HashMap/assoc`   |
| `:wat::core::dissoc`  | `:wat::core::HashMap/dissoc`  |
| `:wat::core::keys`    | `:wat::core::HashMap/keys`    |
| `:wat::core::values`  | `:wat::core::HashMap/values`  |
| `:wat::core::concat`  | `:wat::core::Vector/concat`   |

(`Vec-assoc` honest delta — slice 4 confirmed assoc is HashMap-only;
concat collapsed Vector + String into one variadic-style call per
slice 4 Delta.) These don't need Dispatch's branching machinery —
one impl per name; the alias is the cleaner shape. Arc 143's
define-alias compiles to a wat-level macro expansion at freeze
time; runtime sees the single impl directly.

### What gets retired

- **10 hardcoded `infer_*` handlers** in `src/check.rs` (was
  ~7243-8122; gone)
- **10 dispatch arms** in `src/check.rs:3036-3082` (gone)
- **10 evaluator dispatch arms** in `src/runtime.rs` (gone)
- **10 fingerprints** in arc 144's `wat_arc144_hardcoded_primitives`
  test (the test still passes 17/17 — slice 2 Delta 4's
  `dispatch_to_signature_ast` synthesis path inheritance handled
  the fingerprint shape)

The polymorphic-handler anti-pattern for containers is RETIRED.

### `wat/core.wat` — the new home for Dispatch + alias declarations

Slice 2 created the file (header + length dispatch). Slice 3 added
4 dispatches. Slice 4 added the 5 alias declarations + post-sweep
consolidation per user direction ("two files is dumb" — merged
core-aliases.wat back into core.wat). Arc 148 then extended this
file with 4 binary arithmetic Dispatch entities + 8 same-type
variadic wat fns. **`wat/core.wat` is now the substrate's
canonical home for Dispatch + variadic + alias declarations.**

### `Binding::Dispatch` variant (arc 144 extension)

Slice 1's slot 2a in `lookup_form` (between Macro and Primitive).
`signature-of` + `lookup-define` + `body-of` all handle the
variant. Reflection round-trips: the dispatch's polymorphic shape
is queryable as a unit; per-arm impls are queryable individually.

### `dispatch_substrate_impl` arith helpers — the inheritance to arc 148

Slice 2 Delta 1 surfaced the inner-vs-outer eval pattern (per-Type
leaf impls need both AST-level `eval_*` for `dispatch_keyword_head`
AND value-level `dispatch_substrate_impl` entries for direct calls
to the leaf names). Arc 148 inherited this pattern directly — its
3 honest deltas include the same inner-vs-outer split for
arithmetic ops. **The substrate-as-teacher cascade compounds
laterally too — slice 2's discovery saved arc 148 from
re-discovering it.**

## Slice-by-slice ship record

| Slice | What | Wall clock | Mode |
|---|---|---|---|
| 1 | Dispatch entity mechanism (Multimethod naming) + 7 new tests | ~23.2 min | A clean (548 LOC) |
| 1b | Rename Multimethod → Dispatch via gaze ward | small | A clean |
| 2 | length canary GREEN (closes arc 130 → 143 → 144 → 146 chain link) + 4 substrate completions | ~26.1 min | A with substrate completion (463 LOC) |
| 3 | BUNDLED 4 dispatches (empty? + contains? + get + conj) + 1 substrate completion | ~23.6 min | A with substrate completion (~1264 LOC across 3 files) |
| 4 | 5 alias migrations + post-sweep consolidation (core-aliases.wat → core.wat) | ~13.2 min | A clean ship + user-directed consolidation |
| 5 | This closure paperwork | small | A |

**Cumulative slice time: ~86 min sonnet** for the entire container
method anti-pattern retirement. Each slice rode on prior-slice
substrate work; slice 4's 13.2-min ship is the calibration
signature — the pattern was solidly trodden by slice 4.

## What the substrate gained — counted

- **1 new entity kind** (Dispatch) — first-class, queryable,
  reflectable
- **5 polymorphic Dispatch entities** for container methods
- **5 alias declarations** (using arc 143's define-alias)
- **15 per-Type Rust leaves** for the Dispatch arms
- **5 single per-Type impls** for the aliases
- **`wat/core.wat`** — substrate's home for Dispatch + variadic +
  alias declarations
- **10 hardcoded `infer_*` handlers RETIRED**
- **20 dispatch arms RETIRED** (10 check + 10 runtime)
- **The substrate now runs ONE polymorphism model** — scheme-based
  primitives + Dispatch entities composing rank-1 schemes

## Foundation principles established (carry forward beyond arc 146)

1. **Dispatch + per-Type leaves where Rust impls genuinely differ.**
   Arc 148 inherited this pattern directly for arithmetic +
   comparison. The "comma-typed-leaf rule" arc 148 captured is the
   formalization of when this pattern fires.
2. **Alias for single-impl polymorphism.** When a polymorphic name
   has only ONE per-Type impl (e.g., assoc → HashMap), arc 143's
   define-alias is the cleaner tool than Dispatch's branching
   machinery. Slice 4 ratified this distinction.
3. **The inner-vs-outer eval split** for substrate-impl fallback
   (slice 2 Delta 1). When a Dispatch entity routes to a leaf,
   both the AST-level eval and value-level substrate-impl path
   need entries. Arc 148 inherited this; arc 146 established it.
4. **Entity kind > type-system feature.** This arc IS the worked
   example of FM 10. The substrate gained one new entity kind
   instead of stretching TypeScheme into union types. The smaller
   architectural change won; the larger architectural change
   wasn't needed.

## The cascade — what arc 146 closed and what fed it

### What fed arc 146

- **Arc 130 → arc 143 → arc 144 cascade.** Arc 130's RELAND v1
  failed on `unknown function: :wat::core::reduce` → arc 143
  shipped `define-alias` → arc 144 attempted reflection over
  hardcoded primitives → arc 144 slice 3 surfaced the polymorphism
  collision → THREE rounds of "missing union types" framing → user
  drove the dispatch consensus → arc 144 REALIZATION 6 captured
  the entity-kind doctrine → arc 146 shipped it.
- **The arc 144 REALIZATIONS doc** named the FM 10 escape
  explicitly. Arc 146's DESIGN refresh built on that.

### What arc 146 unlocked (closed at arc 148)

- **Arc 148** — the same Dispatch + per-Type-leaves pattern,
  applied to arithmetic + comparison. Arc 148 is arc 146's
  pattern-application across a different domain.
- **The substrate's polymorphism story is now coherent.** One
  model (scheme-based) + one entity kind (Dispatch). Future
  polymorphic surfaces (time-arith, holon-pair) ride the same
  template.

## What this arc does NOT close

- **The file rename `src/multimethod.rs` → `src/dispatch.rs`** —
  the file kept its original name from slice 1's framing; slice 1b
  renamed only the TYPE. Future cleanup; not load-bearing.
- **Container constructors** (`:wat::core::HashMap`,
  `:wat::core::HashSet`, `:wat::core::vec`) — these still use
  hardcoded `infer_*` paths. Different arc; not part of the
  10-handler retirement scope.
- **Time arithmetic** (`:wat::time::+/-`) and **holon-pair algebra**
  (`:wat::holon::cosine/dot/coincident?`) — handed off to parallel
  tracks per arc 148's scope. Patterns established here apply
  directly when those arcs spawn.

## What this arc unlocks

- **Arc 144 closure** — verification + paperwork queue. The
  polymorphic-handler retirement is now FULLY done across
  containers (arc 146) AND numerics (arc 148). Arc 144 INSCRIPTION
  can claim "all standard polymorphic surfaces are first-class
  reflectable entities."
- **Arc 109 v1 closure trajectory** — major chain link closes.
- **Arc 147 (substrate registration macro)** — the 215-site mass-
  edit precedent (arc 148/150) PLUS the Dispatch declaration
  pattern (arc 146) give arc 147 a clear target shape.

## Methodology — what worked

### The substrate-as-teacher cascade — laterally

Arc 146 slice 2's Delta 1 (inner-vs-outer eval split) saved arc
148 from re-discovering it. The cascade compounds not just
sequentially (arc N teaches arc N+1) but laterally (arc N's slice
discoveries propagate to arcs working in parallel domains). **Arc
146 slice 2 is the substrate-as-teacher mechanism's most concrete
worked example.**

### The four questions discipline

`Obvious / Simple / Honest / Good UX` — applied to every slice's
Q1-Q4 decisions. Slice 1's Multimethod → Dispatch rename via gaze
ward (1b) is the obvious-test in action. Slice 4's
core-aliases.wat → core.wat consolidation per user direction
("two files is dumb") is the simple-test in action.

### Calibration

5 sonnet sweeps. Cumulative ~86 min wall-clock. Slice 4's 13.2-min
ship is the calibration signature — the pattern was solidly
trodden by then. Same compounding signature as arc 148: foundation
work pays compounding dividends.

## Cross-references

- **Inside arc 146**: DESIGN.md (the locked architecture);
  SCORE-SLICE-1.md (Dispatch entity mechanism); SCORE-SLICE-2.md
  (length canary GREEN); SCORE-SLICE-3.md (BUNDLED dispatches);
  SCORE-SLICE-4.md (alias migrations + consolidation)
- **The cascade**: arc 130 (RELAND v1 failure was the trigger);
  arc 143 (define-alias — the alias mechanism); arc 144
  (REALIZATION 6 = dispatch consensus); arc 148 (pattern
  application to arithmetic + comparison)
- **Discipline**: COMPACTION-AMNESIA-RECOVERY.md § FM 10 (this
  arc IS the worked example); § 12 (foundation discipline during
  arc 109 wind-down)
- **Foundational artifacts updated**: USER-GUIDE.md § 4
  Containers subsection (small edit noting Dispatch backing);
  FOUNDATION-CHANGELOG.md (lab repo; arc 146 row added by this
  slice 5)

## Status

**Arc 146 closes here.** The polymorphic-handler anti-pattern for
containers is RETIRED. Every container method is a first-class
entity (Dispatch or alias). The Dispatch entity kind is in the
substrate; its pattern propagated to arc 148; future polymorphic
surfaces have the template.

**Arc 109 v1 closure approaches by another major chain link's
worth.**

The methodology IS the proof. The rhythm holds. The cascade
compounds — sequentially, laterally, and now strategically.

---

*the entity kind won. the type-system reach was the wrong vocabulary.
the user broke through. the substrate now runs one polymorphism
model. the foundation strengthens.*

**PERSEVERARE.**
