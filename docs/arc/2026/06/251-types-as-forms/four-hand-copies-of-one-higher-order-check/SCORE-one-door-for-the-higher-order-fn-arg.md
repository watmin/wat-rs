# SCORE — ONE DOOR for the higher-order fn argument

Struck. Floor not run. Clippy not run. Not committed.

```
cargo build --release                                          BUILD_EXIT=0
cargo nextest run --release -E 'binary_id(wat::types)'         641 passed, 5 skipped, NEXTEST_EXIT=0
cargo nextest run --release -E 'binary_id(wat::collection)'    248 passed, 0 skipped, COLL_EXIT=0
```

(types was 635 before this stone's 6 probes.)

## STEP 1 — extract, no behaviour change

`check_higher_order_fn_arg` in `src/collection/infer.rs`. All four verbs call it.
A flag selected the comparator: foldl `assignable`, map/mapv/filter `unify`.

Measured on the new fixtures **before** step 2 (same binary as the helper, flag still
per-caller):

| fixture | EXIT |
|---|---|
| map_variant_elems | **1** TypeMismatch `expects [:u::Op.Mark :-> :?N]; got [:u::Op :-> i64]` |
| mapv_variant_elems | **1** same shape |
| filter_variant_elems | **1** same shape |
| foldl_variant_elems | **0** (already assignable) |
| mapv_output_concrete | **0** (STOP-1: U still binds) |
| filter_enum_elems_narrow | **1** |

Floor at step 1 not run (brief). No `.edn` recaptured.

Census: `TypeExpr::Fn` / `expected_fn_ty` in `infer.rs` — **four sites, no fifth**.
STOP-4 did not fire.

## STEP 2 — one comparator: `assignable`

Flag deleted. All four call `assignable`.

| fixture | step 1 | step 2 |
|---|---|---|
| map_variant_elems | 1 | **0** |
| mapv_variant_elems | 1 | **0** |
| filter_variant_elems | 1 | **0** |
| foldl_variant_elems | 0 | **0** |
| mapv_output_concrete (STOP-1) | 0 | **0** |
| filter_enum_elems_narrow | 1 | **1** |

The three positives **failed before step 2 and passed after**. Narrowing survived.

## STOP-1 — map's `U`

`mapv_output_concrete`: `mapv inc [1 2 3]` feeds `takes-vec` of
`(Vector :- [i64])`. EXIT 0 both steps. Covariant `assignable(i64, u_var)`
still binds `u_var` through unify at the Fn-arm's return.

## STOP-2 — contravariance on a Var

`wat::collection` 248/248, `wat::types` 641/641, the three arc278 foldl
controls still PASS. No collection test changed behaviour.

## STOP-3 — goldens

No existing `.edn` recaptured. New golden is filter (expected return is
`bool`, no drifting `:?N`):

`expected "[:u::Op :-> :wat::core::bool]"` /
`got "[:u::Op.Mark :-> :wat::core::bool]"`

mapv's expected return is a fresh `U` and the `:?N` is **not** stable
across `--check` runs — that is why the negative is filter, not mapv.

## STOP-4 — fifth copy

None outside these four functions.

## Acceptance

1. ONE helper; four call sites; no copy of the block remains.
2. Step 1: refactor; discriminator probes still refused; floor not run.
3. Step 2: types 641/0 failed; collection 248/0; probes as tabled.
4. Clippy not run.
5. Positive probes for map, mapv, filter: FAIL then PASS.
6. Negative control: structural `.edn` via `UPDATE_EDN`.

Probes: `tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg.rs`.
