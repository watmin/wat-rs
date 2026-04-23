# Arc 030 — macroexpand / macroexpand-1

**Status:** opened 2026-04-23. Cut from arc 029's make-deftest
debugging discovery.

**Motivation.** Every Lisp with macros ships a macroexpand tool.
Clojure, Common Lisp, Racket, Scheme, Elisp — all have some form of
`(macroexpand '<quoted-form>)` that runs a form through the expander
WITHOUT evaluating it and returns the resulting AST for inspection.
wat's macro system shipped without this tool. Arc 029's nested-
quasiquote work surfaced a bug in make-deftest's non-empty default-
prelude path; diagnosing required eprintln scaffolding inside
`src/macros.rs` and `src/sandbox.rs`, rebuild cycles, env-var-gated
printf dumps, and pages of output. A proper macroexpand primitive
would diagnose the same bug in three lines of wat at a deftest call
site.

The tool belongs in the substrate, not in debug env vars.

---

## Semantics

Two primitives. Both take a quoted form, return `:wat::WatAST`:

**`(:wat::core::macroexpand-1 <quoted-form>) -> :wat::WatAST`**
One expansion step. If the form is a macro call, expand it once
(apply the macro's template with the call-site bindings). Return
the result. If the form is NOT a macro call, return it unchanged.
Matches Common Lisp / Clojure `macroexpand-1`.

**`(:wat::core::macroexpand <quoted-form>) -> :wat::WatAST`**
Fixpoint expansion. Repeatedly apply macroexpand-1 until the form
stops changing (same AST two iterations in a row). Returns the
final form. Depth-bounded by `EXPANSION_DEPTH_LIMIT` to catch
runaway cycles. Matches Common Lisp / Clojure `macroexpand`.

Both primitives operate on the CURRENT frozen macro registry — the
same registry the compile-time expander uses. They evaluate at
runtime inside `:user::main`, inside test bodies, inside any
expression position that has an `:wat::WatAST` context.

**Why runtime-reachable.** The expander lives in `src/macros.rs` and
is invoked during the freeze pipeline. At runtime the frozen macro
registry is part of the `FrozenWorld`. A macroexpand primitive reads
that registry and applies the same `expand_form` logic — producing
an AST value that the caller can:
- Hand to `:wat::core::atom-value` to inspect pieces.
- Hand to `:wat::eval-ast!` to execute the expanded form.
- Print via stdio for debugging.
- `assert-eq` against a hand-built expected AST in a test.

**What it is NOT.** Not a textual pretty-printer. Not a tool that
evaluates the form. Not something that modifies the macro registry
(the registry is frozen; macroexpand is pure).

---

## Usage — the arc 029 forcing example

The arc 029 bug we want to diagnose:

```scheme
(:wat::test::make-deftest :my-deftest 1024 :error
  ((:wat::load-file! "wat/vocab/shared/time.wat")))

(:my-deftest :my-test body)   ;; fails with broken load in sandbox
```

With macroexpand, the diagnosis becomes:

```scheme
(:wat::core::define (:user::main ... -> :())
  (:wat::core::let*
    (((expanded :wat::WatAST)
      (:wat::core::macroexpand-1
        (:wat::core::quote (:my-deftest :my-test body)))))
    (:wat::io::IOWriter/println stdout
      (:wat::core::ast::to-string expanded))))
```

(Assuming `:wat::core::ast::to-string` exists — arc 030 can ship it
too, or defer to a later arc and use `(atom-value expanded)` piecing.)

The macroexpand-1 call returns the AST the generated inner macro
produces from `(:my-deftest :my-test body)`. If that AST has the
expected `(:wat::test::deftest :my-test 1024 :error ((load)) body)`
shape, the bug is in deftest's expansion. If it has a different
shape (e.g., `(unquote (unquote <list>))` still wrapped), the bug
is in make-deftest's outer expansion / nested-quasi resolution.

Diagnosis in a wat-level test — durable, re-runnable, documented.
No eprintln, no env vars, no rebuild-cycle dance.

---

## Implementation

**One function in src/runtime.rs + two dispatch arms.**

`eval_macroexpand_1(args, env, sym)`:
1. Arity check (1 arg).
2. Evaluate args[0] — expect `Value::wat__WatAST`.
3. Extract the inner `WatAST`.
4. Call `crate::macros::expand_form(ast, &sym.macro_registry, 0)`.
   OR — more precisely — a new `expand_once` helper that runs
   just one macro-call expansion (no fixpoint).
5. Wrap the result in `Value::wat__WatAST`.

`eval_macroexpand(args, env, sym)`:
1. Same 1-3 as above.
2. Call the existing `expand_form` which already runs to fixpoint
   per-subtree via its own recursion.
3. Wrap result.

Dispatch table entries in the main eval match arm:
```rust
":wat::core::macroexpand-1" => eval_macroexpand_1(args, env, sym),
":wat::core::macroexpand"   => eval_macroexpand(args, env, sym),
```

Scheme in src/check.rs:
```rust
":wat::core::macroexpand-1" — :wat::WatAST -> :wat::WatAST
":wat::core::macroexpand"   — :wat::WatAST -> :wat::WatAST
```

**The `expand_once` helper** (new in src/macros.rs) differs from
`expand_form` by NOT recursing into child forms after the single
expansion — matches Clojure's `macroexpand-1` behavior (one step,
not subtree-fixpoint).

**SymbolTable needs the MacroRegistry.** Currently the frozen
SymbolTable doesn't carry the macro registry — macros are consumed
during the freeze pipeline's expand pass and the resulting
expanded forms go through subsequent passes without the registry.
For runtime macroexpand, the registry must be accessible via
`sym.macro_registry()` — a new field on SymbolTable that freeze
populates.

---

## Slices

1. **Slice 1** — runtime primitives + scheme + SymbolTable carries
   the registry. `expand_once` helper. Rust unit tests exercising
   both macroexpand and macroexpand-1 against simple alias macros,
   multi-step chains, and nested-quasiquote templates. Wat-level
   tests in wat-tests/std/test.wat showing the user-facing shape.

2. **Slice 2** — INSCRIPTION + doc sweep + use the new tool to
   diagnose arc 029's remaining bug. Document the root cause in
   arc 029's BACKLOG. Wat-level test at `wat-tests/holon/` or
   similar showing `macroexpand` step-by-step on the problematic
   `make-deftest` call — captures the bug's signature for
   regression testing after the fix lands.

---

## Non-goals

- AST pretty-printer / to-string (deferred; may come in slice 1
  if trivial or a later arc).
- Source-level text reconstruction (the underlying canonical-EDN
  serializer exists from arc 028 era — if useful, can expose it).
- Step-through expansion UI (the CLI could grow one later on top
  of the primitives).
- Macro hygiene introspection (scope IDs on symbols are visible
  via the returned AST but no dedicated tool; use `atom-value`).

---

## Why this is inscription-class

Standard Lisp feature. Absent from wat. A real caller (arc 029
debugging) hit the wall of not-having-it and reached for invasive
Rust-side scaffolding instead. Shipping macroexpand:

- Closes the gap at the substrate
- Enables wat-level macro debugging for every future caller
- Provides the vocabulary for documenting macro behavior in tests
- Matches the absence-is-signal discipline the book named in
  Chapter 22 — the missing feature pointed at real substrate work

Same shape as arcs 017 / 018 / 020 / 023 / 025 / 029 — downstream
work surfaced a substrate gap; substrate gets the fix; downstream
work becomes tractable.
