# Arc 143 — Reflection layer + `:wat::runtime::define-alias`

**Status:**
- drafted 2026-05-02 (evening) as the narrow `:wat::runtime::define-alias`
  arc surfaced from arc 130 slice 1 RELAND's `:reduce` gap
- SCOPE EXPANDED 2026-05-02 (evening) per user direction: the
  narrow 3-primitive design was insufficient. The substrate needs
  the full Ruby-style reflection surface. `define-alias` becomes
  ONE downstream consumer of a broader reflection foundation.
- **SCOPE EXPANDED AGAIN 2026-05-02 (late evening)** after slice 6's
  killed sweep surfaced TWO additional substrate gaps that block
  the macro layer:
  - `defmacro` bodies are pure quasiquote templates; arbitrary
    computation at expand-time is unsupported (verified at
    `src/macros.rs:614-622`: "this slice doesn't do arbitrary macro
    bodies"). Need: COMPUTED UNQUOTE — `,(expr)` inside a
    quasiquote evaluates `expr` at expand-time with macro params
    bound, then splices the result.
  - HolonAST has no wat-side structural decomposition primitives —
    only `:wat::holon::statement-length` (count, no iteration).
    Macros can't read Bundle children to manipulate signature heads.
    Need: HolonAST manipulation primitives in the substrate
    (rename-callable-name + extract-arg-names as Rust functions
    operating on HolonAST).

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
| `Module#instance_methods` / `class_methods` | `(:wat::runtime::all-defines)`, `(:wat::runtime::all-primitives)` |
| `Module.constants` | `(:wat::runtime::all-typealiases)`, `(:wat::runtime::all-structs)` |
| `Object#method(:foo)` | `(:wat::runtime::lookup-define :foo)` |
| `Method#parameters` / `#source_location` | `(:wat::runtime::signature-of :foo)`, `(:wat::runtime::origin-of :foo)` |
| `Method#owner` | (n/a — wat is flat-namespaced) |
| `Object#respond_to?(:foo)` | `(:wat::runtime::callable? :foo)`, `(:wat::runtime::defined? :foo)` |
| `Symbol.all_symbols` | `(:wat::runtime::all-symbols)` |

Ruby's `Module#define_method` is the imperative analog of
`define-alias` — both create a new method by referencing
existing metadata. wat's defmacro doing this at expand-time
parallels Ruby's metaprogramming idioms.

## The full primitive surface

### Enumeration (per-kind + union)

```
(:wat::runtime::all-defines)      -> :Vec<Symbol>
(:wat::runtime::all-macros)       -> :Vec<Symbol>
(:wat::runtime::all-primitives)   -> :Vec<Symbol>
(:wat::runtime::all-typealiases)  -> :Vec<Symbol>
(:wat::runtime::all-structs)      -> :Vec<Symbol>
(:wat::runtime::all-enums)        -> :Vec<Symbol>
(:wat::runtime::all-newtypes)     -> :Vec<Symbol>
(:wat::runtime::all-symbols)      -> :Vec<Symbol>   ;; union of the above
```

### Predicates (per-kind + cross-cutting)

```
;; Per-kind
(:wat::runtime::define?     <name>) -> :bool
(:wat::runtime::macro?      <name>) -> :bool
(:wat::runtime::primitive?  <name>) -> :bool
(:wat::runtime::typealias?  <name>) -> :bool
(:wat::runtime::struct?     <name>) -> :bool
(:wat::runtime::enum?       <name>) -> :bool
(:wat::runtime::newtype?    <name>) -> :bool

;; Cross-cutting
(:wat::runtime::callable?   <name>) -> :bool   ;; define | macro | primitive
(:wat::runtime::type?       <name>) -> :bool   ;; typealias | struct | enum | newtype
(:wat::runtime::defined?    <name>) -> :bool   ;; any kind
```

### Typed lookups (kind-specific AST shape)

```
(:wat::runtime::lookup-define     <name>) -> :Option<HolonAST>  ;; (define <head> <body>)
(:wat::runtime::lookup-macro      <name>) -> :Option<HolonAST>  ;; (defmacro <head> <template>)
(:wat::runtime::lookup-primitive  <name>) -> :Option<HolonAST>  ;; (define <head> <sentinel>)
(:wat::runtime::lookup-typealias  <name>) -> :Option<HolonAST>  ;; (typealias <name> <target>)
(:wat::runtime::lookup-struct     <name>) -> :Option<HolonAST>  ;; (struct <name> <fields>)
(:wat::runtime::lookup-enum       <name>) -> :Option<HolonAST>  ;; (enum <name> <variants>)
(:wat::runtime::lookup-newtype    <name>) -> :Option<HolonAST>  ;; (newtype <name> <target>)
```

### Cross-cutting projections (work on any callable)

```
(:wat::runtime::signature-of  <name>) -> :Option<HolonAST>   ;; head only, any callable
(:wat::runtime::body-of       <name>) -> :Option<HolonAST>   ;; body for defines/macros; :None for primitives
(:wat::runtime::origin-of     <name>) -> :Option<wat::Span>  ;; source location for any binding
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

## Slice plan (8 slices, dependency-ordered)

The third expansion (2026-05-02 late evening) follows the
user's principle: **build complexity up from simplicity
composition; the order is the dependency order**. Each slice
ships only after its dependencies are in place; parallel-safe
slices can ship in any order between their dependency layers.

```
LEVEL 0: existing substrate (registries, slice 1 primitives, macro expander)

LEVEL 1 (parallel; both extend substrate):
  - Slice 2:  Computed unquote in defmacro bodies
              (extends macro expander)
  - Slice 3:  HolonAST manipulation primitives
              (rename-callable-name, extract-arg-names)
              (extends Rust dispatch)

LEVEL 1 (parallel; breadth for arc 143 closure; not on
critical path for define-alias):
  - Slice 4:  Form registry exposure
              (enumeration / predicate / non-callable lookups)
  - Slice 5:  origin-of + span discipline + FQDN rendering

LEVEL 2 (depends on slice 1 ✓ + slices 2 + 3):
  - Slice 6:  :wat::runtime::define-alias defmacro (pure wat)

LEVEL 3 (depends on slice 6):
  - Slice 7:  Apply (:define-alias :reduce :foldl)
              + verify arc 130 stepping stone transitions

LEVEL 4 (closure; depends on all):
  - Slice 8:  INSCRIPTION + 058 row + USER-GUIDE
```

### Slice 1 — point lookups for callables ✅ SHIPPED

Three substrate primitives: `lookup-define`, `signature-of`,
`body-of`. 392 LOC Rust + 11 tests, sweep `a1f16ee3496885ab8`,
Mode A clean ship per SCORE-SLICE-1.md.

### Slice 2 — Computed unquote in defmacro bodies (NEXT — orchestrator)

Extend `expand_template` in `src/macros.rs` so `,(expr)` and
`,@(expr)` inside a quasiquote evaluate `expr` at expand-time
with macro params bound, then splice the result.

Currently (per `src/macros.rs:614-622`) the body MUST be
`(quasiquote X)` AND inside the quasiquote, `,name` substitutes
a parameter (only). Arbitrary `,(expression)` is not supported.

The change: when `expand_template` walks the quasiquote and
encounters an unquote whose argument is NOT a bare parameter
keyword, evaluate the argument as a wat expression with the
macro's bindings as the symbol table, expecting the result to
be a `Value::wat__WatAST` or `Value::holon__HolonAST` that
splices in.

~30-60 LOC Rust + 4-6 tests.

**Orchestrator does this directly** (sonnet hit its limit on
slice 6; substrate work returns to the orchestrator).

### Slice 3 — HolonAST manipulation primitives (parallel with slice 2; orchestrator)

Two new substrate primitives:

- `:wat::runtime::rename-callable-name (head :HolonAST) (from :keyword) (to :keyword) -> :HolonAST` —
  substitute the function name in a signature head. Splits the
  first symbol on `<` (preserving the type-params suffix);
  replaces the name part; rebuilds the head.
- `:wat::runtime::extract-arg-names (head :HolonAST) -> :Vec<keyword>` —
  walks the head's children (HolonAST::Bundle); skips the first
  symbol; filters `(arg-name :type)` Bundle pairs (skipping
  `->` and the trailing return-type symbol); extracts the first
  element of each.

These exist as Rust primitives because: (a) wat lacks
HolonAST structural decomposition primitives (`statement-length`
counts but doesn't iterate); (b) the rename needs string surgery
on a keyword string. Adding wat-side iteration + string primitives
would be a larger arc; the substrate primitives encapsulate the
manipulation cleanly.

~80-120 LOC Rust + 4-6 tests.

### Slice 4 — Form registry exposure (parallel; arc 143 breadth)

The full Ruby-style reflection surface beyond callables. Ships
the enumeration primitives (`:all-defines`, `:all-macros`,
`:all-primitives`, `:all-typealiases`, `:all-structs`,
`:all-enums`, `:all-newtypes`, `:all-symbols`), predicate
primitives (`:define?`, `:macro?`, etc., plus cross-cutting
`:callable?`, `:type?`, `:defined?`), and typed lookups for
non-callables (`:lookup-macro`, `:lookup-typealias`,
`:lookup-struct`, `:lookup-enum`, `:lookup-newtype`,
`:lookup-primitive`).

NOT on the critical path for `define-alias` — that macro only
needs slice 1's `signature-of`. This slice ships the breadth
required for arc 143 closure (per the user's "all symbols"
framing) but doesn't block slices 6/7.

~250-400 LOC Rust + 25-35 tests across all primitives.

### Slice 5 — origin-of + span discipline + FQDN rendering (parallel)

Three combined improvements:

1. Add `define_span: Span` to `Function`. Populate at
   `parse_define_form` time. Use it for the synthesised head
   spans in slice 1's helpers (replacing `Span::unknown()`).
2. Add `register_span: Option<Span>` to `TypeScheme`. Capture
   `Span::new(Arc::new(file!().into()), line!() as i64, 0)` at
   each primitive registration site. Use for synthesised head
   spans for substrate primitives.
3. Add `:wat::runtime::origin-of <name> -> :Option<Span>` primitive.
4. Fix the FQDN rendering concern from SCORE-SLICE-1.md
   (synthesised AST renders `:Vec<T>` not `:wat::core::Vec<T>`).
   Update `format_type` (or add a `format_type_fqdn` variant)
   to resolve bare primitive names to their canonical FQDN.
   The TypeScheme registry's bare names may be a pre-existing
   arc 109 inconsistency worth fixing here.

~80-150 LOC Rust + 6-10 tests.

### Slice 6 — `:wat::runtime::define-alias` defmacro

Pure wat. Lives in `wat/runtime.wat` (NEW file — sets the precedent
for wat-defined `:wat::core::*` macros, mirroring how
`wat/test.wat` mixes substrate primitives + wat-defined macros
under `:wat::test::*`).

```scheme
(:wat::core::defmacro
  (:wat::runtime::define-alias
    (alias-name :AST<wat::core::keyword>)
    (target-name :AST<wat::core::keyword>)
    -> :AST<wat::core::unit>)
  `(:wat::core::define
     ,(:wat::runtime::rename-callable-name
        (:wat::core::Option/expect -> :wat::holon::HolonAST
          (:wat::runtime::signature-of target-name)
          "define-alias: target name not found")
        target-name
        alias-name)
     (,target-name ,@(:wat::runtime::extract-arg-names
                        (:wat::runtime::signature-of target-name)))))
```

Depends on:
- Slice 1's `signature-of` ✅ (shipped)
- Slice 2's computed unquote (`,(expr)` evaluates at expand-time)
- Slice 3's `rename-callable-name` + `extract-arg-names`

~15 LOC wat + 2-3 expansion tests.

`wat/runtime.wat` is loaded via `src/stdlib.rs` like the other
top-level wat files (`test.wat`, `console.wat`, etc.).

### Slice 7 — Apply (:define-alias :wat::list::reduce :wat::core::foldl)

ONE LINE in **`wat/list.wat`** (NEW top-level file; semantic
placement — `reduce` is a list operation; lives adjacent to
fold even while fold still lives under `:wat::core::*`):

```scheme
(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)
```

Per the user's planned `:wat::list::*` namespace move (arc 109
wind-down direction: "we need to move things to `:wat::list::*`
then we can mirror that stuff for lazy seqs"). The eventual
`:wat::core::foldl → :wat::list::foldl` rename in a follow-on
arc updates the alias's TARGET without touching the alias's
NAME.

The clean separation:
- `wat/runtime.wat` — the `:wat::runtime::define-alias` macro
  (runtime-discovery construct)
- `wat/list.wat` — the application + future list-related alias
  accumulations (semantic domain)

ALSO update arc 130's substrate call sites to use
`:wat::list::reduce`:
- `crates/wat-lru/wat/lru/CacheService.wat:213`
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:251`

Verify: re-run `cargo test --release --workspace`. The arc 130
RELAND v1 stepping stone test
(`deftest_wat_lru_test_lru_raw_send_no_recv`, currently failing
with "unknown function: :wat::core::reduce") TRANSITIONS — either
passes (substrate's CacheService Get path resolves `:wat::list::reduce`
and runs to completion), OR fails differently (e.g.,
reply-tx-disconnected — a different arc 130 stepping stone
still on the chain; even that transition is the diagnostic value
of slice 7).

### Slice 5c — REASSIGNED to arc 144

The slice 5c originally planned for arc 143 (register TypeScheme
entries for hardcoded `infer_*` primitives so `signature-of` finds
them) was elevated 2026-05-02 (late evening) per the user's
"nothing is special — `(help :if) /just works/`" principle into
**arc 144 — uniform reflection foundation**. Arc 144 ships the
unified `Binding` enum (UserFunction / Macro / Primitive /
SpecialForm / Type) + `lookup-form` + the special-form registry +
TypeScheme registrations for the 15 hardcoded callable primitives
+ paved-road `:doc-string: Option<String>` field for arc 141.

The slice 6 length test (`define_alias_length_to_user_size_delegates_correctly`)
stays red as a known-defect canary that arc 144's slice 4 turns
green. Documented up-front in INSCRIPTION + arc 144 DESIGN.

### Slice 8 — closure (THIS — shipping)

INSCRIPTION + 058 row + USER-GUIDE entry + arc 144 hand-off.
Cross-references to arc 091 slice 8 (quasiquote precedent), arc
057 (HolonAST polymorphism), arc 037 (introspection precedent),
arc 138 (span coordinates the discipline upgrades), arc 109 (the
canonical-namespace discipline this arc honors by placing the
new macro in `wat/runtime.wat`), arc 144 (the follow-on arc that
generalizes the reflection layer + closes the slice 6 length
canary), `docs/COMPACTION-AMNESIA-RECOVERY.md` (the protocol
forged mid-arc).

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
`(:wat::runtime::lookup name) -> :Option<Binding>` where `Binding`
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

### Q7 — Placement of reflection primitives in stdlib (RESOLVED)

**RESOLVED 2026-05-02 (late evening) per user direction.**

`wat/std/` is being killed by arc 109 (the namespace is being
phased out; arc 109 is ~90% there per user). NO new files in
`wat/std/`.

The wat-side `define-alias` macro lives in **`wat/runtime.wat`**
(NEW top-level file). This sets the precedent for wat-defined
`:wat::core::*` macros, mirroring how `wat/test.wat` mixes
substrate primitives + wat-defined macros under `:wat::test::*`.

`wat/runtime.wat` registers in `src/stdlib.rs` like the other
top-level wat files (test.wat, console.wat, edn.wat, holon.wat,
stream.wat).

The substrate query primitives (slices 1, 3, 4) register in
Rust dispatch as before — they're substrate primitives that
happen to be in the `:wat::core::*` namespace.

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
