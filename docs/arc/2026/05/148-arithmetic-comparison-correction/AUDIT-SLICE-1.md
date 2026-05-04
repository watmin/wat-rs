# Arc 148 Slice 1 — AUDIT

**Drafted 2026-05-03.** Pure audit of the 7 `infer_polymorphic_*`
handlers in `src/check.rs` and their runtime counterparts. Every
claim cites file:line in the current working tree. Source-of-truth
files: `src/check.rs:3286-3351` (handler dispatch),
`src/check.rs:6567-7282` (handler bodies), `src/runtime.rs:2593-2631`
(runtime op dispatch), `src/runtime.rs:4424-4744` (numeric runtime
impls), `src/runtime.rs:15605-15641` (freeze pipeline dispatch),
`src/time.rs:438-547` (time runtime impls), `src/runtime.rs:2799-2820`
(holon runtime arms), `src/check.rs:8716-8750`, `9010-9041` (per-Type
substrate scheme registrations).

Dispatch precedence (`src/check.rs:2984-2990`): the
`dispatch_registry` guard runs BEFORE the `match k.as_str()` arm
that routes to the polymorphic_* handlers. Any future Dispatch
entity declared at one of these names will preempt the legacy
handler — the migration path arc 146 used.

## Handler — `infer_polymorphic_compare` (check.rs:6567)

### User-facing ops served

Six op keywords route here at `src/check.rs:3286-3293`:

- `:wat::core::=`
- `:wat::core::not=`
- `:wat::core::<`
- `:wat::core::>`
- `:wat::core::<=`
- `:wat::core::>=`

### Argument acceptance

Handler body at `src/check.rs:6576-6611`. Arity: strict 2
(`src/check.rs:6578`). Always returns `:bool` (`src/check.rs:6577`,
`6588`, `6611`).

Two-branch acceptance after both args are inferred and resolved
under the substitution (`src/check.rs:6593-6594`):

1. **Numeric branch** (`src/check.rs:6596-6598`): if both resolved
   types satisfy `is_numeric` (`:i64` or `:f64`, defined at
   `src/check.rs:6682-6684`), accept and return `:bool`. Mixed
   `(i64, f64)` and `(f64, i64)` pass.

2. **Non-numeric branch** (`src/check.rs:6601-6609`): require
   `unify(a, b)` to succeed under the type env. Same-type for
   anything (string, bool, keyword, struct, enum, holon, vector,
   tuple, option, result, etc.) — whatever the unifier accepts.

Quote (`src/check.rs:6595-6601`):
```rust
// Numeric cross-type allowed: (i64, f64) and (f64, i64) accepted.
if is_numeric(&a_resolved) && is_numeric(&b_resolved) {
    return Some(bool_ty);
}
// Non-numeric: same-type required (preserves prior
// ∀T. T → T → :bool semantics for strings, bools, etc.).
if unify(&a_resolved, &b_resolved, subst, env.types()).is_err() {
```

### Mixed-type signatures (if applicable)

Only one mixed-type pair the checker explicitly accepts: numeric
cross — `(i64, f64)` and `(f64, i64)` (`src/check.rs:6596`).
No other mixed pairs are check-time-accepted; everything else
must unify same-type.

### Runtime impl

Routed at `src/runtime.rs:2593-2600`:

| Op | Runtime function |
|---|---|
| `:wat::core::=` | `eval_eq` (`src/runtime.rs:4424-4451`) |
| `:wat::core::not=` | `eval_not_eq` (`src/runtime.rs:4464-4481`) — calls `eval_eq` and negates |
| `:wat::core::<` / `>` / `<=` / `>=` | `eval_compare` with closure on `Ordering` (`src/runtime.rs:4603-4645`) |

`eval_eq` delegates pair acceptance to `values_equal`
(`src/runtime.rs:4491-4601`). `values_equal` returns `Some(bool)`
for these pairs: `(i64,i64)`, `(u8,u8)`, `(f64,f64)`,
`(i64,f64)`, `(f64,i64)`, `(String,String)`, `(bool,bool)`,
`(wat__core__keyword,...)`, `(Unit,Unit)`, `(Vec,Vec)` (recurses),
`(Tuple,Tuple)` (recurses), `(Option,Option)` (recurses),
`(Result,Result)` (recurses), `(Enum,Enum)` (matches type_path +
variant + fields), `(Vector,Vector)` (bit-exact),
`(holon__HolonAST, holon__HolonAST)` (delegates to PartialEq),
`(Struct,Struct)` (recurses on fields). Returns `None`
otherwise — eval_eq raises TypeMismatch on `None`.

