//! vigilatum: 2026-06-06T04:56:04Z — UPDATED-vigilia 10-spell guard L1+L2=0
//! (universal-7: intueri/solvere/conformare/purgare/struere/sequi/temperare +
//! exigere + conditional perspicere [fired: nested generics present] +
//! circumspicere LAST; secare not mustered — no parallel primitives in-home;
//! mora not fired; excusare not mustered at cast time — zero runes existed;
//! test-kind not mustered — no in-home test surface). One inward round
//! (9 casts) + the perimeter: 23 findings fought (A–R) + 5 perimeter closures;
//! 5 L1 killed (incl. the List-through-length/empty? check-vs-runtime egress —
//! eliminated, not documented); 3 runes earned through combat
//! (conformare spanless-by-domain on the _inner family; temperare
//! simplicity-win on sort_by; perspicere mumble-alias); clippy-clean in-home.
//! Canonical record:
//! docs/arc/2026/06/249-total-pure-macros/WARD-COLLECTION-REEARN.md.
//! RE-EARNED 2026-06-06T04:56:04Z (diff-scoped, the 245 clear: the unresolved-Var
//! deferral policy made UNIFORM across all four collection intrinsics
//! [contains?/conj/get/assoc — policy stated at infer_contains; runtime
//! TypeMismatch backstop verified per sibling]; gates: transform probe 13/13,
//! corpus 236/0/53, clippy-in-home empty).
//! Declared invariants, each enforced by a living gate:
//! (1) check ≡ runtime for polymorphic length/empty? across all four
//!     containers (tests/probe_collection_transform_ops.rs item1 witnesses);
//! (2) persistent-collection semantics — conj does not mutate its input
//!     (Vector witness in probe_collection_transform_ops.rs; HashSet witness
//!     in probe_arc216_stone5b_hashset_native_storage.rs);
//! (3) the five :wat::std::list::* transform ops hold their boundary
//!     contracts — empty input, n>len windows, out-of-range-returns-unchanged
//!     (probe_collection_transform_ops.rs items 4);
//! (4) record-assoc single-evaluation is TYPE-enforced — record_assoc_inner
//!     receives pre-evaluated Values and cannot evaluate (the signature is
//!     the gate);
//! (5) the corpus (tests/test.rs, 217 deftests) + lib suite exercise the
//!     dispatch perimeter end-to-end on every gate run.
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
//! - `eval.rs` — 50 functions in two tiers, plus 3 constructors and `eval_rest`.
//!
//!   **Tier 1 — `*_inner` helpers (23 fns):** operate on a pre-evaluated `&Value`
//!   (no `WatAST` in scope). Called by both the `eval_*` wrappers and
//!   `dispatch_substrate_impl` (the pre-evaluated dispatch path). Named
//!   `<type>_<op>_inner`, e.g. `vector_length_inner`, `list_empty_q_inner`.
//!
//!   **Tier 2 — `eval_*` wrappers (23 fns):** AST-level; arity-check, evaluate
//!   arguments via `eval_inner`, then delegate to the corresponding `*_inner`
//!   helper. The actual type×op combinations that exist (not all cross-products):
//!   - Vector: `length`, `empty_q`, `contains_q`, `get`, `conj`, `concat`
//!   - HashMap: `length`, `empty_q`, `contains_key_q` (not `contains_q`), `get`,
//!     `assoc`, `dissoc`, `keys`, `values`
//!   - HashSet: `length`, `empty_q`, `contains_q`, `conj`
//!   - List: `length`, `empty_q`, `contains_q`, `get`, `conj`
//!
//!   NB: HashMap dispatches `contains-key?` (`eval_hashmap_contains_key_q`),
//!   not `contains?`. List has no `assoc`/`dissoc`/`keys`/`values`/`concat`.
//!   Vector and HashSet have no `assoc`/`dissoc`/`keys`/`values`.
//!
//!   **Plus:** `eval_rest` (container-polymorphic; Vec/List/WatAST-form arms)
//!   and 3 constructors: `eval_vector_ctor`, `eval_hashmap_ctor`, `eval_hashset_ctor`.
//!   Import world: `Value`, `Environment`, `SymbolTable`, `RuntimeError`.
//! - `transform.rs` — the ~14 seq-HOF/utility ops. Most still enforce `Value::Vec` via
//!   `require_vec` (named `eval_vec_*`). Arc 255 Stone HOME-9 — `zip`/`window`/`remove-at`
//!   graduated off the dead `:wat::std::list::*` namespace to `:wat::seq::*` AND became
//!   Seqable-generic in the same motion (renamed `eval_seq_zip`/`eval_seq_window`/
//!   `eval_seq_remove_at`, via the shared `require_seqable_vec` helper — accepts `Vector`,
//!   `PersistentVector`, `List`, or `Stream`, not just `Value::Vec`). The fourth
//!   `:wat::std::list::*` op, `map-with-index`, is DELETED — `eval_vec_map_with_index` no
//!   longer exists; `:wat::core::map-indexed` (`wat/seq.wat`) is its non-drop-in replacement.
//!   `rest` was moved to `eval.rs` (container-polymorphic). Functions:
//!   `eval_vec_map`, `eval_vec_filter`, `eval_vec_foldl`,
//!   `eval_vec_sort_by`, `eval_vec_reverse`, `eval_vec_range`, `eval_vec_take`,
//!   `eval_vec_drop`, `eval_vec_last`, `eval_vec_find_last_index`,
//!   `eval_seq_zip`, `eval_seq_window`, `eval_seq_remove_at`.
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
pub(crate) mod seq_container;
pub(crate) mod map_container;
