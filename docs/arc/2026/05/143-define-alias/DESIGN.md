# Arc 143 — `:wat::core::define-alias` — close the homoiconic reflection loop for callables

**Status:** drafted 2026-05-02 (evening). Surfaced from arc 130
slice 1 RELAND's diagnostic at Layer 1: substrate code reaches
for `:wat::core::reduce`, gets "unknown function." `reduce` is
empirically the cognitive default for the fold-left pattern;
two independent code-writes (LRU + HolonLRU substrate Get
branches) both reached for it. The signature is identical to
`:wat::core::foldl`'s. The bias is data; the absence is a
substrate gap.

**Goal:** ship `(:wat::core::define-alias :wat::core::reduce :wat::core::foldl)`
as a working wat-side primitive. The form expands at macro-time
into a fresh `:wat::core::define` whose head copies the target's
signature with the alias name substituted, and whose body
delegates to the target. The checker re-parses the new head
into a fresh TypeScheme. No TypeScheme copying. Pure
homoiconic AST manipulation.

The arc closes a structural absence the substrate has had since
macros shipped: the inability for wat-side code to ASK
"what's the signature of this named callable?" The macro
substrate has been able to BUILD new defines (via quasiquote)
but not to LOOK UP existing signatures.

## What's there

Verified via filesystem crawl 2026-05-02 evening:

- **`:wat::core::defmacro`** — substrate special form
  (`runtime.rs:15077`). Defmacros store name, params,
  rest_param, body (WatAST), span (`macros.rs:54`). Registry
  `HashMap<String, MacroDef>` (`macros.rs:79`).
- **`:wat::core::quote`** / **`:wat::core::quasiquote`** —
  capture and template AST (`runtime.rs:2406-2407, 5736+`).
  Quasiquote nesting depth handled.
- **`:wat::core::struct->form`** — converts a struct value to
  its WatAST representation (`runtime.rs:5891+`). PARTIAL
  reflection: works for structs.
- **`:wat::holon::leaf`** / **`:wat::holon::from-watast`** —
  HolonAST construction primitives (arc 057 / 065).
- **Typed macros** — every defmacro parameter is `:AST<T>`
  with concrete T per 058-032 (lab proposal). The substrate
  enforces this at parse-time (`macros.rs::parse_defmacro_signature`).