`eval_compare` (`src/runtime.rs:4622-4634`) accepts a NARROWER
set than `values_equal`: `(i64,i64)`, `(u8,u8)`, `(f64,f64)`,
`(i64,f64)`, `(f64,i64)`, `(String,String)`, `(bool,bool)`,
`(wat__core__keyword, wat__core__keyword)`. Anything else raises
TypeMismatch at `src/runtime.rs:4636-4642`.

Freeze pipeline pure-redex inclusion at `src/runtime.rs:15609-15614`:
`:=`, `:not=`, `:<`, `:>`, `:<=`, `:>=` all listed.

### Arc 148 categorization

**SPLIT.** This single handler is the union of:

- **NUMERIC COMPARISON (slice 3 IMMEDIATE):** the numeric branch
  at `src/check.rs:6596-6598`. Six ops × 3 entities (substrate
  primitive + 2 mixed-arm leaves) per DESIGN — 18 names.

- **CATEGORY A UNIVERSAL (NOT deferred — served by slice 3's
  substrate primitive):** the non-numeric branch at
  `src/check.rs:6601-6609`. Per DESIGN, the slice 3 substrate
  primitive's universal same-type delegation handles String,
  Time, Bytes, Vector, Tuple, Option, Result, Holon, etc. via
  Rust's `PartialEq` / `PartialOrd`. **See Open Question 1 —
  the current `eval_compare` allowlist is narrower than DESIGN
  assumes.**

## Handler — `infer_polymorphic_arith` (check.rs:6619)

### User-facing ops served

Four op keywords route here at `src/check.rs:3297-3302`:

- `:wat::core::+`
- `:wat::core::-`
- `:wat::core::*`
- `:wat::core::/`

### Argument acceptance

Handler body at `src/check.rs:6628-6679`. Arity: strict 2
(`src/check.rs:6631`). Promotion result type:

- `(i64, i64)` → `:i64` (`src/check.rs:6673`)
- Either `:f64` and both numeric → `:f64` (`src/check.rs:6674`)
- Otherwise → fallback `:f64` (`src/check.rs:6677`); a TypeMismatch
  has been pushed for the non-numeric arg(s) at lines 6649-6669.

`is_numeric` predicate accepts only `:i64` and `:f64`
(`src/check.rs:6682-6684`) — `:u8` is NOT numeric for arithmetic
purposes despite its presence as a Value variant.

Quote (`src/check.rs:6672-6678`):
```rust
match (&a_resolved, &b_resolved) {
    (Some(a), Some(b)) if is_i64(a) && is_i64(b) => Some(i64_ty),
    (Some(a), Some(b)) if is_numeric(a) && is_numeric(b) => Some(f64_ty),
    // Either non-numeric or unknown — fall back to f64 so downstream
    // inference doesn't cascade more errors.
    _ => Some(f64_ty),
}
```

### Mixed-type signatures (if applicable)

Mixed-numeric: `(i64, f64) → f64`, `(f64, i64) → f64`
(`src/check.rs:6674`). No other mixed acceptance.

### Runtime impl

Routed at `src/runtime.rs:2628-2631`. All four ops dispatch to
`eval_poly_arith` (`src/runtime.rs:4677-4744`) with a `PolyOp` tag
(`Add` / `Sub` / `Mul` / `Div`, defined at `src/runtime.rs:4654-4659`).

`eval_poly_arith` arms (`src/runtime.rs:4697-4744`):

- `(Value::i64, Value::i64)` — wrapping arithmetic, divide-by-zero
  raises `RuntimeError::DivisionByZero` (`src/runtime.rs:4698-4716`).
- `(Value::f64, Value::f64)` — IEEE arithmetic, zero divisor raises
  `DivisionByZero` (`src/runtime.rs:4717-4728`).
- `(Value::i64, Value::f64)` — promote LHS via `as f64`, then f64
  arithmetic (`src/runtime.rs:4729-4744+`).
