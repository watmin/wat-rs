# SCORE — Stone 237.7b-iii — `:wat::core::conj` reborn as a `∀T` intrinsic with a CUSTOM inference arm

## Probe result (verbatim)

```
running 7 tests
test contains_q_vector_hit ... ok
test contains_q_wrong_element_rejected_at_check ... ok
test conj_wrong_element_rejected_at_check ... ok
test get_vector_precise_element_typing ... ok
test conj_vector_preserves_collection_type ... ok
test empty_q_hashset_false ... ok
test empty_q_vector ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Green-gate result

```
green-gate 1/2: cargo build --release --tests --workspace → Finished (0 errors)
green-gate 2/2: cargo test --release --lib -p wat → test result: ok. 834 passed; 0 failed; 1 ignored
green-gate: PASS (test-build clean + lib baseline green)
```

## git diff --stat

```
 src/check.rs   | 101 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 src/runtime.rs |  45 +++++++++++++++++++++++++
 wat/core.wat   |   4 +--
 3 files changed, 147 insertions(+), 3 deletions(-))
```

## Verification counts

- `grep -c "define-dispatch :wat::core::conj" wat/core.wat` → **0** (decl deleted; tombstone comment in place)
- `grep -c "fn infer_conj" src/check.rs` → **1** (custom inference arm present)
- `awk '/fn infer_conj/,/^}/' src/check.rs | grep -c "wat::core::HashMap"` → **0** (no HashMap arm)

## Shape

`infer_conj` mirrors `infer_contains` exactly except:
1. Two collection arms only (Vector<T> + HashSet<T>); HashMap arm intentionally absent (HashMap insertion is `assoc`).
2. Return is `apply_subst(&coll_ty, subst)` (the matched Parametric collection type, type-preserving) instead of `bool_ty()`. The early-return-on-coll_ty path ensures type-preservation propagates; fallback `bool_ty` is only reached on arity-error (no coll_ty known).

`eval_conj` mirrors `eval_contains`: arity-2, eval args, match `arg0_val` on `Value::Vec` → `vector_conj_inner` and `Value::wat__std__HashSet` → `hashset_conj_inner`; no HashMap arm.

## STOP triggers

None triggered.

## Files touched

- `src/check.rs` — added `":wat::core::conj"` dispatch arm (adjacent to `contains?`); added `infer_conj` helper after `infer_contains`.
- `src/runtime.rs` — added `":wat::core::conj"` dispatch arm (adjacent to `contains?`); added `eval_conj` function after `eval_contains`.
- `wat/core.wat` — deleted `(:wat::core::define-dispatch :wat::core::conj ...)` decl; tombstone comment inserted.

No other files touched. No HashMap arm. No List arm. No registry changes. No holon-rs.
