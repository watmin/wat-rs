# Arc 148 — Arithmetic / comparison correction

**Status:** drafted 2026-05-03 mid-arc-146-slice-4-closure;
architecture locked 2026-05-03 (this session) after multi-turn
debate that walked through three rejected naming schemes
(slash-stacking, uniform-comma, hybrid) before converging on
the user's proposal: **Type-as-namespace for same-type +
verb-comma-pair for mixed-type + variadic wat function reducing
over binary Dispatch**.

User direction after orchestrator surfaced the lurking polymorphic
primitives that arc 146 missed:

> *"a new arc is fine with me ... i say... new arc ... 146
> closure is dependent on this new arc being resolved.."*

Arc 146 closure (slice 5) BLOCKS on arc 148 closure.

User direction 2026-05-03 (this session):

> *"get the number ops and comparators figured out now - we'll
> deal with string, bool and time while sonnet makes numbers work"*

Arc 148 immediate scope = NUMERIC arithmetic + NUMERIC comparison.
Non-numeric eq/ord (String, bool, Holon, etc.) and time-arith and
holon-pair = parallel track in user's hands while sonnet works
numerics. Slice 1 audit ENUMERATES all 7 polymorphic_* handler
surfaces for the record, but implementation slices (2-3) ship
numeric-only.

## What arc 146 missed

Arc 146 audited CONTAINER METHODS (length, empty?, contains?,
get, conj, assoc, dissoc, keys, values, concat). The audit
excluded other classes of polymorphic primitives — same anti-
pattern (hardcoded `infer_*` doing ad-hoc dispatch by input
type, with parallel runtime `eval_*`), different domain.

Surfaced by orchestrator's audit 2026-05-03 across `src/check.rs`:

| Class | Handler | Location | Scope in arc 148 |
|---|---|---|---|
| Arithmetic | `infer_polymorphic_arith` | check.rs:6619 | IMMEDIATE (slice 2) |
| Comparison | `infer_polymorphic_compare` | check.rs:6567 | IMMEDIATE numeric (slice 3); non-numeric deferred |
| Time-arith | `infer_polymorphic_time_arith` | check.rs:6698 | DEFERRED — user track |
| Holon-pair → f64 | `infer_polymorphic_holon_pair_to_f64` | check.rs:7075 | DEFERRED |
| Holon-pair → bool | `infer_polymorphic_holon_pair_to_bool` | check.rs:7132 | DEFERRED |
| Holon-pair → path | `infer_polymorphic_holon_pair_to_path` | check.rs:7190 | DEFERRED |
| Holon → i64 | `infer_polymorphic_holon_to_i64` | check.rs:7245 | DEFERRED |

Runtime side: `eval_eq` (runtime.rs:4424), `eval_compare`
(runtime.rs:4603), `eval_poly_arith` (runtime.rs:4677); 9
user-facing op arms at runtime.rs:2593-2631.

Same anti-pattern arc 146 corrected: a polymorphic-name primitive
with hardcoded check-side handler + hardcoded runtime-side handler.
Two type-system models colliding (per arc 144 REALIZATIONS).

## Architecture (LOCKED)

### Architecture differs between arithmetic and comparison

**Arithmetic** needs per-Type Rust leaves because the underlying Rust
impls differ per type (integer addition vs float addition are
different functions). Three layers.

**Comparison** does NOT need per-Type Rust leaves because Rust's
`PartialEq`/`PartialOrd` traits provide one polymorphic impl that
works on any same-type pair. ONE substrate primitive per op +
selective mixed-type arms.

User direction 2026-05-03 (this session) — the simplifying rule:

> *"we have known overrides for mixed types.. same types of anything
> we just delegate to that type's func.... we selectively choose to
> support convenience for mixed values and we raise for those we
> don't"*

This rule applies to BOTH numeric and non-numeric comparison —
arc 148's comparison family AND Category A (non-numeric eq/ord)
collapse into the same architecture: substrate primitive + chosen
mixed-type arms.

### Arithmetic — three layers

1. **Variadic wat function (top-level user surface)** — `:wat::core::+`,
   `:wat::core::-`, `:wat::core::*`, `:wat::core::/`. Variadic; min 2
   args; reduces over the binary Dispatch entity.

2. **Binary Dispatch entity** — arc 146 Dispatch entity at
   `:wat::core::<verb>,2` (sibling name to the variadic). Arms per
   type-pair → routes to per-Type leaf.

