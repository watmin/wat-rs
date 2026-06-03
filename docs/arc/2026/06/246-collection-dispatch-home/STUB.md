# Arc 246 — collection-dispatch home (`src/collection/`) — warded (STUB)

**Status:** ⏸ **STUBBED — execute-ready, enabled by arc 237's closure.** Stubbed 2026-06-03 during 237's wind-down, when the equality/collections clause-vs-intrinsic question forced a precise understanding of the collection dispatch. The builder: *"I want this collection dispatch to be a warded namespace — I never want to deal with these thoughts again."* This is the home where that doctrine lives in self-verifying code. The **doctrine is captured now** (237.9 INSCRIPTION + the in-code comment rewrite in 237.8d + memory `[[project_dispatch_clause_vs_intrinsic]]`); this arc builds the **warded home** that permanently houses it.

## Why this arc — the doctrine made structural

The substrate has two polymorphism mechanisms, sorted by a checkable property:
- **Clause (`defclause`)** — for *monomorphic* ops (concrete arg-match, fixed return, no type-variable flow): numerics, equality.
- **Intrinsic (custom Rust inference)** — for ops needing *type-level computation* (project/flow type variables from the args' parameters): the collection ops.

The collection intrinsics are the **justified asymmetry** — `get : Vector<T> → Option<T>`, `HashMap<K,V> + K → Option<V>` — the return type is a *function of the container's type parameters*, which a clause structurally cannot express (a clause would need one instance per concrete `(K,V)`, an infinite open set; `∀`-quantification collapses it, but clauses can't quantify). This home **inscribes that reason in the code** so the "why isn't this a clause" question can never cost a session again — the warded-homes promise (`[[project_warded_homes_pattern]]`): doctrine inscribed in self-verifying, re-castable code.

## Scope — the components we're perfecting (NOT the central dispatch)

The central `dispatch_keyword_head` (runtime.rs:5295) routes **every** keyword-head call — it stays where it is (its own future home is the 109-level `runtime.rs` reorg, NOT this arc). This arc lifts only the cohesive **collection-ops cluster** it routes to:

**Inference intrinsics (check.rs)** — the type-computing dispatch logic:
- `infer_contains` (10335), `infer_conj` (10415), `infer_get` (10502), `infer_assoc` (12165) (+ `infer_length`/`infer_empty` if separate; confirm at lift). `dispatch_rust_scheme` (12862) — evaluate for inclusion.

**Per-Type impl fns (runtime.rs)** — ~30 standalone collection operations:
- Vector: `eval_vector_length` (9538), `_empty_q` (9651), `_contains_q` (9776), `_get` (9911), `_conj` (10620), `_concat` (10861)
- HashMap: `eval_hashmap_length` (9555), `_empty_q` (9668), `_contains_key_q` (9794), `_get` (9929), `_assoc` (10790), `_dissoc` (10809), `_keys` (10827), `_values` (10844), `_ctor` (11601)
- HashSet: `eval_hashset_length` (9572), `_empty_q` (9685), `_contains_q` (9812), `_conj` (10638), `_ctor` (11720)
- List: `eval_list_ctor` (9382), `_length` (9949), `_empty_q` (9966), `_contains_q` (9983), `_get` (10001), `_conj` (10019), `_zip` (11448), `_window` (11475), `_remove_at` (11507), `_map_with_index` (13920)

**Wiring that STAYS but redirects** — ~110 `:wat::core::(Vector|HashMap|HashSet|List)/…` routing arms in the central match → each calls `collection::eval_*` instead of the local fn (mechanical import change).

## Difficulty — moderate, bounded, clean (a homes-walk sibling)

~35 standalone functions over one cohesive domain (collection ops), plus ~110 mechanical call-site redirects. The functions are *already standalone* (cut/paste-liftable, not inline spaghetti); the entanglement is normal (they import `Value`/`Environment`/`SymbolTable`/`RuntimeError` and `CheckEnv`/`InferCtx`/`Subst`/`TypeExpr` — like every home). Comparable scale to the `function/` and `check/` lifts already done. **Not a `runtime.rs` excavation — a known, bounded walk.**

## Lift plan (execute once 237 closes)

1. **Design stone (246.0)** — name the home via `intueri` (`src/collection/` vs `src/dispatch/`; the domain is *collection operations + their type-computing polymorphic dispatch*). Confirm the exact fn set (verify `infer_length`/`infer_empty`, `dispatch_rust_scheme`).
2. **Lift** — move the inference intrinsics + the ~30 per-Type impls into `src/collection/`, `pub(crate)` as needed; redirect the ~110 central-match arms to `collection::*`. Substrate-as-teacher cascade to green.
3. **Ward** (vigilia 8-spell → L1+L2=0) — annihilate the failure classes; earn the `vigilatum` stamp.
4. **Inscribe the doctrine IN the home** — the module doc states the clause-vs-intrinsic discriminant (type-level computation) with `get` as the worked proof, so the home *answers* the question structurally.
5. **INSCRIPTION** — the home is warded; the thoughts are housed for good.

## Enabled-by

**Arc 237 closure** (237.9). 237 consolidates the *monomorphic* surface (numerics → defclause, equality → defclause); this arc homes the *intrinsic* surface (collections). Do not start until 237 dies — finish the consolidation, then build the home. 237.9 should flag arc 246 unblocked alongside arc 245.
