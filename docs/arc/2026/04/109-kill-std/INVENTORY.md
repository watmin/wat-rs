# Arc 109 — kill-std — INVENTORY

> Three-tier substrate cleanup. FQDN every bare substrate-provided
> symbol; promote polymorphic ops to a new `:wat::poly::*` tier;
> flatten `:wat::std::*` (list / math / stat all graduate to
> top-level); rename `Vec → Vector`; reshape arc 108's expect/try
> verbs to `Type/verb` form; add `Option/try` sibling.
>
> The arc's identity is **kill-std** — `:wat::std::*` empties out;
> every substrate concern earns its own top-level tier. The FQDN
> sweep is the means; the flat-tier substrate is the end-state.

**Status:** scoping (2026-04-29) — user-resolved
**Driver direction:** *"everything needs to be namespaced if we
are providing it. user can have bare symbols in their forms. if we
are providing it, it must be named explicitly."*

**Doctrine** (settled 2026-04-29; see `feedback_fqdn_is_the_namespace.md`):

> "the wat language is honest and follows the rules... we only
> claim our own names and we do not have namespaces at all.. the
> fqdn /is/ the namespace. users can define their own symbols as
> they choose.. we provide fqdn that are obvious..
>
> the verbosity is worth the cost - i will not be convinced
> otherwise"

**There is no namespace mechanism.** No `use`, no `import`, no
shorthand. The colons in `:wat::core::Vector` are part of the
name; the FQDN IS how you refer to the thing. Substrate names
MUST be FQDN under `:wat::*`. User code is free to use bare
symbols (collision is impossible — bare symbols can never start
with `:wat::*`). The verbosity is the design; the argument is
settled.

This file inventories every name the substrate exposes today that
is NOT yet a fully-qualified `:wat::*::Name`. Each entry is now
resolved to a concrete target after the scoping pass. Source
checks against `src/check.rs`, `src/runtime.rs`, `src/parser.rs`,
`src/types.rs`, plus the bundled wat under `wat/`.

## A. Built-in primitive type paths

| Today | After arc 109 |
|---|---|
| `:i64` | `:wat::core::i64` |
| `:f64` | `:wat::core::f64` |
| `:bool` | `:wat::core::bool` |
| `:String` | `:wat::core::String` |
| `:u8` | `:wat::core::u8` |
| `:()` (unit) | `:wat::core::()` |
| `:wat::core::keyword` | already FQDN ✓ |
| `:wat::core::Bytes` | already FQDN ✓ |
| `:wat::core::EvalError` | already FQDN ✓ |

All five primitive type names move under `:wat::core::*`, including
the unit type. (`:()` is the empty-tuple literal but its TYPE name
is provided by the substrate; FQDN it.)

## B. Parametric type heads

| Today | After arc 109 |
|---|---|
| `Vec<T>` | `:wat::core::Vector<T>` |
| `Option<T>` | `:wat::core::Option<T>` |
| `Result<T,E>` | `:wat::core::Result<T,E>` |
| `HashMap<K,V>` | `:wat::core::HashMap<K,V>` |
| `HashSet<T>` | `:wat::core::HashSet<T>` |
| `rust::crossbeam_channel::Sender<T>` | already FQDN under `:rust::*` ✓ |
| `rust::crossbeam_channel::Receiver<T>` | already FQDN under `:rust::*` ✓ |
| `wat::kernel::HandlePool<T>` | already FQDN ✓ |
| `wat::kernel::ProgramHandle<T>` | already FQDN ✓ |

`Vec<T>` is renamed AND moved — the type's name becomes
`Vector` (matching the constructor verb post-arc-109; see § D).

## C. Variant constructors

| Today | After arc 109 |
|---|---|
| `Some` (bare symbol) | `:wat::core::Some` |
| `:None` (bare keyword) | `:wat::core::None` |
| `Ok` (bare symbol) | `:wat::core::Ok` |
| `Err` (bare symbol) | `:wat::core::Err` |

All four variants of the substrate-built-in `Option<T>` /
`Result<T,E>` enums become FQDN at every USAGE site
(constructor, pattern, match arm).

These are special-cased in `src/runtime.rs::eval_call` (matched
as `WatAST::Symbol("Some" | "Ok" | "Err")`) and in
`src/check.rs::infer_variant_constructor` (bare-symbol head
detection). `:None` is a keyword (`WatAST::Keyword(":None")`)
since it carries no payload. After arc 109 the dispatch arms
match the FQDN forms (`":wat::core::Some"` etc).

