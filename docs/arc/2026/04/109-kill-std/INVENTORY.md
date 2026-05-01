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

| Today | After arc 109 | Status |
|---|---|---|
| `:i64` | `:wat::core::i64` | ✓ shipped slice 1c |
| `:f64` | `:wat::core::f64` | ✓ shipped slice 1c |
| `:bool` | `:wat::core::bool` | ✓ shipped slice 1c |
| `:String` | `:wat::core::String` | ✓ shipped slice 1c |
| `:u8` | `:wat::core::u8` | ✓ shipped slice 1c |
| `:()` (unit, as a TYPE) | `:wat::core::unit` (replaces — `:()` retires as a type annotation) | ✓ shipped slice 1d (rename to `Unit` queued as follow-up; see J-PIPELINE.md) |
| `:wat::core::keyword` | already FQDN ✓ | — |
| `:wat::core::Bytes` | already FQDN ✓ | — |
| `:wat::core::EvalError` | already FQDN ✓ | — |

The five named primitive types (`i64`/`f64`/`bool`/`String`/`u8`)
moved under `:wat::core::*` across slices 1a (parser accepts FQDN),
1b (substrate stdlib outer-position sweep), and 1c (full sweep
including parametric inner positions; `BareLegacyPrimitive`
walker enforces; ~1000 sites across ~90 files; commits `f2b5dd4`
→ `e0abbfa`). See `SLICE-1C.md`.

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

| Today | After arc 109 | Status |
|---|---|---|
| `Vec<T>` | `:wat::core::Vector<T>` | ✓ shipped slice 1f (rename + move; § D `vec` verb companion shipped same slice) |
| `Option<T>` | `:wat::core::Option<T>` | ✓ shipped slice 1e |
| `Result<T,E>` | `:wat::core::Result<T,E>` | ✓ shipped slice 1e |
| `HashMap<K,V>` | `:wat::core::HashMap<K,V>` | ✓ shipped slice 1e |
| `HashSet<T>` | `:wat::core::HashSet<T>` | ✓ shipped slice 1e |
| `rust::crossbeam_channel::Sender<T>` | already FQDN under `:rust::*` ✓ | — |
| `rust::crossbeam_channel::Receiver<T>` | already FQDN under `:rust::*` ✓ | — |
| `wat::kernel::HandlePool<T>` | already FQDN ✓ | — |
| `wat::kernel::ProgramHandle<T>` | already FQDN ✓ | — |

Four of the five named heads moved under `:wat::core::*` in slice
1e (`BareLegacyContainerHead` walker + four typealiases; commits
`f8a82be` → `5a96cb0`; ~365 rename sites across 65 files).
`Vec<T>` stays pending — slice 1f territory because the rename
to `Vector` couples with § D's verb companion (`vec` → `Vector`
constructor). See `SLICE-1E.md`.

## C. Variant constructors

| Today | After arc 109 | Status |
|---|---|---|
| `Some` (bare symbol) | `:wat::core::Some` | ✓ shipped slice 1h |
| `:None` (bare keyword) | `:wat::core::None` | ✓ shipped slice 1h |
| `Ok` (bare symbol) | `:wat::core::Ok` | ✓ shipped slice 1i |
| `Err` (bare symbol) | `:wat::core::Err` | ✓ shipped slice 1i |

**§ C structurally complete (post-1h+1i).** All four variant
constructors moved to FQDN under `:wat::core::*`. The substrate
has zero bare-symbol-at-callable-head exceptions; the "callable
heads must be FQDN keywords" rule is universal.

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

| Today | After arc 109 | Status |
|---|---|---|
| `:wat::core::vec` | `:wat::core::Vector` (verb = type) | ✓ shipped slice 1f (Pattern 2 poison; coupled with § B Vec rename) |
| `:wat::core::list` | **retire** (use `:wat::core::Vector`) | ✓ shipped slice 1g (Pattern 2 poison; redirect to `Vector`) |
| `:wat::core::tuple` | `:wat::core::Tuple` | ✓ shipped slice 1g (Pattern 2 poison; verb-equals-type per slice 1f playbook) |
| `:wat::core::HashMap` (constructor) | already aligned ✓ | — |
| `:wat::core::HashSet` (constructor) | already aligned ✓ | — |
| `:wat::core::range` | **moves to `:wat::list::range`** — see Section H | pending § H |

