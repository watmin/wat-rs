# BRIEF — Stone 237.7b-iv — `:wat::core::get` reborn as a `∀T` intrinsic with a CUSTOM inference arm

**Last Tier-B slice.** Mirror `infer_contains` (`fef2c8d9`) + `infer_conj`
(`2d3259ae`) with three twists specific to `get`:

1. **2 collection types, asymmetric arg1 targets**:
   - `Vector<T>` + arg1 = **`i64`** (positional index — NOT the element type T)
   - `HashMap<K,V>` + arg1 = **`K`** (the key)
   - No HashSet arm (HashSet has no positional access — that's `contains?`'s territory).
2. **Return is `Option`-wrapped per collection**:
   - `Vector<T>.get(i64) -> Option<T>`
   - `HashMap<K,V>.get(K) -> Option<V>`
3. **The arg1-unify target diverges from arg0's element type for Vector** — a TRAP the
   pattern doesn't fully prepare you for: `contains?`/`conj` unify arg1 with the
   element type X (`targs.first()`); `get` on Vector unifies arg1 with `i64_ty()`
   independent of X. Easy to miss if you copy `infer_conj` and only change the return.

The probe `tests/probe_arc237_7b_intrinsic_typing.rs` (7/7) is the regression
contract — `get_vector_precise_element_typing` MUST stay green (proves
`(get vec 1)` returns `Option<i64>` with x: i64 usable in arithmetic).

## The work

### 1. `src/check.rs` — `infer_get` helper + dispatch arm

In `infer_list` add a `":wat::core::get"` arm adjacent to `:wat::core::conj`,
calling a new `infer_get` helper. Mirror `infer_conj`'s overall structure but:

- Arity 2 (collection + index/key).
- Match `reduce(arg0_ty, subst, env.types())` against TWO shapes (NO HashSet):
  - `Parametric { head == "wat::core::Vector", args: [X] }` →
    - Unify arg1's type with `i64_ty()` (NOT `X`).
    - Return `Parametric { head: "wat::core::Option", args: [apply_subst(X, subst)] }`.
  - `Parametric { head == "wat::core::HashMap", args: [K, V] }` →
    - Unify arg1's type with `K`.
    - Return `Parametric { head: "wat::core::Option", args: [apply_subst(V, subst)] }`.
  - else → teaching `CheckError::TypeMismatch` (`expected: "Vector<T> or HashMap<K,V>"`).
- On unify-fail of arg1 → push `CheckError::TypeMismatch` naming the expected
  type (`i64` for Vector, `K` for HashMap) and the actual arg1 type.

### 2. `src/runtime.rs` — `eval_get` + dispatch arm

Mirror `eval_conj` shape. Arity 2 → eval arg0 + arg1 → match raw `arg0_val`:
- `Value::Vec(xs)` — eval arg1 must be `Value::i64(idx)`; route to existing
  `vector_get_inner` (or equivalent — grep `eval_vector_get` / `vector_get_inner`)
  which returns `Option<Value>`; wrap in `Value::Option`.
- `Value::wat__std__HashMap(m)` — route to `hashmap_get_inner` (grep) which
  takes the key Value, returns `Option<Value>`; wrap in `Value::Option`.
- else → teaching `RuntimeError::TypeMismatch` (`expected: "Vector<T> or HashMap<K,V>"`).
Wire dispatch arm next to `eval_conj`: `":wat::core::get" => eval_get(...)`.

### 3. `wat/core.wat` — delete the `define-dispatch :wat::core::get` decl

Tombstone comment: "arc 237 Stone 237.7b-iv — `:wat::core::get` is now a Rust
∀T intrinsic with custom inference arm; see `src/check.rs::infer_get` +
`src/runtime.rs::eval_get`".

### 4. KEEP

- Per-type leaves (`:Vector/get`, `:HashMap/get`).
- `DispatchRegistry`.
- All arithmetic `define-dispatch` decls (237.8 territory).
- (`empty?`/`contains?`/`conj` decls already evacuated in 7b-i/ii/iii.)

## Scope

- NO HashSet arm — HashSet has no `get`. If you feel the urge, STOP.
- NO List arm — `length`/`empty?`/`contains?`/`conj` (shipped exemplars) are
  Vector/HashMap/HashSet only, NOT `wat__core__List`. Mirror exactly. List
  coverage is a separate uniform stone.

## Verify (RAW commands — no wrapper scripts)

Run these as SEPARATE simple commands, one per line:

- `cargo build --release -p wat` → 0 errors
- `cargo test --release --test probe_arc237_7b_intrinsic_typing` → `7 passed; 0 failed`
  (the load-bearing `get_vector_precise_element_typing` MUST stay green)
- `cargo build --release --tests --workspace` → 0 errors (the test-build gate)
- `cargo test --release --lib -p wat` → `834 passed; 0 failed` (lib baseline)

Do NOT invoke `./scripts/green-gate.sh` — the wrapper script gets denied;
use the four raw commands above.

## STOP triggers (REJECTION — surface, do not work around)

- If Vector arm unifies arg1 with `X` (element type) instead of `i64_ty()` —
  STOP. Vector.get takes an INDEX (i64), not the element. The probe
  `get_vector_precise_element_typing` calls `(get vec 1)` with `1` as i64 — if
  the arm unified arg1 with X (i64 in this case), it'd *look* right but would
  silently fail on `(get vec_of_strings 0)` or similar.
- If HashMap arm unifies arg1 with `V` (value) instead of `K` (key) — STOP.
  Same K-not-V trap as `contains?`.
- If return type loses Option wrap (returns bare X/V instead of `Option<X>` /
  `Option<V>`) — STOP. The probe matches `((:wat::core::Some x) ...)` — if the
  return isn't Option, the match fails to type-check.
- If `(:wat::core::get ...)` won't resolve to the new intrinsic after the decl
  delete — STOP.
- Any urge to add a HashSet arm, a List arm, touch other ops, the registry, or
  holon-rs — STOP.

## Definition of done

- All 7 probe tests green; test-build 0 errors; lib 834/0.
- `wat/core.wat` no longer has `define-dispatch :wat::core::get`; `src/check.rs`
  has `infer_get` + dispatch arm; `src/runtime.rs` has `eval_get` + dispatch arm.
- Only `src/check.rs` + `src/runtime.rs` + `wat/core.wat` touched. NO holon-rs,
  NO HashSet arm in get, NO List arm, NO other ops, NO registry deletion, NO
  probe edits.
- Write `SCORE-STONE-237.7b-iv.md` (sibling); do NOT commit (orchestrator
  scores + commits).
