# BRIEF — Stone 246.2 — RAZE the duplication + close the dispatch fork + ward `src/collection/`

The vigilia ward on `src/collection/` (7 spells, real casts) found the home's craft clean (temperare/sequi/purgare-on-home converged) but the **surround rotten**: the 246.1 lift left a live duplicate of every collection `*_inner` helper in `runtime.rs`, kept alive by a **second dispatch path that bypasses the home** — a live behavioral fork. Plus two naming/placement lies in the home and stale comments/markers. This stone razes all of it to **L1+L2=0**, then the home earns its `vigilatum` stamp.

**Behavior-preserving where it's a move; behavior-collapsing where it's a fork (both paths must reach the ONE home impl). The existing 895-test suite is the regression guard.**

## 1. THE HERESY — raze the 23 `*_inner` duplicates + close the fork (do this FIRST)

There are two runtime dispatch paths for the same collection ops:
- **Path A (→ home, correct):** `:wat::core::Vector/get` etc. → `collection::eval::*`; and `dispatch_substrate_impl` (runtime.rs:10472) → `crate::collection::eval::*_inner` via `use … as ceval`.
- **Path B (→ flat-file copies, the fork):** bare `:wat::core::get`/`conj`/`contains?`/`assoc` (runtime.rs:5350/5355/5360/5364) → `eval_get`(runtime.rs:14189) / `eval_conj`(14151) / `eval_contains`(14112) / `eval_assoc`(10755) → **local** `vector_get_inner` / `hashmap_assoc_inner` / … which resolve to the **runtime.rs duplicates (9474–10425), NOT the home.**

**Raze:**
1. In `eval_get`/`eval_conj`/`eval_contains`/`eval_assoc` (the Path-B wrappers — these STAY), redirect every bare `*_inner(...)` call to `crate::collection::eval::*_inner(...)` (exactly as `dispatch_substrate_impl` already does — `use crate::collection::eval as ceval;` then `ceval::vector_get_inner(...)`).
2. Then **DELETE all 23 collection `*_inner` definitions from `runtime.rs`** (the 9474–10425 block): `{vector,hashmap,hashset,list}_length_inner`, the four `*_empty_q_inner`, `{vector,hashmap,hashset,list}_contains_q_inner`/`contains_key_q_inner`, `{vector,hashmap,list}_get_inner`, `{vector,hashset,list}_conj_inner`, `hashmap_assoc_inner`, `hashmap_dissoc_inner`, `hashmap_keys_inner`, `hashmap_values_inner`, `vector_concat_inner`. (8 were live via Path B — now redirected; 15 were already dead — `cargo build` warns them today.)
3. **Result:** zero collection `*_inner` defs in `runtime.rs`; both `:wat::core::get` (Path B) and `:wat::core::Vector/get` (Path A) reach the SAME `collection::eval` impl. The fork is closed. (Do NOT touch `eval_inner`, `eval_ok_inner`, `to_holon_inner`, `extract_classifier_inner` — those are not collection ops; leave them.)

## 2. The home's two lies (intueri + solvere + struere)

4. **Rename `eval_list_ctor` → `eval_vector_ctor`** — it builds `Value::Vec`, is wired to `:wat::core::Vector`, every error says `:wat::core::Vector`; the name lies (there's a real distinct List type). Sites: def `collection/eval.rs:868`; dispatch arm `runtime.rs:5723`; prose `mod.rs:51`; `DESIGN.md:22`; stale marker `types.rs:672`.
5. **Move `eval_vec_rest` from `transform.rs` → `collection/eval.rs`** — it branches on `Value::Vec` vs `Value::wat__core__List` (container-polymorphic dispatch), which `transform.rs`'s own module-doc says it does NOT hold; it belongs in `eval.rs` beside the per-Type impls. Update its dispatch arm (`runtime.rs:5527`).

## 3. Stale comments + markers (struere + circumspicere)

6. **Purge the stale `// arc 138: no span — leaf helper without list_span` comments** in `transform.rs` (~136, 220, 259, 298, 336, 363, 494, …) that sit directly above lines doing `list_span.clone()` — the comment contradicts the code. Delete the lying comments (arc 243.M resolved the actual span debt).
7. **Fix the stale source markers** that point readers to runtime.rs for moved impls: `check.rs:4928–4929` (the PARTITION marker says "per-Type collection impls live in `runtime.rs`" → now `src/collection/eval.rs`) and `types.rs:672` (folds into #4's rename).

## DO NOT TOUCH

- The home's op LOGIC (behavior-preserving). Equality (`eval_eq`/`infer_equality`). The Path-B wrapper fns themselves (`eval_get`/`conj`/`contains`/`assoc` STAY — only their `*_inner` *calls* redirect). `dispatch_substrate_impl`'s existing `ceval::` routing (already correct). The non-collection inners (`eval_inner`, `to_holon_inner`, …).

## GREEN-GATE (asserts INTENT — a name-pattern alone is gameable; these check the heresy is GONE)

- `grep -nE "fn (vector|hashmap|hashset|list)_[a-z_]*_inner" src/runtime.rs` → **EMPTY** (all 23 collection inners gone from runtime.rs).
- `grep -rn "eval_list_ctor" src/ docs/` → **EMPTY** (renamed everywhere).
- `grep -n "fn eval_vec_rest" src/collection/transform.rs` → empty; **present** in `src/collection/eval.rs`.
- Both paths → one impl: `eval_get` in runtime.rs calls `crate::collection::eval::vector_get_inner` (Path B reaches the home).
- `cargo build --release -p wat 2>&1 | grep -cE "never used|dead_code"` → **drops** vs today (the 15 dead inners gone); **no NEW** dead_code introduced.
- `cargo test --release --lib -p wat` → **895 / 0 / 1** (behavior preserved; the fork-collapse must not change results — both paths had identical bodies).
- `cargo build --release --tests --workspace` → clean.

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run + re-casts the ward, then commits.
