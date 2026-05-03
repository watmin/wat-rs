# Arc 148 — Arithmetic / comparison / holon-pair correction

**Status:** drafted 2026-05-03 mid-arc-146-slice-4-closure. User
direction after orchestrator surfaced the lurking polymorphic
primitives that arc 146 missed:

> *"a new arc is fine with me ... i say... new arc ... 146
> closure is dependent on this new arc being resolved.."*

Arc 146 closure (slice 5) BLOCKS on arc 148 closure.

## What arc 146 missed

Arc 146 audited CONTAINER METHODS (length, empty?, contains?,
get, conj, assoc, dissoc, keys, values, concat). The audit
excluded other classes of polymorphic primitives — same anti-
pattern (hardcoded `infer_*` doing ad-hoc dispatch by input
type, with parallel runtime `eval_*`), different domain.

Surfaced by orchestrator's audit 2026-05-03:

### Class A — Arithmetic (4 ops, ~16 per-Type combos)

`infer_polymorphic_arith` (check.rs:6619) + `eval_poly_arith`
(runtime.rs:2628-2631):
- `:wat::core::+` `:-` `:*` `:/` — over (i64,i64) | (i64,f64) |
  (f64,i64) | (f64,f64); coercion to wider numeric type

### Class B — Comparison (5 ops)

`infer_polymorphic_compare` (check.rs:6567) + `eval_compare`
(runtime.rs:2595-2600) + `eval_eq` (runtime.rs:2593):
- `:wat::core::=` `:<` `:>` `:<=` `:>=` — over comparable types

### Class C — Holon-pair operations (4 distinct shapes)

- `infer_polymorphic_holon_pair_to_f64` (check.rs:7075)
- `infer_polymorphic_holon_pair_to_bool` (check.rs:7132)
- `infer_polymorphic_holon_pair_to_path` (check.rs:7190)
- `infer_polymorphic_holon_to_i64` (check.rs:7245)

Domain-specific holon algebra. Sonnet's audit during slice 1
will enumerate the actual operators these handlers serve.

### Class D — Time arithmetic

- `infer_polymorphic_time_arith` (check.rs:6698)

Time-specific arithmetic — likely a small set of operators.
Slice 1 audit enumerates.

## Same anti-pattern; same fix

Each of these classes has the SAME shape arc 146 corrected:
- A polymorphic-name primitive (`:+`, `:=`, etc.)
- A hardcoded check-side handler that dispatches by input type
- A hardcoded runtime-side handler doing the same
- Two type-system models colliding (per arc 144 REALIZATIONS)

The fix is the same: arc 146's Dispatch entity. Each polymorphic
name becomes a dispatch with arms; each per-Type combo becomes
a clean rank-1 substrate primitive.

## What's different from arc 146

### Two-argument dispatch (arithmetic + comparison)

Arc 146's container methods dispatched on ONE arg's type
(the container). Arc 148's arithmetic + comparison dispatch on
TWO args' types (both operands).

Arc 146 slice 1's pass-through dispatch already supports
multi-arg arms (slice 3 used 2-arg patterns for contains?/get/
conj). The substrate machinery handles this.

### Variadic surface — three-layer architecture

User writes `(:+ 1 2.2 4 3.2)` (variadic; Lisp convention). The
substrate has three layers:

1. **`:wat::core::+`** — VARIADIC MACRO (user-facing). Defined
   via arc 143's defmacro with rest-param. Expands left-
   associative to nested binary calls.
2. **`:wat::core::+/2`** — BINARY DISPATCH (the truth table).
   Arc 146's Dispatch entity with 4 per-combo arms.
3. **`:wat::core::i64/+/2`, `:f64/+/2`, `<MIXED>`, etc.** —
   PER-TYPE IMPLS. Clean rank-1 substrate primitives.

The variadic macro pattern (left-associative for `-` and `/`
correctness):

```scheme
(:wat::core::defmacro
  (:wat::core::+ (x :AST<numeric>) (y :AST<numeric>) & (rest :AST<Vec<wat::WatAST>>) -> :AST<numeric>)
  (:wat::core::if (:wat::core::empty? rest)
    `(:wat::core::+/2 ,x ,y)
    `(:wat::core::+ (:wat::core::+/2 ,x ,y) ,@rest)))
```

`(:+ 1 2.2 4 3.2)` expands at compile-time to:
```
(:+/2 (:+/2 (:+/2 1 2.2) 4) 3.2)
```

