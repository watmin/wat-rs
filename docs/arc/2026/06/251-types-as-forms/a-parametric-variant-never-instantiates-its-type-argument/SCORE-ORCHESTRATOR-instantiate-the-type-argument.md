# SCORE (independent re-run) — instantiate the type argument

Independent re-run of the rider's SCORE. Floor not run. Clippy not run.
No re-implementation.

HEAD is `090525650` STONE(251/parametric) — the strike is **committed**
(working tree clean). `assignable` is not in the commit diff
(`assignable_diff_hits=0`).

```
cargo build --release                                          BUILD_EXIT=0
cargo nextest run --release -E 'binary_id(wat::types)'         614 passed, 5 skipped, NEXTEST_EXIT=0
```

## Rows, re-run

Independent `--check` of each fixture with `target/release/wat`, plus the
runtime twin. Rider's pre-fix column is historical (not re-measured on a
reverted tree).

| # | row | rider post-fix | this re-run |
|---|---|---|---|
| 1 | parametric RECORD `{:keys}` | EXIT 0 | **EXIT 0** |
| 2 | parametric STRUCT `{:keys}` | EXIT 0 | **EXIT 0** |
| 3 | parametric VARIANT `{:keys}` | EXIT 0 | **EXIT 0** |
| 4 | parametric VARIANT accessor | EXIT 0 | **EXIT 0** |
| 5 | parametric RECORD accessor | EXIT 0 | **EXIT 0** |
| 6 | parametric ENUM via match | EXIT 0 | **EXIT 0** |
| 7 | NON-parametric record `{:keys}` | EXIT 0 | **EXIT 0** |
| 8 | NON-parametric variant `{:keys}` | EXIT 0 | **EXIT 0** |
| 9 | Demo.Has → Demo slot | EXIT 0 | **EXIT 0** |
| 10 | Demo → Demo.Has slot | EXIT 1, TypeMismatch | **EXIT 1**, `TypeMismatch`: expects `(:u::Demo.Has :- [i64])`; got `(:u::Demo :- [i64])`. No `UnknownNamedType`. |
| 11 | `parametric_variant_accessor_runs` | prints `42` | **RUN_EXIT=0**, stdout `42` |
| 12 | `binary_id(wat::types)` | 614 / 5 skipped | **614 passed, 5 skipped, EXIT 0** |
| 13 | 11 named probe tests | all PASS | **11/11 PASS** |

**13/13 match the rider on this re-run.**

## Source (this re-run)

- `instantiate_field_types` at `src/check.rs:17827` — zips params with
  supplied args, `rename`s field types. Called from both Aggregate and
  singleton-Enum arms of keys-destructure.
- Variant accessors minted in `register_enum_methods` after each tagged
  ctor (`src/declare/register.rs:1445`). Scheme: `type_params` = parent
  list, param = `parametric_decl_type(constructor_path, …)`, ret =
  declared field type, body `(:field self)`.
- `fn assignable` at `src/check.rs:17131` — **not in the commit diff**.

## STOP rows (from this re-run)

| STOP | this re-run |
|---|---|
| STOP-1 subtyping / `assignable` | **held** — no `assignable` lines in the commit; widening/narrowing fixtures match |
| STOP-2 same-head covariance | **held** — not in the diff |
| STOP-3 floor failures > 8 | floor not run; `wat::types` 614/5, no new red in that binary |
| STOP-4 third site same shape | not independently censused; rider's remaining `Parametric { head, .. }` claim not re-walked |

## Related bare `(:has d)` — confirmed

The rider's four-line keyword-accessor placeholder repro `--check`s
**BARE_HAS_EXIT=0** on this tree. Different shape, not this stone.

No further commit from this re-run.

---

## STOP-4 closed by the orchestrator (the census was mine, per the brief)

Censused every `TypeExpr::Parametric { head, .. }` in `src/check.rs` that discards its args — 13
sites — then **READ** the three whose next six lines also touch `types().get` / `.fields` / 
`variant_fields`, because a count is not a classification.

```
:2443   is_enum predicate — "is this an Enum?"          membership only, args irrelevant
:2508   the same predicate, second copy                 membership only, args irrelevant
:5824   "is this a SINGLETON Enum?"                     a classifier — AND the third shape
```

**No fourth "look up the TypeDef, use its declared field types" site exists.** The rider's claim
holds, now verified by reading rather than by pattern.

★ `:5824` is the HARVEST placeholder the peer reported as the related-but-different shape, and its
own comment names it:

```rust
// HARVEST (236.2): silent-by-intent — polymorphic accessor placeholder.
let ty = fresh.fresh();
```

So the false green on bare `(:has d)` is **not an accident — it is a recorded deferral from arc
236.2**, silent by design and silent ever since. It is the same mechanism as the expander bug (an
unresolvable read yielding a type that satisfies every expectation), but it was *chosen*, which
makes it a ruling to revisit rather than a defect to fix.

⚠ And `:Enum.Variant/field` now EXISTS, which is what the placeholder was standing in for. Its
premise has changed under it — the disposition (instantiate the placeholder, or retire it) is the
builder's, and belongs to its own stone.