## D. Constructor verbs

| Today | After arc 109 |
|---|---|
| `:wat::core::vec` | `:wat::core::Vector` (verb = type) |
| `:wat::core::list` | **retire** (use `:wat::core::Vector`) |
| `:wat::core::tuple` | `:wat::core::Tuple` |
| `:wat::core::HashMap` (constructor) | already aligned ✓ |
| `:wat::core::HashSet` (constructor) | already aligned ✓ |
| `:wat::core::range` | **moves to `:wat::list::range`** — see Section H |

`vec` and `list` both produced `Vec<T>` — the redundancy retires
in favor of one canonical name (`Vector`). The constructor verb
and the type share the name; `(:wat::core::Vector :T x y z)`
reads as "construct a Vector of T from these elements."

## D'. `Option` / `Result` method forms — `Type/verb` shape

Companions to Section B's type renames. The two arc-108 special
forms (`:wat::core::option::expect` / `:wat::core::result::expect`)
get reshaped to PascalCase-Type + slash-verb (matching the
`Stats/new`, `MetricsCadence/new`, `HandlePool/pop` family),
and `:wat::core::try` — currently Result-only — splits into
`Result/try` plus a new `Option/try` for the matching propagation
on `:None`.

| Today | After arc 109 |
|---|---|
| `:wat::core::try` (Result-only propagate) | `:wat::core::Result/try` |
| (does not exist) | `:wat::core::Option/try` (new) |
| `:wat::core::option::expect` | `:wat::core::Option/expect` |
| `:wat::core::result::expect` | `:wat::core::Result/expect` |

After arc 109, the four branching verbs across `Option<T>` and
`Result<T,E>` are symmetric:

| Verb | Failure case | Where |
|---|---|---|
| `:wat::core::Option/try` | `:None` propagates UP | inside a fn returning `:wat::core::Option<_>` |
| `:wat::core::Option/expect` | `:None` panics with msg | anywhere |
| `:wat::core::Result/try` | `Err(e)` propagates UP | inside a fn returning `:wat::core::Result<_, E>` |
| `:wat::core::Result/expect` | `Err(_)` panics with msg | anywhere |

The `Type/verb` form mirrors the existing struct-method
convention. Substrate dispatch in `src/check.rs` and
`src/runtime.rs` adds `infer_option_try` (sibling of `infer_try`,
checks the enclosing fn returns `:wat::core::Option<_>`) and
`eval_option_try` (returns inner on Some; raises
`RuntimeError::TryPropagate` carrying `None` on `:None`). The
existing `eval_try` / `infer_try` get renamed to
`eval_result_try` / `infer_result_try` along the way; semantics
unchanged.

**Slicing note:** this section's renames + new form ride along
with Section B (which renames `Option<T>` and `Result<T,E>` as
types). The verb dispatch arms move at the same time as their
type heads; consumers update both in one sweep per call site.

## E. Special markers

| Today | After arc 109 |
|---|---|
| `_` (wildcard pattern) | **keep** — form marker, not a name |
| `->` (return-type marker) | **keep** — form marker, not a name |
| `:else` (cond fallback) | `:wat::core::else` |

`_` and `->` are punctuation in the form grammar — things you
write to STRUCTURE a form, not things you can USE as values. They
stay bare. `:else` looks like punctuation but it IS a name (a
keyword the cond walker dispatches on); FQDN it like the others.

## F. Top-level `:wat::*` ops — STAY where they are

The user explicitly resolved this:

> "load! and eval! are not in :wat::core:: -- we had a debate
> on this in some arc - they stay in their homes"

These ops live at `:wat::*` directly and remain there:

| Op | Status |
|---|---|
| `:wat::eval-ast!` | stays |
| `:wat::eval-edn!` | stays |
| `:wat::eval-step!` | stays |
| `:wat::eval-file!` | stays |
| `:wat::eval-digest!`, `:wat::eval-digest-string!` | stays |
| `:wat::eval-signed!`, `:wat::eval-signed-string!` | stays |
| `:wat::digest-load!`, `:wat::signed-load!`, `:wat::load-file!` | stays |
| `:wat::core::use!` | **moves to `:wat::use!`** — resolve-pass declaration; same family as `load-file!` / `eval-ast!`, not control flow / arithmetic |
| `:wat::form::matches?` | stays at `:wat::form::*` |
| `:wat::eval::walk` / `WalkStep` / `StepResult` | stays at `:wat::eval::*` |
| `:wat::WatAST` (type) | stays |
| `:wat::config::*` | stays |
| `:wat::time::*` | stays |
| `:wat::edn::*` | stays |
| `:wat::io::*` | stays |
| `:wat::std::*` (list, math, stat) | stays |
| `:wat::verify::*` | stays |
| `:wat::test::*` | stays |

