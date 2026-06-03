# BRIEF — Arc 248 Stone 248.1 — the `for` template comprehension

**Mission.** Add a **bounded template comprehension** to `defmacro`: a `for` form that, at macro-expansion time, maps a sub-template over a finite list and splices the results. Full rationale + the bounded-not-Turing principle in **`DESIGN.md`** (same dir) — read it first; this is the strike order.

The contract is the probe **`tests/probe_arc248_macro_for_comprehension.rs`** (currently 1 passed / 2 ignored). Done when all **3** pass with zero `#[ignore]`.

## The form

```
,@(:wat::core::for [<binder> <list>] <element-template>)
```
- `<list>` — resolves from the macro's bindings (a `WatAST::List`, typically a `&`-rest-binder param).
- `<binder>` — bound, per iteration, to the current element; **unquote-able in the template as `~<binder>`** (same as a macro param: `~name`).
- `<element-template>` — walked once per element, with `<binder>` bound.
- In splice position (`,@`), the collected results are spliced into the surrounding form.

Worked (from the probe):
```
`(:wat::core::Vector :wat::core::i64 ~@(:wat::core::for [x items] (:wat::core::i64::+ ~x 1)))
```
`(:my::inc-vof 10 20 30)` → `(:wat::core::Vector :wat::core::i64 (i64::+ 10 1) (i64::+ 20 1) (i64::+ 30 1))` → `[11 21 31]`.

## The plug-in (src/macros.rs)

- **`fn walk_template`** (~:839) — the quasiquote walker. Its `List` arm recognizes `quasiquote`/`unquote`/`unquote-splicing` via `match_unquote` (~853/877/909). **Add a `for` recognition.** When a child in splice position is `(:wat::core::for [binder list] template)`:
  1. Resolve `list` from `bindings` (the `binder`-name's bound `WatAST::List`).
  2. For each element: produce a per-iteration binding `binder → element` (layered onto `bindings`), `walk_template` the `<element-template>` with it, collect the result.
  3. Splice the collected vec into `out` (the same way `splice_argument` results are `out.extend`'d at ~915).
- **`fn splice_argument`** (~:1153) — the existing `,@name` path; generalize or sit beside it for the `for` case.
- **Hygiene** — each iteration's `binder` binding is local (does not leak to the call site or across iterations); template-origin symbols still get the macro scope (`add_scope`) as today. Reuse the existing sets-of-scopes machinery — do not invent a new hygiene path.

## Bounded — NOT Turing-complete

`for` iterates a **finite list** and instantiates a **template** per element. That is the whole capability: `map`, not `eval`.
- NO recursion, NO arbitrary expansion-time computation, NO conditionals/branching in the macro body (the spec's quasiquote-only virtue is preserved — `for` is the one sanctioned extension).
- Finite list → terminates. N elements → N instances. Predictable.
- If you find yourself wanting more than "iterate a list, instantiate a template," STOP — that's a separate, deliberate decision, out of this stone.

## The work

1. Recognize `(:wat::core::for [binder list] template)` in `walk_template`'s splice handling.
2. Resolve `list` from bindings; iterate; layer `binder → element` per iteration; walk the template; collect; splice.
3. Hygiene: `binder` scoped per-iteration, no leak; template symbols keep the macro scope.
4. Un-ignore the 2 mint tests in the probe; drive it to 3/3.

## Green-gate (raw commands)

- `cargo test --release --test probe_arc248_macro_for_comprehension` → **3 passed / 0 ignored**.
- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged — existing macros/`,@` must not regress).
- `cargo build --release --tests --workspace` → clean.

Leave all changes uncommitted. Do not commit/tag/push.