`vec` and `list` both produced `Vec<T>` — the redundancy retires
in favor of one canonical name (`Vector`). The constructor verb
and the type share the name; `(:wat::core::Vector :T x y z)`
reads as "construct a Vector of T from these elements."

## D'. `Option` / `Result` method forms — `Type/verb` shape

**§ D' structurally complete (post-1j).** All four branching
verbs across `Option<T>` and `Result<T,E>` ship in the symmetric
`Type/verb` shape; `Option/try` is brand new (slice 1j's only
substrate addition).

| Today | After arc 109 | Status |
|---|---|---|
| `:wat::core::try` (Result-only propagate) | `:wat::core::Result/try` | ✓ shipped slice 1j (Pattern 2 rename) |
| (did not exist) | `:wat::core::Option/try` | ✓ shipped slice 1j (substrate addition) |
| `:wat::core::option::expect` | `:wat::core::Option/expect` | ✓ shipped slice 1j (Pattern 2 rename) |
| `:wat::core::result::expect` | `:wat::core::Result/expect` | ✓ shipped slice 1j (Pattern 2 rename) |

The four branching verbs across Option<T> and Result<T,E> are now
symmetric:

| Verb | Failure case | Where |
|---|---|---|
| `:wat::core::Option/try` | `:None` propagates UP | inside a fn returning `:wat::core::Option<_>` |
| `:wat::core::Option/expect` | `:None` panics with msg | anywhere |
| `:wat::core::Result/try` | `Err(e)` propagates UP | inside a fn returning `:wat::core::Result<_, E>` |
| `:wat::core::Result/expect` | `Err(_)` panics with msg | anywhere |

Substrate work shipped: new `RuntimeError::OptionPropagate`
variant + `apply_function` trampoline arm (caught at the
innermost function/lambda boundary, converts to
`Value::Option(Arc::new(None))`); new `eval_option_try` +
`infer_option_try` (mirror of the Result-side path; checks
enclosing fn returns `:Option<_>`). The dispatcher
parameterized `infer_try` / `infer_option_expect` /
`infer_result_expect` (and their eval counterparts) with a
leading `callee: &str` so diagnostics name the user-typed head.

20 files swept (5 stdlib + 15 consumer); 197 rename sites total
(15 stdlib + 182 consumer); zero substrate-gap fixes. cargo test
workspace 1476/0 (commits `ebeb6be` → `853fbdc`; SLICE-1J.md).

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

### The error hierarchy mirrors the type hierarchy

```
:wat::kernel::Program<I,O>      ⟸  :wat::kernel::Thread<I,O>
                                 |  :wat::kernel::Process<I,O>

:wat::kernel::ProgramDiedError  ⟸  :wat::kernel::ThreadDiedError
                                 |  :wat::kernel::ProcessDiedError
```

`:wat::kernel::ProgramDiedError` is the supertype/protocol for
"a Program (Thread or Process) died." Both `ThreadDiedError`
(arc 060) and `ProcessDiedError` (arc 112) satisfy it via the
typeclass mechanism slice 10d mints. Same three variants in
both — `Panic { message, failure }`, `RuntimeError { message }`,
`ChannelDisconnected` — only the type-name distinguishes the
subject (Thread vs. Process).

User-facing reading:

- **Code that doesn't care about host** matches against
  `:wat::kernel::ProgramDiedError`. One arm covers both. Most
  receivers live at this level — they want "did the peer die?"
  not "WHICH host's peer died?"
