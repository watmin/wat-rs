# Arc 144 — Uniform reflection foundation: every form has `:name :sig :body :doc-string`

**Status:** drafted 2026-05-02 (late evening), post-arc-143-slice-7.

**The architectural framing:**

> *"i think we need all forms... when we get to working on our repl
> we should be able to call (help :some-func) no matter what it is..
> we can call (help :if) and it'll /just work/?... nothing is special?..."*

> *"user forms can provide doc strings - we just satisfy the
> interface of :name :sig :body (and later :doc-string) -- all
> things need to have this -- we'll prove we did this correct when
> we go to add doc strings, that's a proving point that we can
> build paved roads"*

> *"if we need to inline define these in rust such the registry
> becomes aware then we must"*

The user articulating: **uniform reflection — nothing is special.**
Every known wat form (functions, macros, types, type-aliases,
substrate primitives, special forms like `if`/`cond`/`let*`/`lambda`)
satisfies a uniform interface: `:name :sig :body` (and `:doc-string`
when arc 141 ships docstrings). `lookup-form` returns Some for
ANY known symbol; reflection works without per-kind special-cases
at the consumer layer.

This arc ships the foundation. **Doc strings aren't in this arc.**
Arc 141 (already pending) handles populating doc strings. THIS arc's
plumbing must ACCEPT docstrings when arc 141 ships — paved-road
discipline. The `:doc-string` field exists on Binding from day 1
(always `None` until arc 141 fills in user-define + macro doc
strings; substrate-primitive doc strings populate alongside the
synthetic Bindings this arc creates).

`(help X)` is NOT in this arc — it's a future REPL-layer consumer.
This arc ships the FOUNDATION such that `help` becomes a small wat
function over the uniform interface.

## Why this isn't a slice of arc 143

Arc 143 shipped `:wat::runtime::define-alias` end-to-end. Slice 7
demonstrated the substrate-as-teacher cascade through to arc 130
unblock. Arc 143's substantive purpose is FULFILLED.

Arc 143's slice 6 surfaced a SECOND gap (length not in TypeScheme
registry) which hinted at a broader architectural absence
(`infer_*` hardcoded handlers bypassing reflection entirely). The
user articulated the principle on 2026-05-02 late evening:
"nothing is special" — `(help :if) /just works/` — the reflection
layer must cover ALL known forms uniformly.

That's a NEW arc, not a slice of 143:
- Arc 143's scope was define-alias (point alias for one primitive)
- Arc 144's scope is the full reflection foundation (uniform
  surface across all kinds)

Splitting them keeps each arc coherent. Arc 143 closes; arc 144
delivers the broader foundation.

## Acceptance criteria

After arc 144 ships, the following ALL work uniformly:

```scheme
(:wat::runtime::lookup-form :wat::core::if)         ;; SpecialForm Binding
(:wat::runtime::lookup-form :wat::core::let*)       ;; SpecialForm Binding
(:wat::runtime::lookup-form :wat::core::lambda)     ;; SpecialForm Binding
(:wat::runtime::lookup-form :wat::core::foldl)      ;; Primitive Binding
(:wat::runtime::lookup-form :wat::core::length)     ;; Primitive Binding (was hardcoded)
(:wat::runtime::lookup-form :wat::core::Vector)     ;; Primitive Binding (constructor)
(:wat::runtime::lookup-form :wat::test::deftest)    ;; Macro Binding
(:wat::runtime::lookup-form :wat::lru::Spawn)       ;; Type Binding (typealias)
(:wat::runtime::lookup-form :user::my-fn)           ;; UserFunction Binding
(:wat::runtime::lookup-form :no::such::thing)       ;; None
```

Every known form returns `Some(Binding)`. Unknown forms return
`None`.

`signature-of`, `body-of`, `origin-of` (when arc 143's slice 5b
extends) all dispatch on the Binding variant uniformly.

## The Binding interface

```rust
pub enum Binding {
    UserFunction {
        name: String,
        signature: HolonAST,         // (name<T,...> (arg :Type) ... -> :Ret)
        body: HolonAST,              // the function's body
        doc_string: Option<String>,  // populated by arc 141 in user defines
    },
    Macro {
        name: String,
        signature: HolonAST,         // (name (param :AST<T>) ... -> :AST<R>)
        template: HolonAST,          // the quasiquote template
        doc_string: Option<String>,
    },
    Primitive {
        name: String,
        signature: HolonAST,         // synthesized from TypeScheme
        // no body — Rust-implemented
        doc_string: Option<String>,  // populated for substrate primitives
    },
    SpecialForm {
        name: String,
        signature: HolonAST,         // (head <slot1> <slot2> ...) syntax sketch
        // no body — semantic operation, not a function
        doc_string: Option<String>,  // hand-authored at substrate registration
    },
    Type(TypeDef),  // already unified per arc 057 (struct/enum/newtype/alias)
}
```

The `:doc-string` field is the paved road for arc 141. Every
variant carries an `Option<String>` from day 1; arc 141 populates
the `Some` cases as docstring sources arrive.

## What needs to ship

### Substrate side

1. `Binding` enum + per-variant accessors (`name`, `signature`,
   `body`/`template`/None, `doc_string`)
