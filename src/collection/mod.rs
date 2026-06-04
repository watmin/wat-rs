//! vigilatum: 2026-06-04T00:17:13Z — vigilia 8-spell L1+L2=0, clippy-clean in-home
//!
//! # Collection — namespaced home for the container-polymorphic dispatch core.
//!
//! ## Why this module exists
//!
//! Arc 246 — lifts the container-polymorphic collection intrinsic dispatch out
//! of the flat `src/check.rs` and `src/runtime.rs` into this warded home.
//! The home's reason to exist is the **clause-vs-intrinsic partition**: these
//! ops are intrinsics because their types require type-level computation that a
//! monomorphic `defclause` cannot express. The doctrine lives in `docs/DISPATCH.md`.
//!
//! ## The clause-vs-intrinsic partition
//!
//! wat has two mechanisms for polymorphic operations. Which one an op uses is
//! not a taste call — it is decided by a checkable property of the op's type.
//!
//! **Clause (`defclause`)** — monomorphic: concrete argument types, a fixed
//! return, and no type variable flows anywhere. Numerics (`+`, `-`, `<`, `>`)
//! live here.
//!
//! **Intrinsic (custom Rust inference)** — reached for when the op's type
//! requires type-level computation. The collection ops that live in this home
//! are the **projective intrinsic** flavor: a type variable flows from an
//! argument's type parameters into the return (or into another argument).
//!
//! ## `get` as the worked proof
//!
//! ```text
//! get : Vector<T>     + i64 -> Option<T>
//! get : HashMap<K, V> + K   -> Option<V>
//! ```
//!
//! The return `Option<T>` is computed from the container's `T`; the key
//! argument's type *is* the container's `K`. A `defclause` is monomorphic
//! (no `∀`) — to cover this it would need one clause per concrete `(K, V)`,
//! an **infinite open set** (users mint new types forever). Unexpressible.
//! `infer_get` projects `T`/`K`/`V` out of the container and flows them into
//! the key argument and the return. **Projective ⇒ intrinsic.**
//!
//! The same logic applies to `conj`, `assoc`, and `contains`: each demands
//! type-level computation over the container's type parameters. See
//! `docs/DISPATCH.md` for the full decision procedure and worked
//! classifications table.
//!
//! ## Internal layout
//!
//! - `infer.rs` — the 4 check-side inference intrinsics
//!   (`infer_contains`, `infer_conj`, `infer_get`, `infer_assoc`).
//!   Import world: `CheckEnv`, `InferCtx`, `Subst`, `TypeExpr`.
//! - `eval.rs` — the ~30 runtime per-Type dispatch impls + 3 constructors
//!   (`eval_<vector|hashmap|hashset|list>_<length|empty_q|contains_q|get|conj|assoc|dissoc|keys|values|concat>`
//!   + `eval_vector_ctor`, `eval_hashmap_ctor`, `eval_hashset_ctor`).
//!     Import world: `Value`, `Environment`, `SymbolTable`, `RuntimeError`.
//! - `transform.rs` — the ~16 Vector/List-specific utility ops
//!   (`eval_vec_map`, `eval_vec_filter`, `eval_vec_foldl`, `eval_vec_foldr`,
//!   `eval_vec_sort_by`, `eval_vec_reverse`, `eval_vec_range`, `eval_vec_take`,
//!   `eval_vec_drop`, `eval_vec_last`, `eval_vec_rest`, `eval_vec_find_last_index`,
//!   `eval_list_zip`, `eval_list_window`, `eval_list_remove_at`,
//!   `eval_list_map_with_index`).
//!
//! ## Declaration sites (source markers)
//!
//! - **Check-side declaration:** `fn infer_list` in `src/check.rs` — the
//!   `":wat::core::<op>" => collection::infer::<op>(...)` arms declare which
//!   ops are intrinsics. The PARTITION marker comment there is the live gate.
//! - **Runtime-side declaration:** `fn dispatch_keyword_head_value` in
//!   `src/runtime.rs` — routes to `collection::eval::*` /
//!   `collection::transform::*`; equality routes separately to `eval_eq` /
//!   `eval_not_eq` (the relational intrinsic, not a collection op).
//!
//! DO NOT make collection ops `defclause`s. See `docs/DISPATCH.md`.

pub(crate) mod infer;
pub(crate) mod eval;
pub(crate) mod transform;