The `:wat::*` namespace IS the substrate. `:wat::core::*` is its
core (built-in types, control flow, value constructors,
arithmetic, collections); the rest of `:wat::*` is the rest of
the substrate (load/eval/macroexpand machinery, IO, time, EDN,
config, std). Everything is named explicitly; the partition
within `:wat::*` is by concern, not by FQDN-ness.

## I. Three-tier substrate organization

**User direction (2026-04-29):**
> "right now :wat::core::- handles floats and ints... that's
> dishonest.. that's not a core thing.. but it is a thing we want...
> it makes the UX better... we need a :wat::<good-UX-namespace>::- 
> and others.. hash-map, hash-set, vec iteration and can all be
> done here..."
> ":wat::list::* could be :wat::list::* and anything who is
> 'list-like' can have methods from here who just make sense?..."

The user named a hidden tier. Three honest tiers replace the
two-tier core/std split:

| Tier | What lives here | Honesty rule |
|---|---|---|
| `:wat::core::*` | **Single-type primitives.** Mono-typed. No polymorphism. Lisp-canonical irreducibles + Rust `std::ops`-style type-attached methods. | If the op dispatches on operand TYPE, it does NOT live here. |
| `:wat::poly::*` (new) | **Polymorphic conveniences.** Cross-type operators that runtime-dispatch on operand type (numeric `+`/`-`/`*`/`/`, polymorphic `empty?`/`length`/`contains?`/`get`, `show`). Reasonable defaults for ergonomics. | If the op exists ONLY because it makes the surface less verbose, this is its home. |
| `:wat::list::*` | **List-like (iterable) operations.** HOF over collection types (`map`, `foldl`, `filter`, `range`, etc.). Implementation can be Rust; the namespace acknowledges the conceptual tier. | "Composable from primitives, but worth shipping for ergonomics." Same Lisp-stdlib / Rust-`Iterator` flavor as before — flattened from `:wat::list::*` to `:wat::list::*` since list is a substrate concern, not a sub-niche of std. |

### What `:wat::std::*` becomes after this re-org

`:wat::std::*` empties out entirely. Every substrate concern
graduates to its own top-level tier:

- `:wat::std::list::*` → `:wat::list::*`
- `:wat::std::math::*` → `:wat::math::*`
- `:wat::std::stat::*` → `:wat::stat::*`

Each tier's name now says exactly what lives there at first
contact, with no "library miscellany" indirection. The pattern
matches the rest of the substrate (`:wat::core::*`,
`:wat::kernel::*`, `:wat::holon::*`, `:wat::io::*`,
`:wat::time::*`, etc.) — every namespace is a substrate concern,
not a tier-of-organization.

### Name resolved by /gaze (2026-04-29) — `:wat::poly::*`

Gaze ward verdict:

> Winner: `:wat::poly::*`. It is the only candidate where the
> namespace name names the actual mechanism — runtime
> polymorphism over operand type. At the call site,
> `(:wat::poly::+ a b)` and `(:wat::poly::empty? c)` read cleanly
> alongside `(:wat::core::i64::+ a b)` and `(:wat::list::map f xs)`:
> core is mono-typed primitives, list is collection composition,
> poly is "the same name across many types." The tier's name
> carries its dispatch story without forcing a doc lookup.

Rejected with reasoning:

| Candidate | Verdict |
|---|---|
| `:wat::ops::*` | **Level 1 lie** — `:wat::core::*` already contains operators (`i64::+`); `ops` fails to distinguish |
| `:wat::auto::*` | Level 2 mumble — auto WHAT? |
| `:wat::common::*` | Level 2 mumble — describes a property, not the tier's identity |
| `:wat::ergo::*` | Level 2 mumble — "ergonomics" describes motivation, not content |

`dispatch`, `multi`, `any` were also considered and each mumbles
harder than `poly`.

### What moves where (after the user picks the UX name)

