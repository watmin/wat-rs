# SCORE — Stone 237.7b-ii — `:wat::core::contains?` reborn as a `∀T` intrinsic with custom inference arm

## Probe result (verbatim)

```
running 7 tests
test conj_vector_preserves_collection_type ... ok
test conj_wrong_element_rejected_at_check ... ok
test contains_q_wrong_element_rejected_at_check ... ok
test empty_q_hashset_false ... ok
test contains_q_vector_hit ... ok
test get_vector_precise_element_typing ... ok
test empty_q_vector ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Green-gate result

```
== green-gate 1/2: cargo build --release --tests --workspace (compile all test units) ==
Finished `release` profile [optimized] target(s) in 1m 38s

== green-gate 2/2: cargo test --release --lib -p wat (lib run baseline) ==
test result: ok. 834 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.24s

green-gate: PASS (test-build clean + lib baseline green)
```

## git diff --stat

```
 src/check.rs   | 94 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 src/runtime.rs | 45 ++++++++++++++++++++++++++++
 wat/core.wat   |  5 +---
 3 files changed, 140 insertions(+), 4 deletions(-)`
```

## Verification checks

- `grep -c "define-dispatch :wat::core::contains?" wat/core.wat` → **0** (decl deleted)
- `grep -c "fn infer_contains" src/check.rs` → **1** (custom inference arm present)

## Files touched

- `src/check.rs` — `infer_contains` helper function (Tier B custom arm: Vector<T>/HashSet<T>/HashMap<K,V> shapes; arg1 unified with element/key type; TypeMismatch on collection-shape mismatch and on wrong-element); `":wat::core::contains?"` arm in the `infer_list` head-symbol dispatch (adjacent to first/second/third)
- `src/runtime.rs` — `eval_contains` function (arity-2; delegates to existing per-type inner helpers `vector_contains_q_inner`/`hashset_contains_q_inner`/`hashmap_contains_key_q_inner`); dispatch arm wired adjacent to `length`/`empty?` at runtime.rs:5327
- `wat/core.wat` — `define-dispatch :wat::core::contains?` decl deleted; one-line tombstone comment left in its place

## STOP triggers

None triggered.

## Notes

- HashMap arm correctly checks arg1 against **K** (first type param), not V — `contains?` on HashMap is `contains-key?` semantics.
- Runtime `eval_contains` delegates to the existing per-type inner helpers rather than re-implementing the `xs.contains()`/`s.contains()`/`m.contains_key()` logic directly — this keeps the per-type leaf behavior (including the hashability guard in `hashmap_contains_key_q_inner` and `hashset_contains_q_inner`) as the single source of truth.
- No List arm added (out of scope per BRIEF).
- Only three files touched; no holon-rs, registry, or other ops modified.