3. **Per-Type Rust primitives (the leaves)** — actual binary ops:
   - **Same-type:** `:wat::core::<Type>::<verb>` is the variadic wat
     wrapper for that Type; binary Rust leaf lives at
     `:wat::core::<Type>::<verb>,2`.
   - **Mixed-type:** `:wat::core::<verb>,<type1>-<type2>` (verb +
     comma + hyphenated operand pair). Always binary; no variadic
     possible (variadic over a fixed type-pair has no coherent
     semantics).

### Comparison — substrate primitive + selective mixed arms

Each comparison op (`:=`, `:not=`, `:<`, `:>`, `:<=`, `:>=`) is a
SINGLE substrate primitive that:

- **Same-type:** delegates to the type's existing comparator via
  Rust's `PartialEq`/`PartialOrd` on the underlying Value. Works
  universally for `=`/`not=` (any type with `PartialEq` — basically
  everything). For ord (`<`/`>`/`<=`/`>=`) the substrate maintains an
  opinionated allowlist of types with meaningful order (numeric,
  String, time, Bytes, Vector\<T\>, Tuple\<T...\>, Option\<T\>,
  Result\<T,E\>); types not on the allowlist raise at compile time.

- **Mixed-type:** routes to an EXPLICIT named arm if one exists;
  raises at compile time otherwise. Arc 148 ships only numeric mixed
  arms ((i64, f64) and (f64, i64) for all 6 ops). Other mixed pairs
  (`(:= "1" 1)`, `(:< :keyword 5)`) are type errors, not silently-
  false coercions.

