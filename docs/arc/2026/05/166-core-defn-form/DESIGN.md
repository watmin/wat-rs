# Arc 166 — `:wat::core::defn` foundational named-function form

**Status:** queued 2026-05-08; opens immediately.

**Gates:** none. Substrate has both `:wat::core::def` (arc 157) and
`:wat::core::fn` (arc 155); defmacro with quasiquote-template bodies
is established (`wat/runtime.wat:17`, `wat/test.wat:304`). Arc 166
ships `defn` as a wat-provided macro; no substrate change.

## Background

User direction 2026-05-08:
> *"i want exactly one way to define a function. defn just binds the
> function to a name and has doc strings, etc... (defn :some-name
> :some-sig :some-body) ↦ (def :some-name (fn :some-sig :some-body))
> ... i think fn should be the one and only way to actually
> construct a function — defn is just a wrapper on it."*

The composition is the standard Clojure shape, applied to wat's
typed-substrate vocabulary. Clojure's actual `defn` IS a macro
(`(source defn)` shows the macro definition). Heritage check passes;
substrate-minimalism check passes; "exactly one way to construct"
discipline passes.

## Vocabulary mint

- **Form:** `:wat::core::defn`
- **Implementation:** wat-provided defmacro in `wat/core.wat`
- **Position rule:** inherited from `:wat::core::def` — top-level OR
  direct child of top-level `:wat::core::do` OR direct child of
  top-level `:wat::core::let` body. Conditional / function bodies /
  iteration constructs reject `defn` because `def`'s expansion target
  rejects them. The position-rule walker runs over the post-expansion
  AST so the macro inherits the rule for free.
- **Type story:** def infers from RHS; RHS is fn; fn carries its own
  type from its sig. The composition is type-honest end-to-end.
- **Recursive self-reference:** def registers `:name` in the
  SymbolTable BEFORE evaluating the RHS expression at runtime
  (per arc 157 closure); the fn body can reference `:name` and
  resolves recursively. `(defn fact (...) (... (fact ...) ...))`
  works without special handling.

### Macro body

```scheme
(:wat::core::defmacro
  (:wat::core::defn
    (name :AST<wat::core::nil>)
    (sig  :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def ,name (:wat::core::fn ,sig ,body)))
```

### Usage

```scheme
;; canonical
(:wat::core::defn
  :user::add
  ((x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
  (:wat::core::i64::+,2 x y))

;; expands to (post-macro)
(:wat::core::def
  :user::add
  (:wat::core::fn
    ((x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
    (:wat::core::i64::+,2 x y)))

;; recursive — fact body sees `fact` because def registers the name
;; before evaluating RHS (arc 157)
(:wat::core::defn
  :user::fact
  ((n :wat::core::i64) -> :wat::core::i64)
  (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
    1
    (:wat::core::i64::*,2 n (:user::fact (:wat::core::i64::-,2 n 1)))))
```

## Why arc 166 is the right shape

Four questions:

- **Obvious?** Reader sees `(defn name sig body)` ↔ `(def name (fn
  sig body))`. The desugaring is the documentation. Heritage shape
  every Lisp/Clojure programmer recognizes.
- **Simple?** Pure macro composition. ~6 lines. No substrate change.
  Position rule + type inference + recursive self-reference all
  inherited from existing primitives.
- **Honest?** Doesn't claim to be the function constructor — fn IS.
  Defn is sugar over composition. The macro-as-tutorial pattern: read
  the macro body to see the desugaring.
- **Good UX?** Familiar shape. Trade-off: type-check errors point at
  the post-expansion form (def or fn), not the source `defn`. For a
  thin wrapper this is acceptable; if defn-specific diagnostics ever
  earn their keep, that becomes a separate substrate-promotion arc.

## Out of arc 166's scope (affirmative)

