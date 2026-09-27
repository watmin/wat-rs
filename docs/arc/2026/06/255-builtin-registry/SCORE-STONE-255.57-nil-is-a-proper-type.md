# SCORE — STONE 255.57: nil is a proper type, then 255.56 finishes

Two commits on main, not pushed. Part 1 is `782f5fcda`. Part 2 is the
commit that carries this file.

## Part 1 — nil is the path `:wat::core::nil`

`:wat::core::nil` is a builtin leaf, the same door as `:wat::core::i64`.
It is not an `AliasDef`. `wat.type/nil` parses to that path. `Value::Unit`
is `Value::Nil`. EDN still writes and reads `nil`. `values_equal` keeps
`(Nil, Nil) => Some(true)`. `values_compare` has no `Nil` arm.

A tuple type needs at least one slot. `:()` does not parse. `(Tuple :- [])`
does not parse. The reason names nil. `BareLegacyUnitType` still fires if
a `TypeExpr::Tuple` with no slots is walked. Nothing in `src` builds one.
`grep` of `Tuple(vec![])` and `Tuple(Vec::new())` in `src` is empty.

`:()` remains in four places, and none of them is nil's rendering:

- `src/check/error.rs` records the retired spelling on `BareLegacyUnitType`
  (`primitive = ":()"`). `src/edn/derive_tests.rs` mirrors that attribute.
- `src/types.rs` asserts that `parse_type_expr(":()")` is
  `MalformedTypeExpr` with raw `":()"`.
- An empty call list is `NoStepRule` with op `"()"`. An empty parametric
  form is `MalformedTypeExpr` with raw `"()"`.

Display of the value is `nil`. `type_name` is `"nil"`.

`:test::Wrapper` is a `defrecord` with the same fields and `:- [E]`.
The three `_` scratches compare through a typed consumer
(`:probe::matches?` on `Equatable` for the two `eval-ast!` pairs,
`:probe::same-tag?` on `i64` for the struct field). The call of
`:user::eq-generic` on two functions is
`tests/types/probe_arc255_56_eq_generic_fn.wat.bad`. The test asserts
`BoundNotSatisfied`, function `:user::eq-generic`, param `T`, bound
`:wat::core::Equatable`, got `[:wat::core::i64 :-> :wat::core::i64]`.

## The red floor, not re-run

`.floor/2026-09-27T08-34-56Z` was red and was not re-run.

```
Summary [ 361.504s] 6193 tests run: 6186 passed (14 slow), 7 failed, 23 skipped
```

`[floor] ⛔ RED — exit=100`. The seven arms were nil spellings and the
empty tuple form: the alias floor still required nil; two runtime tests
still wrote `-> :()`; one golden still expected `:()` inside a function
type; `wat.type/nil` was a different path that formatted as
`:wat::core::nil`; `(wat.type/Tuple :- [])` was still `TypeExpr::Tuple`
with no slots; the new bound test's got-string opened with `[`.

## Proof

Green floor `.floor/2026-09-27T08-47-25Z`:

```
Summary [ 360.627s] 6193 tests run: 6193 passed (16 slow), 23 skipped
```

Exit 0. Against 6186 passed at `55fe36769`: 255.56 added 7 operator tests
and `different_newtype_classes_do_not_compare`, and removed the agreement
test and the different-class integration test; this stone adds
`eq_generic_refuses_a_function`. 6186 + 7 = 6193. Skipped stays 23.

Clippy `cargo clippy --release --all-targets -- -D warnings` rc 0.

Census `.census/2026-09-27T08-54-40Z.txt` against
`.census/2026-09-27T06-55-26Z.txt`: 2294 → 2295. The new file is
`tests/types/probe_arc255_56_operators.wat` at rc 0. Nonzero 218.
Three flips, all 0→1, the amend's expected goldens:

- `tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-06b-less-equal.wat`
- `tests/resolve/probe_arc251_fix_source_local_rules__contract-07-greater-than.wat`

`--diff` exits 8 on those three. No other flip.

Delta `.delta/2026-09-27T08-55-39Z`: ORIG-CLEAN 160/179, CONV-CLEAN 158/179,
NEW 2, RECOVERY 0, exit 0. The two NEW files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Goldens whose content changed because `:()` was nil's old rendering:
the eleven `signature declares :()` files, and
`probe_arc251_enrol_the_variant_in_the_lattice__fn_narrow_param_for_wide_slot.edn`.
`probe_arc251_keyword_to_type_form__contract-07-empty-tuple.wat` no longer
holds `(wat.type/Tuple :- [])`. The test asserts the refusal.