- **Macros-that-build-macros precedent**: `wat::test::make-deftest`
  in `wat/test.wat:387+` is a defmacro whose body is
  ```scheme
  `(:wat::core::defmacro
     (,name ...) `...)
  ```
  Quasiquote splices the user-passed name into a generated
  defmacro registration.
- **AST-introspection precedent**: `statement-length` (arc
  037) operates on AST values. INSCRIPTION 224: "the
  introspection primitive."

## What's missing

Verified via filesystem crawl — these primitives DO NOT exist:

- `:wat::core::lookup-define`
- `:wat::core::signature-of`
- `:wat::core::body-of`
- `:wat::core::source-of`
- Any macro/binding inspection from wat code

The macro substrate can BUILD new forms via quasiquote but
cannot LOOK UP existing forms by name. The inspection loop
is half-closed: forward (build) works; backward (introspect)
doesn't.

## The narrow target

```scheme
(:wat::core::define-alias :wat::core::reduce :wat::core::foldl)
```

Macro-expands to:

```scheme
(:wat::core::define
  (:wat::core::reduce<T,Acc>
    (xs :Vec<T>) (init :Acc) (f :fn(Acc,T)->Acc) -> :Acc)
  (:wat::core::foldl xs init f))
```

The checker processes the new define normally — re-parses the
head into a fresh TypeScheme that's identical to `foldl`'s.
At runtime, calls to `:wat::core::reduce` evaluate the body
which delegates to `foldl`. Same dispatch path; +1 stack
frame (could be optimized away later via a tail-call or
direct alias).

## The substrate addition

ONE primitive: `:wat::core::signature-of`.

```
(:wat::core::signature-of <name :Symbol>) -> :Option<HolonAST>
```

Returns the signature HEAD as a HolonAST node:
`(name<TypeParams> (arg-name :ArgType) ... -> :ReturnType)`.

For a USER-DEFINED `:wat::core::define`, returns the head
extracted from the stored AST (the head is `(define <head>
<body>)`'s second element).

For a SUBSTRATE PRIMITIVE (Rust-implemented like `foldl`),
synthesizes the head from the registered TypeScheme. The
TypeScheme already carries name + parameter types + return
type; we materialize them as a HolonAST signature node.

For a name with no binding, returns `:None`.

This is the minimal reflection primitive. It does NOT expose
function bodies (no `body-of`); it does NOT expose macro
templates (no `template-of`). Just signatures. That's
sufficient for `define-alias` and for any future macro that
wants to wrap or re-export a callable.

## The wat-side macro

```scheme
(:wat::core::defmacro
  (:wat::core::define-alias
    (alias-name :AST<wat::core::Symbol>)
    (target-name :AST<wat::core::Symbol>)
    -> :AST<wat::core::unit>)
  (:wat::core::let*
    (((sig :wat::core::Option<wat::holon::HolonAST>)
      (:wat::core::signature-of target-name))
     ((renamed :wat::holon::HolonAST)
      (:rename-callable-name sig target-name alias-name))
     ((arg-names :wat::core::Vector<wat::core::Symbol>)
      (:extract-arg-names sig)))
    `(:wat::core::define
       ,renamed
       (,target-name ,@arg-names))))