- **Multi-arity overloads.** Per user direction 2026-05-08:
  *"multi arity will be defn-clause — we'll make that later — defn
  first... we'll handle multi arity like erlang... i dislike
  clojure's N-ary approach."* Arc 166 ships single-arity defn.
  Erlang-style multi-clause dispatch (`defn-clause`) is its own
  later arc when the user directs.

- **Docstrings.** Per user direction 2026-05-08: *"doc strings will
  come later, we need them on structs, enums, defs, defns,
  defn-clauses, etc etc — not now — later."* Arc 166 does NOT take a
  docstring slot. Arc 141 (pending #225) is queued to wire docstring
  source-extraction broadly across substrate forms; when arc 141
  ships, defn extends to take an optional docstring (likely
  `(defn name doc? sig body)`) which threads through `def` (also
  extended to accept a docstring) into the populated `doc_string:
  Option<String>` slot.

- **`define` → `defn` migration sweep.** Per user direction
  2026-05-08: *"we'll flip from define to defn after its in place."*
  Arc 166 SHIPS defn; the sweep that migrates `define` consumers and
  retires `define` is its own later arc. Sequencing: defn ships
  additive; define stays operational; migration sweep drives `define`
  call sites to `defn`; retirement closes `define`. Three discrete
  arcs.

## Slice plan

### Slice 1 — macro + tests

- **`wat/core.wat`** — add the `defn` defmacro definition. Place near
  existing `define`-related forms or in a clearly-marked "named-fn
  binding" section.
- **`tests/wat_arc166_defn.rs`** — integration tests covering:
  1. Simple defn (basic add function) compiles + runs
  2. Recursive defn (factorial) compiles + runs (verifies arc 157's
     name-registered-before-RHS-eval contract)
  3. Defn at top-level
  4. Defn inside `(:wat::core::do ...)` at top-level
  5. Defn inside `(:wat::core::let (...) ...)` at top-level (the body
     position; def rule allows this per arc 157 closure)
  6. Defn rejected inside an `if` branch (inherits def's position
     rule via post-expansion walk)
  7. Defn with no-args fn (`((:nil) -> :T)` shape)
  8. Defn with type-mismatch in body — surfaces ReturnTypeMismatch
     from the fn-form's check
  9. Defn redefining same name → `DefRedefForbidden` per def's
     strict-default
  10. Reflection — `(:wat::runtime::lookup-form :user::name)`
      resolves post-defn (verifies the name lands in SymbolTable
      via def's existing path)

### Slice 2 — closure

- INSCRIPTION
- 058 changelog row in trading-lab repo
- USER-GUIDE row introducing `defn` (placed in the same section as
  `define` / `def`; explicit cross-reference to `fn` and `def` so
  readers see the composition)
- WAT-CHEATSHEET row

## Cross-references

- **Arc 157** — `:wat::core::def` foundational top-level value-binding
  form. Arc 166 composes def + fn; inherits position rule + redef
  gating + recursive name binding.
- **Arc 155** — `:wat::core::fn` function constructor. Arc 166 keeps
  fn as the single way to construct function values.
- **Arc 141** — core-form docstrings (pending). Future arc that wires
  docstring extraction; arc 166's defn extends to accept docstrings
  when 141 ships.
- **`wat/test.wat:304`** — `:wat::test::deftest` defmacro: the exact
  shape pattern arc 166 mirrors (name + sig + body, all
  `:AST<wat::core::nil>`, body is quasiquoted def-form template).

## Discipline notes

The decision to ship defn-as-macro rather than substrate-primitive
followed the four questions consulted explicitly:
- Obvious favored macro (composition is honest)
- Simple favored macro (no substrate change)
- Honest favored macro (defn IS sugar; saying so directly is honest)
- Good UX slightly favored substrate (better diagnostics) but trade
  is acceptable for a thin wrapper

User direction *"when we need to make a decision we consult the
questions"* — invoked here. The macro path wins three of four; the
fourth is acceptable trade.
