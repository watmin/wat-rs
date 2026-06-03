# BRIEF — Stone 246.1 — lift the collection dispatch into `src/collection/`

**Mission.** Lift the collection operations out of the flat `src/check.rs` + `src/runtime.rs` into a new warded-home-in-waiting `src/collection/`, and redirect the central dispatch arms into it. **Behavior-preserving — a pure move + redirect, no logic changes.** Full rationale + the intueri-cast names: `DESIGN.md` (same dir) + `docs/DISPATCH.md`. Read both first.

This stone does NOT ward (that's 246.2) and does NOT change any op's behavior. The existing `cargo test` suite is the regression guard — it must stay green by construction.

## The home to create

```
src/collection/
  mod.rs        — home root. Module-doc states the clause-vs-intrinsic partition
                  doctrine (the word "intrinsic" lives HERE in prose; `get` as the
                  worked proof; cite docs/DISPATCH.md). Declares `mod infer; mod eval;
                  mod transform;`. NO vigilatum stamp yet (earned in 246.2). Mirror
                  the `src/function/mod.rs` doc shape (without the stamp line).
  infer.rs      — the 4 check-side inference intrinsics.
  eval.rs       — the ~30 runtime per-Type dispatch impls + the 3 constructors.
  transform.rs  — the ~12 Vector/List-specific utility ops.
```

Register the home: add `pub mod collection;` to `src/lib.rs` (alongside `pub mod check;` / `pub mod types;` etc.).

## What moves (by name — `cargo build` names any straggler)

**→ `collection/infer.rs`** (from `src/check.rs`), `pub(crate)`:
- `infer_contains`, `infer_conj`, `infer_get`, `infer_assoc`.

**→ `collection/eval.rs`** (from `src/runtime.rs`), `pub(crate)` — the container-polymorphic dispatch:
- `eval_<vector|hashmap|hashset|list>_<length|empty_q|contains_q|get|conj|assoc|dissoc|keys|values|concat>` (the ~30 you find by that pattern) + the constructors `eval_list_ctor`, `eval_hashmap_ctor`, `eval_hashset_ctor`.

**→ `collection/transform.rs`** (from `src/runtime.rs`), `pub(crate)` — the Vector/List-specific utilities:
- `eval_vec_map`, `eval_vec_filter`, `eval_vec_foldl`, `eval_vec_foldr`, `eval_vec_sort_by`, `eval_vec_reverse`, `eval_vec_range`, `eval_vec_take`, `eval_vec_drop`, `eval_vec_last`, `eval_vec_rest`, `eval_vec_find_last_index`, `eval_list_zip`, `eval_list_window`, `eval_list_remove_at`, `eval_list_map_with_index`.

**Do NOT move:** `dispatch_keyword_head` / `dispatch_keyword_head_value` (the central match — STAYS, redirects), `dispatch_rust_scheme` (rust-deps, not collections), `eval_eq`/`infer_equality` (equality — relational intrinsic, separate), `infer_list` (the check-side keyword dispatch — STAYS, redirects).

## Redirect (the arms STAY, point into the home)

- **`src/runtime.rs` `dispatch_keyword_head_value`** — the ~110 `:wat::core::(Vector|HashMap|HashSet|List)/<op>` arms + the seq-HOF/utility arms now call `collection::eval::*` / `collection::transform::*` instead of the local fn (mechanical import/path change).
- **`src/check.rs` `infer_list`** — the 4 collection arms (`:wat::core::conj`/`get`/`assoc`/`contains?` at ~4936/4949/4962 + contains) now call `collection::infer::*`. **Leave the PARTITION marker comment in place** (it documents the declaration site).

Substrate-as-teacher: move the fns, add `mod collection;`, then `cargo build --release` and fix each path the compiler names. The fns import the usual home types (`infer.rs` ← CheckEnv/InferCtx/Subst/TypeExpr; `eval.rs`/`transform.rs` ← Value/Environment/SymbolTable/RuntimeError) — `pub(crate)` + `use crate::...` as needed.

## The doctrine in `mod.rs` (the home's reason to exist)

State the partition rule in prose (cite `docs/DISPATCH.md`): collections are the **projective intrinsic** — `get : Vector<T> → Option<T>`; the return is a function of the container's type params, which a monomorphic `defclause` cannot express. This is why these ops are intrinsics and live here. (The word `intrinsic` belongs here, in prose — not in a filename.)

## Green-gate (raw commands)

- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged — the move is behavior-preserving).
- `cargo build --release --tests --workspace` → clean.
- `grep -rnE "^fn (infer_(contains|conj|get|assoc)|eval_(vector|hashmap|hashset|list|vec)_)" src/check.rs src/runtime.rs` → **empty** (the fns left the flat files).
- `grep -rn "mod collection" src/lib.rs` → present; the home compiles.

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run and commits.
