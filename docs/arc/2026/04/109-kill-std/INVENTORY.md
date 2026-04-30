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
| `:()` (unit, as a TYPE) | `:wat::core::unit` (replaces — `:()` retires as a type annotation) |
| `:wat::core::keyword` | already FQDN ✓ |
| `:wat::core::Bytes` | already FQDN ✓ |
| `:wat::core::EvalError` | already FQDN ✓ |

The five named primitive types (`i64`/`f64`/`bool`/`String`/`u8`) move
under `:wat::core::*` and the bare forms retire (slice 1a → 1b → 1c).

The unit TYPE moves under `:wat::core::*` (the same home as
`i64` / `f64` / `bool` / `String` / `u8`). Pre-arc-109 it's
spelled `:()` (zero-element structural form, parses as empty
tuple); slice 1d retires that spelling as a type annotation. The
empty-tuple LITERAL VALUE `()` stays untouched — only the TYPE
annotation `:()` is renamed.

`unit` is the structural-honest name — matches Rust / ML / Haskell
tradition. (No `unit?` predicate — per arc 110 / 111 doctrine,
absence is `:None`, emptiness is `empty?`, errors are `:Err`; the
unit type's static-known one-inhabitant makes a runtime predicate
tautological.)

User direction (2026-04-30, captured during arc 112 slice 2a sweep,
following the Program / Thread / Process supertype split conversation
in Section J):

> i want us to remove () as the type in 109 - it needs to be swapped
> to :wat::core::unit

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
| `:wat::std::*` (list, math, stat) | **flattens** — see Section G; std empties out |
| `:wat::verify::*` | stays |
| `:wat::test::*` | stays |

The `:wat::*` namespace IS the substrate. `:wat::core::*` is its
core (built-in types, control flow, value constructors,
arithmetic, collections); the rest of `:wat::*` is the rest of
the substrate (load/eval/macroexpand machinery, IO, time, EDN,
config, std). Everything is named explicitly; the partition
within `:wat::*` is by concern, not by FQDN-ness.

## G. Three-tier substrate organization

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
| `:wat::poly::*` (new) | **Runtime-polymorphic dispatchers.** One name; runtime selects the implementation based on operand type. Numeric `+`/`-`/`*`/`/`, polymorphic `empty?`/`length`/`contains?`/`get`, `show`. | **Admission rule (Hickey, 2026-04-29 review):** an op earns `:wat::poly::*` ONLY if it dispatches on operand type to give one name across many types. "It feels convenient" is NOT enough — that's the rule that turns this tier into a new `std`. Every member must answer: which type tag selects which mono-typed primitive? If the op has no type-driven story, it does not belong here. |
| `:wat::list::*` | **List-like (iterable) operations.** HOF over collection types (`map`, `foldl`, `filter`, `range`, etc.). Implementation can be Rust; the namespace acknowledges the conceptual tier. | "Composable from primitives, but worth shipping for ergonomics." Same Lisp-stdlib / Rust-`Iterator` flavor as before — flattened from `:wat::std::list::*` to `:wat::list::*` since list is a substrate concern, not a sub-niche of std. |

### What `:wat::std::*` becomes after this re-org

`:wat::std::*` empties out entirely. Every substrate concern
graduates to its own top-level tier:

- `:wat::std::list::*` → `:wat::list::*`
- `:wat::std::math::*` → `:wat::math::*`
- `:wat::std::stat::*` → `:wat::stat::*`
- `:wat::std::stream::*` → `:wat::stream::*` (the 14 stream HOFs:
  `spawn-producer`, `from-receiver`, `map`, `filter`, `inspect`,
  `fold`, `for-each`, `chunks`, `chunks-by`, `take`, `flat-map`,
  `with-state`, `drain-items`, `collect`, `window`, plus the
  `Stream<T>` / `Producer<T>` / `ChunkStep<T>` / `KeyedChunkStep<K,T>`
  typealiases). Stream is to channels what `:wat::list::*` is to
  Vecs — collection-shaped HOFs at the honest tier.
- `:wat::std::service::Console::*` → `:wat::console::Console::*`
  (the typealiases `Message` / `Tx` / `Rx` / `AckTx` / `AckRx` /
  `Handle` / `DriverPair` / `Spawn` plus the verbs `Console/loop` /
  `Console/ack-at` / `Console/out` / `Console/err` / `Console/spawn`).
  Parallels `:wat::lru::CacheService`, `:wat::telemetry::Service`,
  `:wat::holon::lru::HologramCacheService` — services live at
  concept-named tiers, not under `service::*`. The "service" word
  is what they ARE; the tier is what they WORK ON.

