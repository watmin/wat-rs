# Arc 157 — `:wat::core::def` foundational value-binding form

**Status:** opened 2026-05-07.

## User direction (verbatim)

> *"let's introduce a new form... (:wat::core::def :some-name :some-value)"*

> *"i don't think a type declaration is appropriate for the same
> reason that (let ...) and (do ...) don't need a type declaration"*

> *"define will be swapped to a wrapper on (def :name (fn ...)) -
> don't worry about this for now. let's get (def ...) as a
> foundational form first."*

Q1 (scope):
> *"top level - its modifying the ambient monad"*

Q2 (redef):
> *"i want this to be user choosable... 2 config fields ...
> (:wat::config::set-redef! true) and (:wat::config::set-eval-redef! true)
> ... the redef must satisfy two constraints - the signature must not
> change - input args+type must be identical, the ret type must be the
> same"*

Q3 (namespacing):
> *"we have no namespace... the symbol (:wat::core::def :pi 3.14) is
> a user legal form.. wat provided forms are always defined as fqdn..
> users can do whatever they want"*

## Goal

Mint `:wat::core::def` as the foundational top-level value-
binding special form. Shape:

```
(:wat::core::def :my::app::pi 3.14159)
```

The form binds a name to the result of evaluating an expression.
The bound name's type is the inferred type of the expression —
no type annotation on the form itself.

This is the missing primitive. Today `:wat::core::define` only
takes function shape (`(define (name args -> T) body)`); there's
no top-level analog of `let` for non-function values. Module-
level constants currently route through `define` with a no-arg
function or aren't expressible at all.

## Why no type declaration

Substrate is statically typed via inference + recipient
unification (memory `feedback_substrate_already_typed.md`).
Arc 145 (typed-let) was BACKED OUT for this exact reason: the
expression already has a type; mandating `-> :T` is redundant
noise.

The `def` analog of let's recipient is **future call sites** —
which lookup `:name` in the module env and find the inferred
type already attached. No recipient-pressure gap; no annotation
needed.

Optional `-> :T` as disambiguator for genuinely ambiguous
expressions (e.g., `[]` of unknown element type) is intentionally
**out of scope** for arc 157. The honest answer to ambiguity is
"add the hint inside the expression," not "bolt it onto the
binding form." If a real use case surfaces post-arc, opens a new
arc.

## Final shape

```
(:wat::core::def :name expr)
```

where:
- `:name` is a keyword identifier (FQDN per arc 109 conventions
  for cross-module visibility, or local for module-private)
- `expr` is any expression; its inferred type becomes the
  registered type of `:name`

## Cross-references

- **Arc 145** (typed-let) — back-out lesson: don't require type
  annotations when substrate inference already produces them.
- **`feedback_substrate_already_typed.md`** — paid-for memory
  driving the no-annotation decision.
- **Arc 109** — FQDN convention applies to `def`'d names.
- **Future** — `define` retirement: `(define (name args -> T) body)`
  becomes the macro `(defn name args body) → (def :name (fn args body))`.
  Out of arc 157's scope.

## Settled design

### Scope (Q1) — Clojure top-level rule

`def` is legal at **top-level position only**. User direction:
"top level — it's modifying the ambient monad." Module env IS
the ambient monad; `def` is a state-modifying operation on it.

User direction 2026-05-07:
> *"clojure is our guiding light - we're just building a
> strongly typed clojure on rust"*

**Precise predicate** (recursive, runs post-expansion):

> A form is at *top-level position* iff it is either:
> 1. a direct child of the file's form list, OR
> 2. a direct child of a `:wat::core::do` form at top-level
>    position, OR
> 3. a direct child of a `:wat::core::let` body at top-level
>    position.

Both `do` and `let` **splice** because both are sequential —
each direct child runs once, in order, at module load. `let`
additionally introduces locals that the `def`'s expression can
capture as closures (the load-bearing case for Path B):

```clojure
(:wat::core::let [config (:my::app::load-config)]
  (:wat::core::def :get-port
    (:wat::core::fn [] (:port config)))
  (:wat::core::def :get-host
    (:wat::core::fn [] (:host config))))
```

`config` stays local to the `let`; only the `def`'d names enter
the module env, with closures that capture `config`.

**Why other forms are NOT splice positions** — the governing
principle is: **`def` is a once-per-program declaration**. Only
positions guaranteed to execute exactly-once at load time may
splice `def`.

User direction 2026-05-07:
> *"def in try, loop, recur makes no sense. the stance we should
> have is that a def /should/ only be declared once. using def in
> an explicit iterable is not acceptable"*

Rejection categories:

- **Conditional** (binding existence would depend on runtime
  value): `if`, `cond`, `match`, `and`, `or` (short-circuit),
  `Result/try`, `Option/try`, future `when` / `unless`
- **Function / closure bodies** (run at call time, not load
  time; multiple invocations would fight the redef discipline):
  `fn`, `define` body, `defmacro` template
- **Iteration** (zero-or-many executions; "explicit iterable"
  per user direction): future `loop`, `recur`, `for`, `doseq`,
  `while`, `map`-as-control-flow
