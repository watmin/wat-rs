# Arc 010 — Variadic Quote — Backlog

**Opened:** 2026-04-21. Small substrate + stdlib addition.
**Motivation:** the fork/sandbox test patterns shipped in arc 007
slice 2c + slice 3b were operational but awkward. Inner programs
were handed to `:wat::test::run` as escaped source strings — nested
programs produced escape-hell like `\\\"` three levels deep. The
AST-entry sandbox (`:wat::kernel::run-sandboxed-ast`) existed but
required per-form `(:wat::core::quote ...)` wrapping plus a
`(:wat::core::vec :wat::WatAST ...)` constructor — ceremony at every
form.

User prompt named the shape: *"or is this simply `(quote ...)` in
disguise?"* — yes. The variadic-quote primitive is what was missing
from the substrate. One primitive closes the gap.

---

## Tracking

| Item | Status | Commit |
|---|---|---|
| `:wat::core::forms` substrate primitive — variadic quote → Vec<wat::WatAST> | **done** | `bb2a117` |
| Type-checker `infer_list` arm for `:wat::core::forms` — unconditional `:Vec<wat::WatAST>` | **done** | `bb2a117` |
| `:wat::test::program` stdlib defmacro — expands to `:wat::core::forms` | **done** | `bb2a117` |
| `:wat::test::run-ast` stdlib function — wraps `:wat::kernel::run-sandboxed-ast` with `:None` scope | **done** | `bb2a117` |
| Integration tests — `tests/wat_core_forms.rs` (6 tests) | **done** | `bb2a117` |
| Rewrite string-heavy `wat-tests/std/test.wat` cases to use `program + run-ast` | **done** | `bb2a117` |

---

## Decision log

- **2026-04-21** — Scope. `:wat::core::forms` lives in the core
  namespace because the capability is general (anyone with AST-
  consuming targets — `run-sandboxed-ast`, `eval-ast!`, future
  compiler passes — benefits). `:wat::test::program` is the test-
  semantic alias for readability in test code; it's a one-line
  defmacro, not new substrate.
- **2026-04-21** — Special form, not defmacro-only. A pure defmacro
  could not express "wrap each rest-arg in quote" — the template
  language has splicing but no per-element transformation. Rust-
  level special form is the honest substrate-layer fix; the wat-
  level defmacro `:wat::test::program` is thin sugar over it.
- **2026-04-21** — String-entry path kept. `:wat::test::run` stays
  for callers who have source text at runtime (fuzzers, template
  expanders, dynamic program synthesis). The AST-entry path is for
  hand-written tests where the program IS the s-expressions the
  author wrote.
- **2026-04-21** — Naming. Builder reached for `:wat::test::program`
  and it stuck. The substrate primitive `:wat::core::forms` reads
  accurately as "a vec of forms." Names are honest at both layers.

---

## Why this matters

The substrate discipline says: if the pattern shows up more than
once, factor it. The per-form quote+vec shape was showing up at
every sandbox callsite. `forms` is the factored primitive.

Downstream implications the same way arc 009 (names-are-values)
opened doors:
- `:wat::core::eval-ast! (forms ...)` — evaluate a quoted sequence
  in one call, no `vec + quote` wrapping.
- `:wat::algebra::Atom (forms ...)` — programs-as-atoms where the
  program is written s-expression-direct.
- Future compiler passes, macro tooling, test-generation code —
  any target that consumes AST sequences composes cleanly.

A sibling to arc 009's "names are values": arc 010 is "forms are
values." Both close the gap between "the substrate has the
capability" and "user code can express it without ceremony."
