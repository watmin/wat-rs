# Arc 143 — Reflection layer + `:wat::core::define-alias`

**Status:**
- drafted 2026-05-02 (evening) as the narrow `:wat::core::define-alias`
  arc surfaced from arc 130 slice 1 RELAND's `:reduce` gap
- **SCOPE EXPANDED 2026-05-02 (evening)** per user direction: the
  narrow 3-primitive design was insufficient. The substrate needs
  the full Ruby-style reflection surface. `define-alias` becomes
  ONE downstream consumer of a broader reflection foundation.

**The architectural framing the user landed:**

> *"we need to be able to query /all/ symbols that are
> registered... the users can filter out what they want.. we can
> recall every definition... think ruby... we can ask all the
> questions we want about symbols in the environment... all
> modules... all classes.. all functions on them... all methods...
> instance and class methods... all the constants... we must have
> an identical query interface.. yes?... we must be able to query
> our env to do macro work proper..."*

The wat substrate is closed under macro + AST construction. It
lacks the ability to OBSERVE the runtime's own symbol table. This
arc closes that gap with a Ruby-analog reflection surface — the
primitives that let userland macros enumerate + lookup + test +
inspect any binding, regardless of kind.

`(:define-alias :reduce :foldl)` becomes the FIRST USER of this
foundation. Every future reflection-driven macro (sweep generators,
spec validators, doc extractors) sits on top of the same surface.

## Why this isn't a "v2" deferral

Per the v1 wind-down: "all arcs after 109 must be implemented
without deferral." Arc 143 surfaced from arc 130's reland (which
is part of arc 109's chain). The reduce gap blocks arc 130.
Closing it via Rust-side aliasing (the 5-line patch I originally
proposed) addresses the symptom. Closing it via the full
reflection foundation addresses the structural absence and
unlocks every future reflection-driven userland macro.

The bias to reach for `reduce` was the empirical signal. The
absence of reflection was the deeper diagnosis.

## The Ruby analog

| Ruby | wat |
|---|---|
| `Module#instance_methods` / `class_methods` | `(:wat::core::all-defines)`, `(:wat::core::all-primitives)` |
| `Module.constants` | `(:wat::core::all-typealiases)`, `(:wat::core::all-structs)` |
| `Object#method(:foo)` | `(:wat::core::lookup-define :foo)` |
| `Method#parameters` / `#source_location` | `(:wat::core::signature-of :foo)`, `(:wat::core::origin-of :foo)` |
| `Method#owner` | (n/a — wat is flat-namespaced) |
| `Object#respond_to?(:foo)` | `(:wat::core::callable? :foo)`, `(:wat::core::defined? :foo)` |
| `Symbol.all_symbols` | `(:wat::core::all-symbols)` |

Ruby's `Module#define_method` is the imperative analog of
`define-alias` — both create a new method by referencing
existing metadata. wat's defmacro doing this at expand-time
parallels Ruby's metaprogramming idioms.

## The full primitive surface

### Enumeration (per-kind + union)

```
(:wat::core::all-defines)      -> :Vec<Symbol>
(:wat::core::all-macros)       -> :Vec<Symbol>
(:wat::core::all-primitives)   -> :Vec<Symbol>
(:wat::core::all-typealiases)  -> :Vec<Symbol>
(:wat::core::all-structs)      -> :Vec<Symbol>
(:wat::core::all-enums)        -> :Vec<Symbol>
(:wat::core::all-newtypes)     -> :Vec<Symbol>
(:wat::core::all-symbols)      -> :Vec<Symbol>   ;; union of the above
```

### Predicates (per-kind + cross-cutting)

```
;; Per-kind
(:wat::core::define?     <name>) -> :bool
(:wat::core::macro?      <name>) -> :bool
(:wat::core::primitive?  <name>) -> :bool
(:wat::core::typealias?  <name>) -> :bool
(:wat::core::struct?     <name>) -> :bool
(:wat::core::enum?       <name>) -> :bool
(:wat::core::newtype?    <name>) -> :bool

;; Cross-cutting
(:wat::core::callable?   <name>) -> :bool   ;; define | macro | primitive
(:wat::core::type?       <name>) -> :bool   ;; typealias | struct | enum | newtype
(:wat::core::defined?    <name>) -> :bool   ;; any kind
```

### Typed lookups (kind-specific AST shape)

