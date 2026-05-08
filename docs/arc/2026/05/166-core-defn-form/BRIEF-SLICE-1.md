# Arc 166 slice 1 — `:wat::core::defn` macro + tests

## Goal

Ship `:wat::core::defn` as a wat-provided macro composing `def + fn`.
Single-arity; no docstrings (both deferred per DESIGN.md). Add
integration test file covering the 10 cases listed below. No
substrate change.

## Background context (read these before starting)

- `docs/arc/2026/05/166-core-defn-form/DESIGN.md` — full scope
- `wat/runtime.wat:17` — worked defmacro example (`define-alias`)
- `wat/test.wat:304` — worked defmacro example (`deftest`); sig
  shape is identical to defn's
- Arc 157 INSCRIPTION (`docs/arc/2026/05/157-core-def-form/INSCRIPTION.md`)
  — def's position rule + recursive-name-binding contract
- Arc 155 INSCRIPTION (`docs/arc/2026/05/155-fn-rename/INSCRIPTION.md`)
  — fn's shape + sig vocabulary

## Site to add

### `wat/core.wat`

Add the `defn` defmacro AFTER the existing dispatch declarations,
under a new section header. Macro shape:

```scheme
;; ─── Named-function binding ───────────────────────────────────────
;;
;; `:wat::core::defn` is the user-facing named-function form. It
;; composes the two foundational primitives:
;;
;;   (:wat::core::defn :name :sig :body)
;;     ↓ macro-expansion
;;   (:wat::core::def :name (:wat::core::fn :sig :body))
;;
;; Per user direction 2026-05-08: `:wat::core::fn` is the ONE AND
;; ONLY function constructor. defn just binds the function value to
;; a name; def just binds any value to a name. Composition over
;; multiplication of primitives.
;;
;; Inherits from `:wat::core::def`:
;; - position rule (top-level OR direct child of top-level do/let body)
;; - strict-default redef-error
;; - recursive name binding (the fn body sees `:name` as bound)
;;
;; Multi-arity overloads are NOT in this form's scope; a separate
;; `defn-clause` form (Erlang-style) ships later.
;;
;; Docstrings are NOT in this form's scope; arc 141 wires docstring
;; extraction broadly across substrate forms; defn extends to take a
;; docstring at that time.

(:wat::core::defmacro
  (:wat::core::defn
    (name :AST<wat::core::nil>)
    (sig  :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def ,name (:wat::core::fn ,sig ,body)))
```

The `:AST<wat::core::nil>` AST-shape annotations match the deftest
macro's pattern (wat/test.wat:304-310) — generic AST shape, not
constrained to specific types.

### `tests/wat_arc166_defn.rs`

New integration test file. Use the standard `startup_ok` /
`startup_err` helpers per arc 153/154/155 precedent. Test cases:

1. **`defn_simple_compiles_and_runs`** — `(:wat::core::defn
   :user::add ((x :wat::core::i64) (y :wat::core::i64) ->
   :wat::core::i64) (:wat::core::i64::+,2 x y))` plus a `:user::main`
   that calls `(:user::add 2 3)` returns 5.

2. **`defn_recursive_factorial_works`** — defn defines `:user::fact`
   with a body that recursively calls `:user::fact`. Verifies arc
   157's name-registered-before-RHS-eval contract carries through
   defn unchanged. main returns `(:user::fact 5)` = 120.

3. **`defn_at_top_level_position`** — defn at file root works.

4. **`defn_inside_top_level_do_works`** — `(:wat::core::do (:defn
   ... ) (:defn ... ))` at top level — both names register.

5. **`defn_inside_top_level_let_body_works`** — defn inside the body
   of a top-level let. Per arc 157 closure, def's position rule
   permits this; defn inherits.

6. **`defn_rejected_inside_if_branch`** — `(:wat::core::if cond
   -> :T (:wat::core::defn ...) (:wat::core::defn ...))` should
   surface the position-rule error from def. The walker runs over
   the post-expansion AST so the macro inherits the rule.

7. **`defn_zero_arg_function_works`** — `(:wat::core::defn
   :user::greet (-> :wat::core::nil) (:wat::console::println
   "hello"))` — wait, check what zero-arg sig shape is. Per arc 155
   the no-arg fn sig is `(-> :T)` (just arrow + ret type, no params).
   If that doesn't compile, fall back to a fn that takes a unit
   `((_ :wat::core::nil) -> :wat::core::i64)` shape and report the
   delta.

8. **`defn_body_type_mismatch_surfaces`** — defn declares `->
   :wat::core::nil` but body returns `:wat::core::i64`. Surface
   `ReturnTypeMismatch` from fn's check.

9. **`defn_redef_same_name_forbidden_by_default`** — two defn forms
   with the same name. Surface `DefRedefForbidden` per def's
   strict-default. Verifies defn inherits redef gating.

10. **`defn_reflection_lookup_form_resolves`** — after defn,
    `(:wat::runtime::lookup-form :user::add)` returns a non-None
    Binding (the def-bound fn). Verifies the def-bound name lands
    in SymbolTable via the same path `(:wat::core::define :name ...)`
    used pre-arc-166.

Use `startup_ok` for cases 1-5, 7, 10. Use `startup_err` for cases
6, 8, 9 — assert the relevant error string appears.

## Discipline

The macro is small (~6 lines plus the section comment). The bulk of
the work is the test file. Each test case is self-contained — write
a minimal program with `:user::main` that exercises just one
behavior.

Do NOT modify any substrate code. If a test reveals a substrate gap
(e.g., position rule doesn't propagate through macro expansion as
expected), STOP and report; orchestrator decides whether to scope-in
the substrate fix or rescope the test.

Run `cargo test --release --test wat_arc166_defn` after each round
of edits; the test binary should compile and pass all 10 cases.
Then run the full workspace suite to confirm no regression:
`cargo test --release --workspace --no-fail-fast`.

DO NOT commit. Orchestrator commits after scoring.

## Time-box

30-60 min upper. The macro is trivial; the tests are the bulk and
take time to write + iterate.

## Report shape

Per EXPECTATIONS-SLICE-1.md, report:

1. cargo test final summary (workspace pass/fail counts)
2. Test names + pass/fail status of each of the 10 cases
3. Any honest deltas — surprises in defn expansion, position-rule
   propagation, recursive-name-binding behavior, or test-shape
   mismatches with the BRIEF's expected sig syntax (e.g., the
   no-arg sig shape detail in case 7)
4. Actual runtime in minutes vs predicted 30-60 min band