- `(Value::f64, Value::i64)` — promote RHS (continues past line 4744).

Per-Type strict leaves ALREADY EXIST as substrate primitives:

| Op | Runtime arm | TypeScheme registered |
|---|---|---|
| `:wat::core::i64::+` / `-` / `*` / `/` | `src/runtime.rs:2514-2529` (`eval_i64_arith`) | `src/check.rs:8718-8732` |
| `:wat::core::f64::+` / `-` / `*` / `/` | `src/runtime.rs:2552-2561` (`eval_f64_arith`) | `src/check.rs:8736-8750` |

These are the per-Type same-type leaves DESIGN proposes to ship as
`:wat::core::i64::+,2` etc. — **slice 2 needs to reconcile naming
between the existing names and DESIGN's `,2` arity tag.** See
Open Question 2.

Freeze pipeline pure-redex inclusion at `src/runtime.rs:15605-15641`:
`:+`, `:-`, `:*`, `:/`, `:i64::+/-/*//`, `:f64::+/-/*//`, plus
`:f64::abs/max/min` all listed for canonical-form reduction.

### Arc 148 categorization

**IMMEDIATE numeric (slice 2).** Per DESIGN: 4 ops × 8 entities =
32 names. Note DESIGN's enumeration assumes 6 of those 8 entities
do not exist yet (variadic wat wrappers + binary Dispatch entity +
mixed-type leaves); the 2 same-type binary Rust primitives
(`:i64::v` / `:f64::v` × 4) DO exist already at the bare names
(no `,2` suffix).

## Handler — `infer_polymorphic_time_arith` (check.rs:6698)

### User-facing ops served

Two op keywords route here at `src/check.rs:3308-3312`:

- `:wat::time::-`
- `:wat::time::+`

### Argument acceptance

Handler body at `src/check.rs:6707-6785`. Arity: strict 2
(`src/check.rs:6711`). LHS must be `:wat::time::Instant`
(`src/check.rs:6732-6746`); RHS-variant + op together pick the
result type.

### Mixed-type signatures (if applicable)

Three accepted (op, b-type) → result-type combinations
(`src/check.rs:6749-6764`):

| Op | a (LHS) | b (RHS) | Result |
|---|---|---|---|
| `:wat::time::-` | `:wat::time::Instant` | `:wat::time::Instant` | `:wat::time::Duration` |
| `:wat::time::-` | `:wat::time::Instant` | `:wat::time::Duration` | `:wat::time::Instant` |
| `:wat::time::+` | `:wat::time::Instant` | `:wat::time::Duration` | `:wat::time::Instant` |

`(Duration + Instant)` is NOT accepted — LHS-must-be-Instant rule
at `src/check.rs:6736`. `(Duration + Duration)`, `(Duration -
Duration)`, `(Instant + Instant)` all rejected. See Open Question 4.

### Runtime impl

Routed at `src/runtime.rs:3053-3054`:

| Op | Runtime function |
|---|---|
| `:wat::time::-` | `eval_time_sub` (`src/time.rs:438-505`) |
| `:wat::time::+` | `eval_time_add` (`src/time.rs:508-547`) |

`eval_time_sub` matches on RHS variant (`src/time.rs:456-503`):
`Duration` → Instant subtraction; `Instant` → signed duration
(panics if negative, `src/time.rs:486-494`); else TypeMismatch.

`eval_time_add` requires `Duration` RHS only (`src/time.rs:526-537`);
LHS validated by `require_instant`. **Mirrors the check-time
asymmetry — `Duration + Instant` is not supported anywhere.**

### Arc 148 categorization

**DEFERRED Category B (parallel user track).** Out of slices 2-3
scope per DESIGN. Future arc; small contained surface (2 ops × 3
signatures).

## Handler — `infer_polymorphic_holon_pair_to_f64` (check.rs:7075)

### User-facing ops served

Two op keywords route here at `src/check.rs:3324-3328`:

- `:wat::holon::cosine`
- `:wat::holon::dot`

### Argument acceptance

