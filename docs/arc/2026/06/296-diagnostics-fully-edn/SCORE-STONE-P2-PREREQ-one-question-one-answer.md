# SCORE — STONE P-2 PREREQ: one question, one answer

No commit. Floor and clippy left to the orchestrator. Lands on P-1 RELAND-1.
Nothing reverted. `is-type?` was not plumbed; it asks `TypeEnv::contains`
and now sees this program's `use!` names because they are seeded as
membership-only leaves.

## What landed

`TypeEnv::register_use_declared_leaf` — `pub(crate)`, idempotent, inserts
into `builtin_names` only. `register_builtin_leaf` stays private; both
`debug_assert!`s stay. After user-source `use!` is collected from residue,
each path is seeded, then `types_insert` clones the env (so the seed
survives the fork).

`is-type?` is untouched. The subject fixture stdout is `true`.

## The collapse did not hold — STOP-3, arm restored

Seeding `use_decls.list()` (stdlib + user) into TypeEnv makes
`is-type? :rust::sqlite::Connection` true in **every** program: stdlib
`wat/sqlite.wat` already `use!`s it. That flips `no_use_is_not_a_type`
to `true` — STOP-1. Not seeded from the build-time registry; seeded from
stdlib's `use!`, which RELAND-1 collects into the same `use_decls` the
sketch walked.

Seeding **user residue only** keeps STOP-1 green. Removing
`use_decls.covers` then turns every program red. Verbatim, first firing
on `…__use_then_is_type.wat` (and the other three fixtures, and P-1's
control):

```
#wat.type/UnknownNamedType {:message "annotation names unknown type :rust::sqlite::ReadConnection — not a declared type, not a type variable, and not a builtin" :location #wat.core/Span {:file "src/check.rs" :line 15260 :col 13 :end #wat.core/Option.None {}} :causes [] :path ":rust::sqlite::ReadConnection"}
```

The two stores are not equivalent. Stdlib `use!` of `:rust::sqlite::*` /
`:rust::cache::Lru` lives in `use_decls` for the annotation wall and must
**not** live in `TypeEnv` (or STOP-1 fires). `covers` restored.
`grep -c use_decls src/declare/typevar.rs` → `3`.

The remaining disagreement is exactly that split: a program with no user
`use!` still has the wall accept `:rust::sqlite::Connection` (stdlib
covers) while `is-type?` answers `false`. That is STOP-1 holding, not a
missed seed.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 `no_use` turns true | **held** after user-only seed. Fired on the first sketch (seed all `use_decls`). Fixture not adjusted. |
| STOP-2 `register_builtin_leaf` pub / debug_assert removed | **held.** New `pub(crate)` entry point beside it. |
| STOP-3 collapse turns red | **fired, arm restored.** Verbatim block above. |
| STOP-4 hand-list row deleted | **held.** Crossbeam Sender/Receiver untouched. |

## Expectations

| # | result |
|---|---|
| 1 | `use_then_is_type` stdout `true` |
| 2 | `no_use_is_not_a_type` stdout `false` |
| 3 | `handlist_control` stdout `true` |
| 4 | `phantom_rust_name` stdout `false` |
| 5 | `cargo nextest run --release -E 'test(p2prereq)'` **4 passed, 0 skipped** |
| 6 | `test(p1_annotation)` **10 passed, 0 skipped** |
| 7 | P-1 phantom param+return EXIT=1, path `:usr::TotallyMadeUp` |
| 8 | collapse | **STOP-3.** `use_decls` count in typevar.rs is 3, not 0. |
| 9 | `get` stays None | `types::tests::stone_296_use_declared_leaf_has_membership_without_structure` — contains true, get None, second seed no-op, seed of `:wat::core::i64` no-op |
| 10 | hand-list untouched | `git diff src/types.rs` does not delete Group 3 rows |

## Targeted checks

```
./target/release/wat …__use_then_is_type.wat        stdout true
./target/release/wat …__no_use_is_not_a_type.wat    stdout false
./target/release/wat …__handlist_control.wat        stdout true
./target/release/wat …__phantom_rust_name.wat       stdout false
cargo nextest run --release -E 'test(p2prereq)'     4 passed, 0 skipped
cargo nextest run --release -E 'test(p1_annotation)' 10 passed, 0 skipped
```

Floor **orchestrator**. Clippy **orchestrator**.

## Sites inspected

- `tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union.rs` + four fixtures
- `src/freeze/env.rs` stdlib collection (214–221) vs user residue (254–274)
- `src/types.rs` `register_builtin_leaf` (private, two `debug_assert!`s), Group 3 crossbeam rows
- `src/resolve/rust_use.rs` `collect_use_declarations` (registry.has_type gate)
- `src/resolve/walk.rs` Pass 1 — walks **user residue only**, same per-program rule as the seed
- `src/reflect/verbs.rs` `eval_is_type` — not edited; still `contains` ∪ `is_builtin_primitive`
- `src/declare/typevar.rs` / `src/check.rs` — covers removed, then restored

## What surprised

The sketch's `for path in use_decls.list()` walks a set RELAND-1 already
filled with stdlib `use!`. The DESIGN's "Where" cites the residue block
and does not mention the stdlib_post_types collection ten lines above.
The two-fixture isolation and the collapse cannot both hold for
`:rust::sqlite::Connection`: it is the subject name **and** a stdlib
`use!`. No `:rust::*` in wat-rs defaults is use!-able and not already
use!'d by stdlib, so there is no other name the isolation could have
picked without adjusting the fixture (forbidden).

## Working tree

```
src/types.rs           register_use_declared_leaf + get-stays-None test
src/freeze/env.rs      seed user use! into TypeEnv; stdlib stays in use_decls only
src/declare/typevar.rs covers restored (STOP-3)
src/check.rs           use_decls parameter stays
tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union.rs  un-ignore
```

Do not commit unless a later brief says to.
