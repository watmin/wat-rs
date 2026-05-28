# SCORE — Stale Test Sweep (arc 239)

Three integration tests asserted a retired reality. Each was updated to verify
the current-correct behavior without weakening its intent. Zero substrate
changes.

---

## Fix 1 — `tests/wat_arc144_uniform_reflection.rs`

**Test:** `dispatch_length_lookup_define_emits_define_dispatch_head`
**Renamed to:** `dispatch_empty_lookup_define_emits_define_dispatch_head`

**Why stale:** Arc 237.7a evacuated `:wat::core::length` to a Rust ∀T
intrinsic. The `define-dispatch` declaration was removed from `wat/core.wat`.
`lookup-define` now returns the intrinsic form (a `:wat::core::define` with an
`__internal/primitive` body), not a `define-dispatch` form.

**Fix:** Switched the exemplar from `:wat::core::length` to
`:wat::core::empty?`, which remains a `define-dispatch` with three arms
(Vector, HashMap, HashSet — `wat/core.wat:31-34`). Updated:
- section comment (line ~278)
- test function name
- `lookup-define` argument (`:wat::core::length` → `:wat::core::empty?`)
- all assertion substrings (`define-dispatch`, `:wat::core::empty?`,
  `Vector/empty?`, `HashMap/empty?`) plus added a third arm assertion
  (`HashSet/empty?`) to match the three-arm reality.

**Result:** test was FAIL, now PASS; all 9 arc144 tests pass.

---

## Fix 2 — `tests/probe_arc234_stone15_namespace_promotion.rs`

**Test:** `probe_5_class_fqdn_extraction_post_rename`

**Why stale:** The `make_record()` helper (lines 53-67) was updated in arc 237
S-C.2c (base/holonic split) to build `Value::wat__holon__Record` (the holonic
variant carrying `holon_form`). Probe 5's `match` arm still said
`Value::wat__Record` (the base variant, no `holon_form`), which never matched,
causing the `_ => panic!` arm to fire.

**Fix:** Changed the `match` arm from `Value::wat__Record` to
`Value::wat__holon__Record` and updated the panic message to match. Added a
comment explaining that `make_record()` produces the holonic variant. The
`class_fqdn` assertion (the substance of the probe) is unchanged.

**Result:** test was FAIL, now PASS; all 5 probe_arc234_stone15 tests pass.

---

## Fix 3 — `tests/wat_arc201_structured_signature_types.rs`

**Test:** `signature_of_defn_foldl_emits_structured_parametric_and_fn`

**Why stale:** The test's comment and assertion for the `:T` type variable
said: "EDN renderer emits Symbol payloads as quoted strings … observed
substring is `\":T\"`". That was true when `watast_to_holon` emitted
`HolonAST::Symbol` for keywords. Post arc 221 Stone 221.5, the path now emits
`HolonAST::Keyword` — the EDN renderer writes `#wat-edn.holon/Keyword :T`
(no surrounding quotes). The `r#"\":T\""#` substring no longer appears.

Actual output confirmed:
```
...#wat-edn.holon/Bundle [#wat-edn.holon/Keyword :Fn #wat-edn.holon/Keyword :Acc #wat-edn.holon/Keyword :T ...
```

**Fix:** Replaced the `r#"\":T\""#` assertion with `line.contains("Keyword :T")`,
which matches the structured `#wat-edn.holon/Keyword :T` token. Updated the
comment to explain the Keyword (not Symbol) rendering path. The negative
assertion (`!line.contains("wat::core::Fn(Acc")`) is unchanged. The positive
structural assertions (`:wat::core::Vector`, `:Fn`, `:Acc`) are unchanged.

**Result:** test was FAIL, now PASS; all 5 arc201 tests pass.

---

## Verification

```
cargo test --release \
  --test wat_arc144_uniform_reflection \
  --test probe_arc234_stone15_namespace_promotion \
  --test wat_arc201_structured_signature_types
```

```
running 5 tests  [probe_arc234_stone15_namespace_promotion]
test probe_1_variant_compiles_and_constructs ... ok
test probe_2_type_name_returns_wat_record ... ok
test probe_3_eq_hash_consistency_under_rename ... ok
test probe_4_namespace_type_registration ... ok
test probe_5_class_fqdn_extraction_post_rename ... ok
test result: ok. 5 passed; 0 failed

running 9 tests  [wat_arc144_uniform_reflection]
test dispatch_empty_lookup_define_emits_define_dispatch_head ... ok
test dispatch_length_signature_and_body_shape ... ok
test length_canary_hashmap_via_define_alias ... ok
test macro_lookup_define_smoke ... ok
test primitive_lookup_define_and_signature_smoke ... ok
test special_form_lookup_define_smoke ... ok
test type_lookup_define_smoke ... ok
test user_function_lookup_define_emits_define_head ... ok
test user_function_signature_and_body_return_some ... ok
test result: ok. 9 passed; 0 failed

running 5 tests  [wat_arc201_structured_signature_types]
test define_alias_round_trips_on_parametric_signature ... ok
test signature_of_defn_emits_atomic_for_monomorphic_path_types ... ok
test signature_of_defn_emits_structured_parametric_user_fn ... ok
test signature_of_defn_emits_structured_tuple_return_type ... ok
test signature_of_defn_foldl_emits_structured_parametric_and_fn ... ok
test result: ok. 5 passed; 0 failed
```

```
cargo build --release --tests --workspace --keep-going
```
```
Finished `release` profile [optimized] target(s) in 1m 25s
```

0 errors. 19 target tests green. Full workspace build clean.