- **Type definitions** (declarative-only context; no execution
  semantics): `struct`, `enum`, `newtype`, `typealias`
- **Quote / template positions** (quoted forms aren't executed):
  `quote`, `quasiquote`, `unquote`, `unquote-splicing`

The discipline is forward-compatible: any new control-flow form
landing in the substrate must explicitly be classified as splice
(`do`/`let` lineage) or not. Default is NOT — must justify
splice eligibility per the once-per-load-time-execution rule.

Clojure's runtime surface accepts `def` everywhere because
Clojure is dynamic; the Clojure community treats nested `def`
as a footgun and lints reject it. Strongly-typed wat-rs
ENFORCES that discipline at substrate level — the rule above
is exactly the Clojure surface that production Clojure code
actually uses, restricted to statically-tractable positions.

**Legal positions for `def`:**

```clojure
;; ✓ direct top-level
(:wat::core::def :a 1)

;; ✓ inside top-level (do ...) — splices sequentially
(:wat::core::do
  (:wat::core::def :a 1)
  (:wat::core::def :b 2))

;; ✓ inside top-level (let ...) — splices sequentially;
;;   let's locals can be captured as closures
(:wat::core::let [config (:my::app::load-config)]
  (:wat::core::def :get-port
    (:wat::core::fn [] (:port config)))
  (:wat::core::def :get-host
    (:wat::core::fn [] (:host config))))

;; ✓ nested do/let inside top-level let/do — recursive splice
(:wat::core::let [x 1]
  (:wat::core::do
    (:wat::core::def :a x)
    (:wat::core::def :b (:wat::core::* x 2))))

;; ✓ macro at top-level expanding to a top-level (do ...)
(:my::app::declare-pair)
;; ↳ expands to (:do (:def :a 1) (:def :b 2))
;;   — splice rule applies; defs land at top-level
```

**Illegal positions** (rejected with `DefNotTopLevel` diagnostic):

```clojure
;; ✗ if is not a splice form (conditional execution)
(:wat::core::if cond
  (:wat::core::def :a 1)
  (:wat::core::def :b 2))

;; ✗ when / cond / match — also conditional
(:wat::core::when cond
  (:wat::core::def :a 1))

;; ✗ inside function body (runs at call time, not load time)
(:wat::core::define (:my::app::f -> :wat::core::Unit)
  (:wat::core::def :a 1))

;; ✗ inside fn literal body
(:wat::core::def :maker
  (:wat::core::fn []
    (:wat::core::def :a 1)))    ;; nested in fn body

;; ✗ macro called inside a function body, even if its expansion
;;   contains a top-level-shaped (do (def ...))
(:wat::core::define (:my::app::f -> :wat::core::Unit)
  (:my::app::declare-pair))
```

The macro provenance question dissolves: the substrate doesn't
need to track "this came from a macro." Post-expansion, the
shape rule applies uniformly. Macros that need to emit defs
must be called at top-level position; their expansion lands at
top-level naturally.

`let` at top-level splices `def` legality (Path B); `let` inside
a function body still covers nested local binding without `def`
state mutation.

### Re-binding discipline (Q2)

**Default:** error on redef. Compile error: "name `:foo` already
bound at <prior location>; use a different name or enable
`(:wat::config::set-redef! true)`".

**Configurable via two flags** on the SymbolTable capability
carrier (memory `feedback_capability_carrier.md`). **Both default
to `false`** — users must opt in:

| Flag | Default | Effect when `true` |
|---|---|---|
| `(:wat::config::set-redef! true)` | `false` | Compile-time / load-time redef permitted with type-stability check |
| `(:wat::config::set-eval-redef! true)` | `false` | Eval-time redef (interactive `eval-ast!` flow) permitted with type-stability check |

Opinionated default: re-defining a name is an error unless the
user has explicitly enabled it. Catches typo'd-name collisions
by default; allows hot-reload workflows on opt-in.

**Type-stability constraint** (mandatory whenever redef happens,
regardless of which flag): the redef's expression must produce
the same type as the original binding. Specifically:

- For `def` of a value: the inferred type of the new expression
  must equal the previously-registered type for that name.
- For `def` of a function (post-`define`-retirement): the input
  arg types and return type must be identical to the prior
  binding. Body may change; signature cannot.

If the type changes, redef is rejected even with the flag on:
"redef of `:foo` changes type from `:wat::core::i64` to
`:wat::core::String`; only body may change". This preserves the
contract downstream callers depend on.

Bang convention follows established `:wat::config::*` precedent
(`set-capacity-mode!`, `set-dim-count!`, `set-presence-sigma!`,
etc.).

### Namespacing (Q3)

Substrate has no namespace concept. `def` accepts any keyword as
the name — bare (`:pi`) or FQDN (`:my::app::pi`). User picks.

**Discipline:** wat-provided forms ship at FQDN paths
(`:wat::core::def`, `:wat::config::set-redef!`, etc.). User code
chooses freely. Substrate doesn't enforce.

### Final shape

```
(:wat::core::def :pi 3.14159)                  ;; bare name; legal
(:wat::core::def :my::app::pi 3.14159)         ;; FQDN; legal
```

`:pi`'s registered type = `3.14159`'s inferred type = `:wat::core::f64`.

## Slice plan

### Slice 1a — substrate (def + redef config + scope check)

**Special form registration** (`src/special_forms.rs`):

- Register `:wat::core::def` with shape `<name> <expr>`.
- Top-level recognition in `src/freeze.rs` alongside `define`,
  `defmacro`, `define-dispatch`, `struct/enum/newtype/typealias`.

**SymbolTable carrier additions** (`src/runtime.rs`):

- `redef_allowed: bool` (default `false`)
- `eval_redef_allowed: bool` (default `false`)
- `defined_values: HashMap<String, (TypeScheme, ValueOrigin)>` —
  bound name → (type, where-defined) for type-stability checking
  + diagnostic location on collision

**Type-check arm** (`src/check.rs`):

- Position check: `def` outside top-level position rejected with
  `DefNotTopLevel` diagnostic naming the wrapper that violates
  the rule (e.g. "def found inside `:wat::core::if` — only
  literal top-level or top-level `:wat::core::do` /
  `:wat::core::let` splice positions are legal"). Predicate
  per § Scope (Q1) above (recursive: top-level `do` and top-
  level `let` both splice).
- Infer `<expr>`'s type.
- If `<name>` already in `defined_values`:
  - If `redef_allowed == false` → reject with location-of-prior-def
    diagnostic.
  - If `redef_allowed == true` → check type-stability; reject if
    inferred type ≠ prior registered type with diagnostic naming
    both types.
- Register `<name>` → inferred type in `defined_values` (insert
  or replace).

**Eval arm** (`src/runtime.rs`):

- Evaluate `<expr>` in current module's value env.
- If `<name>` already bound at runtime:
  - If `eval_redef_allowed == false` → runtime error.
  - If `eval_redef_allowed == true` → check runtime-type
    matches prior binding; reject if not.
- Bind `<name>` → value.

**Config primitives** (`src/runtime.rs` or `src/config.rs`):

- `:wat::config::set-redef!` — takes `:wat::core::bool`; sets
  `redef_allowed`. Bang convention.
- `:wat::config::set-eval-redef!` — takes `:wat::core::bool`;
  sets `eval_redef_allowed`. Bang convention.
- Pattern mirrors existing `set-capacity-mode!` /
  `set-dim-count!` / `set-presence-sigma!` shape.

**Tests** (8-12 covering):

- Simple `def` with literal value; `:name` resolves to value
- Computed `def` referencing prior `def`
- Type registered: subsequent reference type-checks against the
  registered type
- Type error in expr surfaces at `def` site with location
- `def` at literal top-level → succeeds
- `def` inside top-level `(do ...)` splice → succeeds (multiple
  defs, each registered)
- `def` inside top-level `(let ...)` splice → succeeds; let's
  locals captured by closure-shaped def expressions
- `def` nested through top-level `do`/`let` chain (e.g. `let`
  containing `do` containing `def`) → succeeds (recursive splice)
- `def` inside `(if ...)` → rejected with `DefNotTopLevel`
- `def` inside `(when ...)` → rejected with `DefNotTopLevel`
- `def` inside function body (`define` body) → rejected with
  `DefNotTopLevel`
- `def` inside `fn` literal body → rejected with `DefNotTopLevel`
- Macro called at top-level expanding to `(do (def ...) (def ...))`
  → succeeds (post-expansion shape is top-level-do-splice)
- Macro called inside function body, even if its expansion
  contains `(do (def ...))` → rejected (the wrapping context
  isn't top-level)
- Redef with default flags off → error with location-of-prior-def
- Redef with `set-redef!` + same type → succeeds
- Redef with `set-redef!` + different type → rejected (type-stability)
- Eval-redef with `set-eval-redef!` off → runtime error
- Eval-redef with `set-eval-redef!` + same type → succeeds
- Eval-redef with `set-eval-redef!` + different type → rejected

`model: "sonnet"` explicit on Agent spawn per FM 12.

DO NOT COMMIT (atomic with 1b if any consumer migration exists).

### Slice 1b — consumer migration (likely tiny)

Sweep wat sources + tests for places that "wanted" `def` but had
to use `define` or top-level computed expressions. Likely a
small handful; possibly zero. Atomic commit with 1a when
workspace = 0-failed.

### Slice 2 — closure paperwork

- INSCRIPTION + 058 changelog row + USER-GUIDE update
  (new `def` section + new `:wat::config::set-redef!` /
  `set-eval-redef!` rows)
- WAT-CHEATSHEET update (control-form table)
- Pre-INSCRIPTION grep mandatory per FM 11
- Orchestrator-side per `feedback_paperwork_orchestrator_side.md`

## Estimated effort

- Slice 1a: ~45-75 min Sonnet (def special form + 2 config
  primitives + redef discipline + 8-12 tests)
- Slice 1b: ~5-15 min Sonnet (consumer scan, likely empty)
- Slice 2: ~25 min orchestrator (closure paperwork)
- Total: ~1.5-2 hours wall-clock if Mode A clean