```
(:wat::core::lookup-define     <name>) -> :Option<HolonAST>  ;; (define <head> <body>)
(:wat::core::lookup-macro      <name>) -> :Option<HolonAST>  ;; (defmacro <head> <template>)
(:wat::core::lookup-primitive  <name>) -> :Option<HolonAST>  ;; (define <head> <sentinel>)
(:wat::core::lookup-typealias  <name>) -> :Option<HolonAST>  ;; (typealias <name> <target>)
(:wat::core::lookup-struct     <name>) -> :Option<HolonAST>  ;; (struct <name> <fields>)
(:wat::core::lookup-enum       <name>) -> :Option<HolonAST>  ;; (enum <name> <variants>)
(:wat::core::lookup-newtype    <name>) -> :Option<HolonAST>  ;; (newtype <name> <target>)
```

### Cross-cutting projections (work on any callable)

```
(:wat::core::signature-of  <name>) -> :Option<HolonAST>   ;; head only, any callable
(:wat::core::body-of       <name>) -> :Option<HolonAST>   ;; body for defines/macros; :None for primitives
(:wat::core::origin-of     <name>) -> :Option<wat::Span>  ;; source location for any binding
```

~25 primitives total. Each is small (registry iteration or
HashMap lookup wrapped as `eval_*` + check.rs scheme
registration). Per-primitive cost is low; cumulative
substrate weight matters and is justified by the unlocked
userland macro surface.

## Findings (open questions resolved 2026-05-02 evening)

### Q1 — User-define AST preservation: RESOLVED

`Value::wat__core__lambda(Arc<Function>)` per `runtime.rs:158`.
The `Function` struct (line 499) carries everything needed:

```rust
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,           // ARG NAMES preserved
    pub type_params: Vec<String>,
    pub param_types: Vec<TypeExpr>,
    pub ret_type: TypeExpr,
    pub body: Arc<WatAST>,             // BODY preserved
    pub closed_env: Option<Environment>,
}
```

For a user-defined function, all callable-projecting
primitives reconstruct trivially.

Lookup path: `Environment::lookup(name)` returns
`Option<Value>` per `runtime.rs:563`.

### Q2 — Arg name preservation in TypeSchemes: RESOLVED via synthesis

TypeScheme has no param names. For substrate primitives,
synthesize: `:_a0`, `:_a1`, ..., `:_a<n-1>`. Generated
alias bodies use synthetic names — type-checks identically;
cosmetic loss only.

Lookup path: `env.schemes.get(name)` per `check.rs:1149-1150`.
Substrate primitives are NOT in `Environment` (env.lookup
returns None) — they live in the TypeScheme registry.

### Q3 — Span discipline (NEW, raised 2026-05-02 evening)

`Span::unknown()` for synthesized AST nodes ACTIVELY undermines
arc 138 (errors carry coordinates). The right discipline:

- **For user defines**: `Function` gains `pub define_span: Span`
  populated at `parse_define_form` time. Synthesized head AST
  uses this span. Body keeps native spans.
- **For substrate primitives**: TypeScheme gains
  `pub register_span: Option<Span>`. Each primitive registration
  captures `Span::new(Arc::new(file!().into()), line!() as i64, 0)`
  via Rust's `file!()` / `line!()` macros at the call site.
- **For other kinds (typeliases / structs / enums / newtypes)**:
  same pattern — preserve registration span at parse time;
  thread it through to all reflection projections.

This is **slice 5** of the arc (origin-of + span discipline
fix together). Slice 1 (currently in flight) ships with
`Span::unknown()` per its existing brief; slice 5 retroactively
upgrades the helpers to use real spans.

## Resolution-order semantics

For each callable's lookup (lookup-define, signature-of,
body-of):

1. **First**: check `Environment::lookup(name)` for a user define
2. **Then**: check `env.schemes.get(name)` for a substrate primitive
3. **Else**: return `:None`

User defines shadow primitive registrations (matches normal
call dispatch precedence).

For typed lookups (`lookup-macro`, `lookup-typealias`, etc.),
each consults the registry that owns its kind:

- `lookup-macro` → `MacroRegistry` (`src/macros.rs`)
- `lookup-typealias` → typealias registry (likely in `check.rs` —
  TBD slice 4 investigation)
- `lookup-struct` / `lookup-enum` / `lookup-newtype` → similar

For predicates: each just runs the corresponding `lookup-X` and
checks `Some` vs `None`.

