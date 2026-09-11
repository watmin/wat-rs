# SCORE — enrol the variant in the subtype lattice

Struck. Floor not run. Clippy not run. Not committed.

```
cargo build --release                                          BUILD_EXIT=0
cargo nextest run --release -E 'test(probe_arc278_call_context::)
  + test(probe_arc278_arming_is_internal_only::control_arming)
  + test(wrong_response_type_at_reply_site)'                   8 passed, NEXTEST_EXIT=0
cargo nextest run --release -E 'binary_id(wat::types)'         635 passed, 5 skipped, NEXTEST_EXIT=0
```

## Diagnosis correction (measured)

The brief's strike — `register_subtype(variant, enum)` in `register_variant_types` —
**is already landed.** A-2 (`0d4f4c46b`) added it at `src/types.rs:1135`. This tree:

```
(:wat::core::subtype? :u::Demo.Has :u::Demo)   => true
(:wat::core::subtype? :u::Demo :u::Demo.Has)   => false
Alarm<Op.Mark> → Alarm<Op> value slot           EXIT 0   (SAME-head arm already asks is_subtype)
```

`is_subtype` returns true. The SAME-head parametric arm of `assignable` already
consumes the edge for a **value** slot. The 6 reds are not a missing edge.

### Why foldl still failed

`infer_foldl` (`src/collection/infer.rs:987`) used **`unify`** on the reducer,
not `assignable`. Unify of `TypeExpr::Fn` is invariant on arguments. Direct
foldl calls never reached `assignable` at all (`infer_list` intercepts
`:wat::core::foldl`).

Even after switching foldl to `assignable`, Fn types fell through to the same
invariant unify (no Fn arm). Discriminator measured pre-fix:

```
fn(Alarm<Op>)  where  fn(Alarm<Op.Mark>) expected    EXIT 1 TypeMismatch
```

That is the foldl shape.

## What changed

1. `assignable`: Fn arm — arguments CONTRAVARIANT, return COVARIANT. The
   SAME-head `is_subtype` question now runs inside a function argument.
2. `infer_foldl`: `assignable(&f_ty, &expected_fn_ty, …)` instead of `unify`.

`src/types.rs` **untouched**. No new `register_subtype`. No `assignable`
subtype-lattice rewrite — the existing Path-Path / SAME-head arms are the
consumer; they were unreachable from foldl.

## STOP-1 — `is_subtype_parent` reclassification

**Not newly triggered.** Every enum is already a value of `subtype_edges`
because A-2 registered the edges. `is_subtype_parent(":wat::core::Option")`
was already true before this stone. We added no edges.

## STOP-2 — cycle check

Did not call `register_subtype`. No `CyclicSubtype`.

## STOP-3 — stdlib scale / floor wall time

Floor not run (brief). No new edges, so the linear scan over values is
unchanged.

## STOP-4 — passing tests

`probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error`
**still PASS** (the golden that pins a narrow variant name).

## Acceptance

| row | result |
|---|---|
| 6 arc278 reds | **6/6 PASS** (plus `two_param_public_arm` already green) |
| Demo.Has → Demo | **EXIT 0** |
| Demo → Demo.Has | **EXIT 1** TypeMismatch (narrowing refused) |
| Demo.Has → Demo.Has | **EXIT 0** |
| fn(Alarm<Op>) where fn(Alarm<Op.Mark>) expected | **EXIT 0** (was 1) |
| fn(Alarm<Op.Mark>) where fn(Alarm<Op>) expected | **EXIT 1** TypeMismatch |
| Alarm<Op.Mark> → Alarm<Op> | **EXIT 0** |
| Alarm<Op> → Alarm<Op.Mark> | **EXIT 1** |
| floor / clippy | not run |

Probes: `tests/types/probe_arc251_enrol_the_variant_in_the_lattice.rs` (7 tests).

## Related, not fixed

`infer_map` / `infer_mapv` / `infer_filter` still `unify` the function
(`collection/infer.rs:761,826,898`). Same shape as foldl. Not in the 6 reds.
Not a fourth *read* path (STOP-4 of this brief). Flagged for the orchestrator.

## Uncertain

Unit-variant keywords in value position still infer as `[:-> :Op.Mark]`
(a fn type), so an Alarm constructed with a bare `:Op.Mark` keyword does
not type as `Alarm<Op.Mark>`. Tagged ctors and the generated `:-mark`
path are Paths. Pre-existing; not opened here.