- **Code that DOES care about host** pattern-matches on the
  concrete satisfier. `((ThreadDiedError-Panic msg _) ...)` for
  thread-specific handling; `((ProcessDiedError-Panic msg _)
  ...)` for process-specific. Available when needed; not
  required.

This is the second instance of typeclass-level dispatch arc
109 § J slice 10d mints — same mechanism handles both
`Program<I,O>` (the running thing) and `ProgramDiedError`
(the failure shape). Arc 113's chained-cause backtrace
(`Vec<ProgramDiedError>`) propagates uniformly across host
boundaries because it reads against the supertype.

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
| **10d** | Mint `:wat::kernel::Thread/join-result` (typed wait on Thread). Mint `:wat::kernel::ProgramDiedError` as the error supertype; both `ThreadDiedError` and `ProcessDiedError` satisfy it. Mint typeclass dispatch for the polymorphic `:wat::kernel::join-result` verb on `Program<I,O>` AND for matching against `ProgramDiedError` at the protocol level (concrete satisfiers still pattern-matchable when subject matters). Bare `:wat::kernel::join-result` on a raw `:wat::kernel::ProgramHandle<R>` (from `:wat::kernel::spawn` arc 060) keeps its current `Result<R, ThreadDiedError>` shape — that's the bare-spawn path; the new poly verb is for typed Programs. |
| **10e** | Sonnet sweep call sites: `Process<...>` annotations from spawn-program → `Thread<...>`; `Process<...>` annotations from fork-program stay; bare `(:wat::kernel::join-result proc)` calls work polymorphically; explicit `(:wat::kernel::Process/join-result ...)` and `(:wat::kernel::Thread/join-result ...)` available for type-explicit code. Match arms against `:wat::kernel::ProgramDiedError` (host-agnostic) work via the typeclass; specific `ThreadDiedError`/`ProcessDiedError` arms available when subject matters. |
| **10f** | Mint typed comm verbs on each concrete satisfier: `:wat::kernel::Thread/send` + `:wat::kernel::Thread/recv` (zero-copy crossbeam under the hood; arc 114 transport asymmetry); `:wat::kernel::Process/send` + `:wat::kernel::Process/recv` (these are arc 112 slice 2b's `process-send`/`process-recv` renamed under §J's naming convention; pipe + EDN under the hood). |
| **10g** | Mint polymorphic `:wat::kernel::send` and `:wat::kernel::recv` over `Program<I,O>` via the same typeclass mechanism slice 10d minted for `join-result`. The current arc-111 `:wat::kernel::send` / `:wat::kernel::recv` operate on `Sender<T>` / `Receiver<T>` (channel halves) — they keep that shape; the poly extension adds Program<I,O> as an additional satisfier. Sonnet sweep call sites: explicit `Thread/send`, `Process/send`, `Thread/recv`, `Process/recv` available when subject matters; bare `send` / `recv` works polymorphically across channel-halves AND Programs. |

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

Follow-up direction (same conversation, after arc 113 sketch):

> ProgramDiedError is satisfied by ThreadDiedError and
> ProcessDiedError -- yea?

Confirmed — the error hierarchy mirrors the type hierarchy. Same
typeclass mechanism slice 10d mints serves both. See "The error
hierarchy mirrors the type hierarchy" subsection above.

Further direction (same conversation, after arc 112 slice 2a
closure decision):

> we need to update 109 to have thread-{send,recv} and a poly
> {send,recv}

