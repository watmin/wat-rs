# DESIGN — fence-HOF: the 6a purity fence handles higher-order fold fns + fn-literals

## What + why (block-and-build prerequisite for 8-custom)
The 6a fence (`src/rete/purity.rs`, `pure?`/`deterministic?`) was built for simple `where`/`:test` predicates
(`(= ?n 3)`). It does **not** recognize the higher-order combinators (`foldl`/`foldr`/`map`/`filter`/`reduce`)
or `:wat::core::fn` literals — so it rejects **every real fold fn**, which all use both. This blocks custom
accumulators (8-custom) AND silently limits "filter with your own fn" in `where`/`:test` to non-fold fns. Fix
the class: teach the fence the HOF combinators (with **conditional** purity) + fn-literals. Confirmed RED:
`probe_arc278_fence_hof` — `pure?`/`deterministic?` return false for a pure fold.

## The contract (conditional purity — NOT blanket-allow)
A HOF is pure∧det **iff its fn-argument is** — i.e., `(foldl (fn …) 0 xs)` is pure iff the `(fn …)` body is
pure. This **falls out of the existing arg-recursion** (`classify_expr:251` already recurses every arg), so
marking the combinator pure∧det + classifying the fn-literal body gives conditional purity for free: an impure
fn-arg still fails. The guard test (`impure_fold_is_not_pure`) pins this — the fix must keep it false.

## The fix (three parts, `purity.rs` only)
1. **`intrinsic_meta`** (`:76`–`:157`): add `:wat::core::foldl` / `foldr` / `map` / `filter` / `reduce` to the
   `pure_det` set. (Verify none are in `is_effectful_op` — they aren't; if any were, that'd deny on the Pure
   axis regardless.)
2. **`classify_fn` Native arm** (`:279`, currently `FunctionBody::Native => false`): a Native fn must
   **consult `intrinsic_meta`** instead of blanket-deny —
   `FunctionBody::Native => intrinsic_meta(fqdn).is_some_and(|m| match axis { Pure => m.pure, Deterministic => m.deterministic })`.
   This is the load-bearing piece: foldl is **native AND in `sym.functions`**, so `head_ok` (`:171`) reaches
   `classify_fn` *before* `intrinsic_meta`; without this, step 1 is never consulted for foldl.
3. **`classify_expr` `:wat::core::fn` lambda arm** (add before the general-list arm `:243`): a fn-literal is
   `(:wat::core::fn [params] -> :ret body…)`. Classify the **body** (the forms after the `-> :ret`
   ascription); the param vector + ret-type are not evaluated. Mirror the `match`-arm's `->`-locating logic
   (`:224`) to skip to the body. Without this, a fn-literal hits the general-list arm with head
   `:wat::core::fn` (unknown call) → denied.

## Scope
`src/rete/purity.rs` ONLY. No `rete.wat`/`kernel.rs`/`runtime.rs`. The 8-custom dispatch (already in the tree,
uncommitted) is unblocked by this — it is the NEXT stone, not this one.

## Done = green
`probe_arc278_fence_hof` → 4/4 (pure fold/map pure∧det; impure fold rejected). No regressions: `--test
probe_arc278_6b_ii_a_where_oracle` (the where-fence still works) + the lib floor 941/36. (8-custom stays RED —
it greens in the next stone.)
