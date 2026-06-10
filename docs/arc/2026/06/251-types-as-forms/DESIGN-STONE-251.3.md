# DESIGN — Stone 251.3: parametrics-as-FORMS (`(wat.type/Vector wat.type/i64)`)

**Status: STRIKE-READY (drawn 2026-06-10, on a full crawl of the binder↔type-slot grammar).
Probe RED at HEAD before build.**

Predecessor: 251.2a (scalar type atoms → `wat.type/` symbols; `:wat::type::` aliases to the
internal canonical). Successor: 251.4 (`:-`/`ann-form`). Home: `src/types.rs` + the type-slot
readers in `src/argspec/` and `src/function/`.

## The change — types become genuine FORMS

Today a type in a binder/return slot is a **keyword string** read by `parse_type_expr_with_span`.
A parametric type is spelled with the `<>` keyword surface (`:wat::core::Vector<T>`), which the
lexer's `lex_keyword` keeps as a single token via dedicated `angle_depth` tracking
(lexer.rs:637–730). 251.3 moves parametrics to **s-expr forms**: `(wat.type/Vector wat.type/i64)`,
a `WatAST::List` whose head is the constructor symbol and whose args are type forms. "Types
become forms the macro engine computes over" (arc DESIGN) — the payoff that ends `keyword/of`'s
string-concat type construction and the run-threads Bundle-surgery type destructuring.

## The disk (grounded)

Type-position readers accept ONLY `WatAST::Keyword` today — a `WatAST::List` in a type slot is
not handled:
- `src/argspec/parse.rs:187` — binder type slot (`[x <- TYPE]`).
- `src/function/parse.rs:170` — return type slot (`-> TYPE`).
- `src/types.rs:1834 / 1873 / 1940` — struct/enum/typeunion field-type readers.

Each is `WatAST::Keyword(k, span) => parse_type_expr_with_span(&k, &span)`. A parametric FORM
`(wat.type/Vector …)` (a List) falls through to the "expected type keyword" error → the RED.

## The strike

1. **`parse_type_form(node: &WatAST) -> Result<TypeExpr, TypeError>` (src/types.rs):** read a
   `(wat.type/Ctor arg…)` List → `TypeExpr::Parametric { head, args }`, recursively parsing each
   arg (atom symbol `wat.type/i64` → already normalized to `:wat::type::i64` → the 251.2a alias →
   `Path(":wat::core::i64")`; nested form → recurse). The head maps to the internal `Parametric`
   head convention (`wat::core::Vector` etc. — same storage the `<>` surface produces, so the
   checker is UNCHANGED).
2. **Wire the readers:** at the five sites above, accept `WatAST::List` → `parse_type_form` in
   addition to `WatAST::Keyword` → `parse_type_expr_with_span`. One shared dispatch
   (`parse_type_node(node)` matching Keyword|List) keeps the five sites uniform — candidate for a
   single helper both the runtime and check paths call (mirrors argspec's existing unification).
3. **Dual-read:** `:wat::core::Vector<T>` keyword AND `(wat.type/Vector T)` form both produce the
   same `Parametric` — both valid through the transition.

## Sequencing (churn-once, consistent with 251.2)

- **251.3a** — `parse_type_form` + reader dual-read; probe RED→GREEN.
- **(DEFERRED to the unified 251.5 sweep):** the `angle_depth` `<>` lexer machinery DELETION
  (lexer.rs:637–730) and the corpus migration (`:wat::core::Vector<T>` ×126 → `(wat.type/Vector
  T)`). You cannot delete `<>` lexing while the corpus still uses it; both ride the one sweep.
- **251.3w** — diff-scoped check of the touched readers (gated by the probe + full suite).

## The probe (RED at HEAD)

`tests/probe_arc251_stone3_parametric_form.rs`:
- **C01 (RED→GREEN):** a parametric type FORM `(wat.type/Vector wat.type/i64)` in a binder slot
  type-checks, with a LOAD-BEARING body (a Vector<i64> op, not an identity fn — learn from 251.2's
  hollow-probe catches). RED at HEAD (the type slot rejects a List).
- **C02 (dual-read):** the `:wat::core::Vector<:wat::core::i64>` keyword spelling still checks.

## Out of scope (named)

- `<>` lexer deletion + parametric corpus migration → **251.5** unified sweep.
- `:-` / `ann-form` annotation surface → **251.4**.
- Internal `Parametric`-head flip to `wat::type::` → **251.5** (one-canonical-path).
