# BRIEF — Arc 247 — Clojure-honest seq-HOF order (fn-first)

**Mission.** Flip the 5 coll-first seq-HOFs to Clojure's **fn-first** order, so the substrate stops lying about being clojure-on-rust. Full rationale in `DESIGN.md` (same dir) — read it first; this is the strike order.

The contract is the probe **`tests/probe_arc247_hof_fn_first.rs`** (currently 1 passed / 4 ignored). Done when all **5** pass with zero `#[ignore]`.

## The flip — 5 HOFs, coll-first → fn-first

| op | now | target | runtime impl |
|---|---|---|---|
| `:wat::core::map` | `(map xs f)` | `(map f xs)` | `eval_vec_map` (runtime.rs ~11300) |
| `:wat::core::filter` | `(filter xs pred)` | `(filter pred xs)` | `eval_vec_filter` (~11410) |
| `:wat::core::foldl` | `(foldl xs init f)` | `(foldl f init xs)` | `eval_vec_foldl` (~11336) |
| `:wat::core::foldr` | `(foldr xs init f)` | `(foldr f init xs)` | `eval_vec_foldr` (~11373) |
| `:wat::core::sort-by` | `(sort-by xs keyfn)` | `(sort-by keyfn xs)` | `eval_vec_sort_by` (~11225) |

Each flip is three coordinated edits:
1. **Runtime impl** (`src/runtime.rs`): swap the `args[i]` extraction so the fn/pred/keyfn is now `args[0]` (map/filter), or `args[0]` with init `args[1]` and coll `args[2]` (foldl/foldr), keyfn `args[0]` + coll `args[1]` (sort-by).
2. **Check inference** (`src/check.rs`): the type scheme / custom arm for each — the `f`/`pred`/`keyfn` argument now in first position, the collection in last; the function's expected signature unchanged, only its position moves.
3. **Call sites** (the cascade): every invocation flips.

## The cascade

Flipping arg order breaks every call site — the type checker flags each (the moved `f` now mis-types in its old slot). This is the substrate-as-teacher cascade; migrate each in turn.
- **First:** the internal folds in `wat/core.wat:101-187` — the arithmetic defclauses call `(:wat::core::foldl rest seed (fn ...))` → `(:wat::core::foldl (fn ...) seed rest)`. These flip immediately or the lib won't build.
- **Then:** `wat/list.wat`, `wat/stream.wat`, `wat/Record.wat`, `wat/holon/*`, `wat-tests/`, `tests/`, `examples/` — every `(map xs f)`/`(filter xs p)`/`(foldl xs i f)`/`(foldr xs i f)`/`(sort-by xs k)` → fn-first.

## HARD CUT

The old coll-first order is **deleted**, not aliased. No `(map xs f)` compatibility shim. After the strike, coll-first is a check error (the `mint_map_coll_first_is_gone` gate confirms).

## Un-ignore the 4 probe confirmers as the flip lands; drive the probe to 5/5.

## Affirmative scope — do NOT touch

- **`apply`** — `(apply f args)` is already fn-first. Leave it.
- **Collection ops** `get`/`conj`/`assoc`/`length`/`empty?`/`contains?`/`dissoc`/`keys`/`values` — coll-first is *Clojure-correct* for these. Do NOT flip them.
- **Threading:** if you find the flip wants thread-last `->>` to be ergonomic, NOTE it for a sibling arc — do not build `->>` here unless it already exists and the flip trivially needs a one-word touch.
- No `holon-rs`.

## Green-gate (raw commands)

- `cargo test --release --test probe_arc247_hof_fn_first` → **5 passed / 0 ignored**.
- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged).
- `cargo build --release --tests --workspace` → clean.

Leave all changes uncommitted. Do not commit/tag/push.