Handler body at `src/check.rs:7084-7125`. Arity: strict 2
(`src/check.rs:7086`). Each arg independently must satisfy
`is_holon_or_vector` (`src/check.rs:7100-7123`), which accepts
`:wat::holon::HolonAST` OR `:wat::holon::Vector` (defined at
`src/check.rs:7061-7067`). Result: always `:f64`
(`src/check.rs:7085`, `7124`).

### Mixed-type signatures (if applicable)

All four combinations of `{HolonAST, Vector} × {HolonAST, Vector}`
are accepted. The runtime promotes the AST side by encoding at the
Vector side's dimensionality (per the doc comment at `src/check.rs:7072-7074`).

### Runtime impl

Routed at `src/runtime.rs:2799` and `2819`:

| Op | Runtime function |
|---|---|
| `:wat::holon::cosine` | `eval_algebra_cosine` (`src/runtime.rs:10446`+) |
| `:wat::holon::dot` | `eval_algebra_dot` (`src/runtime.rs:10976`+) |

Both use `pair_values_to_vectors` to promote the HolonAST input
through the encoding context (`src/runtime.rs:10454`, `10976`).

### Arc 148 categorization

**DEFERRED Category C (parallel user track).** Algebraic surface,
not arithmetic or comparison. Out of slices 2-3 scope per DESIGN.

## Handler — `infer_polymorphic_holon_pair_to_bool` (check.rs:7132)

### User-facing ops served

One op keyword routes here at `src/check.rs:3329-3333`:

- `:wat::holon::coincident?`

### Argument acceptance

Handler body at `src/check.rs:7141-7182`. Identical shape to
`infer_polymorphic_holon_pair_to_f64`: arity 2, each arg is
HolonAST-or-Vector (`src/check.rs:7159-7180`). Result: `:bool`
(`src/check.rs:7142`, `7181`).

### Mixed-type signatures (if applicable)

All four combinations of `{HolonAST, Vector}²` accepted. Same as
the f64 sibling — only the return type differs.

### Runtime impl

Routed at `src/runtime.rs:2801`:

- `:wat::holon::coincident?` → `eval_algebra_coincident_q`
  (`src/runtime.rs:10532`+).

### Arc 148 categorization

**DEFERRED Category C.** Same parallel user track as the f64 sibling.

## Handler — `infer_polymorphic_holon_pair_to_path` (check.rs:7190)

### User-facing ops served

One op keyword routes here at `src/check.rs:3334-3346`:

- `:wat::holon::coincident-explain`

The dispatch site passes `":wat::holon::CoincidentExplanation"` as
the `return_path` argument (`src/check.rs:3344`).

### Argument acceptance

Handler body at `src/check.rs:7200-7240`. Arity 2; each arg is
HolonAST-or-Vector via `is_holon_or_vector` (`src/check.rs:7216-7239`).
Result: the caller-supplied struct path (`src/check.rs:7201`,
`7240`) — always `:wat::holon::CoincidentExplanation` at the
single dispatch site, but the handler accepts ANY return_path
string (a generalization beyond the one current call site).

### Mixed-type signatures (if applicable)

All four combinations of `{HolonAST, Vector}²` accepted.

### Runtime impl

Routed at `src/runtime.rs:2802-2804`:

- `:wat::holon::coincident-explain` → `eval_algebra_coincident_explain`
  (`src/runtime.rs:10579`+).

### Arc 148 categorization

**DEFERRED Category C.** Same family as cosine/dot/coincident?.

## Handler — `infer_polymorphic_holon_to_i64` (check.rs:7245)

### User-facing ops served

One op keyword routes here at `src/check.rs:3347-3351`:

- `:wat::holon::simhash`

### Argument acceptance

Handler body at `src/check.rs:7254-7281`. Arity: **strict 1**
(`src/check.rs:7256`) — the only arity-1 handler in this audit.
Single arg is HolonAST-or-Vector (`src/check.rs:7268-7280`).
Result: `:i64` (`src/check.rs:7255`, `7281`).

### Mixed-type signatures (if applicable)

N/A — single arg. Two accepted input types (HolonAST or Vector).

### Runtime impl

Routed at `src/runtime.rs:2820`:

- `:wat::holon::simhash` → `eval_algebra_simhash`
  (`src/runtime.rs:11015`+). Requires encoding context
  (`src/runtime.rs:11022`) for HolonAST input.

### Arc 148 categorization