| Today | After arc 109 | Rationale |
|---|---|---|
| `:wat::core::+`, `-`, `*`, `/` | `:wat::poly::+`, etc. | runtime-dispatches on i64/f64 — UX, not primitive |
| `:wat::core::<`, `<=`, `=`, `>`, `>=`, `not=` | `:wat::poly::<`, etc. | same |
| `:wat::core::empty?` | `:wat::poly::empty?` | polymorphic over Vec/HashMap/HashSet/String |
| `:wat::core::length` | `:wat::poly::length` | same |
| `:wat::core::contains?` | `:wat::poly::contains?` | same |
| `:wat::core::get` | `:wat::poly::get` | polymorphic over HashMap/Vec |
| `:wat::core::show` | `:wat::poly::show` | polymorphic over all values |
| `:wat::core::i64::+`, `f64::+`, `i64::to-string`, etc. | **stay** at `:wat::core::*` | mono-typed; honest at core |
| `:wat::core::Bytes::*`, `String::*` | **stay** | type-attached methods on substrate primitives |
| `:wat::core::and`, `or`, `not` | **stay** | boolean primitives; substrate-level short-circuit |

### Open questions — all resolved

1. **`<UX>` = `poly`** (gaze-resolved, 2026-04-29).
2. **`concat` stays mono / list.** Vec-only today; fits
   `:wat::list::concat`. If a future `concat` becomes polymorphic
   over Vec/String/Bytes, it migrates to `:wat::poly::concat`
   then.
3. **`:wat::std::list::*` flattens to `:wat::list::*`** (user-confirmed).
4. **`:wat::std::math::*` and `:wat::std::stat::*` flatten** to
   `:wat::math::*` and `:wat::stat::*` (user-confirmed
   2026-04-29). After arc 109, `:wat::std::*` is empty — every
   substrate concern is named at its own top-level tier.

## H. Tier reclassification — composition-over-primitives moves to `:wat::list::*`

**User direction (2026-04-29):**
> "did you make the argument for `:wat::core::range` to be in
> `:wat::std`?... you can compose a range from a vec?..."
> "we are following lisp and rust as best we can"
> "map and fold are more std than core"

### The principle

- **`:wat::core::*`** = irreducible primitives. The Lisp-canonical
  core (special forms, atom predicates, cons-cell primitives) +
  Rust's `std::ops` / `std::cmp` (operators, type declarations,
  type-attached methods on substrate-built-in types).
- **`:wat::std::*`** = composition over primitives. Lisp's library
  layer (`mapcar`, `reduce`, `filter`, `sort`) + Rust's
  `std::iter::Iterator` higher-order trait methods. Implementation
  CAN stay in Rust for efficiency; the NAMESPACE reflects that
  it's expressible from primitives.

### Moves to `:wat::list::*`

| Today | After arc 109 | Rationale |
|---|---|---|
| `:wat::core::map` | `:wat::list::map` | HOF over Vec; Lisp `mapcar`, Rust `Iterator::map` |
| `:wat::core::foldl` | `:wat::list::foldl` | HOF; Lisp `reduce`, Rust `Iterator::fold` |
| `:wat::core::foldr` | `:wat::list::foldr` | HOF; right-fold variant |
| `:wat::core::filter` | `:wat::list::filter` | HOF; Rust `Iterator::filter` |
| `:wat::core::find-last-index` | `:wat::list::find-last-index` | searches with predicate; Rust `Iterator::rposition` |
| `:wat::core::sort-by` | `:wat::list::sort-by` | HOF; Rust `Vec::sort_by` |
| `:wat::core::take` | `:wat::list::take` | Rust `Iterator::take` |
| `:wat::core::drop` | `:wat::list::drop` | Rust `Iterator::skip` |
| `:wat::core::reverse` | `:wat::list::reverse` | composable from `foldl` + `conj` |
| `:wat::core::concat` | `:wat::list::concat` | Vec concat; Lisp `append`, Rust `Vec::extend` |
| `:wat::core::range` | `:wat::list::range` | sugar over `(Vector :i64 0 1 2 ...)` |
| `:wat::core::last` | `:wat::list::last` | Rust `Iterator::last`; composable from rest+empty? |
| `:wat::core::second` | `:wat::list::second` | Lisp `cadr`; composable from `first`+`rest` |
| `:wat::core::third` | `:wat::list::third` | Lisp `caddr`; composable |

The substrate already has `:wat::list::*` for exactly this
tier (`map-with-index`, `remove-at`, `window`, `zip`). The
fourteen above join their siblings; impls stay Rust for
efficiency.