No per-Type Rust leaves for comparison. Same-type comparison goes
through the polymorphic substrate primitive (Rust's trait dispatch);
mixed-type comparison goes through the substrate primitive's named
arm dispatch.

### Variadic semantics (arithmetic only)

`(:wat::core::+ 0 40.0 2)` walks left-to-right:
- Step 1: pair `(0:i64, 40.0:f64)` → dispatch finds (i64, f64) →
  calls `:wat::core::+,i64-f64`(0, 40.0) → 40.0:f64
- Step 2: pair `(40.0:f64, 2:i64)` → dispatch finds (f64, i64) →
  calls `:wat::core::+,f64-i64`(40.0, 2) → 42.0:f64
- Result: **`:f64 42.0`**

Same-type variadic `(:wat::core::i64::+ 1 2 3 4 5)` skips dispatch
because type is fixed by the call signature: walks the list reducing
directly via `:wat::core::i64::+,2` (the i64 binary leaf). Compile-
time type error if any arg is non-i64.

### Comparison semantics

Strict binary. `(:wat::core::< 1 2 3)` is rejected (arity mismatch).
Chained comparison written explicitly via `:and`:
`(:wat::core::and (:wat::core::< 1 2) (:wat::core::< 2 3))`. Reasoning:
the only meaningful variadic comparison semantics is Python-style
pairwise-AND, which would introduce a SECOND "what does variadic mean"
rule alongside arithmetic's fold. Two semantics for variadic fails
the four questions on Simple. Strict binary keeps one rule per family.

### Arity rules — Lisp/Clojure tradition for arithmetic; strict 2 for comparison

User direction 2026-05-03 (mid-session reversal): mirror Clojure's
identity-element rule for arithmetic.

**Arithmetic arity rules:**

| Op | 0-ary | 1-ary | 2+-ary |
|---|---|---|---|
| `:+` | returns `:i64 0` (identity) | returns the arg unchanged | reduces via dispatch |
| `:*` | returns `:i64 1` (identity) | returns the arg unchanged | reduces via dispatch |
| `:-` | ARITY ERROR | `(:- x)` → `(:- 0 x)` = `-x` (negation) | reduces via dispatch |
| `:/` | ARITY ERROR | `(:/ x)` → `(:/ 1 x)` = `1/x` (reciprocal, type-preserving) | reduces via dispatch |

The 1-ary identity is inserted on the LEFT (the only ordering that
gives useful semantics for `-` / `/`).

**Type preservation for 1-ary** — implicit identity takes the SAME
TYPE as the argument:
- `(:- 5)` → `-5:i64` (insert `0:i64`)
- `(:- 5.0)` → `-5.0:f64` (insert `0:f64`)
- `(:/ 5)` → `0:i64` (insert `1:i64`; integer truncation — 1/5 in i64 = 0)
- `(:/ 5.0)` → `0.2:f64` (insert `1.0:f64`; float reciprocal)

The integer reciprocal returning 0 is honest about i64 semantics.
Users wanting f64 reciprocal write `(:wat::core::/ 1.0 5)` explicitly,
or call `:wat::core::f64::/` if Ratio support lands later.

**Comparison arity rule: strict 2.** Comparison has no identity
element; chained comparison is meaningless under fold semantics
(`(:< 1 2 3)` would fold to `(:< (:< 1 2) 3) = (:< true 3)` —
nonsense). Pairwise-AND would introduce a SECOND "what does variadic
mean" rule alongside arithmetic's fold (fails Simple). Comparison
ops require exactly 2 args; chains via `:and`:
`(:wat::core::and (:< 1 2) (:< 2 3))`.

### Why this architecture wins the four questions

- **Obvious?** YES — `(:+ 1 2.0 3)` does what users mean; per-Type
  leaves callable directly for explicit-Type usage.
- **Simple?** YES — three layers each doing ONE thing; uniform rule
  per layer (top = variadic; middle = binary dispatch; leaves =
  Rust). Per-Type variadic at the Type-namespace gives users
  type-locked variadic when wanted.
- **Honest?** YES — composition (variadic fold) lives in wat where it
  belongs; metal work (binary ops) lives in Rust where it belongs.
  Names speak: `,2` = binary form; `,<pair>` = mixed-type leaf;
  `<Type>::<verb>` = Type's verb.
- **Good UX?** YES — variadic Lisp tradition; type-locked variadic
  available; per-Type leaves reachable per arc 109 no-privacy.

## Naming convention (LOCKED)

Three shapes; each uses one rule.

### Same-type per-Type — Type as namespace segment

```
:wat::core::<Type>::<verb>      ; arithmetic: variadic wat function over Type
:wat::core::<Type>::<verb>,2    ; arithmetic: binary Rust leaf
:wat::core::<Type>::<verb>      ; comparison: binary Rust leaf (no variadic)
```

Type sits in the namespace path. `::` is the standard namespace
separator. Mirrors the existing `:wat::core::i64::+` convention
already in use across `resolve.rs`, `freeze.rs`, `macros.rs`,
`string_ops.rs` per CONVENTIONS.md line 23.

### Mixed-type per-Type — verb + comma + hyphenated pair

```
:wat::core::<verb>,<type1>-<type2>
```

Comma separates the verb from the operand-pair tag. Hyphen joins
the two types in the pair. Always binary. Both orderings get
distinct names (`+,i64-f64` vs `+,f64-i64`); not commutative-
collapsed because subtraction needs operand order preserved and
we want one shape uniform across all ops.

The comma is a NEW structural separator in wat keyword grammar
(alongside `/` for `<Type>/<method>` precedent and `::` for
namespaces). Lexer accepts commas inside keyword bodies per
`src/lexer.rs:335` ("Every other character (including `<`, `>`,
`/`, `-`, `,`, `!`, `?`) is pushed as-is"). Verified empirically
2026-05-03: comma-bearing keywords lex/parse/register/check/execute.

### Binary form of variadic (arithmetic only)

```
:wat::core::<verb>,2
```

Comma + arity digit `2`. Distinguishes the binary Dispatch entity
(arc 146 Dispatch; one fixed surface arity) from the variadic wat
function at the bare verb name. Sibling name pattern; needed only
where the variadic wrapper exists (arithmetic). Comparison uses
the bare name for the Dispatch directly (no wrapper).

The arity digit is COMMA-separated, not slash-separated. Erlang's
`/N` was rejected during this session because it conflicts with
the namespace-suffix `/method` precedent (`HashMap/get`); the
slash-stacking that resulted (`:i64///2`, `:////i64-f64/2`) failed
the gaze ward on Lies (templates structurally diverged between
same-type and mixed-type cases).

### Why these two separators (not one)

The substrate now has TWO name-internal structural separators:
- `/` — `<Type>/<method>` (existing per `HashMap/get`, etc.)
- `,` — verb/operand-pair seam AND verb/arity-digit seam (NEW)

Single-separator alternatives were considered and rejected (gaze
2026-05-03): unifying everything to slash creates the slash-
stacking visual collision for division; unifying everything to
comma breaks the existing `Type/method` precedent. The two-
separator decision is the gaze-converged answer; documentation
must cover this clearly per CONVENTIONS.md update at closure.

## Full enumeration — NUMERIC arc 148 surface

### Arithmetic family (4 ops × 8 entities = 32 names)

For each op `<v>` ∈ {`+`, `-`, `*`, `/`}:

```
:wat::core::<v>                  variadic wat function (folds via :<v>,2)
:wat::core::<v>,2                binary Dispatch entity (arms per type-pair)
:wat::core::i64::<v>             variadic wat function over i64-only (folds via :i64::<v>,2)
:wat::core::i64::<v>,2           binary Rust primitive — (i64, i64) → i64
:wat::core::f64::<v>             variadic wat function over f64-only (folds via :f64::<v>,2)
:wat::core::f64::<v>,2           binary Rust primitive — (f64, f64) → f64
:wat::core::<v>,i64-f64          binary Rust primitive — (i64, f64) → f64
:wat::core::<v>,f64-i64          binary Rust primitive — (f64, i64) → f64
```

### Comparison family (6 ops × 3 entities = 18 names)

For each op `<v>` ∈ {`=`, `not=`, `<`, `>`, `<=`, `>=`}:

```
:wat::core::<v>                  substrate primitive (universal same-type via PartialEq/PartialOrd; mixed via named arm)
:wat::core::<v>,i64-f64          mixed-type leaf — (i64, f64) → bool
:wat::core::<v>,f64-i64          mixed-type leaf — (f64, i64) → bool
```

NO per-Type same-type leaves (`:i64::<` etc. omitted) — the substrate
primitive's universal same-type delegation handles them via Rust's
trait dispatch; carrying separate names would be redundant.

The 6th op `:not=` was missed in earlier enumeration; per-handler
audit at check.rs:3287-3293 confirms the dispatch site lists all six.

**Total: 32 + 18 = 50 names for the immediate arc 148 scope.**

### What "same-type universal delegation" actually serves

Because the comparison primitive uses Rust's `PartialEq` / `PartialOrd`
traits, the SAME 18 entities cover same-type comparison on EVERY
type the substrate's Value enum supports. This means **Category A
of the previously-deferred surface (non-numeric eq/ord) is solved
by arc 148 itself, not deferred to a parallel track**:

| Same-type pair | `:=` `:not=` | `:<` `:>` `:<=` `:>=` |
|---|---|---|
| `:i64`, `:f64` | yes | yes |
| `:String` | yes | yes (lexicographic) |
| `:wat::time::Instant`, `:Duration` | yes | yes (chronological) |
| `:wat::core::Bytes` | yes | yes (byte-wise) |
| `:wat::core::Vector<T>` | yes | yes if T has ord (parametric) |
| `:wat::core::Tuple<T...>` | yes | yes if all elements have ord |
| `:wat::core::Option<T>` | yes | yes if T has ord |
| `:wat::core::Result<T,E>` | yes | yes if both have ord |
| `:bool` | yes | **NO** — meaningless (false < true is technically true but useless) |
| `:wat::core::keyword` | yes | **NO** — no compelling case |
| `:wat::core::HashMap` `:HashSet` | yes | **NO** — no canonical order |
| `:wat::holon::HolonAST` | yes | **NO** — algebraic surface; no canonical order |
| `:wat::core::unit` | yes | **NO** — only one value; meaningless |
| user-defined enums/structs | yes | **NO by default** — unless user opts in (future feature) |

Equality (`:=`, `:not=`) is universal across ALL same-type pairs.
Ord is selective: the substrate's compare primitive's check-time
logic enforces the allowlist; ord on a non-allowlisted type raises
a compile-time `TypeMismatch` diagnostic naming the offending type.

### Mixed-type — what arc 148 chooses to be convenient about

| Mixed pair | Supported? | Reason |
|---|---|---|
| `(:i64, :f64)`, `(:f64, :i64)` | yes — explicit named arms | Numeric promotion is conventional and useful |
| Anything else | no — raise compile error | Honest: `(:= "1" 1)` is a type confusion, not false |

The substrate primitive's mixed-type dispatch consults a registry
of named mixed arms; misses raise. Future mixed conveniences (e.g.,
String/Bytes equality if compelling) can be added by registering new
arms — additive, no architectural change.

### What's NOT in arc 148's immediate scope

Per user direction 2026-05-03 + the simplifying rule: only TWO
categories remain genuinely deferred (down from three):

- **Category B — time arithmetic** (`:wat::time::+`, `:wat::time::-`)
  — handler `infer_polymorphic_time_arith` (check.rs:6698). Two ops
  with three signatures: `(Instant, Duration) → Instant`,
  `(Instant, Instant) → Duration` (only `-`), `(Instant, Duration)
  → Instant` (only `+`). Small, contained.

- **Category C — holon-pair algebra** (5 ops across 4 handlers):
  `:wat::holon::cosine`, `:wat::holon::dot`, `:wat::holon::coincident?`,
  `:wat::holon::coincident-explain`, `:wat::holon::simhash`. All
  consume HolonAST or Vector. Algebraic surface, not comparison.

**Category A — non-numeric eq/ord — IS NOT DEFERRED.** It's served
by arc 148's comparison architecture directly via the universal
same-type delegation rule. String/bool/time/holon/etc. equality
just works through `:wat::core::=` because Rust's `PartialEq` is
universal. Ord on the allowlisted types works the same way.

The "parallel user track" reduces from three categories to two
(B time-arith + C holon-pair).

## Worked examples — wat call shapes

```scheme
;; Polymorphic variadic — folds over dispatch
(:wat::core::+ 0 40.0 2)               => :f64 42.0
(:wat::core::+ 1 2 3 4 5)              => :i64 15
(:wat::core::* 1.0 2 3)                => :f64 6.0

;; Same-type variadic — type-locked
(:wat::core::i64::+ 1 2 3 4 5)         => :i64 15
(:wat::core::i64::+ 1 2.0)             => COMPILE ERROR (2.0 is :f64)
(:wat::core::f64::* 1.0 2.0 3.0)       => :f64 6.0

;; Per-Type leaf — direct binary call
(:wat::core::i64::+,2 1 2)             => :i64 3
(:wat::core::+,i64-f64 1 2.0)          => :f64 3.0

;; Comparison — strict binary
(:wat::core::< 1 2)                    => :bool true
(:wat::core::< 1 2.0)                  => :bool true       ; mixed routes via dispatch
(:wat::core::< 1 2 3)                  => COMPILE ERROR    ; arity mismatch
(:wat::core::and (:< 1 2) (:< 2 3))    => :bool true       ; chains via :and

;; Rejected unary (no implicit negation/reciprocal)
(:wat::core::- 1)                      => COMPILE ERROR
(:wat::core::/ 5)                      => COMPILE ERROR
```

## Substrate registration — sketch shape

Implementation slices (2-3) realize this; sketch here is for design
clarity, not literal slice 1 deliverable.

```scheme
;; ─── Per-Type Rust primitives — registered via env.register in
;;     register_builtins (or per arc 147's macro when shipped) ───
;; (Rust impls; not shown — bind_i64_plus_i64, etc.)

;; ─── Binary Dispatch entity — wat declaration in wat/core.wat ───
(:wat::core::define-dispatch :wat::core::+,2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::+,2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::+,2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::+,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::+,f64-i64))

;; ─── Variadic wat function — wat declaration in wat/core.wat ───
;; (Polymorphic top-level)
(:wat::core::define
  (:wat::core::+ & (xs :wat::core::Vector<numeric>) -> :numeric)
  (:wat::core::reduce :wat::core::+,2 xs))

;; (Same-type i64)
(:wat::core::define
  (:wat::core::i64::+ & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  (:wat::core::reduce :wat::core::i64::+,2 xs))

;; ─── Comparison Dispatch entity (no wrapper needed) ───
(:wat::core::define-dispatch :wat::core::<
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::<)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::<)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::<,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::<,f64-i64))
```

The `numeric` shorthand in the variadic signatures is approximate;
slice 2 audits whether wat has a numeric union type or whether
the variadic uses inference + min-2-args validation. Sonnet's
slice 2 brief resolves this concretely.

## Slice plan (revised post-audit, 2026-05-03)

The audit (`AUDIT-SLICE-1.md`) surfaced three discrepancies between
DESIGN's assumptions and the substrate's current state. Resolutions
adopted under the durability discipline ("eliminate failure domains;
build the substrate up, don't bridge over its gaps"):

- **OQ1 — `eval_compare` allowlist narrower than DESIGN's ord
  allowlist.** Resolution: build the substrate up. Add `values_compare`
  arms for `:wat::time::Instant`, `:wat::time::Duration`,
  `:wat::core::Bytes`, `:wat::core::Vector<T>`, `:wat::core::Tuple<T...>`,
  `:wat::core::Option<T>`, `:wat::core::Result<T,E>` mirroring
  `values_equal`'s recursive shape. KEEP `bool` and `keyword` ord
  (substrate already supports; PartialOrd is honest). Eliminates
  failure domain "I tried to compare X and got a runtime error."

- **OQ2 — Per-Type arithmetic leaves already exist at bare names
  (no `,2` suffix).** Resolution: rename existing
  `:wat::core::i64::+`/`-`/`*`/`/` and `:wat::core::f64::+`/`-`/`*`/`/`
  to add `,2`. Bare names become available for the variadic wat
  wrappers slice 4 ships. Mass call-site sweep across tests/examples
  /lab. Eliminates failure domain "same name, sometimes binary,
  sometimes variadic." (Comparison per-Type leaves are NOT renamed —
  they get RETIRED in slice 5 instead, see below.)

- **OQ3 — `:not=` is computed, not per-Type leaved.** Resolution:
  keep current substrate behavior — `:wat::core::not=` computes
  `not(:wat::core::= ...)` internally. No per-Type `:not=` leaves.
  Drops comparison count from 6 ops × 3 = 18 names to 5 ops × 3 +
  `:not=` × 1 = **16 names**. Eliminates failure domain "per-Type
  `:not=` drifts from `:=`."

**Revised total: 32 (arithmetic) + 16 (comparison) = 48 names.**

### Slice 1 — AUDIT (SHIPPED 2026-05-03)

Sonnet enumerated all 7 `infer_polymorphic_*` handler surfaces.
Produced `AUDIT-SLICE-1.md` (616 lines). Surfaced OQ1 + OQ2 + OQ3
above. No code changes.

### Slice 2 — Rename per-Type arithmetic leaves (OQ2 fix)

Foundation slice. Renames the 8 existing per-Type arithmetic Rust
primitives to add `,2` suffix; sweeps all call sites; verifies
green. NO new entities; NO architectural change beyond the rename.

Affected substrate registrations:
- `src/runtime.rs:2514-2529` (`eval_i64_arith` arms — register at
  new names)
- `src/runtime.rs:2552-2561` (`eval_f64_arith` arms — register at
  new names)
- `src/check.rs:8718-8732` (TypeScheme registrations for i64)
- `src/check.rs:8736-8750` (TypeScheme registrations for f64)
- `src/runtime.rs:15605-15641` (freeze pipeline pure-redex list)

Comparison per-Type leaves (`:wat::core::i64::<` etc.) are NOT
renamed — they get retired entirely in slice 5.

`:f64::abs`, `:f64::max`, `:f64::min` are NOT renamed — they have
no polymorphic counterpart (they're Type-specific functions; no
variadic wrapper needed at the bare name).

**Unlocks:** slice 4 (arithmetic migration can place variadic wat
fns at the freed bare names).

### Slice 3 — Build out `values_compare` for ord coverage (OQ1 fix)

Foundation slice. Extends `src/runtime.rs:4622-4634` (`values_compare`)
with arms mirroring `values_equal` (`src/runtime.rs:4491-4601`):

- `(time::Instant, time::Instant)` — chronological
- `(time::Duration, time::Duration)` — chronological
- `(Bytes, Bytes)` — byte-wise
- `(Vec, Vec)` — element-wise recursive
- `(Tuple, Tuple)` — element-wise recursive
- `(Option, Option)` — None < Some(_); Some(x) cmp Some(y) = x cmp y
- `(Result, Result)` — Err < Ok; Err(x) cmp Err(y) = x cmp y; same for Ok

Plus tests for each new ord arm. Existing comparison tests must
still pass (bool/keyword/String/numeric).

**Unlocks:** slice 5 (comparison migration can retire
`infer_polymorphic_compare`'s non-numeric branch without
regressing ord on the new types — universal delegation now works).

### Slice 4 — Numeric arithmetic migration (32 names)

For each of `+`, `-`, `*`, `/`: ship 8 entities:
- 1 polymorphic variadic wat function at `:wat::core::<v>`
- 1 binary Dispatch entity at `:wat::core::<v>,2`
- 2 same-type variadic wat functions at `:wat::core::<Type>::<v>` (uses bare names freed by slice 2)
- (2 same-type binary Rust primitives at `:wat::core::<Type>::<v>,2` — already exist post-slice-2)
- 2 mixed-type binary Rust primitives at `:wat::core::<v>,i64-f64` and `,f64-i64`

Retire `infer_polymorphic_arith` + `eval_poly_arith` + 4 runtime
dispatch arms + 4 freeze pipeline pure-redex entries.

### Slice 5 — Numeric comparison migration (16 names)

For each of `=`, `<`, `>`, `<=`, `>=`: ship 3 entities:
- 1 substrate primitive at `:wat::core::<v>` (uses Rust's
  PartialEq/PartialOrd polymorphically for same-type; routes
  named mixed arms; raises otherwise)
- 2 mixed-type binary Rust primitives at `:wat::core::<v>,i64-f64`
  and `,f64-i64`

Plus `:wat::core::not=` as 1 substrate primitive computing
`not(:wat::core::= ...)` internally — no per-Type or mixed leaves.

Retire numeric portion of `infer_polymorphic_compare` + numeric
arms of `eval_eq` / `eval_compare` + 6 runtime dispatch arms +
6 freeze pipeline pure-redex entries.

**Also retire per-Type comparison leaves** (`:wat::core::i64::=`,
`:i64::<`, `:i64::>`, `:i64::<=`, `:i64::>=` and same for f64 —
10 names total) — the substrate primitive's universal delegation
makes them redundant; keeping them creates "two ways to do the
same thing" cruft that fails the four questions on Simple. Update
all call sites to use the polymorphic substrate primitive.

### Slice 6 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry + arc 146 slice 5 unblock
note + Ratio forward-reference (future arc — see "Future work"
below).

### Deferred (parallel user track) — Categories B + C only

After arc 148 closes, two categories remain in the parallel
user-track:

- **Category B** — Time arithmetic (`:wat::time::-`,
  `:wat::time::+`). Handler `infer_polymorphic_time_arith`.
  2 ops × 3 signatures.
- **Category C** — Holon-pair algebra. 4 handlers, 5 user-facing
  ops (`:cosine`, `:dot`, `:coincident?`, `:coincident-explain`,
  `:simhash`).

Category A (non-numeric eq/ord) is NOT deferred — slice 5's
substrate primitive handles it via the universal-delegation rule
made tractable by slice 3's `values_compare` buildout.

## Future work — Ratio support (separate arc)

User direction 2026-05-03: **Ratio is a separate arc, not arc 148.**
Adding a native `:wat::core::Ratio` type for exact rational
arithmetic is a coherent substrate addition (Clojure-style), but
it expands the numeric tower from {i64, f64} to {i64, Ratio, f64}
with new mixed combos and a real semantics question (Ratio + f64
coerces which way? equality across types?). That work waits on
explicit arc spawn; arc 148 ships {i64, f64} per the locked
architecture above.

When Ratio arc lands, the implicit-identity rule for 1-ary `:/`
revises: `(:/ 5)` would return `1/5:Ratio` instead of `0:i64`
(integer truncation). That's a deliberate behavior change captured
in Ratio arc's INSCRIPTION.

## Why arc 146 closure depends on this

User direction: 146 slice 5 (closure paperwork) BLOCKS on arc 148
completion. Reasoning:

Arc 146's INSCRIPTION would claim "every defined symbol queryable
at runtime" / "substrate has 6 entity kinds with honest
representation." Both claims are FALSE while arithmetic +
comparison are still using the hardcoded-handler anti-pattern.

Honest closure of 146 requires arc 148 to finish the numeric
correction. Non-numeric work continues in parallel without
blocking either arc.

Arc 146 slice 5 closure paperwork becomes:
- "arc 146 closes the CONTAINER METHODS chapter"
- "arc 148 closes the NUMERIC ARITHMETIC + NUMERIC COMPARISON
  chapter"
- Non-numeric polymorphism work in user-track; future arc when ready

## Decision history (this session, 2026-05-03)

The naming + architecture went through three rejected schemes
before converging. Recorded for future-self compaction recovery
and so the rationale doesn't get re-litigated.

### What was tried and rejected

1. **Slash-stacking (initial DESIGN)** — `:wat::core::i64/+/2`
   for same-type; `:wat::core::+/i64-f64/2` for mixed; `:i64///2`
   and `://i64-f64/2` for division. Gaze converged on this scheme
   earlier in the session. REJECTED in this session because:
   - Same-type and mixed templates structurally diverge (verb
     migrates slots) — gaze L1 lie on later re-audit
   - Division produces 3-4 consecutive slashes — visual debt
   - Doesn't leverage the existing `:wat::core::i64::+`
     convention already in `resolve.rs` etc.

2. **Uniform comma everywhere** — `:wat::core::+,i64-i64` for
   same-type; `:+,i64-f64` for mixed. REJECTED:
   - Same-type form breaks the established `<Type>/<method>` /
     `<Type>::<verb>` precedent
   - Forces the `,` separator into name positions where slash/
     namespace already speaks

3. **Hybrid (gaze's surfaced third path)** — slash for same-type;
   comma for mixed. REJECTED in favor of the user's proposal
   below because:
   - Same-type slash form (`:i64/+/2`) still has division collision
   - Doesn't unify with existing `:i64::+` convention

### What converged (the locked architecture)

User proposal 2026-05-03 — Type-as-namespace for same-type
(extends existing `:wat::core::i64::+` precedent in
CONVENTIONS.md line 23 + already-shipped impls); verb-comma-pair
for mixed-type (new but lands in the gap where existing convention
has nothing to say). Min-2 arity. Variadic wat function for
arithmetic surfaces; strict binary for comparison.

### Gaze trail (this session, compaction recovery)

- Gaze 1: A=slash-stacking vs B=uniform-comma. B closer to
  convergence; flagged that slash-stacking has L1 lie (template
  divergence). Surfaced the existing `:wat::core::i64::+`
  precedent the orchestrator was ignoring. Agent
  `a523e3add6b2b8286`.
- Gaze 2: User's Type-as-namespace + comma-mixed proposal.
  CONVERGED. L1=0, L2=0. The user's "new convention" was actually
  the existing convention rediscovered. Agent `a7f9660b3fa0c5eb5`.
- Gaze 3: Binary dispatch sibling-name candidates A=`,binary`
  vs B=`,2` vs C=`,pair`. B (`,2`) wins head-to-head (zero L1,
  borderline-zero L2). A mumbles ("binary as opposed to what?");
  C soft-lies ("pair" suggests pair-typed argument). Agent
  `a1fe13dcce1fcd9a0`.

Ward isolation maintained across all three; ward-converged
architecture holds.

### Reframings the user drove

- **"Reduce over dispatch."** Composition lives in wat (variadic
  fold via `:wat::core::reduce`); leaves live in Rust (per-Type
  binary primitives). Orchestrator had been over-engineering the
  variadic resolver as a Rust mega-impl; user reframed as a
  trivially-small wat function. Architecturally lighter; honest
  about where each concern belongs.
- **"`(:- 1)` is an incomplete statement."** Min-2 arity rule;
  no implicit negation/reciprocal from unary forms. Mint
  `:wat::core::negate` separately if wanted.
- **Same-type variadic gets its own surface.** `(:i64::+ 1 2 3)`
  works as type-locked variadic, folding over `:i64::+,2` (the
  binary leaf). User caught that the orchestrator's "8 entities
  has 2 redundant" claim was wrong — the variadic + binary
  distinction holds at the Type level too when same-type variadic
  is a desired UX.

## Cross-references

- arc 146 — container method correction (precedent + Dispatch
  mechanism; its slice 5 closure blocks on this arc)
- arc 144 REALIZATIONS — the two-type-system-models collision
  (same root cause this arc closes for arithmetic + comparison)
- COMPACTION-AMNESIA-RECOVERY § FM 10 — entity-kind-not-
  type-system-feature discipline
- CONVENTIONS.md line 23 — `:wat::core::i64::+` Type-as-namespace
  convention (which this arc consciously extends to arithmetic +
  comparison families)
- arc 109 INVENTORY § L — pending naming consistency. Closure of
  arc 148 will likely touch CONVENTIONS.md to formalize the
  comma separator + the Type-as-namespace pattern uniformly.

## Status notes

- DESIGN locked 2026-05-03 after multi-session debate.
- Slice 1 (AUDIT) ready to spawn; BRIEF + EXPECTATIONS shipped
  alongside this DESIGN.
- Slices 2-3 (numeric arithmetic + numeric comparison) wait on
  slice 1 audit deliverable.
- Slice 4 closure ships after slices 2-3 land; unblocks arc 146
  slice 5.
- Non-numeric / holon / time work proceeds in user-track; not
  blocking; lands in future arc when ready.
- Arc 109 v1 closure now waits on arc 144 + arc 130 + arc 145 +
  arc 146 + arc 147 + arc 148. The "impeccable foundation"
  milestone moves further out — but each arc compounds; the
  foundation strengthens with each.