### Filesystem path mirrors FQDN

The same FQDN doctrine applies to **file paths** as to symbol
paths. A file at `wat/std/edn.wat` shipping `:wat::edn::*` is
exactly as dishonest as a file at `wat/edn.wat` shipping
`:wat::std::edn::*` — the `std/` segment is in the lie either
way; what matters is whether file path and shipped symbols agree.

**The rule:** the directory tree under `wat/` mirrors the FQDN
tree under `:wat::*`. A file shipping `:wat::edn::read` /
`:wat::edn::write` lives at `wat/edn.wat`. A file shipping
`:wat::kernel::*` extensions lives at `wat/kernel/<name>.wat`
(joining `wat/kernel/queue.wat`, which is already honest).

Already-honest filesystem layout (path matches shipped FQDN):

- `wat/holon/*.wat` ships `:wat::holon::*` ✓
- `wat/kernel/queue.wat` ships `:wat::kernel::*` typealiases ✓

Dishonest layout (path does NOT match shipped FQDN — six files):

| Today's file | Today's shipped paths | After arc 109 |
|---|---|---|
| `wat/std/edn.wat` | `:wat::edn::*` | `wat/edn.wat` (file move; symbols unchanged) |
| `wat/std/test.wat` | `:wat::test::*` | `wat/test.wat` (file move; symbols unchanged) |
| `wat/std/sandbox.wat` | `:wat::kernel::run-sandboxed*` | `wat/kernel/sandbox.wat` |
| `wat/std/hermetic.wat` | `:wat::kernel::run-sandboxed-hermetic*` etc. | `wat/kernel/hermetic.wat` |
| `wat/std/stream.wat` | `:wat::std::stream::*` | `wat/stream.wat` (path AND symbols both rename per slice 9d) |
| `wat/std/service/Console.wat` | `:wat::std::service::Console::*` | `wat/console/Console.wat` (path AND symbols both rename per slice 9e) |

After arc 109 closes, `wat/std/` is gone. Every file's path
matches its shipped FQDN by inspection. A reader navigating the
substrate sees one tree, not two.

Each tier's name now says exactly what lives there at first
contact, with no "library miscellany" indirection. The pattern
matches the rest of the substrate (`:wat::core::*`,
`:wat::kernel::*`, `:wat::holon::*`, `:wat::io::*`,
`:wat::time::*`, `:wat::lru::*`, `:wat::telemetry::*`,
`:wat::edn::*`, `:wat::test::*`, etc.) — every namespace is a
substrate concern, not a tier-of-organization.

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

The substrate has four ops that already fit this tier today —
they live at `:wat::std::list::*` (`map-with-index`,
`remove-at`, `window`, `zip`). Slice 10 flattens that namespace
to `:wat::list::*`, where the fourteen above join them. Impls
stay Rust for efficiency.

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
Section G's three-tier reorganization treats `list` as a
substrate concern (not a sub-niche of std); the existing four
ops are no exception.

After arc 109's slice 8, every list-like op lives at
`:wat::list::*`. The old `:wat::std::list::*` namespace empties
out; `:wat::std::*` retains only `:wat::std::math::*` and
`:wat::std::stat::*` (subject to question 4 in Section G —
whether those also flatten).

## I. Already FQDN — out of scope

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

1. **Slice 1 — Section A (primitive types).** Four sub-slices:
   - **1a** — both `:i64` and `:wat::core::i64` accepted (additive).
   - **1b** — substrate stdlib + lab swept to FQDN.
   - **1c** — bare `:i64`/`:f64`/`:bool`/`:String`/`:u8` errors at
     startup with self-describing redirect.
   - **1d** — `:wat::core::unit` minted; `:()` retires as a TYPE
     annotation (the empty-tuple literal VALUE `()` stays). Three
     sub-sub-slices mirror 1a → 1b → 1c: (1d-α) both `:()` and
     `:wat::core::unit` accepted as type annotations (additive);
     (1d-β) substrate stdlib + lab swept to `:wat::core::unit`;
     (1d-γ) bare `:()` as a type annotation errors at startup with a
     self-describing redirect.
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
6. **Slice 6 — Retire deprecated aliases.** `:wat::core::try`,
   `:wat::core::option::expect`, `:wat::core::result::expect`
   error at startup; only the `Type/verb` forms remain.
