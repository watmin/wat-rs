# DESIGN — Stone 251.4c: `:->` the function-type arrow (`[i64 :-> i64]`)

**Status: STRIKE-READY (drawn 2026-06-10, on a grounded crawl). Probe RED at HEAD before build.**

Predecessor: 251.4b (`ann-form`). Closes the 251.4 annotation cluster. Home: `src/types.rs`.

## The move (core.typed parity)

core.typed writes a function type as `[arg… :-> ret]` — a bracketed form with `:->` separating
the argument types from the return type. wat today writes fn types as the keyword form
`:wat::core::Fn(i64)->bool` (`parse_fn_body`, types.rs:2563). 251.4c adds the bracket form
`[wat.type/i64 :-> wat.type/i64]` as a DUAL-READ alias producing the SAME `TypeExpr::Fn`.
Usage: `[f :- [wat.type/i64 :-> wat.type/i64]]` (a param `f` of function type).

## The disk (grounded)

- `TypeExpr::Fn { args, ret }` (types.rs:86) is the target representation; `parse_fn_body`
  (types.rs:2563→2594) builds it from the keyword `:wat::core::Fn(args)->ret` form.
- `parse_type_node` (types.rs:2223) dispatches Keyword | Symbol | List — **no Vector arm**. A
  `[…]` fn-type in a type slot is unparseable today → the RED.
- `:->` is a keyword token (`WatAST::Keyword(":->")`), parallel to `:-` (which lexes natively,
  per 251.4a). Confirm it lexes via the probe's HEAD failure mode (do NOT assume — the `:-`
  precedent showed lexing must be verified, and `:->` contains `>` which the keyword lexer's
  angle-bracket tracking touches).

## The strike (251.4c)

1. **`parse_fn_type_bracket(items, span) -> Result<TypeExpr, TypeError>` (src/types.rs):**
   a `[T… :-> R]` Vector → `TypeExpr::Fn { args, ret }`. Split the items on the lone `:->`
   keyword: everything before = arg types (each parsed via `parse_type_node`), everything after
   = the single return type. Errors: no `:->` → "fn-type bracket needs a `:->` arrow"; >1 return
   type or no return → clear error. Produce BYTE-IDENTICAL `TypeExpr::Fn` to what `parse_fn_body`
   yields (so unification with the `:wat::core::Fn(...)->...` keyword form succeeds — the probe's
   sink proves this).
2. **`parse_type_node` gains a `WatAST::Vector(items, span)` arm** → `parse_fn_type_bracket`.
3. **Dual-read:** the `:wat::core::Fn(i64)->bool` keyword form (parse_fn_body) is UNCHANGED.

## The probe (RED at HEAD)

`tests/probe_arc251_stone4c_fn_type_arrow.rs`:
- **C01 (RED→GREEN):** a `[wat.type/i64 :-> wat.type/i64]` fn-typed param, load-bearing — its
  value is passed to a sink fn typed with the keyword `:wat::core::Fn(...)->...` spelling, so the
  bracket form must produce the SAME `TypeExpr::Fn` for unification. RED at HEAD (no Vector arm).
- **C02 (dual-read):** the `:wat::core::Fn(wat::core::i64)->wat::core::i64` keyword spelling
  still checks (PRESERVATION — the keyword fn-type retires at 251.5).

## Out of scope (named)

- Corpus adoption of `[… :-> …]` + the keyword fn-type retirement → the unified **251.5** sweep.
- Multi-arg / zero-arg fn-type edge spellings beyond what the probe exercises → covered by the
  same split logic; extend the probe if the corpus surfaces a shape the split misses.
