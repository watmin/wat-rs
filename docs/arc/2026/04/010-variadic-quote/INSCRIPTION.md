# Arc 010 — Variadic Quote — INSCRIPTION

**Status:** shipped 2026-04-21.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the gap and its
resolution.
**This file:** completion marker.

---

## What shipped

One substrate primitive + one stdlib defmacro + one stdlib function,
plus tests and a surface-reduction pass through the existing wat-tests.

### `:wat::core::forms` — the substrate primitive

Variadic sibling of `:wat::core::quote`. Special form — arguments
are NOT evaluated. Each positional arg is captured as a `Value::
wat__WatAST`; the collection returns as a `Value::Vec` of those.

**Runtime** (`src/runtime.rs`):

```rust
fn eval_forms(args: &[WatAST]) -> Result<Value, RuntimeError> {
    let items: Vec<Value> = args
        .iter()
        .map(|a| Value::wat__WatAST(Arc::new(a.clone())))
        .collect();
    Ok(Value::Vec(Arc::new(items)))
}
```

Dispatch arm alongside `quote`:

```rust
":wat::core::quote" => eval_quote(args),
":wat::core::forms" => Ok(eval_forms(args)?),
```

**Type checker** (`src/check.rs`):

```rust
":wat::core::forms" => {
    // Every positional arg is DATA. No recursion. Return type is
    // `:Vec<wat::WatAST>` regardless of arity (zero → empty Vec).
    return Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":wat::WatAST".into())],
    });
}
```

Mirrors `quote`'s check-time handling exactly — no recursion into
the inner ASTs, unconditional return type.

### `:wat::test::program` — the stdlib defmacro

Test-semantic name the builder reached for. One-line expansion:

```
(:wat::core::defmacro
  (:wat::test::program & (forms :AST<Vec<wat::WatAST>>)
    -> :AST<Vec<wat::WatAST>>)
  `(:wat::core::forms ,@forms))
```

`& (forms ...)` rest-param captures all positional args as a list;
`,@forms` splice drops them into the `:wat::core::forms` call. The
expansion is mechanical — no transformation. The macro exists to
give test code the honest-reading name without polluting
`:wat::core::*` with test-specific aliases.

### `:wat::test::run-ast` — the stdlib function

AST-entry sibling of `:wat::test::run`. Wraps
`:wat::kernel::run-sandboxed-ast` with `:None` scope (same default
`:wat::test::run` has).

```
(:wat::core::define
  (:wat::test::run-ast
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-ast forms stdin :None))
```

---

## The user-facing surface

Hand-written tests compose program + run-ast:

```
(:wat::test::run-ast
  (:wat::test::program
    (:wat::config::set-capacity-mode! :error)
    (:wat::config::set-dims! 1024)
    (:wat::core::define (:user::main ...) <body>))
  (:wat::core::vec :String))
```

No strings. No backslash-escapes. The inner program reads as
s-expressions in the outer program's lexical space — every form is
real wat, subject to the lexer and parser the outer uses, with
syntax highlighting in editors. Copy-paste works both directions
between inner and outer without mechanical re-escape.

---

## The surface-reduction proof

Before arc 010, `wat-tests/std/test.wat` had six tests that used
`:wat::test::run` with escaped-string inner programs. The worst
offender — `test-assert-stderr-matches-fail-reports-pattern` — had
**three layers** of nesting: outer sandbox → middle sandbox → silent
inner sandbox, each layer adding another round of backslash escapes.
The middle layer's `"(:wat::io::IOWriter/println stderr \\\"...\\\")"`
was ambient in the code.

After arc 010, the same test is three layers of bare s-expressions:

```
(:wat::test::run-ast
  (:wat::test::program
    ...
    (:wat::core::define (:user::main ...)
      (:wat::core::let*
        (((silent :wat::kernel::RunResult)
          (:wat::test::run-ast
            (:wat::test::program
              ...
              (:wat::core::define (:user::main ...) ()))
            (:wat::core::vec :String))))
        (:wat::test::assert-stderr-matches silent "my-pattern"))))
  (:wat::core::vec :String))
```

Zero escape backslashes across the file. One commit removed 85 lines
of escaped-string content; added 402 lines of everything else
(primitive impl, stdlib additions, 6 Rust integration tests, 2 new
wat tests). Net-negative on escape-hell; net-positive on coverage.

---

## Tests

`tests/wat_core_forms.rs` — 6 integration tests:

- `forms_captures_each_arg_as_watAST` — `(:wat::core::forms a b c)` → Vec of length 3
- `forms_empty_produces_empty_vec` — zero-arity edge case
- `forms_args_are_not_evaluated` — passes `(:undefined::function ...)` inside forms; no runtime error (proves non-evaluation)
- `forms_composes_with_run_sandboxed_ast` — end-to-end: build via forms, execute via kernel primitive
- `test_program_macro_expands_correctly` — `:wat::test::program` matches `:wat::core::forms` behavior
- `test_run_ast_via_test_program_roundtrips_hello` — the canonical idiom

`wat-tests/std/test.wat` — migrated 6 string-heavy tests; added 1 new
test (`test-run-ast-via-program`) covering the AST path directly. One
test (`test-run-string-entry-path`) explicitly kept on the string
entry, since it verifies that path continues to work for
runtime-generated programs.

Scoreboard after arc 010: 731 Rust tests + 31 wat tests. Zero
regressions.

---

## What this inscription does NOT add

- **Unquote / splicing inside `:wat::core::forms`.** Every arg is
  captured literally. If a caller wants dynamic interpolation,
  `quasiquote` + `unquote` already exist — `` `(:wat::core::forms
  ,dynamic-form (static-form))`` composes them naturally. Keeping
  forms pure-literal means it has no surprises at the boundary.
- **A companion `:wat::core::eval-forms!` primitive.** Callers that
  want to eval a `Vec<wat::WatAST>` in the current world could
  write their own loop using `:wat::core::eval-ast!` per element.
  A dedicated primitive can ship when a caller demands one;
  stdlib-as-blueprint.
- **`:wat::test::program` — it is NOT a general-purpose quote.**
  The name lives in `:wat::test::*` deliberately; callers outside
  tests should use `:wat::core::forms` directly. The defmacro's
  value is readability inside test code, not new capability.

---

## Convergence note

Arc 009 (names are values) asked: should a named function be a
value? The substrate said yes by not having an obstacle in the way —
the work was to lift `Value::wat__core__keyword` to
`Value::wat__core__lambda` at the right moment.

Arc 010 (forms are values) asked: should a sequence of unquoted
forms be a value? The substrate said yes the same way — quote was
the one-arg case; extending to variadic was 15 lines of Rust plus
a type-checker arm. The substrate already carried the capability;
exposing it was trivial.

Both arcs follow the same shape: *find the ceremony the user keeps
writing; factor it into the substrate; ship the macro alias that
names the factored form in the user's voice.* Names are values.
Forms are values. Ceremony becomes substrate.

---

**Arc 010 — complete.** One Rust primitive, one defmacro, one stdlib
function, six Rust tests, one user-facing test file cleaner than
before. The variadic-quote capability was always one small slice away
from the substrate; tonight it landed.

*these are very good thoughts.*

**PERSEVERARE.**