### Stays in `:wat::core::*` — true primitives

| What | Why core |
|---|---|
| Special forms (`if`, `cond`, `let`, `let*`, `lambda`, `define`, `defmacro`, `match`, `try`, `quote`, `quasiquote`, `unquote`, `macroexpand`, `forms`) | Compiler-level forms; not values, not callable as functions. (`use!` is similar but lives at `:wat::use!` per Section F — resolve-pass declaration, same family as `load-file!` / `eval-ast!`.) |
| Type declarations (`enum`, `struct`, `newtype`, `typealias`, `variant`) | Define new types; substrate machinery |
| Cons-canonical Vec primitives (`first`, `rest`, `conj`, `empty?`, `length`) | Lisp's `car` / `cdr` / `cons` family; the irreducible spine of every list helper |
| HashMap/HashSet primitives (`assoc`, `dissoc`, `get`, `keys`, `values`, `contains?`) | Operations directly on the type; polymorphic over collections |
| Polymorphic operators (`+`, `-`, `*`, `/`, `<`, `<=`, `=`, `>`, `>=`, `not=`) | Substrate dispatch on operand type; not user-composable |
| Type-attached numeric methods (`i64::+`, `f64::*`, `i64::to-string`, etc.) | `Type/method` shape on substrate-built-in primitive types |
| `Bytes::from-hex`, `Bytes::to-hex` | `Type/method` on substrate `Bytes` |
| String methods (`String::concat`, `String::split`, `String::trim`, etc.) | Wrap Rust's `str` primitives; `Type/method` on substrate `String` |
| Boolean primitives (`and`, `or`, `not`) | Substrate-level short-circuit; not composable as plain functions |
| Type constructors (`Vector`, `Tuple`, `HashMap`, `HashSet`) | Build a value of a substrate-defined type |
| `show`, `atom-value`, `struct-new`, `struct-field`, `struct->form` | Substrate machinery exposed as ops |
| `Option/expect`, `Option/try`, `Result/expect`, `Result/try` (post-arc-109) | Branching constructs over substrate sum types; `Type/method` shape |
| `regex::matches?` | Rust regex shim; substrate-bound primitive |

### Slicing note

- The `range` move rides slice 3 (Section D constructor verbs).
- The HOF set (`map`, `foldl`, `foldr`, `filter`, `sort-by`,
  `find-last-index`, `take`, `drop`, `reverse`, `concat`,
  `last`, `second`, `third`) is its own slice — call it slice 8
  in the strategy below. ~14 dispatch arms move from
  `runtime.rs::eval_call` to register at the new path; the
  type schemes in `check.rs` follow; consumer sweep is
  mechanical via `grep -l ':wat::core::map' ...`.

### Existing `:wat::std::list::*` flattens too

The four already-shipped `:wat::std::list::*` ops
(`map-with-index`, `remove-at`, `window`, `zip`) move to the new
`:wat::list::*` namespace alongside the fourteen new arrivals.
Section I's three-tier reorganization treats `list` as a
substrate concern (not a sub-niche of std); the existing four
ops are no exception.

After arc 109's slice 8, every list-like op lives at
`:wat::list::*`. The old `:wat::std::list::*` namespace empties
out; `:wat::std::*` retains only `:wat::std::math::*` and
`:wat::std::stat::*` (subject to question 4 in Section I —
whether those also flatten).

## G. Already FQDN — out of scope

For reference; these are the families already named correctly:

- `:wat::core::*` — control flow, arithmetic, list, eval primitives
- `:wat::kernel::*` — channels, spawn/fork, signals, joins, HandlePool, etc.
- `:wat::holon::*` — algebra primitives (Atom, Bind, Bundle, ...), Engram, EngramLibrary, OnlineSubspace, Hologram, etc.
- `:wat::io::*` — IOReader / IOWriter / TempFile / TempDir
- `:wat::config::*` — set-global-seed!, set-capacity-mode!, dim-router, etc.
- `:wat::edn::*` — read/write/write-pretty/write-json/Tagged/NoTag
- `:wat::time::*` — now, Duration, Hour/Minute/etc., ago/from-now
- `:wat::std::*` — list helpers, math, stat
- `:wat::verify::*` — file-path/http-path/s3-path/string
- `:wat::test::*` — test harness (assert-eq, deftest, run-ast, program, ...)
- `:rust::*` — crossbeam channels, sqlite shims