7. **Slice 7 — Section H tier reclassification.** Move the
   HOF set (`map`, `foldl`, `foldr`, `filter`, `sort-by`,
   `find-last-index`, `take`, `drop`, `reverse`, `concat`,
   `range`, `last`, `second`, `third`) from `:wat::core::*` to
   `:wat::list::*`. Both names accepted in the additive
   phase; consumers swept; bare `:wat::core::map` etc. error at
   startup once green.
8. **Slice 8 — `:wat::poly::*` graduation.** Move polymorphic
   ops (`+`, `-`, `*`, `/`, `<`, `<=`, `=`, `>`, `>=`, `not=`,
   `empty?`, `length`, `contains?`, `get`, `show`) from
   `:wat::core::*` to `:wat::poly::*`. Sweep consumers. Bare
   `:wat::core::+` etc. error once green.
9. **Slice 9 — `:wat::std::*` and `wat/std/` both empty.** Each
   sub-slice fixes BOTH the symbol path AND the file path so they
   mirror — that's the rule. Symbol-only sweeps (9a-9c) just move
   the wat:: paths; symbol+file sweeps (9d, 9e) move both;
   file-only sweeps (9f-9i) just relocate already-honest files.
   - **9a** — `:wat::std::list::*` → `:wat::list::*` (substrate
     register-side; `wat/std/list/` doesn't exist as a file).
   - **9b** — `:wat::std::math::*` → `:wat::math::*` (same shape).
   - **9c** — `:wat::std::stat::*` → `:wat::stat::*` (same shape).
   - **9d** — `:wat::std::stream::*` → `:wat::stream::*`; AND
     `wat/std/stream.wat` → `wat/stream.wat`. The 14 HOFs + 4
     typealiases repath.
   - **9e** — `:wat::std::service::Console::*` →
     `:wat::console::Console::*`; AND
     `wat/std/service/Console.wat` → `wat/console/Console.wat`.
     Parallels concept-tiered services
     (`:wat::lru::CacheService` / `:wat::telemetry::Service` /
     `:wat::holon::lru::HologramCacheService`).
   - **9f** — `wat/std/edn.wat` → `wat/edn.wat` (file move only;
     already ships `:wat::edn::*`).
   - **9g** — `wat/std/test.wat` → `wat/test.wat` (file move only;
     already ships `:wat::test::*`).
   - **9h** — `wat/std/sandbox.wat` → `wat/kernel/sandbox.wat`
     (file move only; already extends `:wat::kernel::*`).
   - **9i** — `wat/std/hermetic.wat` → `wat/kernel/hermetic.wat`
     (file move only; already extends `:wat::kernel::*`).
   After 9 closes: `wat/std/` directory deleted, `:wat::std::*`
   namespace empty, every file's path matches its shipped FQDN.
   File moves require updating `register_stdlib_*` paths in
   `src/stdlib.rs` (where files are baked into the binary via
   `include_str!`).

Each slice ends with cargo test --workspace green + lab green
before the bare form errors out.

Per the user's
[`feedback_iterative_complexity.md`](../../../../../../../home/watmin/.claude/projects/-home-watmin-work-holon/memory/feedback_iterative_complexity.md):
build small steps, prove each, never one-shot.

## J. Program / Thread / Process — typed-program supertype split

Arc 112 slice 2a UNIFIED the spawn-program (in-thread) and
fork-program (out-of-process) returns under a single
`:wat::kernel::Process<I,O>` struct, with the wait mechanism
hidden inside `ProgramHandle`'s internal `InThread` / `Forked`
enum variant. That was the right structural mirror for arc 111
(one verb pair on one type), but it conflates two genuinely
different runtime contexts behind one name. The honest naming —
captured here for arc 109's resolution pass — is a **supertype
split**:

```
:wat::kernel::Program<I,O>      ← abstract supertype
  ├─ :wat::kernel::Thread<I,O>  ← concrete; spawn-program returns
  └─ :wat::kernel::Process<I,O> ← concrete; fork-program returns
```

A **Program** is the abstract notion of "a wat program running
somewhere with a typed I/O channel." A Thread is a Program
running in-thread (current OS process); a Process is a Program
running in a separate OS process via fork. Both have the same
field shape (stdin / stdout / stderr / wait-mechanism). They
differ only in failure-reporting flavor:

- `:wat::kernel::Thread/join-result` returns
  `:Result<:wat::core::unit, :wat::kernel::ThreadDiedError>`.
- `:wat::kernel::Process/join-result` returns
  `:Result<:wat::core::unit, :wat::kernel::ProcessDiedError>`.

Arc 112 introduced `ProcessDiedError` for slice 2a. The
**Thread/join-result** verb does not yet exist; its work is
done by today's bare `:wat::kernel::join-result` (arc 060) which
operates on `:wat::kernel::ProgramHandle<R>`. Arc 109's resolution
pass migrates that handle name to `Thread<R>`.

### `:wat::kernel::join-result` becomes polymorphic over `Program<I,O>`

The polymorphism arc 109 lands:

```
(:wat::kernel::join-result p :Program<I,O>)
   -> :Result<(), :{Thread,Process}DiedError>
```

Dispatch site sees the concrete type of `p` and routes to either
`Thread/join-result` or `Process/join-result`. The user-visible
verb name stays `join-result` (no Thread/ or Process/ prefix
required at the call site). The typed forms remain available
when the user wants to be explicit.

This is the first wat substrate verb that needs **typeclass-level
dispatch** — picking implementation based on a concrete-type's
satisfaction of a supertype. Today's substrate doesn't have this
mechanism; arc 109 is where it's minted.

### Slice plan (slots into the existing slice numbering)

| Slice | Work |
|---|---|
| **10a** | Mint `:wat::kernel::Program<I,O>` as supertype kind. Define satisfaction rule (a struct satisfies Program<I,O> if it has fields `stdin: IOWriter, stdout: IOReader, stderr: IOReader` and a wait-handle that resolves to `Result<(), E>` where `E` extends a "DiedError" supertype). |
| **10b** | Rename arc-112's unified `:wat::kernel::Process<I,O>` (returned by both spawn-program and fork-program) to `:wat::kernel::Program<I,O>`. Sonnet sweep against TypeMismatch output. |
| **10c** | Split `Program<I,O>` back into two concrete types: `Thread<I,O>` (returned by `spawn-program` / `spawn-program-ast`; has wait yielding `ThreadDiedError`) and `Process<I,O>` (returned by `fork-program` / `fork-program-ast`; has wait yielding `ProcessDiedError`). Both satisfy the abstract `Program<I,O>`. |
| **10d** | Mint `:wat::kernel::Thread/join-result` (typed wait on Thread). Mint typeclass dispatch for the polymorphic `:wat::kernel::join-result` verb on `Program<I,O>`. Bare `:wat::kernel::join-result` on a raw `:wat::kernel::ProgramHandle<R>` (from `:wat::kernel::spawn` arc 060) keeps its current `Result<R, ThreadDiedError>` shape — that's the bare-spawn path; the new poly verb is for typed Programs. |
| **10e** | Sonnet sweep call sites: `Process<...>` annotations from spawn-program → `Thread<...>`; `Process<...>` annotations from fork-program stay; bare `(:wat::kernel::join-result proc)` calls work polymorphically; explicit `(:wat::kernel::Process/join-result ...)` and `(:wat::kernel::Thread/join-result ...)` available for type-explicit code. |

### Why this is arc 109's problem and not arc 112's

Arc 112's structural mirror to arc 111 demands one verb pair on
one type — slice 2a delivered that. The honest name for that
type is "Program" (abstract running thing), not "Process" (which
in OS terms specifically means out-of-process). Arc 109's stated
goal is "name everything correctly"; carrying the
Process-as-misnamed-Program through arc 112's closure into 109
is the right place to fix it. Arc 113 (cascading runtime errors
as `Vec<*DiedError>`) generalizes naturally over the
`Thread/Process` distinction once it exists.

User direction (2026-04-30, captured during arc 112 slice 2a
sonnet sweep):

> :wat::kernel::Process<I,O> — this needs to become
> :wat::kernel::Program<I,O>
>
> :wat::kernel::Thread<I,O> and :wat::kernel::Process<I,O> both
> satisfy this
>
> we need to have :wat::kernel::join-result be poly on this
>
> :wat::kernel::Thread/join-result and
> :wat::kernel::Process/join-result are used in the poly

## Cross-references

- Arc 005 — stdlib naming audit (the inventory this arc updates).
- Arc 077 — chapter 76's "name when ≥ 3 angle brackets"
  type-alias rule. Arc 109 is the broader "name everything"
  generalization.
- Arc 108 — typed `expect` shipped at `:wat::core::option::expect` /
  `:wat::core::result::expect`; arc 109 will reshape to use the
  PascalCase Type/method form (`:wat::core::Option/expect`,
  `:wat::core::Result/expect`) once Section C lands.
- Arc 112 — minted the unified `:wat::kernel::Process<I,O>`
  + `:wat::kernel::ProcessDiedError`; arc 109 section J above
  evolves the naming to Program + Thread/Process refinement.