Captured as slices 10f (typed `Thread/send` / `Thread/recv` and
`Process/send` / `Process/recv` — the latter being arc 112 slice
2b's `process-send` / `process-recv` under §J naming) and 10g
(polymorphic `:wat::kernel::send` / `:wat::kernel::recv` over
Program<I,O>). The verb-naming pattern unifies fully:

| Verb at protocol level | Concrete satisfiers |
|---|---|
| `:wat::kernel::join-result` | `Thread/join-result` (zero-copy crossbeam) \| `Process/join-result` (waitpid + stderr-EDN) |
| `:wat::kernel::send`        | `Thread/send` (crossbeam Sender<I>) \| `Process/send` (EDN + IOWriter) |
| `:wat::kernel::recv`        | `Thread/recv` (crossbeam Receiver<O>) \| `Process/recv` (multiplex stdout/stderr + EDN parse) |

Pre-§J the user reaches for `process-send` / `process-recv` (arc
112 slice 2b's spelling) on the slice-2a unified Process<I,O>.
Post-§J the call sites swap to either typed (`Process/send`) or
polymorphic (`send`); the substrate as teacher pattern handles
the migration.

Further direction (same conversation, after arc 113 slice 3
landed `Vec<*DiedError>` chain typealiases):

> ProgramPanics should be satisfied by ProcessPanics and ThreadPanics

Arc 113 closure shipped concrete typealiases:

| Typealias                      | Body                              |
|---|---|
| `:wat::kernel::ProcessPanics`  | `Vec<wat::kernel::ProcessDiedError>` |
| `:wat::kernel::ThreadPanics`   | `Vec<wat::kernel::ThreadDiedError>`  |

§J adds the supertype `:wat::kernel::ProgramPanics` satisfied by
both — same typeclass mechanism slice 10d uses for join-result.
Polymorphic match arms against `ProgramPanics` work uniformly;
specific `ProcessPanics` / `ThreadPanics` arms remain available
when subject matters. The chain shape at the caller surface
stays identical regardless of transport (this was the arc 113
through-line — threads pass DiedError values through crossbeam,
processes pass them as EDN over kernel pipes; same Result<R,
ProgramPanics> at every call site).

## K. `/` requires a real Type — service-grouping noun cleanup

**User direction (2026-05-01, captured during slice 1j console-rename
exploration):**

> "is this obvious?... is this simple?... is this honest?... is this
> a good ux?...
>
> these questions - always - guide us
>
> we have a lot of cruft from moving fast and making things work.. 109
> is the clean up process to get us on an incredble foundation of
> idealized patterns for others to follow. i do not know if telemetry
> is an idealized pattern, it is /a/ pattern"

### The doctrine

Across wat's existing substrate the `/` separator earns its place
when there's a **real Type** — a struct, a parametric kind, a
substrate primitive that carries a value. In several service crates
the `/` was applied to a **grouping noun** (`Service`, `Console`,
`CacheService`) that has no struct, no value, no kind — it's just a
namespace label dressed up as a Type to hang verbs on. That's
fake-Type cosplay; it fails the honesty test.

**Rule:** `/` is the Type-attached method separator. If the LHS is
not a real Type (struct / parametric kind / substrate primitive),
the verb belongs at the namespace level with `::`, like every other
top-level verb (`:wat::core::map`, `:wat::core::if`,
`:wat::list::map`).

### Real Types that earn `/` (stay)

| Type | Example methods |
|---|---|
| `:wat::kernel::HandlePool<T>` | `/pop`, `/clone-add` |
| `:wat::kernel::Process<I,O>` | `/input`, `/output`, `/join-result` |
| `:wat::kernel::Thread<I,O>` | `/input`, `/output`, `/join-result` |
| `:wat::core::Bytes` | `/to-hex`, `/from-hex` |
| `:wat::core::Result` | `/try`, `/expect` (slice 1j) |
| `:wat::core::Option` | `/try`, `/expect` (slice 1j) |
| `:wat::kernel::RunResult` | `/failure` |
| `:wat::kernel::Failure` | `/message` |
| `:wat::telemetry::Stats` | `/zero`, `/bump`, ... |
| `:wat::telemetry::MetricsCadence<G>` | `/new`, `/gate`, `/tick` |

### Grouping nouns that DON'T earn `/` (clean up)

| Today's name (cruft) | After arc 109 |
|---|---|
| `:wat::std::service::Console::*` (typealiases) | `:wat::console::*` (typealiases at namespace level) |
| `:wat::std::service::Console/spawn` (verb) | `:wat::console::spawn` (top-level verb in namespace) |
| `:wat::std::service::Console/loop` | `:wat::console::loop` |
| `:wat::std::service::Console/out` | `:wat::console::out` |
| `:wat::std::service::Console/err` | `:wat::console::err` |
| `:wat::std::service::Console/ack-at` | `:wat::console::ack-at` |
| `:wat::telemetry::Service::*` (typealiases on grouping noun) | `:wat::telemetry::*` (siblings at namespace level) |
| `:wat::telemetry::Service/spawn` (verb on grouping noun) | `:wat::telemetry::spawn` |
| `:wat::telemetry::Service/loop`, `/tick`, `/extend`, `/maybe`, `/drain`, `/run`, `/bump`, `/batch`, `/null`, `/pair`, `/ack` | flatten to `:wat::telemetry::<verb>` |
| `:wat::lru::CacheService` (grouping noun) + `/handle`, `/get`, `/put` | `:wat::lru::*` siblings + top-level verbs (e.g. `:wat::lru::get`, `:wat::lru::put`) — IF `CacheService` is a grouping noun, not a real struct |
| `:wat::holon::lru::HologramCacheService` ditto | `:wat::holon::lru::*` flattened, IF grouping |
| `:wat::std::service::Console::Tx` (typealias under grouping noun) | `:wat::console::Tx` (typealias at namespace level) |

(Each crate needs an audit pass: identify whether the central name
is a real struct or a grouping noun. Real structs keep their
`/methods`; grouping nouns flatten.)

### Through the four questions

- **Obvious:** every name in `:wat::<concept>::*` attaches the same
  way (`::`). One rule. PascalCase + position signals type-vs-verb.
- **Simple:** `::` always for path; `/` reserved exclusively for
  real Type-attached methods. The grammar tightens.
- **Honest:** `/` tells the truth about what's a real Type;
  grouping nouns admit they're just namespace labels.
- **UX:** call site like `(:wat::console::out handle msg)` reads
  like `(:wat::core::map f xs)` — same substrate-verb shape.

### Mental model — what `Type/method` IS and ISN'T

Captured 2026-05-01 during slice 1j console-rename design conversation.
Preserves the rationale § K's slice plan rests on so future readers
don't have to rederive it.

**The form** — `(:ns::Type/method instance args...)` — is UFCS
(Uniform Function Call Syntax, like Rust's `impl Type { fn method(self) }`),
NOT OOP. The "instance" is just the first positional arg. There is no
`self` binding, no `this`, no inheritance, no virtual dispatch, no
encapsulated mutable state semantics. The `Type/` prefix is a naming
discipline, not a dispatch mechanism.

Two equivalent readings of the same call:

```
(:wat::core::Bytes/to-hex bytes)        ; "method-style"
;; ↕ same machine code ↕
(<the-fn-named-Bytes/to-hex> bytes)     ; "function-style"
```

The function's first parameter can be named anything — `b`, `pool`,
`self` — it's a regular parameter. There's no privileged identifier.

**Honest `Type/method` cases (the only ones § K endorses):**

- **Method:** function takes a value of Type as its first arg
  - `(:wat::core::Bytes/to-hex b)` — takes a Bytes ✓
  - `(:wat::kernel::HandlePool/pop pool)` — takes a HandlePool ✓
- **Constructor:** function returns a value of Type (no instance arg)
  - `(:wat::core::Bytes/from-hex "deadbeef")` — returns Bytes ✓
  - `(:wat::telemetry::Stats/zero)` — returns Stats ✓

Anything else (a fn whose first arg is NOT Type-shaped AND that
doesn't return Type either) is lying about its name. § K's audit is
for catching these lies.

**Stateful instances — two flavors, same call shape:**

1. **Pure-value state (immutable, persistent).** The struct holds
   the state; "mutation" produces a new instance you bind. Same
   pattern as Clojure persistent maps or Haskell State monad.

   ```
   (:wat::telemetry::Stats/bump stats :a-cache-hit)
   ;; returns a NEW Stats with one counter incremented;
   ;; the old `stats` is untouched
   ```

   Examples: `Stats`, `Bytes`, `MetricsCadence`, `Failure`,
   `RunResult`.

2. **Handle state (wraps a mutable resource).** The struct is a
   stable identifier (an `Arc<...>` / a fd / a channel half); the
   resource it points at IS mutable. The handle stays the same
   across calls; what it refers to changes.

   ```
   (:wat::kernel::HandlePool/pop pool)
   ;; pool's underlying state changed; same `pool` handle still in scope
   ```

   Examples: `HandlePool`, `Sender<T>`, `Receiver<T>`, `Thread<I,O>`,
   `Process<I,O>`.

Both flavors fit `(Type/method instance args)`. The difference is
what "operating on the instance" means: pure-value methods compute
a new value (you discard the old); handle methods side-effect the
underlying resource (you keep the same handle).

**Encapsulation is namespace-driven, not language-feature-driven.**

If `:wat::telemetry::*` exposes `Stats/zero`, `Stats/bump`,
`Stats/snapshot` but NOT direct field accessors (`Stats/cache-hits`,
`Stats/cache-misses`), then Stats' fields are effectively private —
the namespace gatekeepers what's reachable. There's no `private`
keyword; you just don't `define` or vend the accessor.

So wat gets the OO-shaped surface (data + methods organized by
type, optional encapsulation) without the OOP machinery (no
inheritance, no dispatch, no `self`-binding, no method-table
overhead). User-direction summary (2026-05-01):

> i do not want object oriented programming.. but ... we need
> something close to it... (fn self args) is the form?..
>
> [confirmation: yes, that's UFCS]
>
> and instance can be stateful?.. some struct of whatever it holds?..
>
> [confirmation: yes, both pure-value and handle flavors]

If wat ever needs true polymorphism (one verb name, runtime
dispatch on operand type), that's § J's typeclass mechanism — a
**separate** mechanism that doesn't touch `Type/method` naming.
The two patterns coexist:

| Need | Form |
|---|---|
| "this function operates on this Type" | `Type/method` (UFCS naming; statically dispatched) |
| "this verb means different things on different concrete types satisfying a protocol" | `:wat::kernel::join-result` (poly verb; typeclass dispatch on `Program<I,O>` per § J) |

§ K's cleanup keeps `Type/method` honest by removing every site
where the LHS isn't a real Type. § J adds typeclass dispatch as a
parallel mechanism for the genuinely-polymorphic cases.

### Slice plan (rolled into 109's J-PIPELINE)

Each affected service crate is its own slice; substrate-as-teacher
discipline (Pattern 2 verb retirement on the OLD `Type/verb` heads,
hint helpers redirecting to the new namespace-level form):

- **Slice K.console** — `:wat::std::service::Console::*` flattens to
  `:wat::console::*`; verbs lose `Console/` prefix. Subsumes § 9e
  (file-path move from `wat/std/service/Console.wat` to
  `wat/console.wat`).
- **Slice K.telemetry** — `:wat::telemetry::Service::*` flattens to
  `:wat::telemetry::*`; `Service/spawn` etc. become bare
  `:wat::telemetry::spawn`. Real types under telemetry (`Stats`,
  `MetricsCadence`) keep their `/methods`.
- **Slice K.lru** — audit `:wat::lru::CacheService`. If grouping
  noun, flatten. If real struct (with field accessors / a value),
  keep `/methods`.
- **Slice K.holon-lru** — same audit + treatment for
  `:wat::holon::lru::HologramCacheService`.

### Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 2 verb retirement is
  the migration mechanism for each grouping-noun → namespace-level
  flatten.
- `docs/arc/2026/04/109-kill-std/SLICE-1J.md` — the precedent for
  Type/verb shape on real Types (`Result/try`, `Option/expect`).
  Slice K is the inverse: when there's NO real Type, the
  `/`-suggesting form retires.

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