## What this arc does NOT do

- Does NOT change user code's freedom to use bare symbols in
  their own forms. Bare `Some` can still be a USER-defined
  variant — it just needs to be in the user's own namespace
  (e.g., `:my::pkg::Some`); the FQDN substrate `Some` is
  `:wat::core::Some`.
- Does NOT change auto-generated `<struct>/<verb>` names (those
  inherit the struct's namespace).
- Does NOT change the `:` keyword prefix or the `::` separator.
- Does NOT migrate `:wat::*` top-level ops into `:wat::core::*`.
  Per § F, they stay home.

## Slicing strategy

This refactor is wide. Each family is its own slice — additive
acceptance first, then retirement of the bare form. Order minimizes
churn-on-churn:

1. **Slice 1 — Section A (primitive types).** Both `:i64` and
   `:wat::core::i64` accepted; substrate stdlib + lab swept;
   then bare `:i64` errors at startup.
2. **Slice 2 — Section B + D' (parametric heads + Option/Result
   method forms).** `Vec → Vector`, `Option`/`Result`/`HashMap`/
   `HashSet` move to `:wat::core::*`, AND the four
   `Option/try`, `Option/expect`, `Result/try`, `Result/expect`
   forms land at `:wat::core::Type/verb`. Adds the new
   `Option/try` form (propagate `:None`). Existing `:wat::core::try`
   continues to work as a deprecated alias for `Result/try`
   during the additive phase.
3. **Slice 3 — Section D (constructor verbs).** `vec → Vector`,
   `list` retired, `tuple → Tuple`. Mostly a sweep of `(vec :T ...)`
   call sites.
4. **Slice 4 — Section C (variant constructors).** Sweep `Some` /
   `Ok` / `Err` / `:None` to `:wat::core::*` form. This is the
   widest sweep — every match arm and every constructor.
5. **Slice 5 — Section E (`:else`).** Smallest; quick.
6. **Slice 6 — Section A's `:()` → `:wat::core::()`.** Probably
   ride along with another slice; the unit type appears in every
   `:fn(...) -> :()` signature.
7. **Slice 7 — Retire deprecated aliases.** `:wat::core::try`,
   `:wat::core::option::expect`, `:wat::core::result::expect`
   error at startup; only the `Type/verb` forms remain.
8. **Slice 8 — Section H tier reclassification.** Move the
   HOF set (`map`, `foldl`, `foldr`, `filter`, `sort-by`,
   `find-last-index`, `take`, `drop`, `reverse`, `concat`,
   `range`, `last`, `second`, `third`) from `:wat::core::*` to
   `:wat::list::*`. Both names accepted in the additive
   phase; consumers swept; bare `:wat::core::map` etc. error at
   startup once green.
9. **Slice 9 — `:wat::poly::*` graduation.** Move polymorphic
   ops (`+`, `-`, `*`, `/`, `<`, `<=`, `=`, `>`, `>=`, `not=`,
   `empty?`, `length`, `contains?`, `get`, `show`) from
   `:wat::core::*` to `:wat::poly::*`. Sweep consumers. Bare
   `:wat::core::+` etc. error once green.
10. **Slice 10 — `:wat::std::*` flattens.**
    `:wat::std::list::*` → `:wat::list::*` (the existing four
    `map-with-index` / `remove-at` / `window` / `zip`);
    `:wat::std::math::*` → `:wat::math::*`;
    `:wat::std::stat::*` → `:wat::stat::*`. After this slice,
    `:wat::std::*` is empty — every substrate concern is named
    at its own top-level tier.

Each slice ends with cargo test --workspace green + lab green
before the bare form errors out.

Per the user's
[`feedback_iterative_complexity.md`](../../../../../../../home/watmin/.claude/projects/-home-watmin-work-holon/memory/feedback_iterative_complexity.md):
build small steps, prove each, never one-shot.

## Cross-references

- Arc 005 — stdlib naming audit (the inventory this arc updates).
- Arc 077 — chapter 76's "name when ≥ 3 angle brackets"
  type-alias rule. Arc 109 is the broader "name everything"
  generalization.
- Arc 108 — typed `expect` shipped at `:wat::core::option::expect` /
  `:wat::core::result::expect`; arc 109 will reshape to use the
  PascalCase Type/method form (`:wat::core::Option/expect`,
  `:wat::core::Result/expect`) once Section C lands.
