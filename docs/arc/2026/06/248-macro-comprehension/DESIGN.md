# Arc 248 — generative-macro comprehension (`for` in templates)

**Status:** OPEN 2026-06-03. **The parent of the chain we descended.** Arc 247 (Clojure-honest `map`) was built as *its* dependency — the comprehension must stand on a `map` that tells the truth about the dialect. Chain: **237 ⇠ 248 (this) ⇠ 247 (closed).** Winding back up. (The `->>` sibling banked in 247's SCORE renumbers to **arc 249**; this — the generative-macro spine — takes 248.)

## Why — the destination of the whole descent

237 is **polymorphism-*consolidation*** — one dispatch mechanism (`defclause`) for the monomorphic ops. Numerics consolidated (8a/8b). Equality (8c) is correct *as a value* (Shape B, the uniform structural primitive) but **un-consolidated as a mechanism** — a bespoke Rust `eval_eq`/`infer_equality`, not a `defclause`. To consolidate it, equality must become a `defclause`. But equality is *uniform*: a hand-written equality defclause is **~22 identical clauses** (`[a <- :T b <- :T] -> :bool (eq a b)` per type) — pure ceremony.

A **macro that maps a clause-template over the type-list** generates those clauses: DRY source, the `defclause` mechanism, zero ceremony. That is how equality joins the one mechanism. But wat's `defmacro` is **quasiquote-only** — it can *splice* a list (`,@rest`) but cannot **map** (transform each element via a sub-template). This arc adds the missing capability. Without it, equality cannot consolidate without ceremony; with it, it can — and so can all the per-Type dispatch boilerplate.

## The capability — a BOUNDED template comprehension

A single new template form: **`for`**, used in splice position.

```
;; generate one clause per type T in `types`:
,@(:wat::core::for [T types]
    `([a <- ,T  b <- ,T] -> :wat::core::bool (:wat::core::eq a b)))
```

At **macro-expansion time**, `walk_template` recognizes `(:wat::core::for [<binder> <list>] <element-template>)`:
1. Resolve `<list>` (a macro parameter bound to a list of AST, e.g. a rest-binder, or a literal list).
2. For each element: bind `<binder>` to it (hygienically — the macro scope per iteration), walk `<element-template>` with that binding.
3. Collect the results; in splice position (`,@`), splice them into the surrounding form.

**Bounded, NOT Turing-complete.** It is `map`, not `eval`: iterate a *finite* list, instantiate a *template* per element. This deliberately preserves every quasiquote virtue the spec was built on:
- **Hygiene** — each iteration is a hygienic template instance (sets-of-scopes per the existing machinery).
- **Termination** — a finite list cannot loop; no recursion, no arbitrary computation.
- **Predictability** — `N` elements → `N` instances; an LLM (and a human) can see the expansion.

This is the "helpers beyond quasiquote" the macro spec already admits (`src/macros.rs:39`), in its most principled form. The line held: *generation, not computation.*

## Crawl — the plug-in point (grounded)

- **`fn walk_template`** (`src/macros.rs:839`) — the quasiquote walker. Its `List` arm recognizes `quasiquote`/`unquote`/`unquote-splicing` via `match_unquote` (853/877/909) and walks children handling splice inline (904-942). The `for` form is recognized here (a new `match_unquote`-style arm) and iterates.
- **`fn splice_argument`** (`src/macros.rs:1153`) — handles `,@name` (drop a list's elements). The `for` iteration produces the list to splice; reuse or generalize this path.
- **`fn substitute_bindings`** (`:1051`) / the binding map — the `<binder>` adds to `bindings` per iteration (scoped).
- Macros are quasiquote-only by current design (`:164` "this slice handles quasiquote-template bodies only"); the `for` arm is the first sanctioned extension.

## FM-2-bis probe (settle before BRIEF)

`tests/probe_arc248_macro_for_comprehension.rs` (to author). Gates:
- **Generates N forms:** a `defmacro` using `,@(for [x xs] template)` over a 3-element list expands to 3 instantiated forms (RED at HEAD — `for` is unrecognized; un-ignore after).
- **Hygiene:** the `<binder>` doesn't capture/leak across iterations or into the call site.
- **Termination/finite:** the list is consumed exactly once; N elements → N forms.
- **The real target:** a macro generating a small `defclause` from a type-list expands + type-checks + dispatches correctly (the equality-defclause shape in miniature — i64 + f64 clauses generated, both work).

## Slicing

Likely **two stones:** (248.1) the `for` template form in `walk_template` + `splice_argument` (the capability + probe green); (248.2) the equality consolidation — replace Shape B's `eval_eq`/`infer_equality` with a macro-generated `=`/`not=` defclause (folds the 237.8c reframe: equality joins the mechanism). 248.2 closes the equality thread 237.8c opened. Confirm slicing at DESIGN-lock.

## Constraints

- Edits: `src/macros.rs` (the `for` recognition + iteration), the probe; then (248.2) `wat/core.wat` (the generated `=`/`not=` defclause) + retire `infer_equality`/`eval_eq`-as-`=`.
- BOUNDED only — no recursion, no arbitrary expansion-time computation. `for` over a finite list, full stop. Anything more is a separate, deliberate decision.
- Hygiene preserved (sets-of-scopes); terminates; predictable expansion.
- Green-gate: `cargo test --release --lib -p wat` + `cargo build --release --tests --workspace`, raw commands.
- No `holon-rs`.

## Lineage

The destination of the day's descent: equality → "a clause" → "22 clauses is ceremony" → "a macro generates them" → "the macro can't map" → "the `map` is a dialect lie" → **247 fixed the `map`** → **248 builds the macro** → equality consolidates. Uses the thing we forged.