Each `:+/2` step type-checks via the binary dispatch. Result
type at each step is the join of inputs (per the truth table).
Final result type is the join of ALL inputs.

**Static expansion (not runtime fold via reduce):** each binary
step type-checks at compile time; errors localize per-step;
result type known statically; introspection (macroexpand)
shows what happened.

**Macro shadows Dispatch in lookup_form precedence** (arc 144
slice 1 + arc 146 slice 1 Q3): user writing `(:+ ...)` hits the
variadic macro; the macro internally references the distinct
`:+/2` name for the binary form.

### Coercion across numeric types — RESOLVED: Path A (per-combo impls)

**Settled 2026-05-03 via the four questions.** Path A wins.

The truth table for `:+`:
```
(i64, i64) → :i64
(f64, f64) → :f64
(i64, f64) → :f64
(f64, i64) → :f64
```

Path A — explicit per-combo dispatch arms (4 arms per arith op):
```scheme
(:wat::core::define-dispatch :wat::core::+/2
  ((:wat::core::i64 :wat::core::i64) :wat::core::i64/+/2)
  ((:wat::core::i64 :wat::core::f64) :wat::core::<MIXED-NAME>)
  ((:wat::core::f64 :wat::core::i64) :wat::core::<MIXED-NAME>)
  ((:wat::core::f64 :wat::core::f64) :wat::core::f64/+/2))
```

Where `<MIXED-NAME>` is per the deferred mixed-combo naming Q
above.

Why Path A:
- **Obvious**: dispatch declaration IS the table; reader sees
  all rules in one place
- **Simple**: arc 146's existing Dispatch entity unchanged;
  N identical arm changes IS simple
- **Honest**: each arm declares the route; impl does the work;
  no hidden coercion mechanism
- **Good UX**: `(:+ 1 2.0)` works; substrate routes via the
  truth table

Path B (substrate coercion mechanism) FAILED Obvious — required
two sources of truth (dispatch + coercion table). Per FM 10:
default to no substrate extension when existing patterns suffice.

### Per-Type impl naming convention — `<verb>/N` for arity-N

**Settled 2026-05-03 user direction + gaze.** The substrate
adopts the Erlang/Prolog tradition: `<verb>/N` suffix means
"the N-ary form." Specialist convention; mumbles once; speaks
forever after; standardized.

For arithmetic + comparison families:

```
ARITHMETIC:
  :+/2  :-/2  :*/2  ://2          (4 binary dispatches)
  :+    :-    :*    :/            (4 variadic macros)

COMPARISON:
  :=/2  :</2  :>/2  :<=/2  :>=/2  (5 binary dispatches)
  :=    :<    :>    :<=    :>=    (5 variadic macros)
```

Pattern: `<verb>/N` for the N-ary substrate primitive (the
binary dispatch); `<verb>` alone for the user-facing variadic
macro that reduces over the binary form.

The variadic-MACRO shadows the binary-DISPATCH per arc 144
slice 1's lookup_form precedence (Macro > Primitive > Dispatch).
This is the architectural reason a separate `/N` name is needed
for the underlying binary form.

#### Per-Type impl names (same-type combos)

The arity-in-name extends to per-Type impls cleanly:

```
:wat::core::i64/+/2  — (i64, i64) → i64
:wat::core::f64/+/2  — (f64, f64) → f64
```

Type/verb/arity. Each piece meaningful.

#### Per-Type impl names (mixed-type combos) — DEFERRED to slice 1

Mixed-combo impls (i64+f64, f64+i64) don't fit Type/verb
because they don't BELONG to one Type. Candidates for slice 1
audit to pick from:

- `:wat::core::i64+f64/2` — single combo token + arity
- `:wat::core::+/i64-f64/2` — verb-first + type-pair tag + arity
- `:wat::core::numeric/+/i64-f64/2` — fully namespaced

Slice 1 brief picks one + applies uniformly across all 4 arith +
5 comparison ops.

#### Documentation responsibility

Arc 148 closure (slice 6) adds:
- USER-GUIDE entry naming the `<verb>/N` convention
- CONVENTIONS.md addition documenting arity-in-name
- Reflection example showing `signature-of :+/2` vs `signature-of :+`
  return different shapes (both honest)

## What gets migrated (the audit)