**DEFERRED Category C.** Single-arg algebra primitive; same parallel
track as the pair-to-* siblings.

## Category mapping

### Category — Numeric arithmetic (arc 148 slice 2)

- Handler: `infer_polymorphic_arith` (`src/check.rs:6619`)
- User-facing ops: `:+`, `:-`, `:*`, `:/` under `:wat::core::*`
- DESIGN surface: 4 ops × 8 entities = 32 names
- **Already wired (per audit):**
  - 8 same-type binary Rust leaves (i64/f64 × +/-/*//) in
    `eval_i64_arith` / `eval_f64_arith` at `src/runtime.rs:2514-2529`,
    `2552-2561`. TypeSchemes registered at `src/check.rs:8718-8750`.
  - These are at the BARE per-Type names (`:wat::core::i64::+`),
    NOT at DESIGN's proposed `,2` arity-tagged form.
- **Slice 2 needs to ship:** 4 polymorphic variadic wat functions,
  4 binary Dispatch entities at `:wat::core::<v>,2`, 8 mixed-type
  leaves at `:wat::core::<v>,i64-f64` / `,f64-i64`, 8 same-type
  variadic wat wrappers at `:wat::core::<Type>::<v>`. Plus reconcile
  the existing per-Type bare-name leaves with DESIGN's `,2` form
  (Open Question 2). And retire `infer_polymorphic_arith` +
  `eval_poly_arith` + 4 runtime arms + 4 freeze redex entries.

### Category — Numeric comparison (arc 148 slice 3)

- Handler: `infer_polymorphic_compare` (`src/check.rs:6567`,
  numeric branch at lines 6596-6598)
- User-facing ops: `:=`, `:not=`, `:<`, `:>`, `:<=`, `:>=` under
  `:wat::core::*` (six ops)
- DESIGN surface: 6 ops × 3 entities = 18 names
- **Already wired:**
  - Per-Type i64 leaves for `:=`, `:<`, `:>`, `:<=`, `:>=` at
    `src/runtime.rs:2607-2613`; TypeSchemes at `src/check.rs:9011-9024`.
    **No `:i64::not=`** anywhere (Open Question 3).
  - Per-Type f64 leaves for `:=`, `:<`, `:>`, `:<=`, `:>=` at
    `src/runtime.rs:2614-2620`; TypeSchemes at `src/check.rs:9027-9040`.
    **No `:f64::not=`.**
- **Slice 3 needs to ship:** 6 substrate primitive entities (per
  DESIGN one Dispatch each — but see DESIGN's note that comparison
  uses bare name for the Dispatch directly with no variadic
  wrapper). 12 mixed-type leaves at `:wat::core::<v>,i64-f64` /
  `,f64-i64`. Reconcile with the existing per-Type leaves
  (already at the right names). Decide `:not=` per-Type story.
  Retire numeric portion of `infer_polymorphic_compare` +
  numeric arms of `eval_eq` / `eval_compare` + freeze redex entries
  (without losing the non-numeric universal-delegation behavior —
  see Open Question 1).

### Category A — Non-numeric eq/ord (NOT deferred; served by slice 3's substrate primitive)

- Handler: `infer_polymorphic_compare` (`src/check.rs:6567`,
  non-numeric branch at lines 6601-6609)
- DESIGN claim: universal same-type delegation handles String,
  Time, Bytes, Vector, Tuple, Option, Result, Holon, keyword,
  HashMap, HashSet, unit, user-defined enums/structs (`:=` /
  `:not=` only on most; `:<` etc. on an opinionated allowlist).
- **Substrate truth (audit finding — discrepancy with DESIGN):**
  - `eval_eq` / `values_equal` accepts a wide set already
    (`src/runtime.rs:4491-4601`): i64, u8, f64, mixed-numeric,
    String, bool, keyword, Unit, Vec, Tuple, Option, Result, Enum,
    Vector (bit-exact), HolonAST, Struct. Returns `None` (→
    TypeMismatch) for everything else. **No HashMap or HashSet
    arms** — these would currently raise.
  - `eval_compare` accepts a NARROWER set
    (`src/runtime.rs:4622-4634`): i64, u8, f64, mixed-numeric,
    String, **bool**, keyword. Anything else → TypeMismatch.
    **DESIGN says NO ord on bool but substrate currently allows it.**
    **DESIGN claims ord on Bytes / Vector<T> / Tuple<T...> /
    Option<T> / Result<T,E> / Time but substrate does NOT support
    any of those — they all raise TypeMismatch today.**
- See Open Question 1.

### Category B — Time arithmetic (parallel user track)

- Handler: `infer_polymorphic_time_arith` (`src/check.rs:6698`)
- User-facing ops: `:wat::time::-`, `:wat::time::+` (2 ops)
- 3 valid signatures (per DESIGN):
  - `:wat::time::-` `(Instant, Duration) → Instant`
  - `:wat::time::-` `(Instant, Instant) → Duration`
  - `:wat::time::+` `(Instant, Duration) → Instant`
- LHS-must-be-Instant; `(Duration ± anything)` rejected
  (`src/check.rs:6736`).
- Runtime: `eval_time_sub` / `eval_time_add` in `src/time.rs:438-547`.
  Symmetric with check rules.

### Category C — Holon-pair algebra (parallel user track)

- 4 handlers + 5 user-facing ops:

| User-facing op | Handler | Arity | Result type |
|---|---|---|---|
| `:wat::holon::cosine` | `infer_polymorphic_holon_pair_to_f64` | 2 | `:f64` |
| `:wat::holon::dot` | `infer_polymorphic_holon_pair_to_f64` | 2 | `:f64` |
| `:wat::holon::coincident?` | `infer_polymorphic_holon_pair_to_bool` | 2 | `:bool` |
| `:wat::holon::coincident-explain` | `infer_polymorphic_holon_pair_to_path` | 2 | `:wat::holon::CoincidentExplanation` |
| `:wat::holon::simhash` | `infer_polymorphic_holon_to_i64` | 1 | `:i64` |

All accept `:wat::holon::HolonAST` OR `:wat::holon::Vector` for
each input position via `is_holon_or_vector` (`src/check.rs:7061-7067`).
Runtime arms in `src/runtime.rs:2799-2820`.

## Open questions

The audit surfaces five UNKNOWNS or DISCREPANCIES that affect
slice 2 / slice 3 planning. None block slice 1 (which ships only
this audit doc). Each names the file:line evidence that surfaced it.

### OQ1 — `eval_compare` allowlist NARROWER than DESIGN's ord allowlist

DESIGN § "What 'same-type universal delegation' actually serves"
lists ord support for `:String`, `:wat::time::Instant`,
`:wat::time::Duration`, `:wat::core::Bytes`, `:wat::core::Vector<T>`,
`:wat::core::Tuple<T...>`, `:wat::core::Option<T>`,
`:wat::core::Result<T,E>` — and explicitly NO ord for `:bool`,
`:keyword`, `:HashMap`, `:HashSet`, `:HolonAST`, `:unit`.

Substrate truth at `src/runtime.rs:4622-4634`: `eval_compare`
accepts ord for ONLY i64, u8, f64, mixed-numeric, String, bool,
keyword. Anything else raises TypeMismatch.

**Discrepancies:**
- `bool` is currently ord-comparable; DESIGN says NO.
- `keyword` is currently ord-comparable; DESIGN says NO.
- `Time` (Instant / Duration), `Bytes`, `Vector<T>`, `Tuple<T...>`,
  `Option<T>`, `Result<T,E>` are NOT ord-comparable today; DESIGN
  says YES.

**Implication for slice 3:** The substrate primitive's "universal
same-type delegation via Rust's PartialOrd" CANNOT just delegate
to today's `eval_compare` for non-numeric types — that path
raises for most of DESIGN's allowlist. Slice 3 needs to either:

(a) Build out the missing arms in `values_compare` (mirror
    `values_equal`) for Time / Bytes / Vector / Tuple / Option /
    Result, AND remove the bool / keyword arms; OR
(b) Scope down to numeric-only and explicitly defer the Category A
    ord allowlist build-out to a parallel track; OR
(c) Re-examine DESIGN's allowlist (perhaps bool/keyword ord IS
    fine; perhaps Tuple ord isn't worth shipping now).

### OQ2 — Per-Type same-type leaves ALREADY EXIST at bare names (no `,2` suffix)

DESIGN § "Arithmetic family" enumerates `:wat::core::i64::<v>,2`
as the binary Rust leaf for each `<v>` ∈ {`+`, `-`, `*`, `/`}.

Substrate truth at `src/runtime.rs:2514-2529` + `src/check.rs:8718-8732`:
`:wat::core::i64::+`, `-`, `*`, `/` already exist as binary Rust
leaves (i64 × i64 → i64). Same for f64 at `src/runtime.rs:2552-2561`
+ `src/check.rs:8736-8750`. No `,2` suffix.

The same situation for comparison: `:wat::core::i64::=`, `:<`, `:>`,
`:<=`, `:>=` exist at the bare name (`src/runtime.rs:2607-2613` +
`src/check.rs:9011-9024`); same for f64 at lines 2614-2620 +
9027-9040. (No `:not=` per-Type — see OQ3.)

**Implication for slice 2:** DESIGN's intent is that the binary
Rust leaf is sibling to a same-type variadic wat wrapper — the
variadic at the bare name, the binary at `,2`. Slice 2 must
either (a) RENAME existing `:wat::core::i64::+` to `:wat::core::i64::+,2`
and define a new variadic wat fn at the bare name — a substantial
break of every existing call site; or (b) keep the existing
binary at the bare name and make the variadic a NEW name (the
bare name retains its current binary semantics — DESIGN's "same-
type variadic" is then deferred or named differently).

This is a real architectural choice that DESIGN doesn't surface.
The user should be consulted before slice 2 spawns.

### OQ3 — `:not=` has no per-Type variant; routes through eval_not_eq → eval_eq

DESIGN § "Comparison family" lists 6 ops including `:not=` and
implies parity with `:=` for per-Type/mixed treatment.

Substrate truth: `:wat::core::not=` routes to `eval_not_eq`
(`src/runtime.rs:2594` → `4464-4481`), which calls `eval_eq` and
negates the bool. No per-Type leaves exist:

```
$ grep "i64::not=\|f64::not=" src/runtime.rs src/check.rs
(no matches)
```

DESIGN's enumeration says 6 ops × 3 entities = 18 names. The
3-entities-per-op shape (substrate prim + 2 mixed leaves) for
`not=` would mean shipping `:wat::core::not=,i64-f64` and
`:wat::core::not=,f64-i64`. But these are pure functions of `=,*`
(`!= ≡ not(=)`); so DESIGN may want to defer them and let the
substrate prim handle `not=` by computing `not(=)` internally.

**Implication for slice 3:** Decide whether `:not=` gets its own
mixed leaves OR computes from `:=` mixed leaves at the substrate
prim level. The latter is one less pair of names per op; the
former is uniform with the other 5.

### OQ4 — Time-arith asymmetry: `(Duration + Instant)` rejected

DESIGN's open-questions section asks whether `(Duration + Instant)`
is legal anywhere. Substrate truth: it is NOT. The check-time
LHS-must-be-Instant rule (`src/check.rs:6736`) and the runtime's
`require_instant` call (`src/time.rs:455`, `525`) BOTH enforce
Instant-LHS only. Asymmetric `+` is intentional, not an oversight.

**Implication for Category B (parallel user track):** When that
work lands, the asymmetry decision can be revisited (commutative
`+` would mean accepting `(Duration + Instant)` and routing to the
same arm). Out of scope for slices 2-3.

### OQ5 — `infer_polymorphic_holon_pair_to_path` is generalized beyond its single call site

The handler at `src/check.rs:7190` takes a `return_path: &str`
parameter, but the dispatch site at `src/check.rs:3334-3346` only
ever calls it with `":wat::holon::CoincidentExplanation"` (line
3344). The generalization is unused.

**Implication for Category C (parallel user track):** When ported
to the Dispatch-entity pattern, this handler doesn't need a
return-path parameter at all — just inline the literal struct
path. The generalization adds no value. Out of scope for slices 2-3
but worth noting for the future user-track work.

---

**End of audit. 7 handlers enumerated. 16 user-facing ops total
(6 numeric compare + 4 numeric arith + 2 time-arith + 4 holon-pair
two-arg + 1 holon-arg one-arg = 17, BUT cosine/dot share one
handler so handler count is 7). 5 open questions surfaced.**