For enumerations: each iterates its corresponding registry's
keys.

## Slice plan (7 slices)

### Slice 1 — point lookups for callables (IN FLIGHT)

Three substrate primitives: `lookup-define`, `signature-of`,
`body-of`. ~80-150 LOC Rust + 9-12 tests. Sonnet sweep launched
2026-05-02 evening with BRIEF-SLICE-1 + EXPECTATIONS-SLICE-1.

NOTE: this slice ships with `Span::unknown()` per its brief;
slice 5 retroactively upgrades the span discipline.

### Slice 2 — enumeration primitives

Eight `:wat::core::all-X` primitives. Each iterates its
registry's keys and returns `Vec<Symbol>`. The `all-symbols`
union concatenates the seven kind-specific lists.

Mechanical to ship together (they share registry-iteration
machinery; differ only by which registry they walk).

~50-100 LOC Rust + 8-10 tests.

### Slice 3 — predicate primitives

Ten `:wat::core::X?` primitives. Each is a 1-line wrapper over
the corresponding registry's `contains_key` (or `.get(name).is_some()`).
The cross-cutting predicates (callable?, type?, defined?) compose
the per-kind ones.

~30-60 LOC Rust + 10-12 tests.

### Slice 4 — typed lookups for non-callables

Six `:wat::core::lookup-X` primitives for macros, typealiases,
structs, enums, newtypes, primitives. Each reconstructs the
kind-appropriate AST from its registry entry.

Investigation: do typealiases / structs / enums / newtypes
preserve their original AST? Likely yes (similar to
`Function.body`); confirm during slice 4 implementation.

~80-150 LOC Rust + 6-12 tests.

### Slice 5 — `origin-of` + span discipline fix

Add `define_span: Span` to `Function`. Add `register_span:
Option<Span>` to `TypeScheme`. Update `parse_define_form` to
populate; update primitive registrations to capture
`file!()` / `line!()`. Update slice 1's helpers to use real
spans. Add `:wat::core::origin-of <name> -> :Option<wat::Span>`
primitive.

Retroactively closes the diagnostic gap I introduced in slice 1.

~50-100 LOC Rust + 5-8 tests.

### Slice 6 — wat-side define-alias

Three pieces in one slice (small enough):

1. wat helpers `:rename-callable-name` + `:extract-arg-names`
   in `wat/std/ast.wat`. ~40 LOC of HolonAST manipulation.
2. `:wat::core::define-alias` defmacro. ~10 LOC atop slice 1's
   primitives.
3. Apply: `(:wat::core::define-alias :wat::core::reduce
   :wat::core::foldl)` in `wat/std/ast.wat` (or `wat/core.wat`).

Verify arc 130's substrate stops failing on the missing-reduce
gap (re-run cargo test --workspace post-application).

~70 LOC wat + 5-8 tests.

### Slice 7 — closure

INSCRIPTION + 058 row + USER-GUIDE entry. Cross-references to
arc 091 slice 8 (quasiquote precedent), arc 057 (HolonAST
polymorphism), arc 037 (introspection precedent), arc 138
(span coordinates the discipline upgrades).

## The four questions (against the expanded scope)

**Obvious?** Yes. Each primitive's name names exactly what it
does. The Ruby analogs are universally familiar; readers from
Clojure / JS / Python / Lisp / Ruby all read `(:all-defines)`
and `(:lookup-define name)` fluently.

**Simple?** The aggregate is substantial (~25 primitives) but
each is tiny (registry walk OR HashMap lookup). The Function
struct addition + TypeScheme addition are surgical. The pattern
is uniform across kinds — write one, the rest mirror it.

**Honest?** Yes. The reflection surface mirrors what the
substrate ACTUALLY holds — defines, macros, primitives,
typealiases, structs, enums, newtypes. No invented categories.
No glossed-over inconsistencies. The synthesized AST for
substrate primitives uses honest sentinel bodies + synthetic
arg names; the limitations are visible.

**Good UX?** Yes. Userland macros become first-class consumers
of the env. `define-alias` is the immediate beneficiary; the
next dozen reflection-driven macros (sweep generators, spec
validators, doc extractors) get the foundation for free.

## Open questions

### Q4 — Unified `Binding` sum type + universal `lookup`?

In addition to the typed lookups, should there be a unified
`(:wat::core::lookup name) -> :Option<Binding>` where `Binding`
is a sum type covering all kinds?

