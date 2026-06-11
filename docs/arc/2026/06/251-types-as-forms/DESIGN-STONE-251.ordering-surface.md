# DESIGN — Stone: the instinct-faithful ordering surface (`sort` / `sort-by` / `reverse`)

**Status: STRIKE-READY (drawn 2026-06-10; intueri-ratified; four-questions + the LLM-instinct law).
Probe RED at HEAD.** A faithful-Clojure correction surfaced by the LLM-instinct test: wat's `sort-by`
takes a COMPARATOR (Clojure's `sort` mis-named), and there is no `sort` — so an LLM's most basic
reaches (`(sort xs)`, `(sort-by :k xs)`) stumble. The name an LLM trips over is the wrong name; we
fix it, not bank it. Dogfoods Arc 256 (generic multi-arity defclause).

## The ratified surface (Clojure-exact, fn-first)
| Verb | Shape | Types |
|---|---|---|
| `sort` | `(sort coll)` natural · `(sort cmp coll)` | `cmp : (T,T)->bool` (boolean less-than — `(sort < xs)`, NOT Java 3-way) |
| `sort-by` | `(sort-by keyfn coll)` · `(sort-by keyfn cmp coll)` | `keyfn : (T)->K`, `cmp : (K,K)->bool` |
| `reverse` | `(reverse coll)` | unchanged ✓ |

Blunt instinct check (must all pass): `(sort xs)`, `(sort < xs)`, `(sort-by :k xs)`, `(reverse xs)`.

## The design — one Rust primitive, the surface in wat (dogfoods 256)
- **Rust: rename the existing comparator-sort to a primitive.** `eval_vec_sort_by` /
  `:wat::core::sort-by` (transform.rs:135; fn-first `(less? xs)`, scheme `(Fn(T,T)->bool, Vector<T>)
  -> Vector<T>`) → **`:wat::core::sort'`** (the engine; `'` = primitive, like `spawn-program'`).
  No logic change — just the name + scheme key.
- **wat: `sort` + `sort-by` as generic multi-arity defclauses in `core.wat`** (256 made generic
  defclause real; arity-overload IS what defclause does), over `sort'` + `<`:
  - `(sort coll)`            → `(sort' (fn [a b] (< a b)) coll)`   *(default comparator = `<`)*
  - `(sort cmp coll)`        → `(sort' cmp coll)`
  - `(sort-by keyfn coll)`   → `(sort' (fn [a b] (< (keyfn a) (keyfn b))) coll)`
  - `(sort-by keyfn cmp coll)` → `(sort' (fn [a b] (cmp (keyfn a) (keyfn b))) coll)`
  Dispatch is purely by ARITY (sort: 1 vs 2; sort-by: 2 vs 3) — no fn-type discrimination needed.
  All clauses generic over `T` (and `K` for `sort-by`) via 256's signature free-var generalization.

## The lenient-orderable seam (honest, not a wart)
`(sort coll)` and `(sort-by keyfn coll)` use `<`, which is `Orderable`-lenient today (it'd accept
`(sort [fn1 fn2])`) — the SAME honest seam already in `<`/`>`/`=`. **Bounded-`∀` later tightens `sort`,
`sort-by`, AND the ordering intrinsics as one family** (reject non-orderable). The LLM-instinct test
RE-PRICED bounded-poly: the `(sort xs)` reach proves it is *instinct-blocking*, not "optional
uniformity" — recorded in the 256 DESIGN out-of-scope.

## Migration (clean cut — `sort-by` changes MEANING, so no dual-read)
Within wat-rs, every `sort-by`-as-comparator caller is in TESTS: `tests/collection/sort_by.rs`
(5 cases) → rename file `sort.rs`, each `(sort-by cmp xs)` → `(sort cmp xs)` (identical arg shape).
(The trading-lab `atr-window.wat` caller is archived for rebuild on the durable substrate — not now.)

## The probe (RED at HEAD)
`tests/probe_arc251_ordering_surface.rs`: C01 `(sort [3 1 2])`→[1 2 3]; C02 `(sort > [1 2 3])`→[3 2 1];
C03 `(sort-by negate [1 2 3])`→[3 2 1]; C04 `(sort-by id > [1 2 3])`→[3 2 1]; C05 `(reverse …)` green.
RED at HEAD (sort UnknownFunction; sort-by rejects a key-fn). C05 confirms reverse preserved.

## Out of scope
- Bounded-`∀` tightening (its own arc; re-priced as instinct-blocking).
- A `compare`/3-way `Ordering` surface — NOT instinct (Clojure has no `Comparator` type; `<` is the
  comparator). Affirmatively cut.
