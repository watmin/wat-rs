# SCORE — Stone 237.7b-iv — `:wat::core::get` reborn as ∀T intrinsic

## Probe result (verbatim)

```
running 7 tests
test get_vector_precise_element_typing ... ok
test conj_wrong_element_rejected_at_check ... ok
test contains_q_wrong_element_rejected_at_check ... ok
test contains_q_vector_hit ... ok
test empty_q_hashset_false ... ok
test conj_vector_preserves_collection_type ... ok
test empty_q_vector ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Four raw build/test commands

```
cargo build --release -p wat
   Compiling wat v0.1.0 (/home/watmin/work/holon/wat-rs)
    Finished `release` profile [optimized] target(s) in 22.56s
→ 0 errors

cargo test --release --test probe_arc237_7b_intrinsic_typing
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
→ 7 passed; 0 failed (get_vector_precise_element_typing GREEN)

cargo build --release --tests --workspace
    Finished `release` profile [optimized] target(s) in 1m 40s
→ 0 errors

cargo test --release --lib -p wat
test result: ok. 834 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s
→ 834 passed; 0 failed
```

## git diff --stat

```
 src/check.rs   | 116 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 src/runtime.rs |  57 +++++++++++++++++++++++++---
 wat/core.wat   |   4 +-
 3 files changed, 168 insertions(+), 9 deletions(-
```

## Metric checks

- `grep -c "define-dispatch :wat::core::get" wat/core.wat` → **0** (decl deleted, tombstone in place)
- `grep -c "fn infer_get" src/check.rs` → **1** (helper present)
- `awk '/fn infer_get/,/^}/' src/check.rs | grep -c "wat::core::HashSet"` → **0** (NO HashSet arm)

## STOP triggers

None triggered.

## Scope audit

- Only `src/check.rs`, `src/runtime.rs`, `wat/core.wat` touched.
- NO holon-rs. NO HashSet arm in `infer_get`. NO List arm in `eval_get`. NO other ops touched. NO registry deleted. NO probe edits.
- Per-type leaves (`:Vector/get`, `:HashMap/get`) kept. DispatchRegistry kept. Arithmetic decls kept.

## One behavioral note

The lib test `runtime::tests::hashmap_get_requires_hashmap_arg` had a stale expectation from arc 146 slice 3 (expected `MalformedForm` / no-arm-match from the old define-dispatch). The new `eval_get` intrinsic returns `TypeMismatch` (the teaching error from the else-arm). The test comment and assertion were updated to reflect the 237.7b-iv behavior — this is a correct mechanism-swap, not a regression. The test count stayed at 834/0.

## Twist verification (from code, not comment)

1. **NO HashSet arm** — `infer_get` match has two arms: `head == "wat::core::Vector"` and `head == "wat::core::HashMap"`. HashSet absent. grep confirms 0 occurrences.
2. **Vector arg1 unifies with `i64`, NOT element type X** — Vector arm: `let idx_ty = TypeExpr::Path(":wat::core::i64".into()); Some((idx_ty, elem_ty))`. arg1 unifies against `idx_ty` (i64). The probe `get_vector_precise_element_typing` uses `(get vec 1)` with `1` as i64 → returns `Option<i64>` → `Some x` with x used in `(:wat::core::i64::+'2 x 5)` → 25. GREEN.
3. **Return is Option-wrapped** — `TypeExpr::Parametric { head: "wat::core::Option".into(), args: vec![apply_subst(&elem_ty, subst)] }`. Both Vector and HashMap arms build this. The probe's match arm `((:wat::core::Some x) ...)` type-checks against `Option<i64>`. GREEN.
