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

### Coercion across numeric types

`(:+ 2 40.0)` should return `42.0` (i64 coerced to f64).
Today's `eval_poly_arith` does this internally. The dispatch-
based replacement needs to either:

- **Option A — coerce in the impl:** dispatch arm `(:i64, :f64)
  → :wat::core::i64+f64` whose impl converts the i64 to f64
  internally + calls f64+f64. ~12 per-Type combo impls.
- **Option B — coerce in the dispatch:** dispatch declares
  arms with optional coercion: `(:i64, :f64) → coerce-to-f64
  → :wat::core::f64/+`. Requires extending arc 146's dispatch
  with coercion arms. Substrate change.
- **Option C — no automatic coercion:** user writes
  `(:f64/+ (:to-f64 x) y)` for mixed types. Cleanest substrate;
  worst UX.

Slice 1 brief decides + the audit informs.

### Per-Type impl naming convention

Arc 146 used `:wat::core::Vector/length` (PascalCase Type +
slash + verb). For arithmetic over PRIMITIVE types (i64, f64),
options:

- `:wat::core::i64/+` (Type/verb shape; matches arc 146)
- `:wat::core::i64::+` (`::` namespace shape)
- Other?

Recommend: `:wat::core::<type>/<op>` — matches arc 146's
convention. Type names are all-lowercase for primitives (i64,
f64) per arc 109's FQDN sweep. Slash separates Type from verb.

For mixed-type combos (i64+f64): `:wat::core::i64+f64` (single
name encoding the combo) OR `:wat::core::f64/+_i64_lhs` (Type/
verb with hint of the mixed shape). Slice 1 brief decides.

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