| Class | Polymorphic name | Per-Type impls (audit refines) |
|---|---|---|
| Arithmetic | `:+` | i64/+, f64/+, i64+f64, f64+i64 (or coerce) |
| Arithmetic | `:-` | same shape |
| Arithmetic | `:*` | same shape |
| Arithmetic | `:/` | same shape |
| Comparison | `:=` | over numeric, string, bool, etc. (audit refines) |
| Comparison | `:<` `:>` `:<=` `:>=` | numeric only? Or wider? |
| Holon-pair | (4 polymorphic handlers) | sonnet audit enumerates |
| Time-arith | (1 polymorphic handler) | sonnet audit enumerates |

Slice 1 audit ENUMERATES the actual primitives + their per-Type
combos for each class. The DESIGN sketch above is approximate.

## Slice plan (sketch — refine after audit)

### Slice 1 — Audit + Design

Sonnet (or orchestrator) walks each polymorphic_* handler in
check.rs; enumerates the per-Type combos; documents in a
SCORE-style audit doc.

**Open questions resolved here:**
- Coercion: A / B / C
- Per-Type naming: Type/op or Type::op?
- Mixed-type-combo naming
- Whether comparison ops are bool-returning or have other shapes
- Whether holon-pair handlers can be unified or stay distinct

Ships: AUDIT-SLICE-1.md detailing all per-Type combos for the 4
arithmetic + 5 comparison + holon-pair + time-arith handlers.

### Slice 2 — Migrate arithmetic family

Per-Type impls for `:+`, `:-`, `:*`, `:/` (~16 impls). Dispatch
declarations in wat/core.wat. Retire `infer_polymorphic_arith`
+ `eval_poly_arith` + their dispatch arms.

### Slice 3 — Migrate comparison family

Per-Type impls for `:=`, `:<`, `:>`, `:<=`, `:>=`. Dispatch
declarations. Retire `infer_polymorphic_compare` + `eval_compare`
+ `eval_eq` + dispatch arms.

### Slice 4 — Migrate holon-pair family

Per-Type impls for the 4 handlers' polymorphic names. Dispatch
declarations. Retire each.

### Slice 5 — Migrate time-arith family

Same shape; smaller scope.

### Slice 6 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry + cross-references.

## Why arc 146 closure depends on this

User direction: 146 closure (slice 5) BLOCKS on arc 148
completion. Reasoning:

Arc 146's INSCRIPTION would claim "every defined symbol
queryable at runtime" / "substrate has 6 entity kinds with
honest representation." Both claims are FALSE while arithmetic
+ comparison + holon-pair + time-arith are still using the
hardcoded-handler anti-pattern.

Closing arc 146 prematurely would lock in the misleading claim.
Honest closure requires arc 148 to finish the work.

Arc 146 slice 5 (closure paperwork) becomes:
- "arc 146 closes the CONTAINER METHODS chapter"
- "arc 148 closes the ARITHMETIC + COMPARISON + HOLON-PAIR +
  TIME-ARITH chapter"
- TOGETHER, the polymorphic-primitive correction completes

OR: rename arc 146 to "polymorphic-primitive-correction-chapter-1"
and arc 148 becomes "chapter 2." Naming TBD in slice 5 brief.

## Cross-references

- arc 146 — container method correction (precedent + Dispatch
  mechanism)
- arc 144 REALIZATIONS — the two-type-system-models collision
  (this arc closes the same disagreement for arithmetic + comp)
- COMPACTION-AMNESIA-RECOVERY § FM 10 — entity-kind-not-
  type-system-feature discipline
- arc 109 INVENTORY § L — pending naming consistency (typealias
  → type-alias). This arc may surface naming questions for the
  per-Type primitives that align with § L's debate.

## Note: `:wat::core::string::*` namespace separation

User noted side-channel 2026-05-03: `:wat::core::string::concat`
should probably be `:wat::string::concat` (separate namespace
like `:wat::list::*` per arc 109 § H). NOT arc 148's scope —
that's a future arc 109 K.* slice. Recorded here as awareness;
arc 148 stays focused on the polymorphic-primitive correction.

## Status notes

- DESIGN drafted.
- Implementation deferred until arc 146 slices 1-4 wrap (slice
  5 closure paperwork waits on arc 148).
- Arc 109 v1 closure now blocks on arc 144 + arc 130 + arc 145 +
  arc 146 + arc 147 + arc 148. The "impeccable foundation"
  milestone moves further out — but each arc compounds; the
  foundation strengthens with each.