2. `lookup_form(name) -> Option<Binding>` — generalized version of
   slice 1's `lookup_callable`. Walks ALL registries:
   - `SymbolTable.functions` → UserFunction
   - `MacroRegistry.macros` → Macro
   - `CheckEnv.schemes` → Primitive
   - Special forms registry (NEW) → SpecialForm
   - `TypeEnv.types` → Type
3. **Special forms registry**: a NEW HashMap (or const array) of
   special form metadata (name + signature sketch + future doc
   string slot). Populated at substrate startup with all special
   forms.
4. Hardcoded callable primitives (the 15 from arc 143's audit:
   Vector, Tuple, HashMap, HashSet, string::concat, assoc, concat,
   dissoc, keys, values, empty?, conj, contains?, length, get) get
   TypeScheme registrations OR special-form-style entries.
5. `signature-of` / `body-of` / `origin-of` refactored to dispatch
   on Binding variant.

### Test side

For each form kind, at least one test:
- `lookup-form` on a known of that kind returns Some(matching variant)
- `signature-of` returns the expected head shape
- `body-of` returns the right thing (Some for define/macro; None for primitive/special)

Plus: a test that confirms `:doc-string` field exists on every
variant (placeholder for arc 141 verification).

## Slice plan (5 slices, dependency-ordered)

### Slice 1 — Binding enum + lookup-form

Define the Binding enum + per-variant accessors. Refactor
`lookup_callable` (arc 143 slice 1) to `lookup_form` returning
`Option<Binding>`. Update arc 143 slice 1's three primitives
(lookup-define, signature-of, body-of) to dispatch on Binding.

This is the GATING substrate change — everything else builds on it.

~150-250 LOC Rust + ~50 LOC tests.

### Slice 2 — Special form registrations

Build a NEW special-form registry (struct + populate at substrate
startup). Each special form gets:
- Name (e.g., `:wat::core::if`)
- Signature sketch (synthetic HolonAST showing the syntax shape)
- Empty doc_string (None for now)

The ~25 special forms identified from check.rs's `infer_list`
dispatch + macro special cases:
- Control: if, cond, match, when, unless
- Binding: let, let*, lambda
- Definitional: define, defmacro, struct, enum, newtype, typealias
- Error: try, option/expect, result/expect
- Concurrency: spawn-thread, spawn-program, fork-program-ast
- Macro plumbing: quote, quasiquote, unquote, unquote-splicing
- AST: forms

Sonnet audits check.rs to enumerate; each one gets a registration.

~100-200 LOC Rust + ~30-50 LOC tests.

### Slice 3 — Hardcoded callable primitives

The 15 callables from arc 143's audit get TypeScheme registrations.
Each scheme is a "callable-fingerprint" — captures arity + return
type even if can't capture full polymorphism (the hardcoded
handlers continue to do actual type-check). The schemes make
lookup_form return Some.

Variadic constructors (Vector, Tuple, HashMap, HashSet) — register
a simplified fixed-arity scheme PLUS a marker that the runtime
uses variadic dispatch. Honest about the limitation.

~75-125 LOC Rust + ~30 LOC tests.

### Slice 4 — Verification + arc 143 cleanup

- Re-run arc 143 slice 6's length test: it now PASSES.
- Verify `lookup-form` works for examples of each kind
  (UserFunction, Macro, Primitive, SpecialForm, Type).
- Add the placeholder doc-string field test (every variant carries
  Option<String>; verify None is the default).

~50 LOC tests.

### Slice 5 — Closure

INSCRIPTION + 058 row + USER-GUIDE + ZERO-MUTEX cross-ref +
end-of-work ritual.

## Why "paved roads" matters here

Arc 141 (pending) ships docstrings on user defines + macros. Without
arc 144's plumbing, arc 141 would have to either:
1. Add docstrings to Function + MacroDef (changing those structs)
2. Add a parallel docstring registry
3. Refactor reflection to handle the new field

With arc 144's plumbing, arc 141 just:
1. Parse `:doc "..."` clauses in define/defmacro forms
2. Set Binding's `doc_string: Some(...)` at registration
3. Done — reflection sees doc strings without further changes

The paved road isn't speculative — it's the discipline that
prevents future arcs from having to retrofit.

## Cross-references

- `docs/arc/2026/05/143-define-alias/` — the prior arc that surfaced
  the need for uniform reflection
- `docs/arc/2026/05/141-core-form-docstrings/` — the future arc
  that populates the doc-string field
- `docs/arc/2026/04/057-holon-ast-polymorphism/` — HolonAST as the
  reflection AST representation
- `docs/COMPACTION-AMNESIA-RECOVERY.md` — the protocol that
  prevents ignorance from propagating into briefs

## What success looks like

After slice 5 closes:
- `lookup-form` works for ALL known wat symbols, regardless of kind
- `(help :if)` would work cleanly once a `help` consumer ships
- Arc 141's docstring work is plumbing-ready (no Binding refactor
  needed when arc 141 lands)
- Arc 109 v1 closure is one arc closer

The principle "nothing is special" is honored at the substrate layer.

## Blocks

Arc 109 v1 closure now blocks on arc 144 in addition to arc 143.