```

Two helper functions written in wat (~20 LOC each):

- `:rename-callable-name (head :HolonAST) (from :Symbol) (to :Symbol) -> :HolonAST` —
  substitute the callable name in the signature head.
- `:extract-arg-names (head :HolonAST) -> :Vec<Symbol>` —
  return the arg-name symbols from the head's
  `(arg-name :Type)` pairs.

Both are pure HolonAST manipulation atop existing
primitives.

## Findings — open questions resolved 2026-05-02 evening

Filesystem investigation closed Q1 + Q2 before slice 1 brief drafted.

### Q1 — User-define AST preservation: RESOLVED

`Value::wat__core__lambda(Arc<Function>)` per `runtime.rs:158`.
The `Function` struct (line 499) carries everything we need:

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

For a user-defined function, all three primitives reconstruct
trivially:

- `signature-of`: build `(name<type_params> (params[i] :param_types[i]) ... -> :ret_type)`
- `body-of`: return `body` directly
- `lookup-define`: build `(:define <head> <body>)`

Lookup path: `Environment::lookup(name)` returns
`Option<Value>` per `runtime.rs:563`. If `Some(Value::wat__core__lambda(f))`,
reconstruct from `f`.

### Q2 — Arg name preservation in TypeSchemes: RESOLVED via synthesis

TypeScheme (per `check.rs:11200+ foldl registration`) has:

```rust
TypeScheme {
    type_params: Vec<String>,
    params: Vec<TypeExpr>,    // TYPES ONLY, no names
    ret: TypeExpr,
}
```

No param names. For substrate primitives, the three primitives
synthesize names: `:_a0`, `:_a1`, ..., `:_a<n-1>`. The resulting
alias body uses the same synthetic names; the generated define
type-checks identically to the pretty version. Cosmetic loss only.

Future polish (not this arc): extend TypeScheme with
`Option<Vec<String>>` param names; primitives that want pretty
aliases register with names. Out of scope.

Lookup path: `env.schemes.get(name)` per `check.rs:1149-1150`.
Substrate primitives are NOT in `Environment` (env.lookup returns
None) — they live in the TypeScheme registry.

## Resolution-order semantics

For each primitive's lookup:

1. **First**: check `Environment::lookup(name)` for a user define
2. **Then**: check `env.schemes.get(name)` for a substrate primitive
3. **Else**: return `:None`

User defines shadow primitive registrations (matches normal call
dispatch precedence).

## Slices (revised post-investigation)

### Slice 1 — three substrate primitives (lookup-define, signature-of, body-of)

Combined slice — they share lookup machinery + AST construction
helpers. Mechanical to ship together.

- Add three `eval_*` functions in `runtime.rs`:
  - `eval_lookup_define(args, env, sym) -> Result<Value::holon__HolonAST, _>`
  - `eval_signature_of(args, env, sym)` — same shape
  - `eval_body_of(args, env, sym)` — same shape, returns `:None`
    for substrate primitives
- Helper: `fn function_to_define_ast(f: &Function) -> WatAST` —
  reconstructs `(:define <head> <body>)` from a user Function.
- Helper: `fn type_scheme_to_signature_ast(name: &str, scheme: &TypeScheme) -> WatAST` —
  synthesizes head with `:_aN` names from a TypeScheme.
- Register all three in:
  - Runtime dispatch (`runtime.rs` near line 2406+)
  - Check.rs schemes (each takes `:Symbol -> :Option<HolonAST>`)
- 6-9 unit tests via `wat-tests/` — for each primitive:
  - User define lookup → returns expected AST
  - Substrate primitive lookup → returns synthesized AST
  - Unknown name → returns `:None`
  - body-of for substrate primitive → returns `:None`

### Slice 2 — wat helpers `:rename-callable-name` + `:extract-arg-names`

- New file: `wat/std/ast.wat`
- `:rename-callable-name (head :HolonAST) (from :Symbol) (to :Symbol) -> :HolonAST` —
  substitute the callable name in the signature head.
- `:extract-arg-names (head :HolonAST) -> :Vec<Symbol>` —
  return arg-name symbols from `(arg-name :Type)` pairs.
- ~40 LOC of HolonAST manipulation.
- 2-3 unit tests.

### Slice 3 — `:wat::core::define-alias` defmacro

- Add to `wat/std/ast.wat`.
- ~10 LOC macro body using slice 1 + slice 2 primitives.
- 2-3 unit tests:
  - Alias a substrate primitive (`foldl` → `reduce`); verify
    type-check passes; verify call-site resolves
  - Alias a user-define; verify the same
  - Verify TypeScheme identity via a probe test
    (call site type-checks with the alias the same as the
    target)

### Slice 4 — use it

- `(:wat::core::define-alias :wat::core::reduce :wat::core::foldl)`
  in `wat/std/ast.wat` (or `wat/core.wat`).
- Re-run `cargo test --workspace`; verify `:wat::core::reduce`
  resolves correctly at the two existing substrate call sites
  (`crates/wat-lru/wat/lru/CacheService.wat:213` +
  `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:251`).
- Confirms arc 130's substrate stops failing on the missing-
  reduce gap; arc 130 slice 1 RELAND can resume Layer 2+.

### Slice 5 — closure

- INSCRIPTION + 058 row + USER-GUIDE entry.
- Cross-references to arc 091 slice 8 (quasiquote precedent)
  + arc 057 (HolonAST polymorphism) + arc 037 (introspection
  precedent).

## Why this isn't a "v2" deferral

The user wind-down rule for arc 109 v1: "all arcs after 109
must be implemented without deferral before we close 109."
Arc 143 surfaced from arc 130's reland, which is itself part
of arc 109's chain. The reduce gap blocks arc 130's stepping
stones from progressing past Layer 1. Either:

1. We add a 5-line Rust alias for `reduce → foldl` (surgical;
   doesn't address the underlying absence)
2. We ship arc 143 (closes the homoiconic gap permanently)

Path 2 is the right architectural answer. The bias is
empirical (two independent miswrites). The absence is felt.
The substrate primitives needed are minimal (one
`signature-of`). The macro is small (~70 LOC of wat across
helpers + macro). The cost-benefit favors closing the gap.

Path 1 would leave the substrate able to register Rust-side
aliases but not wat-side aliases. Future writers (lab,
sonnet, future-us) reach for `reduce` — works for that name
specifically, but the next bias (`fold` instead of `foldl`?
`map-indexed`? `partition`?) requires another Rust addition.
With arc 143, future biases get one-line wat fixes.

## The four questions

**Obvious?** Yes. `(:define-alias :new :existing)` reads as
"define `:new` as an alias for `:existing`." Anyone familiar
with Clojure's `(def new existing)`, JS's `const new =
existing`, or Lisp's `(defalias 'new 'existing)` reads it
fluently.

**Simple?** Yes. ONE substrate primitive (`signature-of`).
Two wat helpers (rename-head, extract-arg-names). One
macro. Total: ~30 LOC Rust + ~70 LOC wat. The substrate
addition is small and bounded — it doesn't touch the type
system; it just exposes existing TypeScheme data as HolonAST.

**Honest?** Yes. The aliasing is REAL — calls to the alias
go through a wat-defined wrapper that delegates. The
TypeScheme identity is preserved through normal define
processing (no shortcutting). The macro expansion is visible
(quasiquote shows the structure). Future readers see what
happens.

**Good UX?** Yes. The user types `(:define-alias :new
:existing)` once. The substrate handles signature lookup,
rename, body delegation. No need to restate signatures.
No TypeScheme manual entry. Bias-aligned naming becomes a
one-line fix for the rest of the substrate's life.

## Open questions (Q1, Q2 resolved 2026-05-02 evening — see Findings)

### Q3 — Placement of the alias macro

Where does `:wat::core::define-alias` live?

- `wat/std/ast.wat` (new file) — clean separation of AST
  primitives + their consumers
- `wat/core.wat` (extend existing core stdlib) — keeps `core`
  primitives together
- `wat/std/alias.wat` (new file) — single-purpose

Defer to slice 2 / 3 implementation. Probably `wat/std/ast.wat`.

### Q4 — Should the alias macro work for macros too?

`define-alias :my-macro :their-macro` — alias one defmacro
to another. Mechanically possible via the same approach (look
up the source macro's signature, generate a defmacro that
delegates). Out of scope for this arc; defer.

### Q5 — Should there be a sibling `:typealias-alias` for type names?

Probably not. `typealias` already exists; aliasing typealiases
is `(:typealias :NewName ExistingType)`. The asymmetry is
honest — types vs callables are different reflection surfaces.

## Cross-references

- `docs/arc/2026/04/091-batch-as-protocol/INSCRIPTION.md`
  (slice 8 — quasiquote + struct→form, the partial reflection
  precedent)
- `docs/arc/2026/04/057-holon-ast-polymorphism/INSCRIPTION.md`
  (HolonAST as wat-readable AST representation)
- `docs/arc/2026/04/037-dim-router/INSCRIPTION.md` (arc 037
  intro of `statement-length` as introspection primitive
  precedent)
- `docs/arc/2026/05/130-cache-services-pair-by-index/SCORE-SLICE-1-RELAND.md`
  (the reland that surfaced the reduce gap)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/058-032-typed-macros/PROPOSAL.md`
  (typed macros)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/058-031-defmacro/PROPOSAL.md`
  (defmacro definition)
- `wat/test.wat:387+` (`:wat::test::make-deftest` — the
  worked example of macros-building-macros via quasiquote)
- `src/macros.rs` (MacroDef, MacroRegistry; parse_defmacro_signature)

## What's next

This DESIGN ships first. The 4 implementation slices follow
once the design is approved. After arc 143 closes, arc 130
slice 1 RELAND continues with the stepping stones (Layer 2+)
against a substrate that has `reduce` working.

Slice 1 of arc 130 RELAND blocks on this arc.