```scheme
(:wat::core::enum :wat::core::Binding
  (Define     (signature :HolonAST) (body :HolonAST))
  (Macro      (signature :HolonAST) (template :HolonAST))
  (Primitive  (signature :HolonAST))
  (Typealias  (target :HolonAST))
  (Struct     (fields :HolonAST))
  (Enum       (variants :HolonAST))
  (Newtype    (target :HolonAST)))
```

Pros: macros that don't know the kind in advance can dispatch
on the variant. Mirrors Ruby's `Object#method` + `Method` class.

Cons: another sum type to maintain alongside the typed lookups.
Most macro use-cases are kind-specific.

**Defer to slice 4 implementation.** If the typed lookups
suffice for `define-alias`, defer the unified `Binding` to a
post-v1 arc. If unified `lookup` becomes load-bearing for some
slice's wat code, add it then.

### Q5 — Should `define-alias` work for macros too?

`(:define-alias :my-macro :their-macro)` — alias one defmacro
to another. Mechanically possible via the same approach (look
up the source macro, generate a defmacro that delegates).

**Defer.** Slice 6 ships callable-aliasing only. A future arc
adds `define-macro-alias` (or extends `define-alias` to
dispatch on kind) if the bias surfaces.

### Q6 — Sibling `:typealias-alias` for type names?

`typealias` already exists; aliasing typealiases is just
`(:typealias :NewName ExistingType)`. The asymmetry between
callables (macro-needed) and types (one-line wat) is honest.

**Don't ship.** Typealiases don't need a defalias macro.

### Q7 — Placement of reflection primitives in stdlib

Where does `:wat::core::all-X` etc. live in wat-side code?

- `wat/std/ast.wat` (new file) — sibling of `wat/std/option.wat`
- `wat/core.wat` (extend existing core stdlib)

**Defer to slice 6 implementation.** The substrate primitives
register in Rust regardless; the wat-side `define-alias` macro
+ helpers need a home. Probably `wat/std/ast.wat`.

## Why this scope is right for v1

The user's wind-down rule: arc 109 v1 doesn't close until all
post-109 arcs implement (no deferrals). The reflection layer
is needed for `define-alias`; `define-alias` unblocks arc 130;
arc 130 unblocks arc 109; arc 109 v1 ships.

Each slice is small enough for one sonnet sweep. The 7-slice
arc is comparable in scope to arc 091 (8 slices), arc 109
(many sub-slices), arc 138 (multiple F-NAMES sub-slices).

The substrate gains a foundational capability (full reflection)
that pays back across years of future macro work. The cost-
benefit favors shipping the proper foundation over the
narrow patch.

## Cross-references

- `docs/arc/2026/04/091-batch-as-protocol/INSCRIPTION.md`
  (slice 8 — quasiquote + struct→form, the partial reflection
  precedent)
- `docs/arc/2026/04/057-holon-ast-polymorphism/INSCRIPTION.md`
  (HolonAST as wat-readable AST representation)
- `docs/arc/2026/04/037-dim-router/INSCRIPTION.md` (arc 037
  intro of `statement-length` as introspection primitive
  precedent)
- `docs/arc/2026/05/138-errors-carry-coordinates/INSCRIPTION.md`
  (the span discipline this arc's slice 5 honors)
- `docs/arc/2026/05/130-cache-services-pair-by-index/SCORE-SLICE-1-RELAND.md`
  (the reland that surfaced the reduce gap → motivated this arc)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/058-031-defmacro/PROPOSAL.md`
  (defmacro definition)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/058-032-typed-macros/PROPOSAL.md`
  (typed macros — every defmacro param :AST<T>)
- `wat/test.wat:387+` (`:wat::test::make-deftest` — the worked
  example of macros-building-macros via quasiquote)
- `src/macros.rs` (MacroDef, MacroRegistry; parse_defmacro_signature)
- `src/runtime.rs:499` (Function struct — Q1 resolution)
- `src/check.rs:11200+` (TypeScheme registration — Q2 resolution)

## What's next

Slice 1 in flight (sonnet, ~15-25 min predicted). After SCORE,
slice 2 BRIEF + EXPECTATIONS gets drafted against slice 1's
calibration; same pattern through slice 7.

Arc 130 slice 1 RELAND v2 picks up at Layer 2+ once arc 143
slice 6 ships the `:reduce` alias.

Arc 109 v1 closure waits on the chain.
